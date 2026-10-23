//! Tek seferlik hayalet kayıt temizliği (yerel menü akışı planı 6.3 / Faz 5.1).
//!
//! Verilen şehir + ay kapsamında, kaynak dosyada BULUNMAYAN `kepce-%` kaynaklı
//! (tarih, öğün) kayıtlarını raporlar ve `--apply` verilirse sert DELETE ile
//! siler. `menu_dishes`, `comments`, `votes` ve `reports` tabloları
//! `ON DELETE CASCADE` ile bağlı olduğu için tek silme yeterlidir.
//!
//! **ZORUNLU ÖN KOŞUL:** Çalıştırmadan önce veritabanı yedeği alın. Silme
//! işlemi o menülere bağlı kullanıcı yorumlarını ve oylarını da kaldırır.
//!
//! Kullanım:
//! ```text
//! cargo run -p worker --example month_atomic_cleanup -- <sehir-slug> <yyyy-mm> <dosya> [--apply]
//! # örnek (dry-run):
//! cargo run -p worker --example month_atomic_cleanup -- istanbul 2026-06 data/menuler/admin/bekleyen/istanbul/Haziran.xlsx
//! ```

use std::collections::HashSet;
use std::path::Path;

use chrono::{Datelike, NaiveDate};
use sea_orm::{ColumnTrait, Database, EntityTrait, QueryFilter};
use shared::entities::{cities, menus};
use worker::parser::models::MenuDatabase;

fn parse_offline(path: &Path) -> anyhow::Result<MenuDatabase> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let path_str = path.to_string_lossy().to_string();
    let mut db = MenuDatabase::new();
    match ext.as_str() {
        "xlsx" | "xls" => {
            worker::parser::excel::parse_excel(&path_str, &mut db)?;
        }
        "json" => {
            db = worker::parser::json::parse_json_file(&path_str, "temizlik")?;
        }
        other => anyhow::bail!("desteklenmeyen uzantı: '{}' (xlsx/json olmalı)", other),
    }
    Ok(db)
}

fn build_keep_set(db: &MenuDatabase) -> HashSet<(NaiveDate, String)> {
    let mut keep = HashSet::new();
    for (date_str, day) in db {
        let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            continue;
        };
        if !day.normal.breakfast.is_empty() || !day.colyak.breakfast.is_empty() {
            keep.insert((date, "breakfast".to_string()));
        }
        if !day.normal.lunch.is_empty() || !day.colyak.lunch.is_empty() {
            keep.insert((date, "lunch".to_string()));
        }
        if !day.normal.dinner.is_empty() || !day.colyak.dinner.is_empty() {
            keep.insert((date, "dinner".to_string()));
        }
    }
    keep
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let apply = args.iter().any(|a| a == "--apply");
    let positional: Vec<&String> = args.iter().filter(|a| *a != "--apply").collect();
    if positional.len() != 3 {
        eprintln!(
            "Kullanım: cargo run -p worker --example month_atomic_cleanup -- <sehir-slug> <yyyy-mm> <dosya> [--apply]"
        );
        std::process::exit(1);
    }
    let city_slug = positional[0].as_str();
    let month_arg = positional[1].as_str();
    let file_path = Path::new(positional[2].as_str());

    let month_start = NaiveDate::parse_from_str(&format!("{}-01", month_arg), "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("geçersiz ay: '{}' (yyyy-mm bekleniyor)", month_arg))?;

    println!("=== ay atomik temizlik (dry_run: {}) ===", !apply);
    if apply {
        println!("!! UYARI: --apply verildi, kayıtlar KALICI OLARAK SİLİNECEK.");
        println!("!! Veritabanı yedeği alındı mı? (yorumlar ve oylar da cascade ile silinir)");
    }

    // 1. Kaynak dosyayı çevrimdışı ayrıştır
    let db_file = parse_offline(file_path)?;
    let keep = build_keep_set(&db_file);
    println!(
        "dosya: {:?} -> {} gün, {} (tarih, öğün) ikilisi",
        file_path,
        db_file.len(),
        keep.len()
    );

    // 2. Veritabanına bağlan
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await?;

    let city = cities::Entity::find()
        .filter(cities::Column::Slug.eq(city_slug))
        .one(&db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("'{}' şehri bulunamadı", city_slug))?;

    // 3. Kapsamdaki mevcut kepce-% kayıtlarını listele
    let next_month = if month_start.month0() == 11 {
        NaiveDate::from_ymd_opt(month_start.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(month_start.year(), month_start.month() + 1, 1).unwrap()
    };
    let existing = menus::Entity::find()
        .filter(menus::Column::CityId.eq(city.id))
        .filter(menus::Column::SourceType.like("kepce-%"))
        .filter(menus::Column::ServeDate.gte(month_start))
        .filter(menus::Column::ServeDate.lt(next_month))
        .all(&db)
        .await?;
    println!(
        "veritabanı: {} kapsamında kepce-% kaynaklı {} kayıt var",
        month_arg,
        existing.len()
    );

    // 4. Silinecekleri raporla
    let meal_str = |m: &shared::entities::sea_orm_active_enums::MealTypeEnum| -> String {
        use shared::entities::sea_orm_active_enums::MealTypeEnum;
        match m {
            MealTypeEnum::Breakfast => "breakfast".to_string(),
            MealTypeEnum::Lunch => "lunch".to_string(),
            MealTypeEnum::Dinner => "dinner".to_string(),
        }
    };
    let obsolete: Vec<&menus::Model> = existing
        .iter()
        .filter(|m| !keep.contains(&(m.serve_date, meal_str(&m.meal_type))))
        .collect();
    println!("\nSilinecek kayıtlar ({}):", obsolete.len());
    for m in &obsolete {
        println!(
            "  - id={} {} {} kaynak={:?} durum={:?}",
            m.id,
            m.serve_date,
            meal_str(&m.meal_type),
            m.source_type,
            m.status
        );
    }

    if obsolete.is_empty() {
        println!("\nKapsam dışında kayıt yok, temizlik gerekmiyor.");
        return Ok(());
    }

    // 5. --apply verilirse sil
    if apply {
        let deleted = worker::tasks::scraper::delete_out_of_scope_menus(
            &db,
            city.id,
            "kepce-",
            month_start,
            &keep,
            None,
        )
        .await?;
        println!(
            "\n✅ {} kayıt silindi (cascade ile bağlı öğün/yorum/oy verileri dahil).",
            deleted
        );
    } else {
        println!("\n(dry-run) Silmek için --apply ekleyin. ÖNCE YEDEK ALIN.");
    }

    Ok(())
}

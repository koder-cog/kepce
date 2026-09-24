use anyhow::Result;
use chrono::{Local, NaiveDate};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::entities::cities;
use std::env;
use std::path::PathBuf;

use crate::parser::models::MenuDatabase;

/// Ağ/servis kaynaklı **geçici** hataları ayırt eder.
///
/// Geçici hatada dosya `bekleyen` klasöründe bırakılır ve sonraki tarama
/// döngüsünde yeniden denenir. Kalıcı hatada ise `hatali` klasörüne taşınır.
/// (Örn. Gemini 503 "high demand" / 429 "rate limit" geçicidir; tek denemede
/// kalıcı sayılıp `hatali` klasörüne atılması veri kaybına yol açar.)
fn is_transient_error(err_msg: &str) -> bool {
    const TRANSIENT_MARKERS: [&str; 13] = [
        "timeout",
        "geçici api hatası",
        "istek atılamadı",
        "service unavailable",
        "service_unavailable",
        "503",
        "429",
        "too many requests",
        "rate limit",
        "rate_limit",
        "high demand",
        "overloaded",
        "temporarily",
    ];
    TRANSIENT_MARKERS
        .iter()
        .any(|marker| err_msg.contains(marker))
}

/// Bir menü veritabanının özeti: gün sayısı, öğün kırılımı ve tarih aralığı.
///
/// Gözlemlenebilirlik için kullanılır: sessiz veri kaybını (ör. çok bölümlü bir
/// PDF'te kahvaltı tablosunun atlanması) görünür kılar.
#[derive(Debug, Default, Clone)]
struct MenuSummary {
    days: usize,
    breakfast: usize,
    lunch: usize,
    dinner: usize,
    first_date: Option<NaiveDate>,
    last_date: Option<NaiveDate>,
}

/// `MenuDatabase` içinden gün sayısı, öğün kırılımı ve tarih aralığını çıkarır.
///
/// Saf fonksiyon (env okumaz), birim testi kolay olsun diye ayrıldı.
fn summarize_menu_db(db: &MenuDatabase) -> MenuSummary {
    let mut s = MenuSummary::default();
    for (date_str, day) in db {
        s.days += 1;
        let has_breakfast = !day.normal.breakfast.is_empty() || !day.colyak.breakfast.is_empty();
        let has_lunch = !day.normal.lunch.is_empty() || !day.colyak.lunch.is_empty();
        let has_dinner = !day.normal.dinner.is_empty() || !day.colyak.dinner.is_empty();
        if has_breakfast {
            s.breakfast += 1;
        }
        if has_lunch {
            s.lunch += 1;
        }
        if has_dinner {
            s.dinner += 1;
        }
        if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            s.first_date = Some(s.first_date.map_or(d, |f| f.min(d)));
            s.last_date = Some(s.last_date.map_or(d, |l| l.max(d)));
        }
    }
    s
}

/// "Beklenenden az gün" eşiği (0 < r <= 1). Varsayılan 0.6.
///
/// `WORKER_MIN_DAYS_RATIO` ile ayarlanır; geçersiz/aralık dışı değerde varsayılana döner.
fn min_days_ratio() -> f64 {
    std::env::var("WORKER_MIN_DAYS_RATIO")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|r| *r > 0.0 && *r <= 1.0)
        .unwrap_or(0.6)
}

/// Tek öğün tipi uyarısı için minimum gün sayısı.
const MEAL_MIX_MIN_DAYS: usize = 10;

/// Beklenenden az gün / eksik öğün tablosu uyarısı üretir (WARN amaçlı; asla bloklamaz).
///
/// İki sezgisel:
/// 1. **Belge içi boşluk:** çıkarılan gün sayısı, belgenin kapsadığı tarih
///    aralığına (`first..last`) göre beklenen minimumun altındaysa.
/// 2. **Öğün karışımı:** çok günlük belgede yalnızca tek öğün tipi varsa
///    (çoklu-tablo atlanmış olabilir).
fn low_days_warning(summary: &MenuSummary) -> Option<String> {
    if let (Some(first), Some(last)) = (summary.first_date, summary.last_date) {
        let span = (last - first).num_days().max(0) as usize + 1;
        if span > 1 {
            let expected_min = ((span as f64) * min_days_ratio()).ceil() as usize;
            if summary.days < expected_min {
                return Some(format!(
                    "Beklenenden az gün: {} gün çıkarıldı, belge {} günlük aralığı kapsıyor (beklenen en az {}). Olası sessiz veri kaybı.",
                    summary.days, span, expected_min
                ));
            }
        }
    }
    if summary.days >= MEAL_MIX_MIN_DAYS {
        let meal_types_present = [summary.breakfast, summary.lunch, summary.dinner]
            .iter()
            .filter(|&&c| c > 0)
            .count();
        if meal_types_present == 1 {
            return Some(format!(
                "{} günlük belgede yalnızca tek öğün tipi bulundu (kahvaltı {}, öğle {}, akşam {}). Çoklu-tablo atlanmış olabilir.",
                summary.days, summary.breakfast, summary.lunch, summary.dinner
            ));
        }
    }
    None
}

pub async fn process_local_files(
    db: &DatabaseConnection,
    reqwest_client: &reqwest::Client,
    gemini_api_key: Option<&str>,
) -> Result<()> {
    let base_dir = env::var("WORKER_MENU_DIR").unwrap_or_else(|_| "../data/menuler".to_string());

    let configs = ["admin", "kullanici", "anonim"];
    let mut processed = 0;

    for folder in configs.iter() {
        let bekleyen_path = PathBuf::from(&base_dir).join(folder).join("bekleyen");
        if !bekleyen_path.exists() {
            continue;
        }

        let mut cities_iter = match tokio::fs::read_dir(&bekleyen_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        while let Ok(Some(city_entry)) = cities_iter.next_entry().await {
            let city_path = city_entry.path();
            if !city_path.is_dir() {
                continue;
            }

            let city_slug = city_entry.file_name().to_string_lossy().to_string();

            // Get city ID from slug
            let city_opt = cities::Entity::find()
                .filter(cities::Column::Slug.eq(&city_slug))
                .one(db)
                .await?;

            let city_id = match city_opt {
                Some(c) => c.id,
                None => {
                    tracing::warn!("Lokal dosya taraması: '{}' adlı şehir veritabanında bulunamadı, atlanıyor.", city_slug);
                    continue;
                }
            };

            let mut files_iter = match tokio::fs::read_dir(&city_path).await {
                Ok(f) => f,
                Err(_) => continue,
            };

            while let Ok(Some(file_entry)) = files_iter.next_entry().await {
                let path = file_entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                tracing::info!("Lokal dosya ayrıştırılıyor: {}/{}", city_slug, filename);

                let path_str = path.to_string_lossy().to_string();

                let source_type = format!("kepce-{}", folder);

                // Gözlemlenebilirlik: bu dosyadan çıkarılan gün/öğün özeti.
                // `file_db` kaydetmeye tüketilmeden ÖNCE hesaplanır; her dosyada sıfırlanır.
                let mut menu_summary: Option<MenuSummary> = None;

                let result = if ext.to_lowercase() == "xlsx" {
                    let mut file_db = crate::parser::models::MenuDatabase::new();
                    match crate::parser::excel::parse_excel(&path_str, &mut file_db) {
                        Ok(_) => {
                            for day_data in file_db.values_mut() {
                                crate::parser::validation::finalize_day_metadata(day_data);
                            }
                            menu_summary = Some(summarize_menu_db(&file_db));
                            crate::parser::save_menu_database(
                                db,
                                city_id,
                                &source_type,
                                file_db,
                                &city_slug,
                            )
                            .await
                        }
                        Err(e) => Err(anyhow::anyhow!("Excel parse hatası: {}", e)),
                    }
                } else if ext.to_lowercase() == "json" {
                    match crate::parser::json::parse_json_file(&path_str, &city_slug) {
                        Ok(mut file_db) => {
                            for day_data in file_db.values_mut() {
                                crate::parser::validation::finalize_day_metadata(day_data);
                            }
                            menu_summary = Some(summarize_menu_db(&file_db));
                            crate::parser::save_menu_database(
                                db,
                                city_id,
                                &source_type,
                                file_db,
                                &city_slug,
                            )
                            .await
                        }
                        Err(e) => Err(anyhow::anyhow!("JSON parse hatası: {}", e)),
                    }
                } else if matches!(
                    ext.to_lowercase().as_str(),
                    "pdf" | "png" | "jpg" | "jpeg" | "webp" | "heic" | "heif"
                ) {
                    if crate::parser::llm::llm_available(gemini_api_key) {
                        match crate::parser::llm::parse_document_with_llm(
                            reqwest_client,
                            gemini_api_key,
                            std::path::Path::new(&path_str),
                        )
                        .await
                        {
                            Ok(file_db) => {
                                menu_summary = Some(summarize_menu_db(&file_db));
                                crate::parser::save_menu_database(
                                    db,
                                    city_id,
                                    &source_type,
                                    file_db,
                                    &city_slug,
                                )
                                .await
                            }
                            Err(e) => Err(anyhow::anyhow!("Belge/Görsel LLM parse hatası: {}", e)),
                        }
                    } else {
                        tracing::warn!(
                            "{}: LLM parsing devre dışı - hiçbir sağlayıcı anahtarı (OPENROUTER_API_KEY/GEMINI_API_KEY) ayarlanmamış, atlanıyor.",
                            filename
                        );
                        continue;
                    }
                } else {
                    continue;
                };

                let success = match result {
                    Ok(_) => {
                        match menu_summary.as_ref() {
                            Some(s) => {
                                tracing::info!(
                                    "{}: {} gün ({} kahvaltı + {} öğle + {} akşam) işlendi (kaynak: {})",
                                    filename,
                                    s.days,
                                    s.breakfast,
                                    s.lunch,
                                    s.dinner,
                                    source_type
                                );
                                if let Some(warning) = low_days_warning(s) {
                                    tracing::warn!("{}: {}", filename, warning);
                                }
                            }
                            None => {
                                tracing::info!("{}: Başarıyla veritabanına işlendi.", filename);
                            }
                        }
                        processed += 1;
                        true
                    }
                    Err(e) => {
                        let err_msg = format!("{:?}", e).to_lowercase();
                        if is_transient_error(&err_msg) {
                            tracing::error!(
                                "{}: Geçici ağ/API hatası, dosya kuyrukta bekletilecek: {:?}",
                                filename,
                                e
                            );
                            continue; // Dosyayı bekleyen klasöründe bırak (taşıma)
                        } else {
                            tracing::error!(
                                "{}: Kalıcı ayrıştırma/kaydetme hatası: {:?}",
                                filename,
                                e
                            );
                            false
                        }
                    }
                };

                // Move file
                if success {
                    let vault_base = PathBuf::from(&base_dir).join("vault");
                    let vault_dir = vault_base
                        .join(ext.to_lowercase())
                        .join(folder)
                        .join(&city_slug);
                    let _ = tokio::fs::create_dir_all(&vault_dir).await;
                    let dest = vault_dir.join(format!(
                        "{}_{}",
                        Local::now().format("%Y%m%d_%H%M%S"),
                        filename
                    ));
                    if let Err(e) = tokio::fs::rename(&path, &dest).await {
                        tracing::warn!(
                            "Dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...",
                            e
                        );
                        if let Err(copy_err) = tokio::fs::copy(&path, &dest).await {
                            tracing::error!("Kopyalama başarısız ({:?}). Sonsuz döngüyü önlemek için dosya uzantısı .failed yapılıyor...", copy_err);
                            let failed_dest = path.with_extension(format!("{}.failed", ext));
                            if let Err(rename_err) = tokio::fs::rename(&path, &failed_dest).await {
                                tracing::error!(
                                    "Dosya .failed olarak yeniden adlandırılamadı: {:?}",
                                    rename_err
                                );
                            }
                        } else {
                            if let Err(remove_err) = tokio::fs::remove_file(&path).await {
                                tracing::error!("Kaynak dosya silinemedi ({:?}): {:?}. Yeniden işlenmemesi için .processed yapılıyor...", path, remove_err);
                                let _ = tokio::fs::rename(
                                    &path,
                                    path.with_extension(format!("{}.processed", ext)),
                                )
                                .await;
                            }
                        }
                    }
                } else {
                    let err_dir = PathBuf::from(&base_dir).join(folder).join("hatali");
                    let _ = tokio::fs::create_dir_all(&err_dir).await;
                    let dest = err_dir.join(filename);
                    if let Err(e) = tokio::fs::rename(&path, &dest).await {
                        tracing::warn!(
                            "Hatalı dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...",
                            e
                        );
                        if let Err(copy_err) = tokio::fs::copy(&path, &dest).await {
                            tracing::error!("Kopyalama başarısız ({:?}). Sonsuz döngüyü önlemek için dosya uzantısı .failed yapılıyor...", copy_err);
                            let failed_dest = path.with_extension(format!("{}.failed", ext));
                            if let Err(rename_err) = tokio::fs::rename(&path, &failed_dest).await {
                                tracing::error!(
                                    "Dosya .failed olarak yeniden adlandırılamadı: {:?}",
                                    rename_err
                                );
                            }
                        } else {
                            if let Err(remove_err) = tokio::fs::remove_file(&path).await {
                                tracing::error!("Kaynak dosya silinemedi ({:?}): {:?}. Yeniden işlenmemesi için .failed yapılıyor...", path, remove_err);
                                let _ = tokio::fs::rename(
                                    &path,
                                    path.with_extension(format!("{}.failed", ext)),
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::info!(
        "Lokal dosya taraması tamamlandı. {} dosya işlendi.",
        processed
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{DayData, MenuComponent, MenuItem};

    fn item(name: &str) -> MenuItem {
        MenuItem {
            takeaway_id: None,
            alternatives: vec![MenuComponent::from(name)],
        }
    }

    fn day(breakfast: usize, lunch: usize, dinner: usize) -> DayData {
        let mut d = DayData::default();
        d.normal.breakfast = (0..breakfast).map(|i| item(&format!("b{}", i))).collect();
        d.normal.lunch = (0..lunch).map(|i| item(&format!("l{}", i))).collect();
        d.normal.dinner = (0..dinner).map(|i| item(&format!("d{}", i))).collect();
        d
    }

    fn db_with_dates(
        dates: &[&str],
        breakfast: usize,
        lunch: usize,
        dinner: usize,
    ) -> MenuDatabase {
        let mut db = MenuDatabase::new();
        for dt in dates {
            db.insert((*dt).to_string(), day(breakfast, lunch, dinner));
        }
        db
    }

    #[test]
    fn test_summarize_menu_db_counts_days_and_meals() {
        let db = db_with_dates(&["2026-09-01", "2026-09-02"], 2, 0, 3);
        let s = summarize_menu_db(&db);
        assert_eq!(s.days, 2);
        assert_eq!(s.breakfast, 2);
        assert_eq!(s.lunch, 0);
        assert_eq!(s.dinner, 2);
        assert_eq!(s.first_date, NaiveDate::from_ymd_opt(2026, 9, 1));
        assert_eq!(s.last_date, NaiveDate::from_ymd_opt(2026, 9, 2));
    }

    /// 30 günlük aralık kapsanırken belirgin şekilde az gün çıkarılırsa uyarı verilmeli.
    #[test]
    fn test_low_days_warning_detects_gap() {
        let mut db = db_with_dates(&["2026-09-01", "2026-09-30"], 0, 0, 3);
        for d in 2..=15 {
            db.insert(format!("2026-09-{:02}", d), day(0, 0, 3));
        }
        let s = summarize_menu_db(&db);
        assert!(low_days_warning(&s).is_some(), "boşluk uyarısı bekleniyor");
    }

    /// Tam ay (kahvaltı + akşam) çıkarıldığında uyarı OLMAMALI.
    #[test]
    fn test_low_days_warning_absent_for_full_month() {
        let dates: Vec<String> = (1..=30).map(|d| format!("2026-09-{:02}", d)).collect();
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let s = summarize_menu_db(&db);
        assert!(low_days_warning(&s).is_none(), "tam ayda uyarı beklenmiyor");
    }

    /// Çok günlük belgede yalnız tek öğün tipi varsa öğün-karışımı uyarısı verilmeli.
    #[test]
    fn test_low_days_warning_single_meal_mix() {
        let dates: Vec<String> = (1..=20).map(|d| format!("2026-09-{:02}", d)).collect();
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 0, 0, 3);
        let s = summarize_menu_db(&db);
        let warning = low_days_warning(&s).expect("öğün-karışımı uyarısı bekleniyor");
        assert!(warning.contains("tek öğün"));
    }
}

//! Kendini Tamir Eden Veritabanı Temizleyicisi (Self-Healing Sanitizer).
//!
//! Sisteme sızmış veya kazıma anomalileriyle oluşmuş çöp menüleri,
//! navigasyon kalıntılarını ve geçersiz alias'ları tespit edip otomatik olarak
//! temizler ve güvenli duruma getirir.

use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseBackend, DatabaseConnection, EntityTrait,
    FromQueryResult, QueryFilter, Set, Statement,
};
use shared::entities::{dish_aliases, menu_dishes, menus, sea_orm_active_enums::MenuStatusEnum};
use shared::services::content_guard::ContentGuard;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SanitizerReport {
    pub junk_aliases_found: usize,
    pub menus_rejected: usize,
    pub menu_dishes_unlinked: usize,
    pub orphaned_aliases_deleted: usize,
}

#[derive(Debug, FromQueryResult)]
struct MenuDishCountRow {
    menu_id: i32,
}

/// Veritabanındaki tüm alias ve menüleri tarayarak:
/// 1. ContentGuard::is_junk_dish_text ile eşleşen çöp alias'ları tespit eder.
/// 2. Bu alias'lara bağlı olan veya toplam geçerli yemek sayısı 2'den az olan menüleri 'rejected' yapar.
/// 3. Çöp yemek bağlantılarını kaldırır ve yetim kalan çöp alias kayıtlarını siler.
pub async fn sanitize_and_repair_database(db: &DatabaseConnection) -> Result<SanitizerReport> {
    let mut report = SanitizerReport::default();

    // 1. Tüm dish_aliases kayıtlarını çek ve is_junk_dish_text ile eşleşenleri bul
    let all_aliases = dish_aliases::Entity::find().all(db).await?;
    let junk_alias_ids: Vec<i32> = all_aliases
        .into_iter()
        .filter(|a| ContentGuard::is_junk_dish_text(&a.name))
        .map(|a| a.id)
        .collect();

    report.junk_aliases_found = junk_alias_ids.len();

    let mut menus_to_reject = std::collections::HashSet::new();

    // Çöp alias içeren menüleri bul
    if !junk_alias_ids.is_empty() {
        let linked_menu_dishes = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::DishAliasId.is_in(junk_alias_ids.clone()))
            .all(db)
            .await?;

        for md in linked_menu_dishes {
            menus_to_reject.insert(md.menu_id);
        }
    }

    // 2. Ayrıca toplam yemek sayısı < 2 olan tekil çöp menüleri bul
    let low_dish_menus = MenuDishCountRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT m.id AS menu_id
        FROM menus m
        LEFT JOIN menu_dishes md ON md.menu_id = m.id
        WHERE m.status != 'rejected'
        GROUP BY m.id
        HAVING COUNT(md.id) < 2;
        "#,
        vec![],
    ))
    .all(db)
    .await?;

    for row in low_dish_menus {
        menus_to_reject.insert(row.menu_id);
    }

    // 3. Tespit edilen menüleri 'rejected' yap ve not ekle
    if !menus_to_reject.is_empty() {
        let menu_ids: Vec<i32> = menus_to_reject.into_iter().collect();
        let target_menus = menus::Entity::find()
            .filter(menus::Column::Id.is_in(menu_ids))
            .filter(menus::Column::Status.ne(MenuStatusEnum::Rejected))
            .all(db)
            .await?;

        for m in target_menus {
            let cur_notice = m.notice.clone();
            let mut active: menus::ActiveModel = m.into();
            active.status = Set(MenuStatusEnum::Rejected);
            let new_notice = match cur_notice {
                Some(existing) if !existing.is_empty() => {
                    format!(
                        "{} | Otomatik sistem temizliği: Geçersiz içerik tespit edildi",
                        existing
                    )
                }
                _ => "Otomatik sistem temizliği: Geçersiz içerik tespit edildi".to_string(),
            };
            active.notice = Set(Some(new_notice));
            active.update(db).await?;
            report.menus_rejected += 1;
        }
    }

    // 4. Çöp alias'lara bağlı menu_dishes satırlarını kaldır
    if !junk_alias_ids.is_empty() {
        let delete_res = menu_dishes::Entity::delete_many()
            .filter(menu_dishes::Column::DishAliasId.is_in(junk_alias_ids.clone()))
            .exec(db)
            .await?;
        report.menu_dishes_unlinked = delete_res.rows_affected as usize;

        // Yetim kalan çöp alias kayıtlarını sil
        let delete_alias_res = dish_aliases::Entity::delete_many()
            .filter(dish_aliases::Column::Id.is_in(junk_alias_ids))
            .exec(db)
            .await?;
        report.orphaned_aliases_deleted = delete_alias_res.rows_affected as usize;
    }

    if report.menus_rejected > 0 || report.orphaned_aliases_deleted > 0 {
        tracing::info!(
            "[SELF-HEAL] Otomatik temizleme tamamlandı: {} menü reddedildi, {} çöp yemek bağı çözüldü, {} çöp alias silindi.",
            report.menus_rejected,
            report.menu_dishes_unlinked,
            report.orphaned_aliases_deleted
        );
    } else {
        tracing::debug!("[SELF-HEAL] Sistem temiz: herhangi bir çöp veya bozuk kayıt bulunamadı.");
    }

    Ok(report)
}

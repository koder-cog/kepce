use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use shared::entities::{cities, menu_dishes, menus, sea_orm_active_enums::MealTypeEnum};
use std::collections::HashMap;

/// Mevcut veritabanındaki tüm menülerde yer alan placeholder Al-Götür paketlerini (örn. "Al-Götür Menü 7")
/// config/takeaway/{sehir}/{donem}/{ogun}.json şablonlarındaki gerçek yemek alternatifleriyle günceller.
pub async fn enrich_all_takeaways(db: &DatabaseConnection) -> Result<usize> {
    tracing::info!("Al-Götür menüleri şablonlardan taranıp zenginleştiriliyor...");

    // 1. Şehirleri slug eşlemesi için yükle
    let all_cities = cities::Entity::find().all(db).await?;
    let city_map: HashMap<i32, String> = all_cities.into_iter().map(|c| (c.id, c.slug)).collect();

    // 2. Al-Götür paketi barındıran menü_dishes kayıtlarını bul
    let takeaway_dishes = menu_dishes::Entity::find()
        .filter(menu_dishes::Column::PackageName.ne("NORMAL"))
        .filter(menu_dishes::Column::PackageName.not_like("%ÇÖLYAK%"))
        .filter(menu_dishes::Column::PackageName.not_like("%COLYAK%"))
        .all(db)
        .await?;

    if takeaway_dishes.is_empty() {
        tracing::info!("Veritabanında zenginleştirilecek Al-Götür paketi bulunamadı.");
        return Ok(0);
    }

    // Menü bazlı paket isimlerini grupla
    let mut menu_packages: HashMap<i32, Vec<String>> = HashMap::new();
    for d in takeaway_dishes {
        let pkgs = menu_packages.entry(d.menu_id).or_default();
        if !pkgs.contains(&d.package_name) {
            pkgs.push(d.package_name);
        }
    }

    let mut total_enriched = 0usize;
    let digit_regex = regex::Regex::new(r"\d+").unwrap();

    for (menu_id, pkg_names) in menu_packages {
        let menu_opt = menus::Entity::find_by_id(menu_id).one(db).await?;
        let menu = match menu_opt {
            Some(m) => m,
            None => continue,
        };

        let city_slug = match city_map.get(&menu.city_id) {
            Some(s) => s.as_str(),
            None => continue,
        };

        let meal_type_str = match menu.meal_type {
            MealTypeEnum::Breakfast => "breakfast",
            MealTypeEnum::Lunch => "lunch",
            MealTypeEnum::Dinner => "dinner",
        };

        // Şablon konfigürasyonunu al
        let config_map = match crate::parser::takeaway::get_takeaway_config(city_slug, meal_type_str) {
            Some(c) => c,
            None => continue,
        };

        for pkg_name in pkg_names {
            // Paket numarasını çıkar (örn: "Al Götür 7", "Al-Götür Menü 1" -> 7, 1)
            let pkg_id_opt = digit_regex
                .find(&pkg_name)
                .and_then(|m| m.as_str().parse::<u32>().ok());

            let pkg_id = match pkg_id_opt {
                Some(id) => id,
                None => continue,
            };

            let parsed_pkg = match config_map.get(&pkg_id) {
                Some(p) => p,
                None => continue,
            };

            // Eğer şablonda gerçek yemek slotları varsa, bu menünün o paketini güncelle
            if parsed_pkg.slots.is_empty() {
                continue;
            }

            let txn = db.begin().await?;

            // Eski placeholder satırları temizle
            menu_dishes::Entity::delete_many()
                .filter(menu_dishes::Column::MenuId.eq(menu_id))
                .filter(menu_dishes::Column::PackageName.eq(&pkg_name))
                .exec(&txn)
                .await?;

            // Şablondaki slot ve alternatifleri ekle
            for (slot_idx, slot) in parsed_pkg.slots.iter().enumerate() {
                for (alt_idx, alt) in slot.iter().enumerate() {
                    let alias_id = crate::tasks::scraper::get_or_create_dish_alias(&txn, &alt.name, None).await?;
                    let cals = alt.calories.as_ref().and_then(|c| {
                        c.chars()
                            .filter(|ch| ch.is_ascii_digit())
                            .collect::<String>()
                            .parse::<i32>()
                            .ok()
                    });
                    let link = menu_dishes::ActiveModel {
                        menu_id: Set(menu_id),
                        dish_alias_id: Set(alias_id),
                        order_index: Set(slot_idx as i32),
                        is_alternative: Set(alt_idx > 0),
                        package_name: Set(parsed_pkg.name.clone()),
                        amount: Set(alt.amount.clone()),
                        calories: Set(cals),
                        ..Default::default()
                    };
                    link.insert(&txn).await?;
                }
            }

            txn.commit().await?;

            if menu.status == shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved {
                let _ = shared::services::immutable_store::ImmutableStore::write_menu_hash(db, menu_id).await;
            }

            total_enriched += 1;
        }
    }

    tracing::info!("Al-Götür zenginleştirmesi tamamlandı: Toplam {} menü paketi güncellendi.", total_enriched);
    Ok(total_enriched)
}

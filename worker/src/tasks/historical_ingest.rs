use anyhow::Result;
use chrono::NaiveDate;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use shared::entities::{cities, sea_orm_active_enums::MealTypeEnum};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct HistoricalDishItem {
    name: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    calories: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HistoricalMenuRecord {
    #[serde(default)]
    city_plate: Option<i32>,
    #[serde(default)]
    city_name: Option<String>,
    date: String,
    meal_type: String,
    dishes: Vec<HistoricalDishItem>,
    #[serde(default)]
    calorie_info: Option<String>,
    #[serde(default)]
    primary_source: Option<String>,
}

/// Ingest unified historical menus JSON using native Worker parsing & upsert pipeline.
pub async fn ingest_historical_menus(db: &DatabaseConnection, file_path: &str) -> Result<()> {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        tracing::warn!("Tarihsel menü arşivi bulunamadı: {}", file_path);
        return Ok(());
    }

    tracing::info!("Tarihsel menü arşivi okunuyor: {}", file_path);
    let content = tokio::fs::read_to_string(&path).await?;
    let records: Vec<HistoricalMenuRecord> = serde_json::from_str(&content)?;
    tracing::info!("Toplam {} adet menü kaydı Worker motoruna alınıyor...", records.len());

    let all_cities = cities::Entity::find().all(db).await?;
    let mut city_by_plate = std::collections::HashMap::new();
    let mut city_by_slug = std::collections::HashMap::new();

    for c in all_cities {
        city_by_plate.insert(c.id, (c.id, c.slug.clone()));
        city_by_slug.insert(c.slug.clone(), c.id);
    }

    let mut total = 0usize;

    for rec in records {
        let city_id = if let Some(plate) = rec.city_plate {
            city_by_plate.get(&plate).map(|(id, _)| *id)
        } else if let Some(ref name) = rec.city_name {
            let slug = name.to_lowercase()
                .replace('ı', "i")
                .replace('ğ', "g")
                .replace('ü', "u")
                .replace('ş', "s")
                .replace('ö', "o")
                .replace('ç', "c");
            city_by_slug.get(&slug).copied()
        } else {
            None
        };

        let city_id = match city_id {
            Some(id) => id,
            None => continue,
        };

        let date = match NaiveDate::parse_from_str(&rec.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let meal_type = match rec.meal_type.as_str() {
            "dinner" => MealTypeEnum::Dinner,
            "breakfast" => MealTypeEnum::Breakfast,
            "lunch" => MealTypeEnum::Lunch,
            _ => continue,
        };

        let raw_src = rec.primary_source.as_deref().unwrap_or("unknown");
        let source_type = if raw_src.contains("kykmenu") {
            "kykmenu".to_string()
        } else if raw_src.contains("yurtmenu") {
            "yurtmenu".to_string()
        } else if raw_src.contains("kykyemek") {
            "kykyemek".to_string()
        } else {
            "unknown".to_string()
        };

        let mut dishes: Vec<Vec<crate::parser::models::MenuComponent>> = Vec::new();
        let mut seen_dishes = std::collections::HashSet::new();

        for d in rec.dishes {
            let raw_name = d.name.trim();
            if raw_name.is_empty() {
                continue;
            }

            // Worker'ın kendi parser/sanitizer motoruyla ayrıştırma
            let alts = crate::parser::kykyemek::clean_and_split_dish(raw_name.to_string());
            let mut valid_alts = Vec::new();

            for alt in alts {
                let normalized_name = crate::parser::normalizer::normalize_food_name(&alt.name);
                if !normalized_name.is_empty() && seen_dishes.insert(normalized_name.clone()) {
                    valid_alts.push(crate::parser::models::MenuComponent {
                        name: normalized_name,
                        amount: alt.amount,
                        calories: alt.calories.or(d.calories.clone()),
                        category: alt.category.or(d.category.clone()),
                    });
                }
            }

            if !valid_alts.is_empty() {
                dishes.push(valid_alts);
            }
        }

        if dishes.is_empty() {
            continue;
        }

        // Kalori parse
        let (min_cal, max_cal) = if let Some(ref cal_str) = rec.calorie_info {
            let cleaned = cal_str
                .to_lowercase()
                .replace("kcal", "")
                .replace("kkal", "")
                .replace("kalori", "")
                .trim()
                .to_string();
            let parts: Vec<&str> = cleaned.split('-').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                (parts[0].parse::<i32>().ok(), parts[1].parse::<i32>().ok())
            } else if parts.len() == 1 {
                let val = parts[0].parse::<i32>().ok();
                (val, val)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        crate::tasks::scraper::upsert_menu(
            db,
            city_id,
            date,
            meal_type,
            source_type,
            None,
            dishes,
            vec![],
            vec![],
            None,
            min_cal,
            max_cal,
        )
        .await?;

        total += 1;
        if total.is_multiple_of(1000) {
            tracing::info!("İlerleyiş: {} menü Worker ile başarıyla işlendi...", total);
        }
    }

    tracing::info!("Tarihsel menü ingest tamamlandı. Toplam {} menü güncellendi.", total);
    Ok(())
}

use anyhow::Result;
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::entities::{cities, sea_orm_active_enums::MealTypeEnum};
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BackupMenuRecord {
    date: String,
    items: Vec<String>,
}

/// Ingest the `kykyemek-şmnmh-yedek/` backup JSON menus into the DB.
///
/// Idempotent (upsert via [`crate::tasks::scraper::upsert_menu`]). Each city
/// folder name is the slug; the meal type is parsed from the filename token
/// (`false` -> breakfast, `true` -> dinner). Unmatched city slugs are collected
/// and reported — never silently skipped.
pub async fn ingest_backup_menus(db: &DatabaseConnection, base_dir: &str) -> Result<()> {
    let base = PathBuf::from(base_dir);
    if !base.exists() {
        tracing::warn!("Backup menü dizini bulunamadı: {}", base_dir);
        return Ok(());
    }

    let mut total = 0usize;
    let mut unmatched: Vec<String> = Vec::new();

    let mut city_iter = tokio::fs::read_dir(&base).await?;
    while let Ok(Some(city_entry)) = city_iter.next_entry().await {
        let city_path = city_entry.path();
        if !city_path.is_dir() {
            continue;
        }
        let city_slug = city_entry.file_name().to_string_lossy().to_lowercase();

        let city_opt = cities::Entity::find()
            .filter(cities::Column::Slug.eq(&city_slug))
            .one(db)
            .await?;

        let city_id = match city_opt {
            Some(c) => c.id,
            None => {
                tracing::warn!("Backup şehir '{}' veritabanında bulunamadı, atlanıyor.", city_slug);
                unmatched.push(city_slug);
                continue;
            }
        };

        let mut file_iter = tokio::fs::read_dir(&city_path).await?;
        while let Ok(Some(file_entry)) = file_iter.next_entry().await {
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

            // Filename format: {City}_{meal}_{shift}_{date}.json  -> token[1] = meal
            let meal_type = match filename.split('_').nth(1) {
                Some("true") => MealTypeEnum::Dinner,
                Some("false") => MealTypeEnum::Breakfast,
                Some("lunch") => MealTypeEnum::Lunch,
                _ => {
                    tracing::warn!("Bilinmeyen öğün token'ı, atlanıyor: {}", filename);
                    continue;
                }
            };

            let content = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Dosya okunamadı {}: {:?}", filename, e);
                    continue;
                }
            };

            let records: Vec<BackupMenuRecord> = match serde_json::from_str(&content) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("JSON ayrıştırma hatası {}: {:?}", filename, e);
                    continue;
                }
            };

            for rec in records {
                let date: NaiveDate = match crate::parser::kykyemek::parse_turkish_date(&rec.date) {
                    Some(d) => d,
                    None => {
                        tracing::warn!("Tarih ayrıştırılamadı: {}", rec.date);
                        continue;
                    }
                };

                let mut dishes: Vec<Vec<crate::parser::models::MenuComponent>> = Vec::new();
                for item in rec.items {
                    let lower = item.to_lowercase();
                    if lower.contains("al götür") || lower.contains("al-götür") {
                        continue; // package marker, no sub-items in this JSON format
                    }
                    
                    let alts = crate::parser::kykyemek::clean_and_split_dish(item);
                        
                    if !alts.is_empty() {
                        dishes.push(alts);
                    }
                }
                if dishes.is_empty() {
                    continue;
                }

                crate::tasks::scraper::upsert_menu(
                    db,
                    city_id,
                    date,
                    meal_type.clone(),
                    "kepce-admin".to_string(), // Keep as admin since it's a manual backup restore
                    None,
                    dishes,
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                )
                .await?;
                total += 1;
            }
        }
    }

    if !unmatched.is_empty() {
        tracing::error!("Eşleşmeyen şehir klasörleri (atlandı): {:?}", unmatched);
    }

    tracing::info!("Backup menü ingest tamamlandı. {} menü işlendi.", total);
    Ok(())
}

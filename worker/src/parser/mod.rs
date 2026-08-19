pub mod anomaly;
pub mod core;
pub mod dictionary;
pub mod excel;
pub mod kykyemek;
pub mod llm;
pub mod models;
pub mod normalizer;
pub mod takeaway;
pub mod validation;

use sea_orm::DatabaseConnection;
use models::MenuDatabase;
use shared::entities::sea_orm_active_enums::MenuStatusEnum;
use chrono::NaiveDate;
use anyhow::Result;

pub async fn save_menu_database(
    db: &DatabaseConnection,
    city_id: i32,
    source_type: &str, // e.g. "kykyemek", "kepce-admin"
    menu_db: MenuDatabase,
    city_slug: &str,
) -> Result<()> {
    for (date_str, day_data) in menu_db {
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;
        
        let target_status = match day_data.metadata.as_ref().map(|m| m.status.as_str()) {
            Some("needs_review") => Some(MenuStatusEnum::Pending),
            Some("approved") => Some(MenuStatusEnum::Approved),
            _ => None, // Fallback to scraper.rs default logic
        };

        // Convert day_data lists to Vec<Vec<MenuComponent>>
        let mut normal_breakfast: Vec<Vec<models::MenuComponent>> = Vec::new();
        let mut normal_lunch: Vec<Vec<models::MenuComponent>> = Vec::new();
        let mut normal_dinner: Vec<Vec<models::MenuComponent>> = Vec::new();
        let mut colyak_breakfast: Vec<Vec<models::MenuComponent>> = Vec::new();
        let mut colyak_lunch: Vec<Vec<models::MenuComponent>> = Vec::new();
        let mut colyak_dinner: Vec<Vec<models::MenuComponent>> = Vec::new();
        
        let mut takeaways_breakfast: Vec<(String, Vec<Vec<models::MenuComponent>>)> = Vec::new();
        let mut takeaways_lunch: Vec<(String, Vec<Vec<models::MenuComponent>>)> = Vec::new();
        let mut takeaways_dinner: Vec<(String, Vec<Vec<models::MenuComponent>>)> = Vec::new();

        type ProcessListResult = (Vec<Vec<models::MenuComponent>>, Vec<(String, Vec<Vec<models::MenuComponent>>)>);
        let process_list = |list: &Vec<models::MenuItem>, is_breakfast: bool| -> ProcessListResult {
            let mut out = Vec::new();
            let mut t_out: Vec<(String, Vec<Vec<models::MenuComponent>>)> = Vec::new();
            
            for item in list {
                if let Some(ref num) = item.takeaway_id {
                    let meal_str = if is_breakfast { "breakfast" } else { "dinner" };
                    if let Some(parsed_packages) = takeaway::parse_takeaway_menu(&format!("Al Götür {}", num), city_slug, meal_str) {
                        t_out.extend(parsed_packages);
                    }
                } else {
                    let alts = item.alternatives.clone();
                    if !alts.is_empty() {
                        out.push(alts);
                    }
                }
            }
            (out, t_out)
        };

        let (b_out, b_take) = process_list(&day_data.normal.breakfast, true);
        normal_breakfast.extend(b_out);
        takeaways_breakfast.extend(b_take);

        let (l_out, l_take) = process_list(&day_data.normal.lunch, false);
        normal_lunch.extend(l_out);
        takeaways_lunch.extend(l_take);

        let (d_out, d_take) = process_list(&day_data.normal.dinner, false);
        normal_dinner.extend(d_out);
        takeaways_dinner.extend(d_take);
        
        let (cb_out, _) = process_list(&day_data.colyak.breakfast, true);
        colyak_breakfast.extend(cb_out);

        let (cl_out, _) = process_list(&day_data.colyak.lunch, false);
        colyak_lunch.extend(cl_out);
        
        let (cd_out, _) = process_list(&day_data.colyak.dinner, false);
        colyak_dinner.extend(cd_out);

        takeaways_breakfast.dedup_by(|a, b| a.0 == b.0);
        takeaways_lunch.dedup_by(|a, b| a.0 == b.0);
        takeaways_dinner.dedup_by(|a, b| a.0 == b.0);

        let source_type_str = source_type.to_string();

        let parse_calories = |kcal_str: &Option<String>| -> (Option<i32>, Option<i32>) {
            if let Some(kcal) = kcal_str {
                let cleaned = kcal
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
            }
        };

        if !normal_breakfast.is_empty() || !takeaways_breakfast.is_empty() || !colyak_breakfast.is_empty() {
            let (min_cal, max_cal) = parse_calories(&day_data.normal.breakfast_kcal);
            crate::tasks::scraper::upsert_menu(
                db,
                city_id,
                date,
                shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast,
                source_type_str.clone(),
                None,
                normal_breakfast,
                colyak_breakfast,
                takeaways_breakfast,
                target_status.clone(),
                min_cal,
                max_cal,
            ).await?;
        }

        if !normal_lunch.is_empty() || !takeaways_lunch.is_empty() || !colyak_lunch.is_empty() {
            let (min_cal, max_cal) = parse_calories(&day_data.normal.lunch_kcal);
            crate::tasks::scraper::upsert_menu(
                db,
                city_id,
                date,
                shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch,
                source_type_str.clone(),
                None,
                normal_lunch,
                colyak_lunch,
                takeaways_lunch,
                target_status.clone(),
                min_cal,
                max_cal,
            ).await?;
        }

        if !normal_dinner.is_empty() || !takeaways_dinner.is_empty() || !colyak_dinner.is_empty() {
            let (min_cal, max_cal) = parse_calories(&day_data.normal.dinner_kcal);
            crate::tasks::scraper::upsert_menu(
                db,
                city_id,
                date,
                shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner,
                source_type_str.clone(),
                None,
                normal_dinner,
                colyak_dinner,
                takeaways_dinner,
                target_status.clone(),
                min_cal,
                max_cal,
            ).await?;
        }
        
        // Also call for colyak if available (currently upsert_menu doesn't cleanly separate colyak unless through meal type or new column, 
        // but for now we follow the structure of existing upsert_menu).
    }
    
    Ok(())
}

#[cfg(test)]
mod tests;

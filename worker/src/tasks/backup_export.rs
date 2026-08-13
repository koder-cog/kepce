use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, PaginatorTrait};
use shared::entities::{cities, menu_dishes, menus, dish_aliases, sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum}};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
struct ExportMenuRecord {
    date: String,
    items: Vec<String>,
    cal: String,
    likes: String,
    dislikes: String,
    comments_count: String,
    menu_id: String,
    vote_id: String,
    city: Option<String>,
}

pub async fn export_backup_menus(db: &DatabaseConnection, output_dir: &str) -> Result<()> {
    fs::create_dir_all(output_dir).context("Backup klasörü oluşturulamadı")?;

    let all_menus = menus::Entity::find()
        .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
        .order_by_asc(menus::Column::ServeDate)
        .all(db)
        .await?;

    use sea_orm::LoaderTrait;
    let all_menu_dishes = all_menus.load_many(menu_dishes::Entity, db).await?;
    let all_aliases = dish_aliases::Entity::find().all(db).await?;
    let alias_map: HashMap<i32, String> = all_aliases.into_iter().map(|a| (a.id, a.name)).collect();

    let all_cities = cities::Entity::find().all(db).await?;
    let city_map: HashMap<i32, String> = all_cities.into_iter().map(|c| (c.id, c.slug)).collect();

    // city_slug -> meal_type -> year_month -> Vec<ExportMenuRecord>
    let mut export_data: HashMap<String, HashMap<String, HashMap<String, Vec<ExportMenuRecord>>>> = HashMap::new();

    for (menu, dishes_for_menu) in all_menus.into_iter().zip(all_menu_dishes) {
        let city_slug = match city_map.get(&menu.city_id) {
            Some(slug) => slug.clone(),
            None => continue,
        };

        let meal_type_str = match menu.meal_type {
            MealTypeEnum::Breakfast => "false".to_string(),
            MealTypeEnum::Dinner => "true".to_string(),
            MealTypeEnum::Lunch => "lunch".to_string(),
        };

        let date = menu.serve_date;
        let year_month = format!("{:04}_{:02}", date.year(), date.month());
        let date_str = format_turkish_date(date);

        let mut sorted_dishes = dishes_for_menu;
        sorted_dishes.sort_by_key(|md| md.order_index);

        let mut items = Vec::new();
        let mut total_cal = 0;
        for md in &sorted_dishes {
            if let Some(name) = alias_map.get(&md.dish_alias_id) {
                items.push(name.clone());
            }
            if !md.package_name.is_empty() {
                items.push(format!("({} paketi)", md.package_name));
            }
            total_cal += md.calories.unwrap_or(0);
        }

        let cal_str = if total_cal > 0 {
            format!("{} kalori", total_cal)
        } else {
            "650-850 kalori".to_string()
        };

        use shared::entities::{menu_votes, comments};
        use shared::entities::sea_orm_active_enums::SentimentEnum;

        let votes = menu_votes::Entity::find()
            .filter(menu_votes::Column::MenuId.eq(menu.id))
            .all(db)
            .await?;
        let likes_count = votes.iter().filter(|v| v.sentiment == SentimentEnum::Positive).count();
        let dislikes_count = votes.iter().filter(|v| v.sentiment == SentimentEnum::Negative).count();

        let c_count = comments::Entity::find()
            .filter(comments::Column::MenuId.eq(menu.id))
            .filter(comments::Column::IsDeleted.eq(false))
            .count(db)
            .await?;

        let record = ExportMenuRecord {
            date: date_str,
            items,
            cal: cal_str,
            likes: likes_count.to_string(),
            dislikes: dislikes_count.to_string(),
            comments_count: c_count.to_string(),
            menu_id: menu.id.to_string(),
            vote_id: menu.id.to_string(),
            city: Some(city_slug.clone()),
        };

        export_data
            .entry(city_slug)
            .or_default()
            .entry(meal_type_str)
            .or_default()
            .entry(year_month)
            .or_default()
            .push(record);
    }

    // Write to files
    for (city, meals) in export_data {
        let city_dir = Path::new(output_dir).join(&city);
        fs::create_dir_all(&city_dir).context(format!("Şehir klasörü oluşturulamadı: {}", city))?;

        for (meal, months) in meals {
            for (month, records) in months {
                let filename = format!("{}_{}_{}.json", city, meal, month);
                let filepath = city_dir.join(filename);

                let json = serde_json::to_string_pretty(&records)?;
                fs::write(&filepath, json).context(format!("Dosya yazılamadı: {:?}", filepath))?;
            }
        }
    }

    Ok(())
}

fn format_turkish_date(date: NaiveDate) -> String {
    let day = date.day();
    let year = date.year();
    
    let month_str = match date.month() {
        1 => "Ocak",
        2 => "Şubat",
        3 => "Mart",
        4 => "Nisan",
        5 => "Mayıs",
        6 => "Haziran",
        7 => "Temmuz",
        8 => "Ağustos",
        9 => "Eylül",
        10 => "Ekim",
        11 => "Kasım",
        12 => "Aralık",
        _ => "",
    };

    let day_str = match date.weekday() {
        chrono::Weekday::Mon => "Pazartesi",
        chrono::Weekday::Tue => "Salı",
        chrono::Weekday::Wed => "Çarşamba",
        chrono::Weekday::Thu => "Perşembe",
        chrono::Weekday::Fri => "Cuma",
        chrono::Weekday::Sat => "Cumartesi",
        chrono::Weekday::Sun => "Pazar",
    };

    format!("{} {} {} {}", day, month_str, year, day_str)
}

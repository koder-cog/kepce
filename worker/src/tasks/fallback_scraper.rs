use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use reqwest::Client;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::entities::{
    cities,
    sea_orm_active_enums::MealTypeEnum,
};
use std::sync::OnceLock;

use crate::parser::models::MenuComponent;
use crate::tasks::scraper::upsert_menu;

/// Fallback kaynak hiyerarşisi (düşükten yükseğe güven):
///   kykyemek.com (6) > kykmenum.com / yurtmenu.net (5) > kykmenu.com.tr (4)
///
/// Bu task SADECE birincil kaynakta (kykyemek) o şehir/o gün için kayıt
/// bulunamadığında devreye girer ("boşa debelenme"yi önler). Kayıt zaten
/// varsa hiçbir HTTP isteği yapılmaz.
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

struct DayGaps {
    breakfast: bool,
    dinner: bool,
}

pub async fn run_fallback_scrape(
    db: &DatabaseConnection,
    client: &Client,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<usize> {
    let active_cities = [
        "istanbul", "ankara", "izmir", "antalya", "canakkale", "erzurum",
        "eskisehir", "gaziantep", "isparta", "kahramanmaras", "karabuk",
        "kirklareli", "konya", "sakarya", "sivas", "trabzon",
    ];

    let now = chrono::Local::now().naive_local().date();
    let days_in_month = match now.month() {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if now.year() % 4 == 0 && (now.year() % 100 != 0 || now.year() % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    let mut total_saved = 0usize;

    for slug in active_cities {
        if *shutdown_rx.borrow() {
            return Ok(total_saved);
        }

        let Some(city) = cities::Entity::find()
            .filter(cities::Column::Slug.eq(slug))
            .one(db)
            .await?
        else {
            continue;
        };

        for day in 1..=days_in_month {
            if *shutdown_rx.borrow() {
                return Ok(total_saved);
            }

            let date = NaiveDate::from_ymd_opt(now.year(), now.month(), day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap());

            // DB'de bu günün eksik öğünlerini tespit et
            let existing = menus_missing_check(db, city.id, date).await?;
            let gaps = DayGaps {
                breakfast: !existing.0,
                dinner: !existing.1,
            };
            if !gaps.breakfast && !gaps.dinner {
                continue; // Tam dolu -> hiç istek atma
            }

            let saved = fill_day_from_fallbacks(db, client, &city.slug, city.id, date, &gaps, &shutdown_rx).await?;
            total_saved += saved;

            // Kaynak sunucularına nazik ol
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    tracing::info!("[FALLBACK] Alternatif kaynak taraması tamamlandı: {} menü kaydedildi.", total_saved);
    Ok(total_saved)
}

/// (breakfast_var_mi, dinner_var_mi)
async fn menus_missing_check(
    db: &DatabaseConnection,
    city_id: i32,
    date: NaiveDate,
) -> Result<(bool, bool)> {
    let rows = shared::entities::menus::Entity::find()
        .filter(shared::entities::menus::Column::CityId.eq(city_id))
        .filter(shared::entities::menus::Column::ServeDate.eq(date))
        .all(db)
        .await?;

    let has_breakfast = rows.iter().any(|m| m.meal_type == MealTypeEnum::Breakfast);
    let has_dinner = rows.iter().any(|m| m.meal_type == MealTypeEnum::Dinner);
    Ok((has_breakfast, has_dinner))
}

/// Eksik öğünleri sırayla fallback kaynaklardan doldurmaya çalışır.
/// Bir kaynak iki öğünü de tamamlarsa sonraki kaynaklara hiç gidilmez.
async fn fill_day_from_fallbacks(
    db: &DatabaseConnection,
    client: &Client,
    slug: &str,
    city_id: i32,
    date: NaiveDate,
    gaps: &DayGaps,
    shutdown_rx: &tokio::sync::watch::Receiver<bool>,
) -> Result<usize> {
    let mut need_breakfast = gaps.breakfast;
    let mut need_dinner = gaps.dinner;
    let mut saved = 0usize;

    // --- 1) kykmenum.com (JSON-LD Menu) ---
    if need_breakfast || need_dinner {
        if *shutdown_rx.borrow() {
            return Ok(saved);
        }
        let url = format!("https://kykmenum.com/{}/{}", slug, date.format("%Y-%m-%d"));
        if let Ok(res) = client
            .get(&url)
            .header("User-Agent", UA)
            .timeout(std::time::Duration::from_secs(20))
            .send()
            .await
        {
            if let Ok(html) = res.text().await {
                if let Some(menu) = crate::parser::kykmenum::parse_kykmenum_html(&html) {
                    if need_breakfast {
                        if let Some(dishes) = menu.breakfast {
                            upsert_menu(db, city_id, date, MealTypeEnum::Breakfast, "kykmenum.com".to_string(), None, dishes, vec![], vec![], None, None, None).await?;
                            saved += 1;
                            need_breakfast = false;
                        }
                    }
                    if need_dinner {
                        if let Some(dishes) = menu.dinner {
                            upsert_menu(db, city_id, date, MealTypeEnum::Dinner, "kykmenum.com".to_string(), None, dishes, vec![], vec![], None, None, None).await?;
                            saved += 1;
                            need_dinner = false;
                        }
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    // --- 2) yurtmenu.net (SSR HTML kartları) ---
    if need_breakfast || need_dinner {
        if *shutdown_rx.borrow() {
            return Ok(saved);
        }
        let url = format!("https://yurtmenu.net/{}?date={}", slug, date.format("%Y-%m-%d"));
        if let Ok(res) = client
            .get(&url)
            .header("User-Agent", UA)
            .header("Referer", "https://yurtmenu.net/")
            .timeout(std::time::Duration::from_secs(25))
            .send()
            .await
        {
            if let Ok(html) = res.text().await {
                let menu = crate::parser::yurtmenu::parse_yurtmenu_html(&html);
                if need_breakfast {
                    if let Some(dishes) = menu.breakfast {
                        let (min, max) = parse_kcal_range(menu.breakfast_kcal.as_deref());
                        upsert_menu(db, city_id, date, MealTypeEnum::Breakfast, "yurtmenu.net".to_string(), None, dishes, vec![], vec![], None, min, max).await?;
                        saved += 1;
                        need_breakfast = false;
                    }
                }
                if need_dinner {
                    if let Some(dishes) = menu.dinner {
                        let (min, max) = parse_kcal_range(menu.dinner_kcal.as_deref());
                        upsert_menu(db, city_id, date, MealTypeEnum::Dinner, "yurtmenu.net".to_string(), None, dishes, vec![], vec![], None, min, max).await?;
                        saved += 1;
                        need_dinner = false;
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    // --- 3) kykmenu.com.tr (açık JSON API - snapshot bazlı olabilir) ---
    if need_breakfast || need_dinner {
        if *shutdown_rx.borrow() {
            return Ok(saved);
        }
        let url = format!(
            "https://kykmenu.com.tr/api/menu-view?city={}&date={}",
            slug,
            date.format("%Y-%m-%d")
        );
        if let Ok(res) = client
            .get(&url)
            .header("User-Agent", UA)
            .header("Referer", "https://kykmenu.com.tr/")
            .timeout(std::time::Duration::from_secs(20))
            .send()
            .await
        {
            if let Ok(body) = res.text().await {
                if let Some((breakfast, dinner)) = parse_kykmenu_api(&body) {
                    if need_breakfast {
                        if let Some(dishes) = breakfast {
                            upsert_menu(db, city_id, date, MealTypeEnum::Breakfast, "kykmenu.com.tr".to_string(), None, dishes, vec![], vec![], None, None, None).await?;
                            saved += 1;
                        }
                    }
                    if need_dinner {
                        if let Some(dishes) = dinner {
                            upsert_menu(db, city_id, date, MealTypeEnum::Dinner, "kykmenu.com.tr".to_string(), None, dishes, vec![], vec![], None, None, None).await?;
                            saved += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(saved)
}

/// Tek bir yemeğin bileşen listesi.
type DishComponents = Vec<MenuComponent>;
/// Bir öğündeki yemeklerin bileşen grupları.
type MealGroups = Vec<DishComponents>;
/// Ayrıştırılan öğün çifti: (kahvaltı, akşam).
type ParsedMeals = (Option<MealGroups>, Option<MealGroups>);

/// kykmenu.com.tr `/api/menu-view` yanıtını ayrıştırır.
/// Şema: {"menu": {"kahvalti": {"yemekler": [...], "kaloriler": {...}},
///                 "aksam":    {"yemekler": [...], "kaloriler": {...}}}}
fn parse_kykmenu_api(body: &str) -> Option<ParsedMeals> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let menu = value.get("menu")?;

    let parse_meal = |key: &str| -> Option<Vec<Vec<MenuComponent>>> {
        let meal = menu.get(key)?;
        let items = meal.get("yemekler")?.as_array()?;
        let calories = meal.get("kaloriler").and_then(|c| c.as_object());

        let mut groups = Vec::new();
        for item in items {
            let Some(name) = item.as_str() else { continue };
            let trimmed = name.trim();
            if is_kykmenu_junk(trimmed) {
                continue;
            }
            let kcal = calories
                .and_then(|c| c.get(trimmed))
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string());
            let group = vec![MenuComponent {
                name: trimmed.to_string(),
                amount: None,
                calories: kcal,
                category: None,
            }];
            groups.push(group);
        }
        if groups.is_empty() {
            None
        } else {
            Some(groups)
        }
    };

    let breakfast = parse_meal("kahvalti");
    let dinner = parse_meal("aksam");
    if breakfast.is_none() && dinner.is_none() {
        None
    } else {
        Some((breakfast, dinner))
    }
}

/// kykmenu.com.tr snapshot'larına sızmış navigasyon/başlık metinlerini eler.
fn is_kykmenu_junk(text: &str) -> bool {
    static JUNK_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = JUNK_RE.get_or_init(|| {
        regex::Regex::new(r"(?i)(yemek listesi|KYK Men|kahvalt.{0,3}ak.{0,3}am|^[-←→•*])").unwrap()
    });
    text.len() < 3 || shared::services::content_guard::ContentGuard::is_junk_dish_text(text) || re.is_match(text)
}

/// "650-850 kcal" -> (Some(650), Some(850))
fn parse_kcal_range(meta: Option<&str>) -> (Option<i32>, Option<i32>) {
    static NUM_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = NUM_RE.get_or_init(|| regex::Regex::new(r"\d+").unwrap());
    let Some(meta) = meta else {
        return (None, None);
    };
    let nums: Vec<i32> = re
        .find_iter(meta)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();
    match nums.as_slice() {
        [min, max] => (Some(*min), Some(*max)),
        [single] => (Some(*single), Some(*single)),
        _ => (None, None),
    }
}

// NOT: upsert_menu içindeki status mantığı "kykmenum.com" / "yurtmenu.net" /
// "kykmenu.com.tr" kaynaklarını otomatik Approved yapar (bkz. scraper.rs).

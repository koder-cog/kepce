use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};
use shared::entities::prelude::*;

#[derive(Debug, Deserialize, Clone)]
pub struct PriceInfo {
    pub amount: String,
    pub price: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MealPricing {
    #[serde(default)]
    pub items: HashMap<String, PriceInfo>,
    #[serde(default)]
    pub categories: HashMap<String, PriceInfo>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PeriodPricing {
    pub period_start: String,
    pub period_end: String,
    #[serde(default)]
    pub breakfast: Option<MealPricing>,
    #[serde(default)]
    pub lunch: Option<MealPricing>,
    #[serde(default)]
    pub dinner: Option<MealPricing>,
}

lazy_static! {
    /// Şehir adı → aktif dönem fiyatlandırması.
    /// `load_pricing_from_db()` çağrıldığında `pricing_periods` ve
    /// `meal_category_prices` tablolarından bugünün tarihine uyan aktif dönem yüklenir.
    static ref PRICING_CACHE: RwLock<HashMap<String, PeriodPricing>> = RwLock::new(HashMap::new());
}

/// Veritabanındaki `pricing_periods` ve `meal_category_prices` tablolarından aktif fiyatları yükler.
pub async fn load_pricing_from_db(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    let today = chrono::Local::now().date_naive();
    tracing::info!("Aktif fiyat dönemleri veritabanından yükleniyor ({:?})...", today);

    let periods = PricingPeriods::find().all(db).await?;
    let mut cache = HashMap::new();

    for period in periods {
        if today >= period.period_start && today <= period.period_end {
            let prices = period.find_related(MealCategoryPrices).all(db).await?;

            let mut breakfast = MealPricing::default();
            let mut lunch = MealPricing::default();
            let mut dinner = MealPricing::default();

            for p in prices {
                let parsed_price = p.price.to_string().parse::<f32>().unwrap_or(0.0);
                let info = PriceInfo {
                    amount: p.portion_amount.unwrap_or_else(|| "-".to_string()),
                    price: parsed_price,
                };

                let target_meal = match p.meal_type.as_str() {
                    "breakfast" => &mut breakfast,
                    "lunch" => &mut lunch,
                    "dinner" => &mut dinner,
                    _ => continue,
                };

                // Büyük harfli isim kategorilere/öğelere eklenir
                let key = p.category_name.to_uppercase();
                target_meal.categories.insert(key.clone(), info.clone());
                target_meal.items.insert(key, info);
            }

            let period_pricing = PeriodPricing {
                period_start: period.period_start.to_string(),
                period_end: period.period_end.to_string(),
                breakfast: Some(breakfast),
                lunch: Some(lunch),
                dinner: Some(dinner),
            };

            tracing::info!(
                "Aktif fiyat dönemi veritabanından yüklendi: {} ({} → {})",
                period.city_slug, period_pricing.period_start, period_pricing.period_end
            );

            cache.insert(period.city_slug.clone(), period_pricing);
        }
    }

    if cache.is_empty() {
        tracing::warn!("Bugüne ait aktif veritabanı fiyat dönemi bulunamadı.");
    }

    if let Ok(mut c) = PRICING_CACHE.write() {
        *c = cache;
    }

    Ok(())
}

/// Belirtilen şehir, tarih ve öğün tipi için fiyat bilgisini döndürür.
/// Temmuz ve Ağustos aylarında (nöbetçi yurt / dönem dışı) veya şehir için tanımlı fiyat yoksa `None` döner.
pub fn get_pricing_info_for_city(
    city: &str,
    serve_date: Option<chrono::NaiveDate>,
    meal_type: &str,
    category: Option<&str>,
    name: &str,
) -> Option<PriceInfo> {
    use chrono::Datelike;

    if let Some(date) = serve_date {
        let month = date.month();
        if month == 7 || month == 8 {
            return None; // Temmuz ve Ağustos tatil/nöbetçi yurt dönemidir, fiyat gösterilmez
        }
    }

    if let Ok(cache) = PRICING_CACHE.read() {
        if let Some(pricing) = cache.get(city) {
            let meal_pricing = match meal_type {
                "breakfast" => pricing.breakfast.as_ref(),
                "lunch" => pricing.lunch.as_ref(),
                "dinner" => pricing.dinner.as_ref(),
                _ => None,
            };

            if let Some(mp) = meal_pricing {
                let upper_name = name.to_uppercase();
                // 1. Try exact item match
                if let Some(info) = mp.items.get(&upper_name) {
                    return Some(info.clone());
                }
                
                // 2. Try explicit category match
                if let Some(cat) = category {
                    let upper_cat = cat.to_uppercase();
                    if let Some(info) = mp.categories.get(&upper_cat) {
                        return Some(info.clone());
                    }
                }

                // 3. Fallback to dynamic rule-based categorizer
                if let Some(detected_cat) = shared::services::categorizer::categorize_dish(name) {
                    let upper_detected = detected_cat.to_uppercase();
                    if let Some(info) = mp.categories.get(&upper_detected) {
                        return Some(info.clone());
                    }
                }
            }
        }
    }
    None
}

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

/// Belirtilen şehir ve öğün tipi için fiyat bilgisini döndürür.
/// `city` verilmezse varsayılan olarak `"istanbul"` kullanılır.
pub fn get_pricing_info(meal_type: &str, category: Option<&str>, name: &str) -> Option<PriceInfo> {
    get_pricing_info_for_city("istanbul", meal_type, category, name)
}

pub fn get_pricing_info_for_city(city: &str, meal_type: &str, category: Option<&str>, name: &str) -> Option<PriceInfo> {
    if let Ok(cache) = PRICING_CACHE.read() {
        if let Some(pricing) = cache.get(city) {
            let meal_pricing = match meal_type {
                "breakfast" => pricing.breakfast.as_ref(),
                "lunch" => pricing.lunch.as_ref(),
                "dinner" => pricing.dinner.as_ref(),
                _ => None,
            };

            if let Some(mp) = meal_pricing {
                // Try items first (exact match)
                if let Some(info) = mp.items.get(&name.to_uppercase()) {
                    return Some(info.clone());
                }
                
                // Try categories
                if let Some(cat) = category {
                    if let Some(info) = mp.categories.get(&cat.to_uppercase()) {
                        return Some(info.clone());
                    }
                }
            }
        }
    }
    None
}

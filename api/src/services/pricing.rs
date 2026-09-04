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
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    #[serde(default)]
    pub breakfast: Option<MealPricing>,
    #[serde(default)]
    pub lunch: Option<MealPricing>,
    #[serde(default)]
    pub dinner: Option<MealPricing>,
}

lazy_static! {
    /// Şehir adı → o şehre ait tüm dönem fiyatlandırmaları (kronolojik sıralı).
    /// `load_pricing_from_db()` çağrıldığında `pricing_periods` ve
    /// `meal_category_prices` tablolarındaki tüm dönemler hafızaya alınır.
    static ref PRICING_CACHE: RwLock<HashMap<String, Vec<PeriodPricing>>> = RwLock::new(HashMap::new());
}

/// Veritabanındaki `pricing_periods` ve `meal_category_prices` tablolarındaki tüm fiyat dönemlerini yükler.
pub async fn load_pricing_from_db(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::info!("Tüm fiyat dönemleri veritabanından yükleniyor...");

    let periods = PricingPeriods::find().all(db).await?;
    let mut cache: HashMap<String, Vec<PeriodPricing>> = HashMap::new();

    for period in periods {
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
            period_start: period.period_start,
            period_end: period.period_end,
            breakfast: Some(breakfast),
            lunch: Some(lunch),
            dinner: Some(dinner),
        };

        tracing::info!(
            "Fiyat dönemi yüklendi: {} ({} → {})",
            period.city_slug, period_pricing.period_start, period_pricing.period_end
        );

        cache.entry(period.city_slug).or_default().push(period_pricing);
    }

    // Dönemleri başlangıç tarihine göre sırala
    for period_list in cache.values_mut() {
        period_list.sort_by_key(|p| p.period_start);
    }

    if cache.is_empty() {
        tracing::warn!("Veritabanında herhangi bir fiyat dönemi bulunamadı.");
    }

    if let Ok(mut c) = PRICING_CACHE.write() {
        *c = cache;
    }

    Ok(())
}

/// Belirtilen şehir, tarih ve öğün tipi için fiyat bilgisini döndürür.
/// Temmuz ve Ağustos aylarında (nöbetçi yurt / dönem dışı) `None` döner.
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

    let target_date = serve_date.unwrap_or_else(|| chrono::Local::now().date_naive());

    if let Ok(cache) = PRICING_CACHE.read() {
        // İlgili şehri bul; eğer o şehirde tanımlı fiyat yoksa 'istanbul' tarifesini fallback kullan
        let period_list = cache.get(city).or_else(|| cache.get("istanbul"));

        if let Some(periods) = period_list {
            // 1. Hedef tarihi kapsayan tam dönem
            // 2. Yoksa en son (veya en yakın) dönem
            let matched_pricing = periods
                .iter()
                .find(|p| target_date >= p.period_start && target_date <= p.period_end)
                .or_else(|| periods.last());

            if let Some(pricing) = matched_pricing {
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
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_pricing_date_matching_and_july_august_exclusion() {
        let mut cache = HashMap::new();
        let mut dinner = MealPricing::default();
        dinner.categories.insert(
            "ANA YEMEK".to_string(),
            PriceInfo {
                amount: "200 g".to_string(),
                price: 50.0,
            },
        );

        let p1 = PeriodPricing {
            period_start: NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
            dinner: Some(dinner.clone()),
            ..Default::default()
        };

        let mut dinner_new = MealPricing::default();
        dinner_new.categories.insert(
            "ANA YEMEK".to_string(),
            PriceInfo {
                amount: "200 g".to_string(),
                price: 65.0,
            },
        );

        let p2 = PeriodPricing {
            period_start: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2027, 8, 31).unwrap(),
            dinner: Some(dinner_new),
            ..Default::default()
        };

        cache.insert("istanbul".to_string(), vec![p1, p2]);

        if let Ok(mut c) = PRICING_CACHE.write() {
            *c = cache;
        }

        // Mayıs 2026 -> 2025-2026 tarifesi (50.0 TL)
        let may_price = get_pricing_info_for_city(
            "istanbul",
            Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
            "dinner",
            Some("ANA YEMEK"),
            "TAVUK SOTE",
        );
        assert!(may_price.is_some());
        assert_eq!(may_price.unwrap().price, 50.0);

        // Eylül 2026 -> 2026-2027 tarifesi (65.0 TL)
        let sept_price = get_pricing_info_for_city(
            "istanbul",
            Some(NaiveDate::from_ymd_opt(2026, 9, 15).unwrap()),
            "dinner",
            Some("ANA YEMEK"),
            "TAVUK SOTE",
        );
        assert!(sept_price.is_some());
        assert_eq!(sept_price.unwrap().price, 65.0);

        // Temmuz 2026 -> Nöbetçi yurt, gizli (None)
        let july_price = get_pricing_info_for_city(
            "istanbul",
            Some(NaiveDate::from_ymd_opt(2026, 7, 10).unwrap()),
            "dinner",
            Some("ANA YEMEK"),
            "TAVUK SOTE",
        );
        assert!(july_price.is_none());

        // Başka şehir (Ankara) -> İstanbul fallback
        let ankara_price = get_pricing_info_for_city(
            "ankara",
            Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
            "dinner",
            Some("ANA YEMEK"),
            "TAVUK SOTE",
        );
        assert!(ankara_price.is_some());
        assert_eq!(ankara_price.unwrap().price, 50.0);
    }
}


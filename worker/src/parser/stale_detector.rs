//! Blok/Dizi Tabanlı Bayat Veri Tespiti (Sequence Stale Cache Detector - Q2).
//!
//! Kaynakların ay başlarında web sitelerini güncellememesi ve bir önceki ayın
//! menüsünü güncel tarihler altında servis etmeye devam etmesi durumunu
//! ardışık gün karşılaştırmasıyla (>= 3 gün) tespit eder.

use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// Tespit edilen bayat veri dizisi detayları.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleSequenceMatch {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub matching_days_count: usize,
    pub sample_signatures: Vec<String>,
}

pub struct StaleSequenceDetector;

impl StaleSequenceDetector {
    /// Verilen yemek adları listesinden karşılaştırılabilir tekil bir imza üretir.
    pub fn compute_dish_signature(dishes: &[String]) -> String {
        let mut normalized: Vec<String> = dishes
            .iter()
            .map(|d| crate::parser::normalizer::normalize_food_name(d))
            .filter(|d| !d.is_empty())
            .collect();
        normalized.sort();
        normalized.join("|")
    }

    /// Yeni kazınan menü dizisi ile önceki ayın onaylı menü dizisini karşılaştırır.
    /// Eğer ardışık 3 veya daha fazla günün menüsü birebir örtüşüyorsa bayat veri alarmı üretir.
    pub fn detect_stale_sequence(
        incoming_days: &[(NaiveDate, Vec<String>)],
        previous_days: &[(NaiveDate, Vec<String>)],
    ) -> Option<StaleSequenceMatch> {
        if incoming_days.is_empty() || previous_days.is_empty() {
            return None;
        }

        // Önceki ayın günlerini gün-ay-numarasına (day of month) göre indeksle
        let mut prev_by_day_num: HashMap<u32, String> = HashMap::new();
        for (date, dishes) in previous_days {
            let sig = Self::compute_dish_signature(dishes);
            if !sig.is_empty() {
                prev_by_day_num.insert(date.day(), sig);
            }
        }

        // Gelen günleri tarihe göre sırala
        let mut sorted_incoming = incoming_days.to_vec();
        sorted_incoming.sort_by_key(|(d, _)| *d);

        let mut current_streak: Vec<(NaiveDate, String)> = Vec::new();
        let mut longest_streak: Vec<(NaiveDate, String)> = Vec::new();

        for (date, dishes) in &sorted_incoming {
            let incoming_sig = Self::compute_dish_signature(dishes);
            if incoming_sig.is_empty() {
                if current_streak.len() > longest_streak.len() {
                    longest_streak = current_streak.clone();
                }
                current_streak.clear();
                continue;
            }

            // Önceki aydaki aynı gün numarasına bak (örn: 1 Eylül vs 1 Ağustos)
            if let Some(prev_sig) = prev_by_day_num.get(&date.day()) {
                if prev_sig == &incoming_sig {
                    // Eşleşme var, seriye ekle
                    current_streak.push((*date, incoming_sig));
                    continue;
                }
            }

            // Eşleşme bozuldu
            if current_streak.len() > longest_streak.len() {
                longest_streak = current_streak.clone();
            }
            current_streak.clear();
        }

        if current_streak.len() > longest_streak.len() {
            longest_streak = current_streak;
        }

        // En az 3 ardışık gün kuralı
        if longest_streak.len() >= 3 {
            let start_date = longest_streak.first().map(|(d, _)| *d).unwrap();
            let end_date = longest_streak.last().map(|(d, _)| *d).unwrap();
            let matching_days_count = longest_streak.len();
            let sample_signatures = longest_streak
                .iter()
                .map(|(_, s)| s.clone())
                .take(3)
                .collect();

            Some(StaleSequenceMatch {
                start_date,
                end_date,
                matching_days_count,
                sample_signatures,
            })
        } else {
            None
        }
    }

    /// Belirtilen şehir ve öğün için veritabanındaki bir önceki ay verilerini çekerek
    /// gelen menü dizisinde bayat veri tekrarı olup olmadığını kontrol eder.
    pub async fn check_stale_for_city(
        db: &sea_orm::DatabaseConnection,
        city_id: i32,
        meal_type: shared::entities::sea_orm_active_enums::MealTypeEnum,
        incoming_days: &[(NaiveDate, Vec<String>)],
    ) -> anyhow::Result<Option<StaleSequenceMatch>> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
        use shared::entities::{dish_aliases, menu_dishes, menus};

        if incoming_days.is_empty() {
            return Ok(None);
        }

        let earliest = match incoming_days.iter().map(|(d, _)| *d).min() {
            Some(d) => d,
            None => return Ok(None),
        };

        // Bir önceki ayın başlangıç ve bitiş tarihleri
        let (prev_year, prev_month) = if earliest.month() == 1 {
            (earliest.year() - 1, 12)
        } else {
            (earliest.year(), earliest.month() - 1)
        };

        let prev_start = match NaiveDate::from_ymd_opt(prev_year, prev_month, 1) {
            Some(d) => d,
            None => return Ok(None),
        };

        let next_month_start = if prev_month == 12 {
            NaiveDate::from_ymd_opt(prev_year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(prev_year, prev_month + 1, 1)
        };

        let prev_end = match next_month_start.and_then(|d| d.pred_opt()) {
            Some(d) => d,
            None => return Ok(None),
        };

        let prev_menus = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::MealType.eq(meal_type))
            .filter(menus::Column::ServeDate.gte(prev_start))
            .filter(menus::Column::ServeDate.lte(prev_end))
            .order_by_asc(menus::Column::ServeDate)
            .all(db)
            .await?;

        if prev_menus.is_empty() {
            return Ok(None);
        }

        let mut previous_days = Vec::new();
        for m in prev_menus {
            let dishes = menu_dishes::Entity::find()
                .filter(menu_dishes::Column::MenuId.eq(m.id))
                .find_also_related(dish_aliases::Entity)
                .all(db)
                .await?;

            let dish_names: Vec<String> = dishes
                .into_iter()
                .filter_map(|(_, alias)| alias.map(|a| a.name))
                .collect();

            if !dish_names.is_empty() {
                previous_days.push((m.serve_date, dish_names));
            }
        }

        Ok(Self::detect_stale_sequence(incoming_days, &previous_days))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_stale_consecutive_days() {
        let prev_days = vec![
            (
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                vec!["Mercimek Çorbası".to_string(), "Tavuk Sote".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
                vec!["Ezogelin Çorbası".to_string(), "Kuru Fasulye".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
                vec!["Yayla Çorbası".to_string(), "Köfte".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
                vec!["Tarhana Çorbası".to_string(), "Balık".to_string()],
            ),
        ];

        // Eylül ayında aynı 3 günü gönderen bayat kaynak
        let incoming_days = vec![
            (
                NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                vec!["Mercimek Çorbası".to_string(), "Tavuk Sote".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
                vec!["Ezogelin Çorbası".to_string(), "Kuru Fasulye".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
                vec!["Yayla Çorbası".to_string(), "Köfte".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 9, 4).unwrap(),
                vec!["Domates Çorbası".to_string(), "Farklı Yemek".to_string()],
            ),
        ];

        let result = StaleSequenceDetector::detect_stale_sequence(&incoming_days, &prev_days);
        assert!(
            result.is_some(),
            "3 ardışık eşleşen gün bayat olarak tespit edilmeli"
        );
        let m = result.unwrap();
        assert_eq!(m.matching_days_count, 3);
        assert_eq!(m.start_date, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap());
        assert_eq!(m.end_date, NaiveDate::from_ymd_opt(2026, 9, 3).unwrap());
    }

    #[test]
    fn test_ignores_non_consecutive_single_day_coincidence() {
        let prev_days = vec![
            (
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                vec!["Mercimek Çorbası".to_string(), "Tavuk Sote".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
                vec!["Ezogelin Çorbası".to_string(), "Kuru Fasulye".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
                vec!["Yayla Çorbası".to_string(), "Köfte".to_string()],
            ),
        ];

        // Sadece 1. gün aynı (tesadüf), 2. ve 3. günler farklı
        let incoming_days = vec![
            (
                NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                vec!["Mercimek Çorbası".to_string(), "Tavuk Sote".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
                vec!["Tarhana Çorbası".to_string(), "Sebze Yemeği".to_string()],
            ),
            (
                NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
                vec!["Domates Çorbası".to_string(), "Kıymalı Makarna".to_string()],
            ),
        ];

        let result = StaleSequenceDetector::detect_stale_sequence(&incoming_days, &prev_days);
        assert!(
            result.is_none(),
            "Tekil gün benzerliği bayat olarak işaretlenmemeli"
        );
    }
}

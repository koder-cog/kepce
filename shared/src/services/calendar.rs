//! Takvim ve özel günler bağlam çözümleme servisi.
//!
//! 81 ilin yerel kurtuluş/anma günleri, ulusal bayramlar, matem günleri
//! ve KYK burs/kredi ödeme döngülerini yönetir. Bot prompt export ve
//! arayüz bilgilendirmesi için merkezi veri kaynağıdır.

use std::collections::HashMap;
use std::sync::OnceLock;
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NationalDay {
    pub name: String,
    #[serde(default)]
    pub is_celebration: bool,
    #[serde(default)]
    pub is_mourning: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct CalendarData {
    national: HashMap<String, NationalDay>,
    cities: HashMap<String, HashMap<String, String>>,
}

static CALENDAR_DATA: OnceLock<CalendarData> = OnceLock::new();

const EMBEDDED_CALENDAR_JSON: &str = include_str!("../../../config/calendar/special_dates.json");

fn get_calendar_data() -> &'static CalendarData {
    CALENDAR_DATA.get_or_init(|| {
        serde_json::from_str(EMBEDDED_CALENDAR_JSON).unwrap_or_else(|e| {
            tracing::error!("Özel günler takvim JSON parse hatası: {:?}", e);
            CalendarData {
                national: HashMap::new(),
                cities: HashMap::new(),
            }
        })
    })
}

/// Verilen tarihin 10 Kasım (Atatürk'ü Anma Günü / milli matem) olup olmadığını döner.
pub fn is_mourning_day(date: NaiveDate) -> bool {
    date.month() == 11 && date.day() == 10
}

/// Verilen tarihin KYK burs/kredi ödeme dönemi (her ayın 6-10'u) olup olmadığını döner.
pub fn is_scholarship_period(date: NaiveDate) -> bool {
    date.day() >= 6 && date.day() <= 10
}

/// Verilen tarih ve şehir için özel gün açıklamasını döner (varsa).
/// Öncelik sırası: Şehrin yerel günü -> Ulusal gün -> Burs dönemi.
pub fn get_special_day_name(date: NaiveDate, city_slug: &str) -> Option<String> {
    let data = get_calendar_data();
    let mm_dd = format!("{:02}-{:02}", date.month(), date.day());

    // 1. Şehre özel yerel gün
    if let Some(city_days) = data.cities.get(city_slug) {
        if let Some(name) = city_days.get(&mm_dd) {
            return Some(name.clone());
        }
    }

    // 2. Ulusal gün
    if let Some(nat) = data.national.get(&mm_dd) {
        return Some(nat.name.clone());
    }

    // 3. KYK burs/kredi dönemi
    if is_scholarship_period(date) {
        return Some("KYK Burs ve Kredi Ödeme Dönemi".to_string());
    }

    None
}

/// Bot prompt girdi metninde günün başlığını formatlar.
/// Örnek:
/// `=== 2026-10-06 [Şehir: İstanbul | Günün Anlamı: İstanbul'un Kurtuluşu] ===`
pub fn format_bot_day_header(date: NaiveDate, city_name: &str, city_slug: &str) -> String {
    match get_special_day_name(date, city_slug) {
        Some(special) => {
            if city_name.is_empty() {
                format!("=== {} [Günün Anlamı: {}] ===", date, special)
            } else {
                format!(
                    "=== {} [Şehir: {} | Günün Anlamı: {}] ===",
                    date, city_name, special
                )
            }
        }
        None => {
            if city_name.is_empty() {
                format!("=== {} ===", date)
            } else {
                format!("=== {} [Şehir: {}] ===", date, city_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_istanbul_kurtulusu() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 6).unwrap();
        let name = get_special_day_name(date, "istanbul");
        assert_eq!(name.as_deref(), Some("İstanbul'un Kurtuluşu"));

        let header = format_bot_day_header(date, "İstanbul", "istanbul");
        assert_eq!(
            header,
            "=== 2026-10-06 [Şehir: İstanbul | Günün Anlamı: İstanbul'un Kurtuluşu] ==="
        );
    }

    #[test]
    fn test_izmir_kurtulusu() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 9).unwrap();
        let name = get_special_day_name(date, "izmir");
        assert_eq!(name.as_deref(), Some("İzmir'in Kurtuluşu"));
    }

    #[test]
    fn test_mourning_day() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        assert!(is_mourning_day(date));

        let non_mourning = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        assert!(!is_mourning_day(non_mourning));
    }

    #[test]
    fn test_scholarship_period() {
        let in_period = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
        assert!(is_scholarship_period(in_period));
        assert_eq!(
            get_special_day_name(in_period, "generic_city").as_deref(),
            Some("KYK Burs ve Kredi Ödeme Dönemi")
        );

        let out_period = NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();
        assert!(!is_scholarship_period(out_period));
    }

    #[test]
    fn test_regular_day_header() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
        let header = format_bot_day_header(date, "Ankara", "ankara");
        assert_eq!(header, "=== 2026-02-02 [Şehir: Ankara] ===");
    }
}

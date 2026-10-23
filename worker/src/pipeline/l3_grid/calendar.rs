//! Takvim ve gün eşleştirme yardımcıları
use chrono::{Datelike, NaiveDate};

pub fn is_weekend(date: NaiveDate) -> bool {
    date.weekday().number_from_monday() > 5
}

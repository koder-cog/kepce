use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::models::{DailyMenu, DayData, DayMetadata, MenuComponent, MenuDatabase, MenuItem};
use super::normalizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IngestAlternative {
    Simple(String),
    Detailed {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        calories: Option<String>,
    },
}

impl IngestAlternative {
    fn name(&self) -> &str {
        match self {
            IngestAlternative::Simple(s) => s,
            IngestAlternative::Detailed { name, .. } => name,
        }
    }

    fn amount(&self) -> Option<&str> {
        match self {
            IngestAlternative::Simple(_) => None,
            IngestAlternative::Detailed { amount, .. } => amount.as_deref(),
        }
    }

    fn calories(&self) -> Option<&str> {
        match self {
            IngestAlternative::Simple(_) => None,
            IngestAlternative::Detailed { calories, .. } => calories.as_deref(),
        }
    }
}

/// Slash ile birleştirilmiş değerleri parçalar: "200 g / 150 g" -> ["200 g", "150 g"]
fn split_slash_values(raw: &str) -> Vec<String> {
    raw.split('/')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestItemJson {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<IngestAlternative>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDayJson {
    pub date: String,
    /// Belgede YAZILI HAM tarih (örn. `01.04.2026`). LLM'in ISO çıktısı bu
    /// alanla deterministik olarak çapraz doğrulanır; tarih yorumu LLM'e
    /// bırakılmaz (Faz 3.4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meal_type: Option<String>,
    #[serde(alias = "total_calories", skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
    #[serde(default)]
    pub items: Vec<IngestItemJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_colyak: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takeaway: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestMenuJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meal_type: Option<String>,
    #[serde(default)]
    pub is_colyak: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_calories: Option<String>,
    #[serde(default)]
    pub days: Vec<IngestDayJson>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetMeal {
    Breakfast,
    Lunch,
    Dinner,
}

fn resolve_meal_type(explicit: Option<&str>, file_name: &str) -> TargetMeal {
    if let Some(m) = explicit {
        let lower = m.to_lowercase();
        if lower.contains("kahvaltı") || lower.contains("kahvalti") || lower.contains("breakfast")
        {
            return TargetMeal::Breakfast;
        }
        if lower.contains("öğle") || lower.contains("ogle") || lower.contains("lunch") {
            return TargetMeal::Lunch;
        }
        if lower.contains("akşam") || lower.contains("aksam") || lower.contains("dinner") {
            return TargetMeal::Dinner;
        }
    }

    let file_lower = file_name.to_lowercase();
    if file_lower.contains("kahvaltı")
        || file_lower.contains("kahvalti")
        || file_lower.contains("breakfast")
    {
        TargetMeal::Breakfast
    } else if file_lower.contains("öğle")
        || file_lower.contains("ogle")
        || file_lower.contains("lunch")
    {
        TargetMeal::Lunch
    } else {
        TargetMeal::Dinner
    }
}

fn normalize_date_str(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%d.%m.%Y") {
        return Ok(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(d.format("%Y-%m-%d").to_string());
    }
    anyhow::bail!(
        "Geçersiz tarih formatı (beklenen YYYY-MM-DD veya DD.MM.YYYY): '{}'",
        raw
    )
}

pub fn parse_json_str(content: &str, file_name_hint: &str) -> Result<MenuDatabase> {
    parse_json_str_with_diagnostics(content, file_name_hint).map(|(db, _)| db)
}

/// `parse_json_str` + LLM `date_raw` çapraz doğrulaması (Faz 3.4).
///
/// Dönen vektör, LLM'in ISO tarihi ile ham yazılı tarihin deterministik
/// (DMY, Türkiye standardı) çözümünün UYUŞMADIĞI günlerin açıklamalarını taşır.
/// Uyuşmazlık karar motorunda `Suspect(DATE_ORDER_MISMATCH)` olur.
pub fn parse_json_str_with_diagnostics(
    content: &str,
    file_name_hint: &str,
) -> Result<(MenuDatabase, Vec<String>)> {
    let parsed: IngestMenuJson =
        serde_json::from_str(content).context("JSON formatı IngestMenuJson şemasına uymuyor")?;

    let mut db: MenuDatabase = HashMap::new();
    let mut date_raw_mismatches: Vec<String> = Vec::new();
    let default_meal = resolve_meal_type(parsed.meal_type.as_deref(), file_name_hint);
    let is_colyak = parsed.is_colyak;

    for day in parsed.days {
        let date_iso = match normalize_date_str(&day.date) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("Tarih atlandı: {}", e);
                continue;
            }
        };

        // Faz 3.4: ham yazılı tarihi deterministik motorla yeniden çöz ve
        // LLM'in ISO çıktısıyla karşılaştır.
        if let Some(raw) = day
            .date_raw
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            && let Some(deterministic) = super::core::parse_date_string(raw)
            && deterministic != date_iso
        {
            let msg = format!(
                "LLM '{}' tarihini {} olarak verdi ama ham '{}' deterministik çözümde {} oluyor",
                raw, date_iso, raw, deterministic
            );
            tracing::warn!("DATE_ORDER_MISMATCH: {}", msg);
            date_raw_mismatches.push(msg);
        }

        let meal_type = match day.meal_type.as_deref() {
            Some(m) => resolve_meal_type(Some(m), file_name_hint),
            None => default_meal,
        };

        let effective_calories = day
            .calories
            .clone()
            .or_else(|| parsed.default_calories.clone());

        let mut menu_items: Vec<MenuItem> = Vec::new();

        if let Some(takeaway_id) = day.takeaway {
            let clean_id = takeaway_id.trim().to_string();
            if !clean_id.is_empty() {
                menu_items.push(MenuItem {
                    takeaway_id: Some(clean_id),
                    alternatives: Vec::new(),
                });
            }
        }

        for item in day.items {
            let mut alternatives = Vec::new();
            let parent_amount = item
                .amount
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty());
            let parent_calories = item
                .calories
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty());

            if let Some(alts) = item.alternatives {
                let total_dishes = 1 + alts.len();

                let amount_parts: Vec<String> = parent_amount
                    .as_deref()
                    .filter(|a| a.contains('/'))
                    .map(split_slash_values)
                    .unwrap_or_default();
                let cal_parts: Vec<String> = parent_calories
                    .as_deref()
                    .filter(|c| c.contains('/'))
                    .map(split_slash_values)
                    .unwrap_or_default();

                let pick_amount = |idx: usize, alt_own: Option<&str>| -> Option<String> {
                    if let Some(own) = alt_own {
                        let trimmed = own.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                    if amount_parts.len() == total_dishes {
                        Some(amount_parts[idx].clone())
                    } else {
                        parent_amount.clone()
                    }
                };
                let pick_cal = |idx: usize, alt_own: Option<&str>| -> Option<String> {
                    if let Some(own) = alt_own {
                        let trimmed = own.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                    if cal_parts.len() == total_dishes {
                        Some(cal_parts[idx].clone())
                    } else {
                        parent_calories.clone()
                    }
                };

                let primary_norm = normalizer::normalize_food_item(&item.name);
                if !primary_norm.name.is_empty() {
                    let amt = pick_amount(0, None).or(primary_norm.amount);
                    alternatives.push(MenuComponent {
                        name: primary_norm.name,
                        amount: amt,
                        calories: pick_cal(0, None),
                        category: None,
                    });
                }

                for (i, alt) in alts.iter().enumerate() {
                    let alt_norm = normalizer::normalize_food_item(alt.name());
                    if !alt_norm.name.is_empty() {
                        let amt = pick_amount(i + 1, alt.amount()).or(alt_norm.amount);
                        alternatives.push(MenuComponent {
                            name: alt_norm.name,
                            amount: amt,
                            calories: pick_cal(i + 1, alt.calories()),
                            category: None,
                        });
                    }
                }
            } else {
                let split_names = normalizer::split_smart_alternatives(&item.name);
                for n in split_names {
                    let norm = normalizer::normalize_food_item(&n);
                    if !norm.name.is_empty() {
                        let amt = parent_amount.clone().or(norm.amount);
                        alternatives.push(MenuComponent {
                            name: norm.name,
                            amount: amt,
                            calories: parent_calories.clone(),
                            category: None,
                        });
                    }
                }
            }

            if alternatives.is_empty() {
                let norm = normalizer::normalize_food_item(&item.name);
                if !norm.name.is_empty() {
                    let amt = parent_amount.or(norm.amount);
                    alternatives.push(MenuComponent {
                        name: norm.name,
                        amount: amt,
                        calories: parent_calories,
                        category: None,
                    });
                }
            }

            if !alternatives.is_empty() {
                menu_items.push(MenuItem {
                    takeaway_id: None,
                    alternatives,
                });
            }
        }

        let day_data = db.entry(date_iso).or_insert_with(|| DayData {
            metadata: Some(DayMetadata {
                trust_score: 100,
                anomaly_score: Some(0.0),
                status: "approved".to_string(),
                source_file: Some(file_name_hint.to_string()),
            }),
            ..Default::default()
        });

        let is_colyak_for_day = day.is_colyak.unwrap_or(is_colyak);
        let target_menu: &mut DailyMenu = if is_colyak_for_day {
            &mut day_data.colyak
        } else {
            &mut day_data.normal
        };

        match meal_type {
            TargetMeal::Breakfast => {
                if let Some(cal) = effective_calories {
                    target_menu.breakfast_kcal = Some(cal);
                }
                target_menu.breakfast.extend(menu_items);
            }
            TargetMeal::Lunch => {
                if let Some(cal) = effective_calories {
                    target_menu.lunch_kcal = Some(cal);
                }
                target_menu.lunch.extend(menu_items);
            }
            TargetMeal::Dinner => {
                if let Some(cal) = effective_calories {
                    target_menu.dinner_kcal = Some(cal);
                }
                target_menu.dinner.extend(menu_items);
            }
        }
    }

    Ok((db, date_raw_mismatches))
}

pub fn parse_json_file(file_path: &str, _city_slug: &str) -> Result<MenuDatabase> {
    let path = Path::new(file_path);
    let content =
        std::fs::read_to_string(path).context(format!("JSON dosyası okunamadı: {}", file_path))?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.json");

    parse_json_str(&content, file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_menu_basic() {
        let sample = r#"{
            "city": "istanbul",
            "meal_type": "breakfast",
            "is_colyak": false,
            "default_calories": "750-900 kcal",
            "days": [
                {
                    "date": "2026-04-01",
                    "items": [
                        { "name": "Patates Kızartması", "amount": "150 g" },
                        { "name": "Haşlanmış Yumurta", "amount": "1 adet L boy" },
                        { "name": "Siyah/Yeşil Zeytin", "amount": "30 g" }
                    ],
                    "takeaway": "1-2"
                },
                {
                    "date": "02.04.2026",
                    "calories": "850 kcal",
                    "items": [
                        { "name": "Kaşarlı Omlet", "amount": "120 g" }
                    ]
                }
            ]
        }"#;

        let db = parse_json_str(sample, "Nisan_2026_Kahvaltı.json").unwrap();
        assert_eq!(db.len(), 2);

        let day1 = db.get("2026-04-01").expect("2026-04-01 bulunmalı");
        assert_eq!(day1.normal.breakfast_kcal.as_deref(), Some("750-900 kcal"));
        assert_eq!(day1.normal.breakfast.len(), 4); // 1 takeaway + 3 items
        assert_eq!(day1.normal.breakfast[0].takeaway_id.as_deref(), Some("1-2"));

        let zeytin = &day1.normal.breakfast[3];
        assert_eq!(zeytin.alternatives.len(), 2); // Siyah Zeytin / Yeşil Zeytin

        let day2 = db
            .get("2026-04-02")
            .expect("02.04.2026 -> 2026-04-02 normalize edilmeli");
        assert_eq!(day2.normal.breakfast_kcal.as_deref(), Some("850 kcal"));
        assert_eq!(day2.normal.breakfast.len(), 1);
    }

    #[test]
    fn test_parse_real_nisan_kahvalti_json() {
        let path = "../data/menuler/admin/bekleyen/istanbul/Nisan_2026_Kahvaltı.json";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let db = parse_json_file(path, "istanbul")
            .expect("Gerçek Nisan 2026 Kahvaltı JSON ayrıştırılmalı");
        assert_eq!(db.len(), 30, "Nisan ayı 30 gün olmalı");

        for d in 1..=30 {
            let key = format!("2026-04-{:02}", d);
            let day_data = db
                .get(&key)
                .unwrap_or_else(|| panic!("{} günü bulunamadı", key));
            assert!(
                !day_data.normal.breakfast.is_empty(),
                "{} kahvaltı listesi boş olamaz",
                key
            );
        }
    }

    #[test]
    fn test_detailed_alternatives() {
        let sample = r#"{
            "meal_type": "dinner",
            "days": [{
                "date": "2026-09-16",
                "items": [{
                    "name": "Izgara Köfte",
                    "amount": "200 g (90 g et)",
                    "calories": "340 kcal",
                    "alternatives": [
                        { "name": "Etsiz Patlıcan", "amount": "200 g", "calories": "164 kcal" }
                    ]
                }]
            }]
        }"#;
        let db = parse_json_str(sample, "test.json").unwrap();
        let day = db.get("2026-09-16").unwrap();
        let slot = &day.normal.dinner[0];
        assert_eq!(slot.alternatives.len(), 2);
        assert_eq!(slot.alternatives[0].name, "Izgara Köfte");
        assert_eq!(
            slot.alternatives[0].amount.as_deref(),
            Some("200 g (90 g et)")
        );
        assert_eq!(slot.alternatives[0].calories.as_deref(), Some("340 kcal"));
        assert_eq!(slot.alternatives[1].name, "Etsiz Patlıcan");
        assert_eq!(slot.alternatives[1].amount.as_deref(), Some("200 g"));
        assert_eq!(slot.alternatives[1].calories.as_deref(), Some("164 kcal"));
    }

    #[test]
    fn test_slash_fallback_distribution() {
        let sample = r#"{
            "meal_type": "dinner",
            "days": [{
                "date": "2026-09-16",
                "items": [{
                    "name": "Izgara Köfte",
                    "amount": "200 g / 150 g",
                    "calories": "340 / 164",
                    "alternatives": ["Etsiz Patlıcan"]
                }]
            }]
        }"#;
        let db = parse_json_str(sample, "test.json").unwrap();
        let day = db.get("2026-09-16").unwrap();
        let slot = &day.normal.dinner[0];
        assert_eq!(slot.alternatives.len(), 2);
        assert_eq!(slot.alternatives[0].amount.as_deref(), Some("200 g"));
        assert_eq!(slot.alternatives[0].calories.as_deref(), Some("340"));
        assert_eq!(slot.alternatives[1].amount.as_deref(), Some("150 g"));
        assert_eq!(slot.alternatives[1].calories.as_deref(), Some("164"));
    }

    #[test]
    fn test_backward_compat_simple_alternatives() {
        let sample = r#"{
            "meal_type": "dinner",
            "days": [{
                "date": "2026-09-17",
                "items": [{
                    "name": "Pirinç Pilavı",
                    "amount": "180 g",
                    "alternatives": ["Bulgur Pilavı"]
                }]
            }]
        }"#;
        let db = parse_json_str(sample, "test.json").unwrap();
        let day = db.get("2026-09-17").unwrap();
        let slot = &day.normal.dinner[0];
        assert_eq!(slot.alternatives.len(), 2);
        assert_eq!(slot.alternatives[0].name, "Pirinç Pilavı");
        assert_eq!(slot.alternatives[1].name, "Bulgur Pilavı");
        assert_eq!(slot.alternatives[1].amount.as_deref(), Some("180 g"));
    }

    #[test]
    fn test_mixed_normal_and_colyak_day_json() {
        let sample = r#"{
            "days": [
                {
                    "date": "2026-05-01",
                    "meal_type": "dinner",
                    "is_colyak": false,
                    "items": [{ "name": "Mercimek Çorbası" }]
                },
                {
                    "date": "2026-05-01",
                    "meal_type": "dinner",
                    "is_colyak": true,
                    "items": [{ "name": "Glutensiz Yayla Çorbası" }]
                }
            ]
        }"#;
        let db = parse_json_str(sample, "test.json").unwrap();
        let day = db.get("2026-05-01").expect("2026-05-01 olmali");
        assert_eq!(day.normal.dinner.len(), 1);
        assert_eq!(
            day.normal.dinner[0].alternatives[0].name,
            "Mercimek Çorbası"
        );
        assert_eq!(day.colyak.dinner.len(), 1);
        assert_eq!(
            day.colyak.dinner[0].alternatives[0].name,
            "Glutensiz Yayla Çorbası"
        );
    }
}

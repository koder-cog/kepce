use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::models::{DailyMenu, DayData, DayMetadata, MenuComponent, MenuDatabase, MenuItem};
use super::normalizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestItemJson {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDayJson {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
    #[serde(default)]
    pub items: Vec<IngestItemJson>,
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
        if lower.contains("kahvaltı") || lower.contains("kahvalti") || lower.contains("breakfast") {
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
    if file_lower.contains("kahvaltı") || file_lower.contains("kahvalti") || file_lower.contains("breakfast") {
        TargetMeal::Breakfast
    } else if file_lower.contains("öğle") || file_lower.contains("ogle") || file_lower.contains("lunch") {
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
    anyhow::bail!("Geçersiz tarih formatı (beklenen YYYY-MM-DD veya DD.MM.YYYY): '{}'", raw)
}

pub fn parse_json_str(content: &str, file_name_hint: &str) -> Result<MenuDatabase> {
    let parsed: IngestMenuJson = serde_json::from_str(content)
        .context("JSON formatı IngestMenuJson şemasına uymuyor")?;

    let mut db: MenuDatabase = HashMap::new();
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

        let meal_type = match day.meal_type.as_deref() {
            Some(m) => resolve_meal_type(Some(m), file_name_hint),
            None => default_meal,
        };

        let effective_calories = day.calories.clone().or_else(|| parsed.default_calories.clone());

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
            let amount = item.amount.map(|a| a.trim().to_string()).filter(|a| !a.is_empty());
            let calories = item.calories.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());

            if let Some(alts) = item.alternatives {
                for alt_name in alts {
                    let clean = normalizer::normalize_food_name(&alt_name);
                    if !clean.is_empty() {
                        alternatives.push(MenuComponent {
                            name: clean,
                            amount: amount.clone(),
                            calories: calories.clone(),
                            category: None,
                        });
                    }
                }
            } else {
                let split_names = normalizer::split_smart_alternatives(&item.name);
                for n in split_names {
                    let clean = normalizer::normalize_food_name(&n);
                    if !clean.is_empty() {
                        alternatives.push(MenuComponent {
                            name: clean,
                            amount: amount.clone(),
                            calories: calories.clone(),
                            category: None,
                        });
                    }
                }
            }

            if alternatives.is_empty() {
                let clean = normalizer::normalize_food_name(&item.name);
                if !clean.is_empty() {
                    alternatives.push(MenuComponent {
                        name: clean,
                        amount,
                        calories,
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

        let target_menu: &mut DailyMenu = if is_colyak {
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

    Ok(db)
}

pub fn parse_json_file(file_path: &str, _city_slug: &str) -> Result<MenuDatabase> {
    let path = Path::new(file_path);
    let content = std::fs::read_to_string(path)
        .context(format!("JSON dosyası okunamadı: {}", file_path))?;
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

        let day2 = db.get("2026-04-02").expect("02.04.2026 -> 2026-04-02 normalize edilmeli");
        assert_eq!(day2.normal.breakfast_kcal.as_deref(), Some("850 kcal"));
        assert_eq!(day2.normal.breakfast.len(), 1);
    }

    #[test]
    fn test_parse_real_nisan_kahvalti_json() {
        let path = "../data/menuler/admin/bekleyen/istanbul/Nisan_2026_Kahvaltı.json";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let db = parse_json_file(path, "istanbul").expect("Gerçek Nisan 2026 Kahvaltı JSON ayrıştırılmalı");
        assert_eq!(db.len(), 30, "Nisan ayı 30 gün olmalı");

        for d in 1..=30 {
            let key = format!("2026-04-{:02}", d);
            let day_data = db.get(&key).unwrap_or_else(|| panic!("{} günü bulunamadı", key));
            assert!(!day_data.normal.breakfast.is_empty(), "{} kahvaltı listesi boş olamaz", key);
        }
    }
}

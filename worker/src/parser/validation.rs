use chrono::{NaiveDate, Utc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

/// Breakfast signal words - if these appear in item names, the sheet is likely breakfast
const BREAKFAST_SIGNALS: &[&str] = &[
    "çay", "peynir", "zeytin", "reçel", "yumurta", "bal", "tereyağı",
    "süt", "simit", "poğaça", "börek", "kahvaltılık", "kaşar", "beyaz peynir",
    "domates", "salatalık", "gevrek", "gözleme", "menemen", "sucuk",
];

/// Dinner / Lunch signal words - if these appear in item names, the sheet is likely hot meal
const DINNER_SIGNALS: &[&str] = &[
    "çorba", "pilav", "makarna", "köfte", "salata", "komposto", "ayran",
    "tatlı", "et ", "tavuk", "balık", "kızartma", "güveç", "sote", "dolma",
    "sarma", "püresi", "corbası", "kebap", "izgara", "haşlama", "rosto",
    "yahni", "türlü", "cacık", "hoşaf",
];

/// Max allowed item name length
const MAX_ITEM_NAME_LEN: usize = 150;

/// Max allowed items per meal
const MAX_ITEMS_PER_MEAL: usize = 25;

/// Max file size in bytes (5 MB)
pub const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Max sheets per workbook
pub const MAX_SHEET_COUNT: usize = 10;

/// Min items for a file to be considered a valid menu
pub const MIN_VALID_ITEMS: usize = 3;

/// Validates that a date string (YYYY-MM-DD) is within ±2 years of today.
pub fn validate_date_range(date: &str) -> bool {
    let parsed = match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return false,
    };

    let today = Utc::now().date_naive();
    let two_years = chrono::Duration::days(365 * 2);
    let min_date = today - two_years;
    let max_date = today + two_years;

    parsed >= min_date && parsed <= max_date
}

/// Validates an item name: must be non-empty and at most 150 characters.
/// Returns the trimmed name if valid, None otherwise.
pub fn validate_item_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_ITEM_NAME_LEN {
        return None;
    }
    Some(trimmed.to_string())
}

/// Validates that a gramaj/calorie value is numeric or a range format.
/// Accepts: "150", "100-200", "1.5", "150 g", "250 kcal", "100,5"
/// Rejects: "çok fazla", "NaN", "biraz", empty-ish text
pub fn validate_numeric_value(val: &str) -> Option<String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }

    let upper = trimmed.to_uppercase();
    if upper == "NAN" || upper == "N/A" || upper == "-" {
        return None;
    }

    // Strip known unit suffixes for validation, but keep the original string
    let check = trimmed
        .trim_end_matches(|c: char| c.is_whitespace())
        .trim_end_matches("kcal")
        .trim_end_matches("kkal")
        .trim_end_matches("gr")
        .trim_end_matches("g")
        .trim_end_matches("ml")
        .trim_end_matches("cc")
        .trim();

    if check.is_empty() {
        return None;
    }

    // Check if it's a valid number or range pattern
    // Allow: digits, commas (Turkish decimal), dots, dashes (range), spaces around dash
    let re = regex::Regex::new(r"^\d+([.,]\d+)?(\s*[-–]\s*\d+([.,]\d+)?)?$").unwrap();
    if re.is_match(check) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Detects meal type using a 3-stage approach:
/// 1. Sheet name matching (KAHVALTI → Breakfast, YEMEK/AKŞAM → Dinner)
/// 2. Content signal analysis from sample item names
/// 3. Returns None if both stages are inconclusive
pub fn detect_meal_type(sheet_name: &str, sample_items: &[String]) -> Option<MealType> {
    let upper = sheet_name.to_uppercase();

    // Stage 1: Sheet name matching
    if upper.contains("KAHVALTI") || upper.contains("SABAH") {
        return Some(MealType::Breakfast);
    }
    if upper.contains("ÖĞLE") || upper.contains("OGLE") {
        return Some(MealType::Lunch);
    }
    if upper.contains("AKŞAM") || upper.contains("AKSAM") {
        return Some(MealType::Dinner);
    }
    if upper.contains("YEMEK") {
        // Generic "YEMEK" in dormitory context defaults to Dinner
        return Some(MealType::Dinner);
    }

    // Stage 2: Content signal analysis
    if sample_items.is_empty() {
        return None;
    }

    let mut breakfast_score: i32 = 0;
    let mut dinner_score: i32 = 0;

    for item in sample_items {
        let lower = item.to_lowercase();
        for signal in BREAKFAST_SIGNALS {
            if lower.contains(signal) {
                breakfast_score += 1;
            }
        }
        for signal in DINNER_SIGNALS {
            if lower.contains(signal) {
                dinner_score += 1;
            }
        }
    }

    if breakfast_score > dinner_score && breakfast_score >= 2 {
        Some(MealType::Breakfast)
    } else if dinner_score > breakfast_score && dinner_score >= 2 {
        Some(MealType::Dinner)
    } else {
        None
    }
}

/// Checks if item count is within the allowed limit.
pub fn validate_meal_item_count(count: usize) -> bool {
    count <= MAX_ITEMS_PER_MEAL
}

/// Checks if the sheet name contains çölyak indicators.
pub fn is_colyak_sheet(sheet_name: &str) -> bool {
    let upper = sheet_name.to_uppercase();
    upper.contains("ÇÖLYAK") || upper.contains("COLYAK")
}

/// Computes the overall trust score of a DayData by averaging the match ratio of its items.
pub fn calculate_trust_score(day: &crate::parser::models::DayData) -> crate::parser::models::DayMetadata {
    use crate::parser::dictionary::calculate_match_ratio;
    
    let mut total_score = 0.0;
    let mut item_count = 0;

    let mut process_items = |items: &[crate::parser::models::MenuItem]| {
        for item in items {
            for alt in &item.alternatives {
                total_score += calculate_match_ratio(&alt.name);
                item_count += 1;
            }
        }
    };

    process_items(&day.normal.breakfast);
    process_items(&day.normal.dinner);
    process_items(&day.colyak.breakfast);
    process_items(&day.colyak.dinner);

    let avg_score = if item_count > 0 {
        (total_score / item_count as f64).round() as u8
    } else {
        0
    };

    let status = if avg_score >= 30 {
        "approved".to_string()
    } else {
        "needs_review".to_string()
    };

    crate::parser::models::DayMetadata {
        trust_score: avg_score,
        status,
        anomaly_score: None,
        source_file: None,
    }
}

/// Finalizes the metadata for a given day by calculating the trust score,
/// extracting all text to compute the anomaly score, and updating the status
/// if it fails anomaly checks (distance > 0.5).
pub fn finalize_day_metadata(day_data: &mut crate::parser::models::DayData) {
    let mut metadata = calculate_trust_score(day_data);
    
    // Extract all text for anomaly detection
    let mut all_text: Vec<String> = Vec::new();
    let mut process_items = |items: &[crate::parser::models::MenuItem]| {
        for item in items {
            for alt in &item.alternatives {
                all_text.push(alt.name.clone());
            }
        }
    };
    
    process_items(&day_data.normal.breakfast);
    process_items(&day_data.normal.dinner);
    process_items(&day_data.colyak.breakfast);
    process_items(&day_data.colyak.dinner);

    let combined_text = all_text.join(" ");
    metadata.anomaly_score = crate::parser::anomaly::calculate_menu_distance(&combined_text);
    
    // Security layer: If the menu is highly anomalous compared to typical menus, flag for review
    if let Some(distance) = metadata.anomaly_score {
        if distance > 0.65 {
            metadata.status = "needs_review".to_string();
        }
    }

    // Retain the source file if it was previously set
    if let Some(existing_meta) = &day_data.metadata {
        metadata.source_file = existing_meta.source_file.clone();
    }
    
    day_data.metadata = Some(metadata);
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Date range tests ---
    #[test]
    fn test_date_range_valid() {
        assert!(validate_date_range("2026-07-21"));
        assert!(validate_date_range("2025-01-01"));
        assert!(validate_date_range("2027-12-31"));
    }

    #[test]
    fn test_date_range_absurd_past() {
        assert!(!validate_date_range("0001-01-01"));
        assert!(!validate_date_range("2020-01-01"));
    }

    #[test]
    fn test_date_range_absurd_future() {
        assert!(!validate_date_range("2099-12-31"));
        assert!(!validate_date_range("2030-06-15"));
    }

    #[test]
    fn test_date_range_invalid_format() {
        assert!(!validate_date_range("not-a-date"));
        assert!(!validate_date_range("21.07.2026")); // wrong format for this function
    }

    // --- Item name tests ---
    #[test]
    fn test_item_name_valid() {
        assert_eq!(
            validate_item_name("Mercimek Çorbası"),
            Some("Mercimek Çorbası".to_string())
        );
    }

    #[test]
    fn test_item_name_trimmed() {
        assert_eq!(
            validate_item_name("  Pilav  "),
            Some("Pilav".to_string())
        );
    }

    #[test]
    fn test_item_name_too_long() {
        let long = "A".repeat(200);
        assert_eq!(validate_item_name(&long), None);
    }

    #[test]
    fn test_item_name_empty() {
        assert_eq!(validate_item_name(""), None);
        assert_eq!(validate_item_name("   "), None);
    }

    // --- Numeric validation tests ---
    #[test]
    fn test_numeric_valid_int() {
        assert_eq!(validate_numeric_value("150"), Some("150".to_string()));
    }

    #[test]
    fn test_numeric_valid_range() {
        assert_eq!(validate_numeric_value("100-200"), Some("100-200".to_string()));
    }

    #[test]
    fn test_numeric_valid_decimal() {
        assert_eq!(validate_numeric_value("1.5"), Some("1.5".to_string()));
    }

    #[test]
    fn test_numeric_valid_with_unit() {
        assert!(validate_numeric_value("150 g").is_some());
        assert!(validate_numeric_value("250 kcal").is_some());
    }

    #[test]
    fn test_numeric_turkish_decimal() {
        assert!(validate_numeric_value("1,5").is_some());
    }

    #[test]
    fn test_numeric_invalid_text() {
        assert_eq!(validate_numeric_value("çok fazla"), None);
        assert_eq!(validate_numeric_value("biraz"), None);
    }

    #[test]
    fn test_numeric_nan() {
        assert_eq!(validate_numeric_value("NaN"), None);
        assert_eq!(validate_numeric_value("NAN"), None);
        assert_eq!(validate_numeric_value("N/A"), None);
    }

    #[test]
    fn test_numeric_empty() {
        assert_eq!(validate_numeric_value(""), None);
        assert_eq!(validate_numeric_value("   "), None);
    }

    #[test]
    fn test_numeric_dash_only() {
        assert_eq!(validate_numeric_value("-"), None);
    }

    // --- Meal type detection tests ---
    #[test]
    fn test_meal_type_sheet_name_kahvalti() {
        assert_eq!(
            detect_meal_type("KAHVALTI", &[]),
            Some(MealType::Breakfast)
        );
        assert_eq!(
            detect_meal_type("Mayıs Kahvaltı", &[]),
            Some(MealType::Breakfast)
        );
    }

    #[test]
    fn test_meal_type_sheet_name_dinner() {
        assert_eq!(
            detect_meal_type("AKŞAM YEMEĞİ", &[]),
            Some(MealType::Dinner)
        );
        assert_eq!(
            detect_meal_type("Yemek Listesi", &[]),
            Some(MealType::Dinner)
        );
    }

    #[test]
    fn test_meal_type_content_breakfast() {
        let items = vec![
            "Çay".to_string(),
            "Beyaz Peynir".to_string(),
            "Zeytin".to_string(),
            "Reçel".to_string(),
            "Yumurta".to_string(),
        ];
        assert_eq!(
            detect_meal_type("Sayfa1", &items),
            Some(MealType::Breakfast)
        );
    }

    #[test]
    fn test_meal_type_content_dinner() {
        let items = vec![
            "Mercimek Çorbası".to_string(),
            "Pirinç Pilavı".to_string(),
            "Tavuk Sote".to_string(),
            "Ayran".to_string(),
        ];
        assert_eq!(
            detect_meal_type("Sayfa1", &items),
            Some(MealType::Dinner)
        );
    }

    #[test]
    fn test_meal_type_unknown() {
        let items = vec![
            "abc".to_string(),
            "xyz".to_string(),
            "123".to_string(),
        ];
        assert_eq!(detect_meal_type("Sayfa1", &items), None);
    }

    #[test]
    fn test_meal_type_no_items_unknown_name() {
        assert_eq!(detect_meal_type("Finansal Rapor Q3", &[]), None);
    }

    // --- Item count tests ---
    #[test]
    fn test_item_count_valid() {
        assert!(validate_meal_item_count(15));
        assert!(validate_meal_item_count(25));
    }

    #[test]
    fn test_item_count_over() {
        assert!(!validate_meal_item_count(26));
        assert!(!validate_meal_item_count(100));
    }

    // --- Çölyak tests ---
    #[test]
    fn test_colyak_detection() {
        assert!(is_colyak_sheet("ÇÖLYAK KAHVALTI"));
        assert!(is_colyak_sheet("Colyak Yemek"));
        assert!(!is_colyak_sheet("KAHVALTI"));
    }
}

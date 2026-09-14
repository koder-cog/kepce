//! Dinamik Menü Kalite Skoru Motoru (Composite Quality Score - CQS).
//!
//! Menülerin içeriğinin zenginliğini, KYK tabldot standartlarına (4 kap + ekmek + su)
//! uygunluğunu ve sözlüksel/anomali bütünlüğünü 0-100 arasında puanlar.

use serde::{Deserialize, Serialize};

/// Kalite puanlamasına girecek yemek girdisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DishInput {
    pub name: String,
    pub weight_g: Option<i32>,
    pub calories: Option<i32>,
    pub is_alternative: bool,
}

/// Kalite değerlendirmesi için menü girdisi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuQualityInput {
    pub meal_type: String, // "breakfast" veya "dinner"
    pub primary_dishes: Vec<DishInput>,
    pub has_celiac: bool,
    pub has_takeaways: bool,
    pub calorie_min: Option<i32>,
    pub calorie_max: Option<i32>,
    pub anomaly_score: Option<f32>,
    pub dictionary_match_ratio: Option<f64>,
}

/// Kalite skoru detay kırılımı.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityScoreBreakdown {
    pub total: i32,
    pub tray_score: i32,
    pub richness_score: i32,
    pub integrity_score: i32,
}

pub struct QualityScoreService;

impl QualityScoreService {
    /// Bir menünün içeriğini analiz ederek 0 ile 100 arasında kompozit kalite skoru üretir.
    pub fn calculate(input: &MenuQualityInput) -> QualityScoreBreakdown {
        let is_breakfast = input.meal_type.to_lowercase().contains("breakfast")
            || input.meal_type.to_lowercase().contains("kahvalti");

        // 1. Tabldot Bütünlüğü (Tray Completeness - Maks 70 puan)
        let mut tray_score = 0;
        let mut has_bread = false;
        let mut has_beverage = false;
        let mut core_dishes_count = 0;

        for dish in &input.primary_dishes {
            if dish.is_alternative {
                continue;
            }
            let name_lower = dish.name.to_lowercase();

            // Ekmek kontrolü
            if name_lower.contains("ekmek") || name_lower.contains("roll") || name_lower.contains("simit") || name_lower.contains("poğaça") {
                has_bread = true;
                continue;
            }

            // Su veya içecek kontrolü
            if name_lower.contains(" su") || name_lower == "su" || name_lower.contains("ayran")
                || name_lower.contains("çay") || name_lower.contains("cay")
                || name_lower.contains("içecek") || name_lower.contains("icecek")
                || name_lower.contains("meyve suyu") || name_lower.contains("limonata")
            {
                has_beverage = true;
                continue;
            }

            core_dishes_count += 1;
        }

        if is_breakfast {
            // Kahvaltı: 4+ ana parça = 40p, 3 = 25p, 2 = 15p, 1 = 10p
            tray_score += match core_dishes_count {
                n if n >= 4 => 40,
                3 => 25,
                2 => 15,
                1 => 10,
                _ => 0,
            };
        } else {
            // Akşam Yemeği: 4 ana kap = 40p, 3 = 25p, 2 = 15p, 1 = 5p
            tray_score += match core_dishes_count {
                n if n >= 4 => 40,
                3 => 25,
                2 => 15,
                1 => 5,
                _ => 0,
            };
        }

        if has_bread {
            tray_score += 15;
        }
        if has_beverage {
            tray_score += 15;
        }
        tray_score = tray_score.clamp(0, 70);

        // 2. Zenginlik & Ek Bilgi Bonusları (Maks 15 puan)
        let mut richness_score = 0;
        let has_weights = input.primary_dishes.iter().any(|d| d.weight_g.unwrap_or(0) > 0);
        let has_calories = input.calorie_min.unwrap_or(0) > 0
            || input.calorie_max.unwrap_or(0) > 0
            || input.primary_dishes.iter().any(|d| d.calories.unwrap_or(0) > 0);

        if has_weights {
            richness_score += 5;
        }
        if has_calories {
            richness_score += 5;
        }
        if input.has_takeaways || input.has_celiac {
            richness_score += 5;
        }
        richness_score = richness_score.clamp(0, 15);

        // 3. Sözlük & Anomali Bütünlüğü (Maks 15 puan)
        let mut integrity_score: i32 = 0;
        if let Some(ratio) = input.dictionary_match_ratio {
            if ratio >= 80.0 {
                integrity_score += 10;
            } else if ratio >= 60.0 {
                integrity_score += 7;
            } else if ratio >= 40.0 {
                integrity_score += 4;
            }
        } else {
            // Oran hesaplanmamışsa nötr kabul
            integrity_score += 7;
        }

        if let Some(anomaly) = input.anomaly_score {
            if anomaly < 0.35 {
                integrity_score += 5;
            } else if anomaly > 0.65 {
                // Şüpheli/çöp metin cezası
                integrity_score -= 20;
            }
        } else {
            integrity_score += 3;
        }
        integrity_score = integrity_score.clamp(0, 15);

        let total = (tray_score + richness_score + integrity_score).clamp(0, 100);

        QualityScoreBreakdown {
            total,
            tray_score,
            richness_score,
            integrity_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_tray_dinner_scores_high() {
        let input = MenuQualityInput {
            meal_type: "dinner".to_string(),
            primary_dishes: vec![
                DishInput { name: "Mercimek Çorbası".to_string(), weight_g: Some(250), calories: Some(180), is_alternative: false },
                DishInput { name: "Tavuk Sote".to_string(), weight_g: Some(200), calories: Some(320), is_alternative: false },
                DishInput { name: "Pirinç Pilavı".to_string(), weight_g: Some(180), calories: Some(260), is_alternative: false },
                DishInput { name: "Ayran".to_string(), weight_g: None, calories: Some(80), is_alternative: false },
                DishInput { name: "Çeyrek Ekmek".to_string(), weight_g: Some(50), calories: Some(120), is_alternative: false },
                DishInput { name: "Mevsim Meyvesi".to_string(), weight_g: Some(150), calories: Some(90), is_alternative: false },
            ],
            has_celiac: false,
            has_takeaways: true,
            calorie_min: Some(800),
            calorie_max: Some(1100),
            anomaly_score: Some(0.12),
            dictionary_match_ratio: Some(95.0),
        };

        let result = QualityScoreService::calculate(&input);
        assert_eq!(result.tray_score, 70); // 4 ana kap (40) + ekmek (15) + ayran/içecek (15) = 70
        assert_eq!(result.richness_score, 15); // gramaj (5) + kalori (5) + takeaway (5) = 15
        assert_eq!(result.integrity_score, 15); // sözlük (10) + anomali < 0.35 (5) = 15
        assert_eq!(result.total, 100);
    }

    #[test]
    fn test_truncated_menu_scores_low() {
        let input = MenuQualityInput {
            meal_type: "dinner".to_string(),
            primary_dishes: vec![
                DishInput { name: "Yemekhane Açık".to_string(), weight_g: None, calories: None, is_alternative: false },
            ],
            has_celiac: false,
            has_takeaways: false,
            calorie_min: None,
            calorie_max: None,
            anomaly_score: Some(0.72), // Yüksek anomali
            dictionary_match_ratio: Some(20.0),
        };

        let result = QualityScoreService::calculate(&input);
        assert!(result.total < 30, "Kusurlu/tek kap menü düşük puan almalı: {}", result.total);
    }
}

//! Deterministik CSV ızgara ve etiketli metin ayrıştırıcısı.
//!
//! Birinci aşama LLM (Gemini Flash) tarafından üretilen etiketli ızgara (`[BELGE_TURU]`,
//! `[TABLO]`, `[DIPNOTLAR]`) veya ham CSV metnini ikinci bir LLM çağrısına gerek kalmadan
//! doğrudan tipli `MenuDatabase` ve `ParsedDocumentPayload` nesnesine dönüştürür.

use anyhow::Result;
use std::collections::HashMap;

use crate::parser::core::{parse_date_string, split_outside_parens};
use crate::parser::models::{
    DayData, DayMetadata, MenuComponent, MenuDatabase, MenuItem, ParsedDocumentPayload,
};

/// Etiketli ızgara veya saf CSV içeriğini tipli `ParsedDocumentPayload` nesnesine dönüştürür.
pub fn parse_csv_grid_to_payload(
    content: &str,
    file_name_hint: &str,
) -> Result<ParsedDocumentPayload> {
    let mut doc_meal_type = "dinner"; // varsayılan
    let mut in_tablo = false;
    let mut in_dipnotlar = false;
    let mut csv_lines = Vec::new();
    let mut dipnot_lines = Vec::new();

    let has_tablo_tag = content.contains("[TABLO]");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[BELGE_TURU") {
            let upper = trimmed.to_uppercase();
            if upper.contains("KAHVALTI") {
                doc_meal_type = "breakfast";
            } else if upper.contains("AKŞAM") || upper.contains("AKSAM") {
                doc_meal_type = "dinner";
            } else if upper.contains("ÖĞLE") || upper.contains("OGLE") {
                doc_meal_type = "lunch";
            }
            in_tablo = false;
            in_dipnotlar = false;
        } else if trimmed.starts_with("[TABLO]") {
            in_tablo = true;
            in_dipnotlar = false;
        } else if trimmed.starts_with("[DIPNOTLAR]") || trimmed.starts_with("[DİPNOTLAR]") {
            in_tablo = false;
            in_dipnotlar = true;
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_tablo = false;
            in_dipnotlar = false;
        } else if in_tablo || (!has_tablo_tag && !in_dipnotlar) {
            if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("```") {
                csv_lines.push(line);
            }
        } else if in_dipnotlar && !trimmed.is_empty() {
            dipnot_lines.push(trimmed.trim_start_matches('-').trim().to_string());
        }
    }

    if csv_lines.is_empty() {
        anyhow::bail!("Ayrıştırılacak CSV tablo satırı bulunamadı.");
    }

    let csv_joined = csv_lines.join("\n");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_joined.as_bytes());

    let mut date_col = 0;
    let mut meal_col = None;
    let mut dish_col = 3;
    let mut amount_col = None;
    let mut cal_col = None;
    let mut total_cal_col = None;
    let mut header_found = false;

    let mut menu_db: MenuDatabase = HashMap::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        if record.is_empty() {
            continue;
        }

        // Header tespiti
        if !header_found {
            let row_joined = record.iter().collect::<Vec<_>>().join(" ").to_lowercase();
            if row_joined.contains("tarih") {
                for (idx, col) in record.iter().enumerate() {
                    let col_lower = col.to_lowercase();
                    if col_lower.contains("tarih") {
                        date_col = idx;
                    } else if col_lower.contains("öğün") || col_lower.contains("ogun") {
                        meal_col = Some(idx);
                    } else if col_lower.contains("yemek")
                        || col_lower.contains("ürün")
                        || col_lower.contains("urun")
                    {
                        dish_col = idx;
                    } else if col_lower.contains("gramaj") || col_lower.contains("miktar") {
                        amount_col = Some(idx);
                    } else if col_lower.contains("kalori") || col_lower.contains("enerji") {
                        if col_lower.contains("toplam") {
                            total_cal_col = Some(idx);
                        } else {
                            cal_col = Some(idx);
                        }
                    }
                }
                header_found = true;
                continue;
            }
        }

        // Veri satırı
        let date_raw = match record.get(date_col) {
            Some(d) => d.trim(),
            None => continue,
        };

        let date_iso = match parse_date_string(date_raw) {
            Some(d) => d,
            None => continue,
        };

        let meal_type = if let Some(mc) = meal_col.and_then(|c| record.get(c)) {
            let m_lower = mc.to_lowercase();
            if m_lower.contains("kahvalt") {
                "breakfast"
            } else if m_lower.contains("akşam") || m_lower.contains("aksam") {
                "dinner"
            } else if m_lower.contains("öğle") || m_lower.contains("ogle") {
                "lunch"
            } else {
                doc_meal_type
            }
        } else {
            doc_meal_type
        };

        let dish_raw = match record.get(dish_col) {
            Some(d) => d.trim(),
            None => continue,
        };

        let dish_clean = dish_raw
            .trim_start_matches('*')
            .trim_start_matches('-')
            .trim_start_matches('•')
            .trim_start_matches('⁃')
            .trim();

        if dish_clean.is_empty() {
            continue;
        }

        let amount_val = amount_col
            .and_then(|c| record.get(c))
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "-")
            .map(|s| s.to_string());

        let cal_val = cal_col
            .and_then(|c| record.get(c))
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "-")
            .map(|s| s.to_string());

        let total_cal_val = total_cal_col
            .and_then(|c| record.get(c))
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "-")
            .map(|s| s.to_string());

        let dish_expanded = crate::parser::kykyemek::expand_dish_shorthands(dish_clean.to_string());
        // Alternatif yemekleri ayrıştır (örn: "Peynirli/Ispanaklı Börek")
        let alt_parts = split_outside_parens(&dish_expanded, '/');
        let alternatives: Vec<MenuComponent> =
            if alt_parts.len() > 1 && !dish_clean.starts_with("500 ml") {
                alt_parts
                    .into_iter()
                    .map(|part| {
                        let norm = crate::parser::normalizer::normalize_food_item(part.trim());
                        let comp_amount = amount_val.clone().or(norm.amount);
                        MenuComponent {
                            name: norm.name,
                            amount: comp_amount,
                            calories: cal_val.clone(),
                            category: None,
                        }
                    })
                    .collect()
            } else {
                let norm = crate::parser::normalizer::normalize_food_item(dish_clean);
                let comp_amount = amount_val.or(norm.amount);
                vec![MenuComponent {
                    name: norm.name,
                    amount: comp_amount,
                    calories: cal_val,
                    category: None,
                }]
            };

        let menu_item = MenuItem {
            takeaway_id: None,
            alternatives,
        };

        let day_data = menu_db.entry(date_iso).or_insert_with(|| DayData {
            metadata: Some(DayMetadata {
                trust_score: 100,
                status: "approved".to_string(),
                anomaly_score: Some(0.0),
                source_file: Some(file_name_hint.to_string()),
            }),
            ..Default::default()
        });

        match meal_type {
            "breakfast" => {
                if day_data.normal.breakfast_kcal.is_none() && total_cal_val.is_some() {
                    day_data.normal.breakfast_kcal = total_cal_val;
                }
                day_data.normal.breakfast.push(menu_item);
            }
            "lunch" => {
                if day_data.normal.lunch_kcal.is_none() && total_cal_val.is_some() {
                    day_data.normal.lunch_kcal = total_cal_val;
                }
                day_data.normal.lunch.push(menu_item);
            }
            _ => {
                if day_data.normal.dinner_kcal.is_none() && total_cal_val.is_some() {
                    day_data.normal.dinner_kcal = total_cal_val;
                }
                day_data.normal.dinner.push(menu_item);
            }
        }
    }

    if menu_db.is_empty() {
        anyhow::bail!("CSV tablosundan geçerli bir menü kaydı çıkarılamadı.");
    }

    for day_data in menu_db.values_mut() {
        crate::parser::validation::finalize_day_metadata(day_data);
    }

    Ok(ParsedDocumentPayload::DailyMenu(menu_db))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_grid_parser_afyon_sample() {
        let sample = r#"[BELGE_TURU: KAHVALTI]

[TABLO]
Tarih,Gün,Öğün,Yemek / Ürün,Gramaj,Kalori / Enerji,Günlük Toplam Kalori
14.09.2026,Pazartesi,Kahvaltı,Sade Omlet,1 adet L boy yumurta,115,
14.09.2026,Pazartesi,Kahvaltı,Patates Kızartması,150 g,470,
14.09.2026,Pazartesi,Kahvaltı,Simit,1 adet,300,
14.09.2026,Pazartesi,Kahvaltı,Beyaz Peynir,40 g,115,
14.09.2026,Pazartesi,Kahvaltı,*Siyah/Yeşil Zeytin,30 g,53,
14.09.2026,Pazartesi,Kahvaltı,Çeyrek Ekmek,,128,
14.09.2026,Pazartesi,Kahvaltı,500 ml Su,,,
14.09.2026,Pazartesi,Kahvaltı,**Çay/Bitki Çayı,,,
15.09.2026,Salı,Kahvaltı,Haşlanmış Yumurta,1 adet L boy,87,
15.09.2026,Salı,Kahvaltı,Menemen,150 g (1 adet L boy yumurta),106,
15.09.2026,Salı,Kahvaltı,Zeytinli/Peynirli Açma,1 adet,300,
15.09.2026,Salı,Kahvaltı,Kaşar Peynir,40 g,160,
15.09.2026,Salı,Kahvaltı,*Siyah/Yeşil Zeytin,30 g,53,
15.09.2026,Salı,Kahvaltı,Çeyrek Ekmek,,128,
15.09.2026,Salı,Kahvaltı,500 ml Su,,,
15.09.2026,Salı,Kahvaltı,**Çay/Bitki Çayı,,,

[DIPNOTLAR]
- *Siyah/Yeşil Zeytin verilen günlerde her iki çeşit de bulundurulacaktır.
- **Çay/Bitki Çayından her ikisi de bulundurulacaktır."#;

        let payload = parse_csv_grid_to_payload(sample, "test_afyon").unwrap();
        match payload {
            ParsedDocumentPayload::DailyMenu(db) => {
                assert_eq!(db.len(), 2);
                let day1 = db.get("2026-09-14").expect("14 Eylül bulunmalı");
                assert_eq!(day1.normal.breakfast.len(), 8);

                // Alternatifli yemek testi: Siyah/Yeşil Zeytin
                let zeytin = day1
                    .normal
                    .breakfast
                    .iter()
                    .find(|i| {
                        i.alternatives
                            .iter()
                            .any(|a| a.name.to_lowercase().contains("zeytin"))
                    })
                    .expect("Zeytin kalemi bulunmalı");
                assert!(zeytin.alternatives.len() >= 2);

                let day2 = db.get("2026-09-15").expect("15 Eylül bulunmalı");
                assert_eq!(day2.normal.breakfast.len(), 8);
            }
            _ => panic!("DailyMenu payload bekleniyordu"),
        }
    }
}

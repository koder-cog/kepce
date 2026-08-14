use std::sync::OnceLock;
use crate::parser::models::{MenuDatabase, MenuItem, MenuComponent, DayData, DayMetadata};
use crate::parser::validation;
use crate::parser::validation::MealType;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SheetGrid {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

pub fn parse_date_string(s: &str) -> Option<String> {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"(\d{1,2})[\./-](\d{1,2})[\./-](\d{4})").unwrap());
    if let Some(caps) = re.captures(s) {
        let day: u32 = caps[1].parse().unwrap_or(1);
        let month: u32 = caps[2].parse().unwrap_or(1);
        let year: i32 = caps[3].parse().unwrap_or(2026);
        return Some(format!("{:04}-{:02}-{:02}", year, month, day));
    }
    None
}

fn is_date_cell(s: &str) -> bool {
    parse_date_string(s).is_some()
}

pub fn split_outside_parens(s: &str, sep: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut parens = 0;
    
    for c in s.chars() {
        match c {
            '(' | '[' => { parens += 1; current.push(c); },
            ')' | ']' => { parens -= 1; current.push(c); },
            x if x == sep && parens == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            },
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

pub fn parse_grid(sheet: &SheetGrid, db: &mut MenuDatabase, file_name_hint: &str) {
    static RE_KCAL: OnceLock<regex::Regex> = OnceLock::new();
    let re_kcal = RE_KCAL.get_or_init(|| regex::Regex::new(r"(?i)(\d+\s*-\s*\d+)\s*(?:kcal|kkal|kalori)").unwrap());
    static RE_NUMS: OnceLock<regex::Regex> = OnceLock::new();
    let re_nums = RE_NUMS.get_or_init(|| regex::Regex::new(r"(\d+)").unwrap());

    let is_colyak = validation::is_colyak_sheet(&sheet.name);
    let mut meal_type_opt = validation::detect_meal_type(&sheet.name, &[]);

    let height = sheet.rows.len();
    if height == 0 {
        return;
    }

    // If meal_type is not obvious from sheet name, pre-detect it from the first few items
    if meal_type_opt.is_none() {
        let mut sample_items: Vec<String> = Vec::new();
        'detect: for r in 0..height {
            for c in 0..sheet.rows[r].len() {
                let cell = &sheet.rows[r][c];
                if parse_date_string(cell).is_some() {
                    let mut item_row = r + 1;
                    let mut empty_count = 0;
                    while item_row < height && empty_count < 2 {
                        let item_row_len = sheet.rows[item_row].len();
                        if c >= item_row_len {
                            empty_count += 1;
                            item_row += 1;
                            continue;
                        }
                        let raw_str = &sheet.rows[item_row][c];
                        if is_date_cell(raw_str) {
                            break;
                        }
                        let item_name = raw_str.trim()
                            .trim_start_matches('*')
                            .trim_start_matches('-')
                            .trim_start_matches('•')
                            .trim_start_matches('⁃')
                            .trim().to_string();
                        if item_name.is_empty() {
                            empty_count += 1;
                        } else {
                            empty_count = 0;
                            sample_items.push(item_name);
                            if sample_items.len() >= 10 {
                                break 'detect;
                            }
                        }
                        item_row += 1;
                    }
                }
            }
        }
        if !sample_items.is_empty() {
            meal_type_opt = validation::detect_meal_type("", &sample_items);
        }
    }

    let resolved_meal_type = match meal_type_opt {
        Some(mt) => mt,
        None => {
            tracing::warn!("SKIP sheet '{}': could not determine meal type from name or content", sheet.name);
            return;
        }
    };

    let mut sheet_kcal: Option<String> = None;
    
    for r in 0..height {
        for c in 0..sheet.rows[r].len() {
            let text = &sheet.rows[r][c];
            if let Some(caps) = re_kcal.captures(text) {
                sheet_kcal = Some(format!("{} kcal", caps[1].replace(" ", "")));
                break;
            }
        }
        if sheet_kcal.is_some() { break; }
    }

    for r in 0..height {
        let row_len = sheet.rows[r].len();
        for c in 0..row_len {
            let cell = &sheet.rows[r][c];
            
            if let Some(mut date_str) = parse_date_string(cell) {
                if (file_name_hint.to_uppercase().contains("HAZIRAN") || file_name_hint.to_uppercase().contains("HAZİRAN"))
                    && date_str == "2026-05-04" {
                        date_str = "2026-06-04".to_string();
                }

                if !validation::validate_date_range(&date_str) {
                    tracing::warn!("SKIP date: {} is outside valid range (±2 years)", date_str);
                    continue;
                }
                
                let mut has_gramaj = false;
                let mut has_enerji = false;
                
                if c + 1 < row_len {
                    let v = sheet.rows[r][c + 1].to_uppercase();
                    if v.contains("GRAMAJ") { has_gramaj = true; }
                    if v.contains("ENERJİ") || v.contains("ENERJI") { has_enerji = true; }
                }
                if c + 2 < row_len && has_gramaj {
                    let v = sheet.rows[r][c + 2].to_uppercase();
                    if v.contains("ENERJİ") || v.contains("ENERJI") { has_enerji = true; }
                }

                let mut item_row = r + 1;
                let mut empty_count = 0;
                let mut item_count: usize = 0;
                
                while item_row < height {
                    let item_row_len = sheet.rows[item_row].len();
                    if c >= item_row_len {
                        empty_count += 1;
                        item_row += 1;
                        if empty_count >= 2 { break; }
                        continue;
                    }

                    let raw_str = &sheet.rows[item_row][c];
                    let raw_upper = raw_str.to_uppercase();
                    
                    let is_takeaway = raw_upper.contains("AL GÖTÜR") || raw_upper.contains("ALGÖTÜR") || raw_upper.contains("AL-GÖTÜR") || raw_upper.contains("AL GÖTUR") || raw_upper.contains("ALGÖTUR");
                    
                    let mut item_name = raw_str.trim()
                        .trim_start_matches('*')
                        .trim_start_matches('-')
                        .trim_start_matches('•')
                        .trim_start_matches('⁃')
                        .trim().to_string();
                    
                    if item_name.contains("Cay") {
                        item_name = item_name.replace("Cay", "Çay");
                    } else if item_name.contains("cay") {
                        item_name = item_name.replace("cay", "çay");
                    }

                    let item_name = match validation::validate_item_name(&item_name) {
                        Some(name) => name,
                        None if !item_name.trim().is_empty() => {
                            tracing::warn!("SKIP item: name too long ({} chars): {}...", item_name.len(), &item_name[..50.min(item_name.len())]);
                            item_row += 1;
                            continue;
                        },
                        None => item_name
                    };

                    let mut amount_str = String::new();
                    let amount_col = if has_gramaj { c + 1 } else { c };
                    if has_gramaj && amount_col < item_row_len {
                        amount_str = sheet.rows[item_row][amount_col].clone();
                    }

                    let lower_name = item_name.to_lowercase();
                    let lower_amt = amount_str.to_lowercase();
                    if lower_name.contains("hazırlanıp") || lower_name.contains("sunulacaktır") || lower_name.contains("garnitür") || lower_name.contains("ortalama") ||
                       lower_amt.contains("hazırlanıp") || lower_amt.contains("sunulacaktır") || lower_amt.contains("garnitür") || lower_amt.contains("ortalama") {
                        item_row += 1;
                        continue;
                    }

                    if is_date_cell(raw_str) || empty_count >= 2 {
                        break;
                    }
                    
                    if item_name.is_empty() {
                        empty_count += 1;
                        item_row += 1;
                        continue;
                    } else {
                        empty_count = 0;
                    }

                    item_count += 1;
                    if !validation::validate_meal_item_count(item_count) {
                        tracing::warn!("WARN: meal for {} exceeded max items ({}), truncating", date_str, item_count);
                        break;
                    }

                    let day_data = db.entry(date_str.clone()).or_insert_with(|| DayData {
                        metadata: Some(DayMetadata {
                            trust_score: 100,
                            anomaly_score: Some(0.0),
                            status: "approved".to_string(),
                            source_file: Some(file_name_hint.to_string()),
                        }),
                        ..Default::default()
                    });
                    let menu_ref = if is_colyak { &mut day_data.colyak } else { &mut day_data.normal };
                    
                    let target_list = match resolved_meal_type {
                        MealType::Breakfast => {
                            if menu_ref.breakfast_kcal.is_none() {
                                menu_ref.breakfast_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.breakfast
                        }
                        MealType::Lunch => {
                            if menu_ref.lunch_kcal.is_none() {
                                menu_ref.lunch_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.lunch
                        }
                        MealType::Dinner => {
                            if menu_ref.dinner_kcal.is_none() {
                                menu_ref.dinner_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.dinner
                        }
                    };

                    if is_takeaway {
                        let mut has_nums = false;
                        for cap in re_nums.captures_iter(&item_name) {
                            has_nums = true;
                            let num = &cap[1];
                            let num_str = num.to_string();
                            if !target_list.iter().any(|i| i.takeaway_id.as_deref() == Some(&num_str)) {
                                target_list.push(MenuItem {
                                    takeaway_id: Some(num_str.clone()),
                                    alternatives: vec![MenuComponent {
                                        name: format!("Al-Götür Menü {}", num),
                                        amount: None,
                                        calories: None,
                                        category: None,
                                    }],
                                });
                            }
                        }
                        if !has_nums && !target_list.iter().any(|i| i.takeaway_id.as_deref() == Some("1")) {
                            target_list.push(MenuItem {
                                takeaway_id: Some("1".to_string()),
                                alternatives: vec![MenuComponent {
                                    name: "Al-Götür Menü 1".to_string(),
                                    amount: None,
                                    calories: None,
                                    category: None,
                                }],
                            });
                        }
                    } else {
                        let mut amount = None;
                        let mut calories = None;

                        let amount_col = if has_gramaj { c + 1 } else { c }; 
                        let cal_col = if has_enerji { if has_gramaj { c + 2 } else { c + 1 } } else { c + 999 };

                        if has_gramaj && amount_col < item_row_len {
                            let a = &sheet.rows[item_row][amount_col];
                            amount = validation::validate_numeric_value(a);
                        }

                        if has_enerji && cal_col < item_row_len {
                            let cal = &sheet.rows[item_row][cal_col];
                            calories = validation::validate_numeric_value(cal);
                        }

                        let mut alternatives = Vec::new();
                        let names: Vec<String> = split_outside_parens(&item_name, '/');
                        
                        let amounts: Vec<String> = if let Some(ref a) = amount {
                            split_outside_parens(a, '/')
                        } else {
                            Vec::new()
                        };
                        
                        let cals: Vec<String> = if let Some(ref cal) = calories {
                            split_outside_parens(cal, '/')
                        } else {
                            Vec::new()
                        };

                        for (i, name) in names.iter().enumerate() {
                            let amt = if amounts.len() > i {
                                Some(amounts[i].to_string())
                            } else if amounts.len() == 1 {
                                Some(amounts[0].to_string())
                            } else {
                                None
                            };

                            let cal_val = if cals.len() > i {
                                Some(cals[i].to_string())
                            } else if cals.len() == 1 {
                                Some(cals[0].to_string())
                            } else {
                                None
                            };

                            alternatives.push(MenuComponent {
                                name: name.to_string(),
                                amount: amt,
                                calories: cal_val,
                                category: None,
                            });
                        }

                        if alternatives.is_empty() {
                            alternatives.push(MenuComponent {
                                name: item_name.clone(),
                                amount,
                                calories,
                                category: None,
                            });
                        }

                        target_list.push(MenuItem {
                            takeaway_id: None,
                            alternatives,
                        });
                    }

                    item_row += 1;
                }
            }
        }
    }
}

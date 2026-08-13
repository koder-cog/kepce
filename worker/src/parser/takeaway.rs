use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

type TakeawayCacheMap = HashMap<String, HashMap<u32, Vec<Vec<crate::parser::models::MenuComponent>>>>;

lazy_static::lazy_static! {
    pub(crate) static ref TAKEAWAY_CACHE: RwLock<TakeawayCacheMap> = RwLock::new(HashMap::new());
}

#[derive(Debug, Deserialize, Clone)]
struct MenuItem {
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct MenuSlot {
    alternatives: Vec<MenuItem>,
}

pub fn parse_takeaway_menu(text: &str, city_slug: &str, meal_type: &str) -> Option<Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>> {
    let text_upper = text.to_uppercase();
    if !text_upper.contains("AL GÖTÜR") && !text_upper.contains("AL-GÖTÜR") {
        return None;
    }
    
    let mut menu_ids = Vec::new();
    let mut current_num = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if !current_num.is_empty() {
            if let Ok(num) = current_num.parse::<u32>() {
                menu_ids.push(num);
            }
            current_num.clear();
        }
    }
    if !current_num.is_empty() {
        if let Ok(num) = current_num.parse::<u32>() {
            menu_ids.push(num);
        }
    }
    
    if menu_ids.is_empty() {
        // Numara bulunamadıysa varsayılan 1. paket say
        menu_ids.push(1);
    }
    
    let mapped_meal_type = match meal_type {
        "breakfast" | "kahvalti" | "kahvaltı" => "breakfast",
        "lunch" | "ogle" | "öğle" => "lunch",
        "dinner" | "aksam" | "akşam" => "dinner",
        other => other,
    };
    let key = format!("{}_{}", city_slug, mapped_meal_type);
    let config = {
        if let Ok(cache) = TAKEAWAY_CACHE.read() {
            cache.get(&key).cloned()
        } else { None }
    }.or_else(|| {
        let file_path = format!("config/takeaway/{}.json", key);
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(parsed) = serde_json::from_str::<HashMap<u32, Vec<MenuSlot>>>(&content) {
                let mut converted: HashMap<u32, Vec<Vec<crate::parser::models::MenuComponent>>> = HashMap::new();
                for (id, slots) in parsed {
                    let mut dish_slots = Vec::new();
                    for slot in slots {
                        let mut alts = Vec::new();
                        for alt in slot.alternatives {
                            alts.push(crate::parser::models::MenuComponent {
                                name: alt.name,
                                amount: None,
                                calories: None,
                                category: None,
                            });
                        }
                        dish_slots.push(alts);
                    }
                    converted.insert(id, dish_slots);
                }
                
                if let Ok(mut cache) = TAKEAWAY_CACHE.write() {
                    cache.insert(key.clone(), converted.clone());
                }
                return Some(converted);
            }
        }
        None
    });
    
    let mut packages = Vec::new();
    
    for id in menu_ids {
        if let Some(ref cfg) = config {
            if let Some(menu_slots) = cfg.get(&id) {
                packages.push((format!("Al Götür {}", id), menu_slots.clone()));
                continue;
            }
        }
        // Dinamik fallback paketi
        let default_slots = vec![vec![crate::parser::models::MenuComponent {
            name: format!("Al-Götür Menü {}", id),
            amount: None,
            calories: None,
            category: None,
        }]];
        packages.push((format!("Al Götür {}", id), default_slots));
    }
    
    if packages.is_empty() {
        None
    } else {
        Some(packages)
    }
}

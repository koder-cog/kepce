use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TakeawayParsedPackage {
    pub name: String,
    pub slots: Vec<Vec<crate::parser::models::MenuComponent>>,
}

type TakeawayCacheMap = HashMap<String, HashMap<u32, TakeawayParsedPackage>>;

lazy_static::lazy_static! {
    pub(crate) static ref TAKEAWAY_CACHE: RwLock<TakeawayCacheMap> = RwLock::new(HashMap::new());
}

#[derive(Debug, Deserialize, Clone)]
struct MenuItem {
    name: String,
    #[serde(default)]
    gramaj: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct MenuSlot {
    alternatives: Vec<MenuItem>,
}

#[derive(Debug, Deserialize, Clone)]
struct TakeawayPackageConfig {
    id: u32,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    items: Vec<MenuSlot>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct TakeawayConfigFile {
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    meal_type: Option<String>,
    #[serde(default)]
    valid_from: Option<String>,
    #[serde(default)]
    valid_to: Option<String>,
    #[serde(default)]
    packages: Vec<TakeawayPackageConfig>,
}

pub fn parse_takeaway_menu(
    text: &str,
    city_slug: &str,
    meal_type: &str,
) -> Option<Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>> {
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
        } else {
            None
        }
    }
    .or_else(|| {
        let file_path = format!("config/takeaway/{}.json", key);
        if let Ok(content) = fs::read_to_string(&file_path) {
            let mut converted: HashMap<u32, TakeawayParsedPackage> = HashMap::new();

            // 1. Yeni format (TakeawayConfigFile) dene
            if let Ok(parsed_file) = serde_json::from_str::<TakeawayConfigFile>(&content) {
                if !parsed_file.packages.is_empty() {
                    for pkg in parsed_file.packages {
                        let mut dish_slots = Vec::new();
                        for slot in pkg.items {
                            let mut alts = Vec::new();
                            for alt in slot.alternatives {
                                alts.push(crate::parser::models::MenuComponent {
                                    name: alt.name,
                                    amount: alt.gramaj,
                                    calories: None,
                                    category: None,
                                });
                            }
                            dish_slots.push(alts);
                        }

                        let pkg_name = if let Some(ref t) = pkg.title {
                            if !t.trim().is_empty() {
                                format!("{}. {}", pkg.id, t.trim())
                            } else {
                                format!("Al Götür {}", pkg.id)
                            }
                        } else {
                            format!("Al Götür {}", pkg.id)
                        };

                        converted.insert(
                            pkg.id,
                            TakeawayParsedPackage {
                                name: pkg_name,
                                slots: dish_slots,
                            },
                        );
                    }
                }
            } else if let Ok(legacy_map) = serde_json::from_str::<HashMap<u32, Vec<MenuSlot>>>(&content) {
                // 2. Eski format (HashMap<u32, Vec<MenuSlot>>) fallback
                for (id, slots) in legacy_map {
                    let mut dish_slots = Vec::new();
                    for slot in slots {
                        let mut alts = Vec::new();
                        for alt in slot.alternatives {
                            alts.push(crate::parser::models::MenuComponent {
                                name: alt.name,
                                amount: alt.gramaj,
                                calories: None,
                                category: None,
                            });
                        }
                        dish_slots.push(alts);
                    }
                    converted.insert(
                        id,
                        TakeawayParsedPackage {
                            name: format!("Al Götür {}", id),
                            slots: dish_slots,
                        },
                    );
                }
            }

            if !converted.is_empty() {
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
            if let Some(parsed_pkg) = cfg.get(&id) {
                packages.push((parsed_pkg.name.clone(), parsed_pkg.slots.clone()));
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

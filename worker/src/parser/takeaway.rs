use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::{OnceLock, RwLock};

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

fn load_config_file(file_path: &str) -> Option<HashMap<u32, TakeawayParsedPackage>> {
    let content = fs::read_to_string(file_path)
        .or_else(|_| fs::read_to_string(format!("../{}", file_path)))
        .ok()?;
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
        Some(converted)
    } else {
        None
    }
}

pub fn get_takeaway_config(city_slug: &str, mapped_meal_type: &str) -> Option<HashMap<u32, TakeawayParsedPackage>> {
    let key = format!("{}_{}", city_slug, mapped_meal_type);
    
    if let Ok(cache) = TAKEAWAY_CACHE.read() {
        if let Some(cached) = cache.get(&key) {
            return Some(cached.clone());
        }
    }

    let turkish_meal_type = match mapped_meal_type {
        "breakfast" => "kahvalti",
        "dinner" => "aksam",
        other => other,
    };

    let candidates = vec![
        format!("config/takeaway/{}_{}.json", city_slug, mapped_meal_type),
        format!("config/takeaway/{}_{}.json", city_slug, turkish_meal_type),
    ];

    for path in candidates {
        if let Some(loaded) = load_config_file(&path) {
            if let Ok(mut cache) = TAKEAWAY_CACHE.write() {
                cache.insert(key.clone(), loaded.clone());
            }
            return Some(loaded);
        }
    }

    None
}

pub fn parse_takeaway_menu(
    text: &str,
    city_slug: &str,
    meal_type: &str,
) -> Option<Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>> {
    let text_upper = text.to_uppercase();
    if !text_upper.contains("AL GÖTÜR") && !text_upper.contains("AL-GÖTÜR") && !text_upper.contains("ALGÖTÜR") && !text_upper.contains("FAST") {
        return None;
    }

    let mapped_meal_type = match meal_type {
        "breakfast" | "kahvalti" | "kahvaltı" => "breakfast",
        "lunch" | "ogle" | "öğle" => "lunch",
        "dinner" | "aksam" | "akşam" => "dinner",
        other => other,
    };

    // Sadece bu şehre ait tanımlı bir şablon varsa paketleri yükle
    let config = get_takeaway_config(city_slug, mapped_meal_type)?;

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

    // Eğer numara bulunamadıysa ama metin config'deki başlıklardan birini içeriyorsa
    if menu_ids.is_empty() {
        for (id, pkg) in config.iter() {
            let pkg_name_upper = pkg.name.to_uppercase();
            if text_upper.contains(&pkg_name_upper) {
                menu_ids.push(*id);
            }
        }
    }

    if menu_ids.is_empty() {
        // Numara bulunamadıysa varsayılan 1. paket say
        menu_ids.push(1);
    }

    let mut packages = Vec::new();

    for id in menu_ids {
        if let Some(parsed_pkg) = config.get(&id) {
            packages.push((parsed_pkg.name.clone(), parsed_pkg.slots.clone()));
        }
    }

    if packages.is_empty() {
        None
    } else {
        Some(packages)
    }
}

pub fn parse_fast_menu_foods_html(html_str: &str) -> Vec<Vec<crate::parser::models::MenuComponent>> {
    use scraper::{Html, Selector};
    let fragment = Html::parse_fragment(html_str);
    let div_selector = Selector::parse("div").unwrap();
    let mut slots = Vec::new();

    static RE_SPLIT: OnceLock<regex::Regex> = OnceLock::new();
    let re_split = RE_SPLIT.get_or_init(|| regex::Regex::new(r"(?i)\s*veya\s*").unwrap());

    static RE_GRAMAJ: OnceLock<regex::Regex> = OnceLock::new();
    let re_gramaj = RE_GRAMAJ.get_or_init(|| regex::Regex::new(r"\(([^)]+)\)$").unwrap());

    for div in fragment.select(&div_selector) {
        let raw_text: String = div.text().collect();
        let decoded = decode_html_entities(&raw_text).trim().to_string();
        if decoded.is_empty() {
            continue;
        }

        let parts: Vec<&str> = re_split.split(&decoded).collect();
        let mut slot_alts = Vec::new();

        for part in parts {
            let part_trimmed = part.trim();
            if part_trimmed.is_empty() {
                continue;
            }

            let (name, amount) = if let Some(caps) = re_gramaj.captures(part_trimmed) {
                let gramaj = caps.get(1).map(|m| m.as_str().trim().to_string());
                let match_start = caps.get(0).unwrap().start();
                let clean_name = part_trimmed[..match_start].trim().to_string();
                (clean_name, gramaj)
            } else {
                (part_trimmed.to_string(), None)
            };

            slot_alts.push(crate::parser::models::MenuComponent {
                name,
                amount,
                calories: None,
                category: None,
            });
        }

        if !slot_alts.is_empty() {
            slots.push(slot_alts);
        }
    }

    slots
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
     .replace("&lt;", "<")
     .replace("&gt;", ">")
     .replace("&quot;", "\"")
     .replace("&#x15F;", "ş")
     .replace("&#x15E;", "Ş")
     .replace("&#x131;", "ı")
     .replace("&#x130;", "İ")
     .replace("&#x11F;", "ğ")
     .replace("&#x11E;", "Ğ")
     .replace("&#xE7;", "ç")
     .replace("&#xC7;", "Ç")
     .replace("&#xFC;", "ü")
     .replace("&#xDC;", "Ü")
     .replace("&#xF6;", "ö")
     .replace("&#xD6;", "Ö")
     .replace("&#x2B;", "+")
     .replace("&#39;", "'")
     .replace("&apos;", "'")
}


use chrono::NaiveDate;
use scraper::{Html, Selector};
use shared::services::content_guard::ContentGuard;

pub struct KykMenuParseResult {
    pub date: NaiveDate,
    pub dishes: Vec<Vec<crate::parser::models::MenuComponent>>,
    pub takeaways: Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>,
}

pub fn parse_kyk_html(html_content: &str, city_slug: &str, meal_type: &str) -> Vec<KykMenuParseResult> {
    let document = Html::parse_document(html_content);
    let card_selector = Selector::parse(".cardStyle").unwrap();
    let date_selector = Selector::parse("p.date, p.cardDate, .cardDate, p[id^='date_']").unwrap();
    let fallback_date_selector = Selector::parse(".card-header span").unwrap();
    let body_selector = Selector::parse(".card-body").unwrap();
    let p_selector = Selector::parse("p").unwrap();
    
    let mut results = Vec::new();
    
    for card in document.select(&card_selector) {
        let date_str = match card.select(&date_selector).next().or_else(|| card.select(&fallback_date_selector).next()) {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => continue,
        };
        
        let date_val = match parse_turkish_date(&date_str) {
            Some(d) => d,
            None => continue,
        };
        
        let body = match card.select(&body_selector).next() {
            Some(b) => b,
            None => continue,
        };
        
        let mut raw_dishes = Vec::new();
        let mut takeaways = Vec::new();
        for p in body.select(&p_selector) {
            let text_nodes: Vec<&str> = p.text().collect();
            let text = text_nodes.join(" / ").trim().to_string();
            let text_lower = text.to_lowercase();
            
            if ContentGuard::is_junk_dish_text(&text) {
                continue;
            }
            
            if p.value().attr("data-fastmenus").is_some() || p.value().attr("onclick").map(|o| o.contains("showFastMenu")).unwrap_or(false) {
                continue;
            }
            
            if text_lower.contains("al götür") || text_lower.contains("al-götür") || text_lower.contains("algötür") || text_lower.contains("al gotur") {
                if let Some(mut pkgs) = crate::parser::takeaway::parse_takeaway_menu(&text, city_slug, meal_type) {
                    takeaways.append(&mut pkgs);
                }
                continue;
            }
            
            // Düzeltmeler (Shorthand expansions) ve Çöp Filtresi
            let dish_group = clean_and_split_dish(text);
            
            if !dish_group.is_empty() {
                raw_dishes.push(dish_group);
            }
        }

        // Fastmenu / Al Götür buton ve özniteliklerini de tara (data-fastmenus veya onclick)
        let btn_selector = Selector::parse("[data-fastmenus], [onclick*='showFastMenu'], button, a, p").unwrap();
        for btn in card.select(&btn_selector) {
            if let Some(fast_json) = btn.value().attr("data-fastmenus") {
                if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(fast_json) {
                    for item in items {
                        let name_val = item.get("name").or_else(|| item.get("title")).and_then(|t| t.as_str()).unwrap_or("Al Götür");
                        if let Some(mut pkgs) = crate::parser::takeaway::parse_takeaway_menu(name_val, city_slug, meal_type) {
                            takeaways.append(&mut pkgs);
                        }
                    }
                }
            }
            if let Some(onclick) = btn.value().attr("onclick") {
                if onclick.contains("showFastMenu") {
                    if let Some(mut pkgs) = crate::parser::takeaway::parse_takeaway_menu(onclick, city_slug, meal_type) {
                        takeaways.append(&mut pkgs);
                    }
                }
            }
        }
        let mut seen_takeaways = std::collections::HashSet::new();
        takeaways.retain(|(pkg_name, _)| seen_takeaways.insert(pkg_name.clone()));
        
        if !raw_dishes.is_empty() {
            results.push(KykMenuParseResult {
                date: date_val,
                dishes: raw_dishes,
                takeaways,
            });
        }
    }
    
    results
}

pub fn clean_and_split_dish(mut text: String) -> Vec<crate::parser::models::MenuComponent> {
    if ContentGuard::is_junk_dish_text(&text) {
        return Vec::new();
    }

    text = text.replace("Siyah/Yeşil Zeytin", "Siyah Zeytin / Yeşil Zeytin")
               .replace("Siyah / Yeşil Zeytin", "Siyah Zeytin / Yeşil Zeytin")
               .replace("Yeşil/Siyah Zeytin", "Yeşil Zeytin / Siyah Zeytin")
               .replace("Yeşil / Siyah Zeytin", "Yeşil Zeytin / Siyah Zeytin")
               .replace("Zeytinli/Peynirli Açma", "Zeytinli Açma / Peynirli Açma")
               .replace("Peynirli/Zeytinli Açma", "Peynirli Açma / Zeytinli Açma")
               .replace("Zeytinli/Peynirli Poğaça", "Zeytinli Poğaça / Peynirli Poğaça")
               .replace("Peynirli/Zeytinli Poğaça", "Peynirli Poğaça / Zeytinli Poğaça")
               .replace("Kakaolu/Sade Tahin Helvası", "Kakaolu Tahin Helvası / Sade Tahin Helvası")
               .replace("Sade/Kakaolu Tahin Helvası", "Sade Tahin Helvası / Kakaolu Tahin Helvası")
               .replace("Kakaolu / Sade Tahin Helvası", "Kakaolu Tahin Helvası / Sade Tahin Helvası")
               .replace("Sade / Kakaolu Tahin Helvası", "Sade Tahin Helvası / Kakaolu Tahin Helvası");
    
    let parts: Vec<&str> = text.split('/').collect();
    let mut dish_group = Vec::new();
    for part in parts {
        let cleaned = part.trim().to_string();
        if !cleaned.is_empty() && !ContentGuard::is_junk_dish_text(&cleaned) {
            dish_group.push(crate::parser::models::MenuComponent {
                name: cleaned,
                amount: None,
                calories: None,
                category: None,
            });
        }
    }
    dish_group
}

pub(crate) fn parse_turkish_date(date_str: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() >= 3 {
        let day: u32 = parts[0].parse().ok()?;
        let month_name = parts[1];
        let year: i32 = parts[2].parse().ok()?;
        
        let month = match month_name.to_lowercase().as_str() {
            "ocak" => 1, "şubat" => 2, "mart" => 3, "nisan" => 4,
            "mayıs" => 5, "haziran" => 6, "temmuz" => 7, "ağustos" => 8,
            "eylül" => 9, "ekim" => 10, "kasım" => 11, "aralık" => 12,
            _ => return None,
        };
        NaiveDate::from_ymd_opt(year, month, day)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyk_menu_parsing() {
        let html = r#"
            <div class="cardStyle">
                <p class="date">15 Mayıs 2026</p>
                <div class="card-body">
                    <p>Tavuk Izgara<br>Mevsim Türlü</p>
                    <p>Etsiz Karışık Dolma Veya Sarma+Yoğurt</p>
                    <p>Mevsim Türlü / Tavuk Hamburger Köfte+Turşu+Marul+Domates+Patates Cips</p>
                    <p>Siyah/Yeşil Zeytin</p>
                </div>
            </div>
        "#;
        
        let results = parse_kyk_html(html, "test-city", "dinner");
        assert_eq!(results.len(), 1);
        let dishes = &results[0].dishes;
        
        // p1: <p>Tavuk Izgara<br>Mevsim Türlü</p> -> split into 2 because of <br> joining with " / "
        assert_eq!(dishes[0], vec!["Tavuk Izgara", "Mevsim Türlü"]);
        
        // p2: <p>Etsiz Karışık Dolma Veya Sarma+Yoğurt</p> -> NOT split by " Veya "
        assert_eq!(dishes[1], vec!["Etsiz Karışık Dolma Veya Sarma+Yoğurt"]);
        
        // p3: <p>Mevsim Türlü / Tavuk Hamburger Köfte+Turşu+Marul+Domates+Patates Cips</p>
        assert_eq!(dishes[2], vec!["Mevsim Türlü", "Tavuk Hamburger Köfte+Turşu+Marul+Domates+Patates Cips"]);
        
        // p4: <p>Siyah/Yeşil Zeytin</p> -> replaced and split
        assert_eq!(dishes[3], vec!["Siyah Zeytin", "Yeşil Zeytin"]);
    }
}

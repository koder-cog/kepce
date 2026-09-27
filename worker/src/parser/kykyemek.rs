use chrono::NaiveDate;
use scraper::{Html, Selector};
use shared::services::content_guard::ContentGuard;

use shared::entities::sea_orm_active_enums::MealTypeEnum;

pub struct KykyemekParseResult {
    pub date: NaiveDate,
    pub dishes: Vec<Vec<crate::parser::models::MenuComponent>>,
    pub takeaways: Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>,
    pub min_calories: Option<i32>,
    pub max_calories: Option<i32>,
    pub detected_meal: Option<MealTypeEnum>,
}

pub type KykMenuParseResult = KykyemekParseResult;

/// "1100-1500 kalori" veya "850 kcal" -> (Some(1100), Some(1500))
pub fn parse_kcal_range(meta: Option<&str>) -> (Option<i32>, Option<i32>) {
    static NUM_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = NUM_RE.get_or_init(|| regex::Regex::new(r"\d+").unwrap());
    let Some(meta) = meta else {
        return (None, None);
    };
    let nums: Vec<i32> = re
        .find_iter(meta)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();
    match nums.as_slice() {
        [min, max] => (Some(*min), Some(*max)),
        [single] => (Some(*single), Some(*single)),
        _ => (None, None),
    }
}

pub fn parse_kykyemek_html(
    html_content: &str,
    city_slug: &str,
    meal_type: &str,
) -> Vec<KykyemekParseResult> {
    let document = Html::parse_document(html_content);
    let card_selector = Selector::parse(".cardStyle").unwrap();
    let date_selector = Selector::parse("p.date, p.cardDate, .cardDate, p[id^='date_']").unwrap();
    let fallback_date_selector = Selector::parse(".card-header span").unwrap();
    let body_selector = Selector::parse(".card-body").unwrap();
    let p_selector = Selector::parse("p").unwrap();
    let cal_selector = Selector::parse(".card-body p, p.text-end, p").unwrap();

    let mut results = Vec::new();

    for card in document.select(&card_selector) {
        let date_str = match card
            .select(&date_selector)
            .next()
            .or_else(|| card.select(&fallback_date_selector).next())
        {
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

        let mut min_calories = None;
        let mut max_calories = None;
        for p in card.select(&cal_selector) {
            let p_text = p.text().collect::<String>().to_lowercase();
            if p_text.contains("kalori") || p_text.contains("kcal") {
                let (min_c, max_c) = parse_kcal_range(Some(&p_text));
                if min_c.is_some() || max_c.is_some() {
                    min_calories = min_c;
                    max_calories = max_c;
                    break;
                }
            }
        }

        let mut raw_dishes = Vec::new();
        let mut takeaways = Vec::new();
        static RICH_SPAN_SEL: std::sync::OnceLock<Selector> = std::sync::OnceLock::new();
        let rich_sel = RICH_SPAN_SEL.get_or_init(|| {
            Selector::parse("span.slash-separator, span.food-main-text, span.food-detail-text")
                .unwrap()
        });

        for p in body.select(&p_selector) {
            // Standart ikram veya bağımsız detay paragrafları (su, çeyrek ekmek vb.)
            let is_standalone_detail = p.value().classes().any(|c| c == "food-detail-text")
                && !p.value().classes().any(|c| c == "food-main-text");
            if is_standalone_detail {
                continue;
            }

            if p.value().attr("data-fastmenus").is_some()
                || p.value()
                    .attr("onclick")
                    .map(|o| o.contains("showFastMenu"))
                    .unwrap_or(false)
            {
                continue;
            }

            let text_nodes: Vec<&str> = p.text().collect();
            let text = text_nodes.join(" / ").trim().to_string();
            let text_lower = text.to_lowercase();

            if ContentGuard::is_junk_dish_text(&text) {
                continue;
            }

            if text_lower.contains("al götür")
                || text_lower.contains("al-götür")
                || text_lower.contains("algötür")
                || text_lower.contains("al gotur")
            {
                if let Some(mut pkgs) =
                    crate::parser::takeaway::parse_takeaway_menu(&text, city_slug, meal_type)
                {
                    takeaways.append(&mut pkgs);
                }
                continue;
            }

            let dish_group = if p.select(rich_sel).next().is_some() {
                parse_rich_dish_paragraph(p)
            } else {
                clean_and_split_dish(text)
            };

            if !dish_group.is_empty() {
                raw_dishes.push(dish_group);
            }
        }

        // Fastmenu / Al Götür buton ve özniteliklerini de tara (data-fastmenus veya onclick)
        let btn_selector =
            Selector::parse("[data-fastmenus], [onclick*='showFastMenu'], button, a, p").unwrap();
        for btn in card.select(&btn_selector) {
            if let Some(fast_json) = btn.value().attr("data-fastmenus") {
                if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(fast_json) {
                    for item in items {
                        let id_val = item.get("id").and_then(|v| v.as_str());
                        let name_val = item
                            .get("name")
                            .or_else(|| item.get("title"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("Al Götür");

                        let mut resolved = false;
                        if let Some(uuid) = id_val {
                            if let Some(slots) = crate::parser::takeaway::get_cached_fastmenu(uuid)
                            {
                                takeaways.push((name_val.to_string(), slots));
                                resolved = true;
                            }
                        }

                        if !resolved {
                            if let Some(mut pkgs) = crate::parser::takeaway::parse_takeaway_menu(
                                name_val, city_slug, meal_type,
                            ) {
                                takeaways.append(&mut pkgs);
                            }
                        }
                    }
                }
            }
            if let Some(onclick) = btn.value().attr("onclick") {
                if onclick.contains("showFastMenu") {
                    if let Some(mut pkgs) =
                        crate::parser::takeaway::parse_takeaway_menu(onclick, city_slug, meal_type)
                    {
                        takeaways.append(&mut pkgs);
                    }
                }
            }
        }
        let mut seen_takeaways = std::collections::HashSet::new();
        takeaways.retain(|(pkg_name, _)| seen_takeaways.insert(pkg_name.clone()));

        let card_html = card.html();
        let detected_meal = if card_html.contains("'Dinner'")
            || card_html.contains("\"Dinner\"")
            || card_html.contains("Akşam Yemeği")
            || card_html.contains("Aksam Yemegi")
        {
            Some(MealTypeEnum::Dinner)
        } else if card_html.contains("'Breakfast'")
            || card_html.contains("\"Breakfast\"")
            || card_html.contains("Kahvaltı")
            || card_html.contains("Kahvalti")
        {
            Some(MealTypeEnum::Breakfast)
        } else {
            None
        };

        if !raw_dishes.is_empty() {
            results.push(KykMenuParseResult {
                date: date_val,
                dishes: raw_dishes,
                takeaways,
                min_calories,
                max_calories,
                detected_meal,
            });
        }
    }

    results
}

pub use parse_kykyemek_html as parse_kyk_html;

/// HTML içeriğindeki tüm kartlardan benzersiz fastmenu (id, name) ikililerini toplar.
/// Scraper bu listeyi kullanarak henüz önbellekte olmayan UUID'leri tek seferde çeker.
pub fn extract_fastmenu_items(html_content: &str) -> Vec<(String, String)> {
    let document = Html::parse_document(html_content);
    let selector = match Selector::parse("[data-fastmenus]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut items_map = std::collections::HashMap::new();

    for el in document.select(&selector) {
        if let Some(fast_json) = el.value().attr("data-fastmenus") {
            if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(fast_json) {
                for item in items {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        let id_str = id.trim().to_string();
                        if !id_str.is_empty() {
                            let name = item
                                .get("name")
                                .or_else(|| item.get("title"))
                                .and_then(|t| t.as_str())
                                .unwrap_or("Al Götür")
                                .trim()
                                .to_string();
                            items_map.entry(id_str).or_insert(name);
                        }
                    }
                }
            }
        }
    }

    items_map.into_iter().collect()
}

pub fn expand_dish_shorthands(mut text: String) -> String {
    text = text
        .replace("Siyah/Yeşil Zeytin", "Siyah Zeytin / Yeşil Zeytin")
        .replace("Siyah / Yeşil Zeytin", "Siyah Zeytin / Yeşil Zeytin")
        .replace("Yeşil/Siyah Zeytin", "Yeşil Zeytin / Siyah Zeytin")
        .replace("Yeşil / Siyah Zeytin", "Yeşil Zeytin / Siyah Zeytin")
        .replace("Zeytinli/Peynirli Açma", "Zeytinli Açma / Peynirli Açma")
        .replace("Peynirli/Zeytinli Açma", "Peynirli Açma / Zeytinli Açma")
        .replace(
            "Zeytinli/Peynirli Poğaça",
            "Zeytinli Poğaça / Peynirli Poğaça",
        )
        .replace(
            "Peynirli/Zeytinli Poğaça",
            "Peynirli Poğaça / Zeytinli Poğaça",
        )
        .replace(
            "Peynirli/Ispanaklı Börek",
            "Peynirli Börek / Ispanaklı Börek",
        )
        .replace(
            "Ispanaklı/Peynirli Börek",
            "Ispanaklı Börek / Peynirli Börek",
        )
        .replace(
            "Kakaolu/Sade Tahin Helvası",
            "Kakaolu Tahin Helvası / Sade Tahin Helvası",
        )
        .replace(
            "Sade/Kakaolu Tahin Helvası",
            "Sade Tahin Helvası / Kakaolu Tahin Helvası",
        )
        .replace(
            "Kakaolu / Sade Tahin Helvası",
            "Kakaolu Tahin Helvası / Sade Tahin Helvası",
        )
        .replace(
            "Sade / Kakaolu Tahin Helvası",
            "Sade Tahin Helvası / Kakaolu Tahin Helvası",
        );
    text
}

#[derive(Default)]
struct AltAccumulator {
    main_parts: Vec<String>,
    detail_parts: Vec<String>,
}

fn flush_alt(acc: &mut AltAccumulator, results: &mut Vec<crate::parser::models::MenuComponent>) {
    let main_raw = acc.main_parts.join(" ");
    let main_str = main_raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let detail_raw = acc.detail_parts.join(" ");
    let detail_str = detail_raw.split_whitespace().collect::<Vec<_>>().join(" ");
    acc.main_parts.clear();
    acc.detail_parts.clear();

    if main_str.is_empty() && detail_str.is_empty() {
        return;
    }

    let combined = if !main_str.is_empty() && !detail_str.is_empty() {
        let formatted_detail = if let Some(stripped) = detail_str.strip_prefix('+') {
            format!("+ {}", stripped.trim())
        } else if detail_str.starts_with('(') {
            detail_str
        } else {
            format!("+ {}", detail_str)
        };
        format!("{} {}", main_str, formatted_detail)
    } else if !main_str.is_empty() {
        main_str
    } else {
        detail_str
    };

    let dishes = clean_and_split_dish(combined);
    results.extend(dishes);
}

pub fn parse_rich_dish_paragraph(
    p: scraper::ElementRef,
) -> Vec<crate::parser::models::MenuComponent> {
    let mut results = Vec::new();
    let mut acc = AltAccumulator::default();

    for child in p.children() {
        match child.value() {
            scraper::Node::Element(el) => {
                let name = el.name();
                let is_slash = el.classes().any(|c| c == "slash-separator") || name == "br";
                let is_detail = el.classes().any(|c| c == "food-detail-text");

                if is_slash {
                    flush_alt(&mut acc, &mut results);
                } else if is_detail {
                    if let Some(wrapped) = scraper::ElementRef::wrap(child) {
                        let text = wrapped.text().collect::<String>().trim().to_string();
                        if !text.is_empty() {
                            acc.detail_parts.push(text);
                        }
                    }
                } else if let Some(wrapped) = scraper::ElementRef::wrap(child) {
                    let text = wrapped.text().collect::<String>();
                    let expanded = expand_dish_shorthands(text);
                    if expanded.contains('/') {
                        let parts: Vec<&str> = expanded.split('/').collect();
                        for (idx, part) in parts.iter().enumerate() {
                            let trimmed = part.trim();
                            if !trimmed.is_empty() {
                                acc.main_parts.push(trimmed.to_string());
                            }
                            if idx < parts.len() - 1 {
                                flush_alt(&mut acc, &mut results);
                            }
                        }
                    } else {
                        let trimmed = expanded.trim();
                        if !trimmed.is_empty() {
                            acc.main_parts.push(trimmed.to_string());
                        }
                    }
                }
            }
            scraper::Node::Text(t) => {
                let text = expand_dish_shorthands(t.text.to_string());
                if text.contains('/') {
                    let parts: Vec<&str> = text.split('/').collect();
                    for (idx, part) in parts.iter().enumerate() {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            acc.main_parts.push(trimmed.to_string());
                        }
                        if idx < parts.len() - 1 {
                            flush_alt(&mut acc, &mut results);
                        }
                    }
                } else {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        acc.main_parts.push(trimmed.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    flush_alt(&mut acc, &mut results);

    results
}

pub fn clean_and_split_dish(text: String) -> Vec<crate::parser::models::MenuComponent> {
    if ContentGuard::is_junk_dish_text(&text) {
        return Vec::new();
    }

    let expanded = expand_dish_shorthands(text);
    let parts: Vec<&str> = expanded.split('/').collect();
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
            "ocak" => 1,
            "şubat" => 2,
            "mart" => 3,
            "nisan" => 4,
            "mayıs" => 5,
            "haziran" => 6,
            "temmuz" => 7,
            "ağustos" => 8,
            "eylül" => 9,
            "ekim" => 10,
            "kasım" => 11,
            "aralık" => 12,
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
        assert_eq!(
            dishes[2],
            vec![
                "Mevsim Türlü",
                "Tavuk Hamburger Köfte+Turşu+Marul+Domates+Patates Cips"
            ]
        );

        // p4: <p>Siyah/Yeşil Zeytin</p> -> replaced and split
        assert_eq!(dishes[3], vec!["Siyah Zeytin", "Yeşil Zeytin"]);
    }

    #[test]
    fn test_kyk_menu_detected_meal() {
        let html_dinner = r#"
            <div class="cardStyle">
                <p class="date">3 Eylül 2026</p>
                <div class="card-body">
                    <p>Mercimek Çorbası</p>
                    <p>Çökertme Kebabı</p>
                </div>
                <button onclick="openModal('guid','0','Dinner','3.09.2026 00:00:00','istanbul')"></button>
            </div>
        "#;
        let results = parse_kyk_html(html_dinner, "istanbul", "breakfast");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].detected_meal, Some(MealTypeEnum::Dinner));
    }

    #[test]
    fn test_kyk_menu_rich_html_parsing() {
        let html = r#"
            <div class='card cardStyle mainPageColor' id="areaAll_0">
                <p class="text-center fw-bold homePageCardElementColor cardDate" id="date_0">15 Eylül 2026 Salı</p>
                <div class='card-body'>
                    <div>
                        <p class="food-main-text">
                            <span class='food-main-text'>Tarhana Çorba</span><span class='slash-separator'>/</span><span class='food-main-text'>Mısır Çorba</span>
                        </p>
                        <p class="food-main-text">
                            <span class='food-main-text'>Tavuk Sote</span><span class='food-detail-text'>+Küp Patates</span><span class='slash-separator'>/</span><span class='food-main-text'>Patlıcan Yemeği</span>
                        </p>
                        <p class="food-main-text">
                            <span class='food-main-text'>Tereyağlı Şehriyeli Pirinç Pilavı</span>
                        </p>
                        <p class="food-main-text">Haydari</p>
                        <p class="fw-bold food-detail-text">200 ml Bardak Su</p>
                        <p class="fw-bold food-detail-text">Çeyrek Ekmek</p>
                    </div>
                    <div>
                        <p class="text-end food-main-text"><i class="fa-solid fa-fire-flame-curved"></i> 1100-1500 kcal</p>
                    </div>
                </div>
            </div>
        "#;

        let results = parse_kyk_html(html, "ankara", "dinner");
        assert_eq!(results.len(), 1);
        let dishes = &results[0].dishes;

        // Su ve ekmek elendiği için tam 4 kap yemek olmalı
        assert_eq!(dishes.len(), 4);

        // 1. Kap: Çorba alternatifleri
        assert_eq!(dishes[0], vec!["Tarhana Çorba", "Mısır Çorba"]);

        // 2. Kap: Garnitür ana yemeğe eklendi, ayrı alternatif yapılmadı
        assert_eq!(
            dishes[1],
            vec!["Tavuk Sote + Küp Patates", "Patlıcan Yemeği"]
        );

        // 3. Kap: Pilav
        assert_eq!(dishes[2], vec!["Tereyağlı Şehriyeli Pirinç Pilavı"]);

        // 4. Kap: Meze
        assert_eq!(dishes[3], vec!["Haydari"]);
    }

    #[test]
    fn test_kyk_menu_rich_html_multiple_garnishes() {
        let html = r#"
            <div class='card cardStyle mainPageColor'>
                <p class="date">16 Eylül 2026 Çarşamba</p>
                <div class='card-body'>
                    <div>
                        <p class="food-main-text">
                            <span class='food-main-text'>Domates Çorba</span><span class='slash-separator'>/</span><span class='food-main-text'>Havuç Çorba</span>
                        </p>
                        <p class="food-main-text">
                            <span class='food-main-text'>Izgara Köfte</span><span class='food-detail-text'>+Köz Domates, Biber, Soğan</span><span class='slash-separator'>/</span><span class='food-main-text'>Tavuk Izgara</span><span class='food-detail-text'>+Köz Domates Biber Soğan</span><span class='slash-separator'>/</span><span class='food-main-text'>Mücver</span><span class='food-detail-text'>+Yoğurt</span>
                        </p>
                        <p class="food-main-text">
                            <span class='food-main-text'>Salçalı Makarna</span>
                        </p>
                        <p class="food-main-text">Bisküvili Pasta</p>
                        <p class="fw-bold food-detail-text">200 ml Bardak Su</p>
                        <p class="fw-bold food-detail-text">Çeyrek Ekmek</p>
                    </div>
                </div>
            </div>
        "#;

        let results = parse_kyk_html(html, "ankara", "dinner");
        assert_eq!(results.len(), 1);
        let dishes = &results[0].dishes;

        assert_eq!(dishes.len(), 4);
        assert_eq!(dishes[0], vec!["Domates Çorba", "Havuç Çorba"]);
        assert_eq!(
            dishes[1],
            vec![
                "Izgara Köfte + Köz Domates, Biber, Soğan",
                "Tavuk Izgara + Köz Domates Biber Soğan",
                "Mücver + Yoğurt"
            ]
        );
        assert_eq!(dishes[2], vec!["Salçalı Makarna"]);
        assert_eq!(dishes[3], vec!["Bisküvili Pasta"]);
    }

    #[test]
    fn test_kyk_menu_rich_parenthesis_detail() {
        let html = r#"
            <div class='card cardStyle mainPageColor'>
                <p class="date">15 Eylül 2026 Salı</p>
                <div class='card-body'>
                    <div>
                        <p class="food-main-text">
                            <span class="food-main-text">Kuru Fasulye</span><span class="slash-separator">/</span><span class="food-main-text">Karışık Kızartma</span><span class="food-detail-text">(Domates Sos+Yoğurt)</span>
                        </p>
                        <p class="food-main-text">
                            <span class="food-main-text">Hamburger</span><span class="food-detail-text">(80-100 Gr Hamburger Ekmeği İçerisinde Domates, Marul, Kornişon Turşu İle)</span><span class="slash-separator">/</span><span class="food-main-text">Karışık Dolma</span>
                        </p>
                    </div>
                </div>
            </div>
        "#;

        let results = parse_kyk_html(html, "konya", "dinner");
        assert_eq!(results.len(), 1);
        let dishes = &results[0].dishes;

        assert_eq!(
            dishes[0],
            vec!["Kuru Fasulye", "Karışık Kızartma (Domates Sos+Yoğurt)"]
        );
        assert_eq!(
            dishes[1],
            vec![
                "Hamburger (80-100 Gr Hamburger Ekmeği İçerisinde Domates, Marul, Kornişon Turşu İle)",
                "Karışık Dolma"
            ]
        );
    }

    #[test]
    fn test_kyk_menu_rich_breakfast_shorthands() {
        let html = r#"
            <div class='card cardStyle mainPageColor'>
                <p class="date">15 Eylül 2026 Salı</p>
                <div class='card-body'>
                    <div>
                        <p class="food-main-text">Haşlanmış Yumurta</p>
                        <p class="food-main-text">Menemen</p>
                        <p class="food-main-text">
                            <span class="food-main-text">Zeytinli/Peynirli Açma</span>
                        </p>
                        <p class="food-main-text">Kaşar Peyniri</p>
                        <p class="food-main-text">
                            <span class="food-main-text">Siyah/Yeşil Zeytin</span>
                        </p>
                        <p class="fw-bold food-detail-text">200 ml Bardak Su</p>
                        <p class="fw-bold food-detail-text">Çeyrek Ekmek</p>
                    </div>
                </div>
            </div>
        "#;

        let results = parse_kyk_html(html, "ankara", "breakfast");
        assert_eq!(results.len(), 1);
        let dishes = &results[0].dishes;

        assert_eq!(dishes.len(), 5);
        assert_eq!(dishes[0], vec!["Haşlanmış Yumurta"]);
        assert_eq!(dishes[1], vec!["Menemen"]);
        assert_eq!(dishes[2], vec!["Zeytinli Açma", "Peynirli Açma"]);
        assert_eq!(dishes[3], vec!["Kaşar Peyniri"]);
        assert_eq!(dishes[4], vec!["Siyah Zeytin", "Yeşil Zeytin"]);
    }

    #[test]
    fn test_kyk_menu_real_downloaded_files() {
        let scratch_dir = std::path::Path::new(
            "/home/omer/.gemini/antigravity-ide/brain/b7f820df-2544-4bec-9e59-76e80dd29219/scratch",
        );
        if !scratch_dir.exists() {
            return;
        }
        for entry in std::fs::read_dir(scratch_dir).unwrap().flatten() {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_str().unwrap();
            if path.extension().and_then(|e| e.to_str()) == Some("html")
                && fname.starts_with("menu_")
            {
                let html = std::fs::read_to_string(&path).unwrap();
                let results = parse_kyk_html(&html, "test", "dinner");
                assert!(!results.is_empty(), "Sonuç boş olmamalı: {:?}", fname);
                for res in results {
                    for slot in res.dishes {
                        assert!(!slot.is_empty());
                        for comp in slot {
                            assert!(
                                !comp.name.contains("Bardak Su"),
                                "Su elenmeli: {:?} ({:?})",
                                comp.name,
                                fname
                            );
                            assert!(
                                !comp.name.contains("Çeyrek Ekmek"),
                                "Ekmek elenmeli: {:?} ({:?})",
                                comp.name,
                                fname
                            );
                            assert!(
                                !comp.name.starts_with('+'),
                                "Garnitür tek başına olmamalı: {:?} ({:?})",
                                comp.name,
                                fname
                            );
                        }
                    }
                }
            }
        }
    }
}

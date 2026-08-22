use crate::parser::models::MenuComponent;
use shared::services::content_guard::ContentGuard;

/// kykmenum.com bir günün menüsünü JSON-LD (`@type: "Menu"`) olarak gömer.
/// Bu struct, o günün öğün bazında ayrıştırılmış halini tutar.
#[derive(Debug, Default, Clone)]
pub struct KykMenumDayMenu {
    pub breakfast: Option<Vec<Vec<MenuComponent>>>,
    pub dinner: Option<Vec<Vec<MenuComponent>>>,
}

impl KykMenumDayMenu {
    pub fn is_empty(&self) -> bool {
        self.breakfast.is_none() && self.dinner.is_none()
    }
}

/// HTML içindeki tüm `<script type="application/ld+json">` bloklarını çıkarır.
fn extract_jsonld_blocks(html: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r#"(?s)<script[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#)
            .unwrap()
    });
    re.captures_iter(html)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .collect()
}

/// `@type: "Menu"` olan JSON-LD bloğunu bulur ve öğün bazında ayrıştırır.
pub fn parse_kykmenum_html(html_content: &str) -> Option<KykMenumDayMenu> {
    let mut result = KykMenumDayMenu::default();

    for block in extract_jsonld_blocks(html_content) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&block) else {
            continue;
        };
        if value.get("@type").and_then(|t| t.as_str()) != Some("Menu") {
            continue;
        }

        let Some(sections) = value.get("hasMenuSection").and_then(|s| s.as_array()) else {
            continue;
        };

        for section in sections {
            let section_name = section
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_lowercase();

            let target = if section_name.contains("kahvalt") {
                &mut result.breakfast
            } else if section_name.contains("akşam") || section_name.contains("aksam") {
                &mut result.dinner
            } else {
                continue;
            };

            let mut dish_groups = Vec::new();
            if let Some(items) = section.get("hasMenuItem").and_then(|i| i.as_array()) {
                for item in items {
                    let Some(name) = item.get("name").and_then(|n| n.as_str()) else {
                        continue;
                    };
                    let trimmed = name.trim().trim_start_matches('*').trim();
                    if trimmed.is_empty() || ContentGuard::is_junk_dish_text(trimmed) {
                        continue;
                    }
                    let group = crate::parser::kykyemek::clean_and_split_dish(trimmed.to_string());
                    if !group.is_empty() {
                        dish_groups.push(group);
                    }
                }
            }
            if !dish_groups.is_empty() {
                *target = Some(dish_groups);
            }
        }

        // İlk geçerli Menu bloğu yeterli.
        if !result.is_empty() {
            return Some(result);
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kykmenum_jsonld() {
        let html = r#"
        <html><head>
        <script type="application/ld+json">
        {"@context":"https://schema.org","@type":"WebPage","name":"x"}
        </script>
        <script type="application/ld+json">
        {
          "@context": "https://schema.org",
          "@type": "Menu",
          "name": "İstanbul KYK Günlük Menü - 20.01.2026",
          "hasMenuSection": [
            {
              "name": "Kahvaltı",
              "hasMenuItem": [
                {"name": "Patates Kızartması"},
                {"name": "*Siyah/Yeşil Zeytin"},
                {"name": "Reklam Alanı"}
              ]
            },
            {
              "name": "Akşam Yemeği",
              "hasMenuItem": [
                {"name": "Mercimek Çorba"},
                {"name": "Tavuk Köfte+Elma Dilim Patates"}
              ]
            }
          ]
        }
        </script>
        </head><body></body></html>
        "#;

        let menu = parse_kykmenum_html(html).expect("Menu bulunmalı");
        let breakfast = menu.breakfast.expect("Kahvaltı bulunmalı");
        // "*Siyah/Yeşil Zeytin" -> iki alternatife bölünmeli
        assert!(breakfast
            .iter()
            .any(|g| g.iter().any(|c| c.name == "Siyah Zeytin")));
        let dinner = menu.dinner.expect("Akşam yemeği bulunmalı");
        assert!(dinner
            .iter()
            .any(|g| g.iter().any(|c| c.name.contains("Mercimek Çorba"))));
    }

    #[test]
    fn test_no_menu_returns_none() {
        let html = r#"<html><body>Bu tarih için menü kaydı bulunamadı.</body></html>"#;
        assert!(parse_kykmenum_html(html).is_none());
    }
}

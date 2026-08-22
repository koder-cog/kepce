use crate::parser::models::MenuComponent;
use scraper::{Html, Selector};
use shared::services::content_guard::ContentGuard;

/// yurtmenu.net şehir/gün sayfasındaki (`/{sehir}?date={YYYY-MM-DD}`)
/// `article#breakfast-card` ve `article#dinner-card` bloklarını ayrıştırır.
#[derive(Debug, Default, Clone)]
pub struct YurtmenuDayMenu {
    pub breakfast: Option<Vec<Vec<MenuComponent>>>,
    pub breakfast_kcal: Option<String>,
    pub dinner: Option<Vec<Vec<MenuComponent>>>,
    pub dinner_kcal: Option<String>,
}

impl YurtmenuDayMenu {
    pub fn is_empty(&self) -> bool {
        self.breakfast.is_none() && self.dinner.is_none()
    }
}

fn extract_card(document: &Html, card_id: &str) -> (Option<Vec<Vec<MenuComponent>>>, Option<String>) {
    let anchor = Selector::parse(&format!("article#{}", card_id)).unwrap();
    let Some(card) = document.select(&anchor).next() else {
        return (None, None);
    };

    // Kalori aralığı: <span class="menu-card-meta">650-850 kcal</span>
    let meta_sel = Selector::parse(".menu-card-meta").unwrap();
    let kcal = card
        .select(&meta_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Yemek satırları: <span class="menu-item-name">...</span>
    let item_sel = Selector::parse(".menu-item-name").unwrap();
    let mut dish_groups = Vec::new();
    for el in card.select(&item_sel) {
        let text = el.text().collect::<String>().trim().to_string();
        if text.is_empty() || ContentGuard::is_junk_dish_text(&text) {
            continue;
        }
        // "Mercimek Çorbası / Köz Biber Çorba" gibi alternatifleri böl
        let group = crate::parser::kykyemek::clean_and_split_dish(text);
        if !group.is_empty() {
            dish_groups.push(group);
        }
    }

    if dish_groups.is_empty() {
        (None, kcal)
    } else {
        (Some(dish_groups), kcal)
    }
}

pub fn parse_yurtmenu_html(html_content: &str) -> YurtmenuDayMenu {
    let document = Html::parse_document(html_content);

    let (breakfast, breakfast_kcal) = extract_card(&document, "breakfast-card");
    let (dinner, dinner_kcal) = extract_card(&document, "dinner-card");

    YurtmenuDayMenu {
        breakfast,
        breakfast_kcal,
        dinner,
        dinner_kcal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yurtmenu_cards() {
        let html = r#"
        <html><body>
        <article id="breakfast-card" class="menu-card breakfast-card">
          <div class="menu-card-header">
            <h2 class="menu-card-title">Kahvaltı</h2>
            <span class="menu-card-meta">650-850 kcal</span>
          </div>
          <div id="breakfast-menu" class="menu-items">
            <div class="menu-item"><span class="menu-item-name">Karışık Pizza</span></div>
            <div class="menu-item"><span class="menu-item-name">Örgü/Çeçil Peynir</span></div>
            <div class="menu-item"><span class="menu-item-name"></span></div>
          </div>
        </article>
        <article id="dinner-card" class="menu-card dinner-card">
          <div class="menu-card-header">
            <h2 class="menu-card-title">Akşam Yemeği</h2>
            <span class="menu-card-meta">1100-1500 kcal</span>
          </div>
          <div id="dinner-menu" class="menu-items">
            <div class="menu-item"><span class="menu-item-name">Mercimek Çorbası / Köz Biber Çorba</span></div>
            <div class="menu-item"><span class="menu-item-name">Ayran</span></div>
          </div>
        </article>
        </body></html>
        "#;

        let menu = parse_yurtmenu_html(html);
        let breakfast = menu.breakfast.expect("Kahvaltı bulunmalı");
        assert_eq!(breakfast.len(), 2);
        assert!(breakfast
            .iter()
            .any(|g| g.iter().any(|c| c.name == "Örgü Peynir") || g.iter().any(|c| c.name.contains("Peynir"))));
        assert_eq!(menu.breakfast_kcal.as_deref(), Some("650-850 kcal"));

        let dinner = menu.dinner.expect("Akşam yemeği bulunmalı");
        // "Mercimek Çorbası / Köz Biber Çorba" iki alternatife bölünmeli
        assert!(dinner
            .iter()
            .any(|g| g.len() >= 2 && g[0].name.contains("Mercimek")));
        assert_eq!(menu.dinner_kcal.as_deref(), Some("1100-1500 kcal"));
    }

    #[test]
    fn test_empty_page() {
        let html = r#"<html><body><p>Menü bulunamadı</p></body></html>"#;
        let menu = parse_yurtmenu_html(html);
        assert!(menu.is_empty());
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use crate::parser::kykyemek::{parse_kyk_html, parse_turkish_date};
    use crate::parser::takeaway::{parse_takeaway_menu, TAKEAWAY_CACHE};
    use crate::parser::models::MenuComponent;

    #[test]
    fn test_parse_turkish_date() {
        assert_eq!(
            parse_turkish_date("14 Temmuz 2026 Salı"),
            Some(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap())
        );
        assert_eq!(
            parse_turkish_date("1 Ocak 2025"),
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        );
        assert_eq!(
            parse_turkish_date("31 Aralık 2300 Cuma"),
            Some(NaiveDate::from_ymd_opt(2300, 12, 31).unwrap())
        );
        assert_eq!(parse_turkish_date("invalid date format"), None);
    }

    #[test]
    fn test_parse_kyk_html_basic() {
        let html = r#"
            <div class="cardStyle">
                <div class="card-header">
                    <span class="date">14 Temmuz 2026 Salı</span>
                </div>
                <div class="card-body">
                    <p>Mercimek Çorbası</p>
                    <p>Tavuk Döner / Pilav</p>
                    <p>Salata / Ayran</p>
                    <p>450 kcal</p>
                </div>
            </div>
        "#;

        let results = parse_kyk_html(html, "ankara", "dinner");
        assert_eq!(results.len(), 1);
        let menu = &results[0];
        assert_eq!(menu.date, NaiveDate::from_ymd_opt(2026, 7, 14).unwrap());
        
        // Assert dishes structure (splits by / and trims)
        assert_eq!(menu.dishes.len(), 3);
        assert_eq!(menu.dishes[0], vec!["Mercimek Çorbası"]);
        assert_eq!(menu.dishes[1], vec!["Tavuk Döner", "Pilav"]);
        assert_eq!(menu.dishes[2], vec!["Salata", "Ayran"]);
    }

    #[test]
    fn test_parse_takeaway_menu_cached() {
        // Pre-populate TAKEAWAY_CACHE to simulate loaded config
        let key = "eskisehir_dinner".to_string();
        let mut mock_slots = HashMap::new();
        mock_slots.insert(1, crate::parser::takeaway::TakeawayParsedPackage {
            name: "Al Götür 1".to_string(),
            slots: vec![vec![MenuComponent::from("Ekmek Arası Köfte")], vec![MenuComponent::from("Ayran")]],
        });
        mock_slots.insert(2, crate::parser::takeaway::TakeawayParsedPackage {
            name: "Al Götür 2".to_string(),
            slots: vec![vec![MenuComponent::from("Ekmek Arası Tavuk")], vec![MenuComponent::from("Meyve Suyu")]],
        });

        {
            let mut cache = TAKEAWAY_CACHE.write().unwrap();
            cache.insert(key, mock_slots);
        }

        // Test matching package 1
        let result = parse_takeaway_menu("Al Götür Paket 1", "eskisehir", "dinner");
        assert!(result.is_some());
        let packages = result.unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].0, "Al Götür 1");
        assert_eq!(packages[0].1, vec![vec![MenuComponent::from("Ekmek Arası Köfte")], vec![MenuComponent::from("Ayran")]]);

        // Test matching both packages
        let result_both = parse_takeaway_menu("Al Götür 1 ve 2. Paketler", "eskisehir", "dinner");
        assert!(result_both.is_some());
        let packages_both = result_both.unwrap();
        assert_eq!(packages_both.len(), 2);
        assert_eq!(packages_both[0].0, "Al Götür 1");
        assert_eq!(packages_both[1].0, "Al Götür 2");
    }
}

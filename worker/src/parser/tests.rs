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

    #[test]
    fn test_parse_takeaway_menu_city_integrity() {
        // İstanbul konfigürasyonundan gerçek paketleri okur
        let result_ist = parse_takeaway_menu("Al Götür 1", "istanbul", "breakfast");
        assert!(result_ist.is_some());
        let pkgs = result_ist.unwrap();
        assert_eq!(pkgs.len(), 1);
        assert!(pkgs[0].0.contains("Soğuk Sandviç"));
        assert!(!pkgs[0].1.is_empty());

        // Konfigürasyonu olmayan bir il (örn: Bayburt) için uydurma veri üretilmez (None döner)
        let result_bayburt = parse_takeaway_menu("Al Götür 1", "bayburt", "breakfast");
        assert!(result_bayburt.is_none());
    }

    #[test]
    fn test_parse_fast_menu_foods_html() {
        let sample_html = r#"
            <div class="pb-1">1 Adet Kaşarlı Soğuk Sandviç(Sandviç Ekmeği+70 G Kaşar)Veya 1 Adet Kaşarlı Salamlı Soğuk Sandviç(Sandviç Ekmeği+50 G Kaşar)</div>
            <div class="pb-1">1 Paket Süt(200 Ml)Veya 1 Paket Meyve Suyu(200 Ml)Veya 1 Paket Ayran(270-330 Ml)</div>
            <div class="pb-1">1 Adet Meyve(150-200 G)Veya 1 Paket Kek(En Az 35 G)</div>
            <div class="pb-1">1 Adet Çay</div>
            <div class="pb-1">1 Adet 500 Ml Su</div>
        "#;

        let slots = crate::parser::takeaway::parse_fast_menu_foods_html(sample_html);
        assert_eq!(slots.len(), 5);
        assert_eq!(slots[0].len(), 2);
        assert_eq!(slots[0][0].name, "1 Adet Kaşarlı Soğuk Sandviç");
        assert_eq!(slots[0][0].amount.as_deref(), Some("Sandviç Ekmeği+70 G Kaşar"));
        assert_eq!(slots[1].len(), 3);
        assert_eq!(slots[1][0].name, "1 Paket Süt");
        assert_eq!(slots[1][0].amount.as_deref(), Some("200 Ml"));
        assert_eq!(slots[3][0].name, "1 Adet Çay");
        assert_eq!(slots[3][0].amount, None);
    }

    #[test]
    fn test_parse_kyk_html_with_data_fastmenus() {
        let card_html = r#"
            <div class="card cardStyle">
                <p class="cardDate">3 Haziran 2026 Çarşamba</p>
                <div class="card-body">
                    <div>
                        <p>Peynirli Omlet</p>
                        <p>Kaşar Peyniri</p>
                        <p data-fastmenus='[{"id":"8f64a9ef","name":"Al Götür Menü 2"},{"id":"d03fe329","name":"Al Götür Menü 1"}]' onclick="showFastMenuGroup(this)">
                            Al Götür Menü
                        </p>
                    </div>
                </div>
            </div>
        "#;

        let parsed = crate::parser::kykyemek::parse_kyk_html(card_html, "istanbul", "breakfast");
        assert_eq!(parsed.len(), 1);
        let menu = &parsed[0];
        assert_eq!(menu.takeaways.len(), 2);
        assert!(menu.takeaways[0].0.contains("Gözleme") || menu.takeaways[0].0.contains("Al Götür 2"));
        assert!(menu.takeaways[1].0.contains("Soğuk Sandviç") || menu.takeaways[1].0.contains("Al Götür 1"));
    }
}

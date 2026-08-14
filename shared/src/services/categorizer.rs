use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryRule {
    pub category: String,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
}

static RULES: OnceLock<Vec<CategoryRule>> = OnceLock::new();

const EMBEDDED_RULES_JSON: &str = include_str!("../../../config/pricing/category_rules.json");

pub fn get_rules() -> &'static [CategoryRule] {
    RULES.get_or_init(|| {
        serde_json::from_str(EMBEDDED_RULES_JSON).unwrap_or_else(|e| {
            tracing::error!("Kategori kuralları JSON parse hatası: {:?}", e);
            Vec::new()
        })
    })
}

/// Türkçe karakterleri küçük harfe dönüştürür ve normalize eder.
pub fn normalize_turkish(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'I' => 'ı',
            'İ' => 'i',
            'Ç' => 'ç',
            'Ş' => 'ş',
            'Ğ' => 'ğ',
            'Ü' => 'ü',
            'Ö' => 'ö',
            other => other.to_ascii_lowercase(),
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn matches_keyword(text: &str, keyword: &str) -> bool {
    if keyword.chars().count() <= 2 {
        text.split(|c: char| !c.is_alphanumeric())
            .any(|w| w == keyword)
    } else {
        text.contains(keyword)
    }
}

/// Verilen yemek adını analiz ederek bakanlığın resmi kategori adını döndürür.
pub fn categorize_dish(dish_name: &str) -> Option<String> {
    let normalized = normalize_turkish(dish_name);
    if normalized.is_empty() {
        return None;
    }

    let rules = get_rules();

    for rule in rules {
        // Excludes kontrolü: Eğer hariç tutulan kelimelerden biri varsa bu kuralı atla
        let excluded = rule.excludes.iter().any(|exc| {
            let exc_norm = normalize_turkish(exc);
            !exc_norm.is_empty() && matches_keyword(&normalized, &exc_norm)
        });

        if excluded {
            continue;
        }

        // Keywords kontrolü: Eğer anahtar kelimelerden biri geçiyorsa eşleşti
        let matched = rule.keywords.iter().any(|kw| {
            let kw_norm = normalize_turkish(kw);
            !kw_norm.is_empty() && matches_keyword(&normalized, &kw_norm)
        });

        if matched {
            return Some(rule.category.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_dishes() {
        assert_eq!(categorize_dish("Ezogelin Çorbası"), Some("ÇORBA ÇEŞİTLERİ".to_string()));
        assert_eq!(categorize_dish("Süzme Mercimek Çorba"), Some("ÇORBA ÇEŞİTLERİ".to_string()));
        assert_eq!(categorize_dish("Şehriyeli Pirinç Pilavı"), Some("PİRİNÇ PİLAVI ÇEŞİTLERİ".to_string()));
        assert_eq!(categorize_dish("Meyhane Bulgur Pilavı"), Some("BULGUR PİLAVI ÇEŞİTLERİ".to_string()));
        assert_eq!(categorize_dish("Sebzeli Tavuk Kavurma"), Some("KEMİKSİZ TAVUK YEMEKLERİ".to_string()));
        assert_eq!(categorize_dish("Fırın Tavuk Baget"), Some("KEMİKLİ TAVUK YEMEKLERİ".to_string()));
        assert_eq!(categorize_dish("Tavuk Şiş Izgara"), Some("KEMİKSİZ IZGARA/KIZARTMA TAVUK YEMEKLERİ".to_string()));
        assert_eq!(categorize_dish("Kıymalı Kuru Fasulye"), Some("ETLİ BAKLAGİLLER".to_string()));
        assert_eq!(categorize_dish("Zeytinyağlı Kuru Fasulye"), Some("ETSİZ BAKLAGİLLER".to_string()));
        assert_eq!(categorize_dish("Kuzu Gerdan Haşlama"), Some("KEMİKLİ ET YEMEKLERİ".to_string()));
        assert_eq!(categorize_dish("İnegöl Köfte"), Some("IZGARA KÖFTELER".to_string()));
        assert_eq!(categorize_dish("İzmir Köfte"), Some("SULU SALÇALI ETLİ YEMEKLER VE TERBİYELİ SEBZELİ KÖFTELER".to_string()));
        assert_eq!(categorize_dish("Fırın Sütlaç"), Some("SÜTLÜ TATLILAR".to_string()));
        assert_eq!(categorize_dish("Fıstıklı Baklava"), Some("BAKLAVA-KADAYIF (FISTIKLI)".to_string()));
        assert_eq!(categorize_dish("Cevizli Ev Baklavası"), Some("BAKLAVA-KADAYIF (CEVİZLİ-FINDIKLI)".to_string()));
        assert_eq!(categorize_dish("Şekerpare"), Some("HAMUR TATLILARI".to_string()));
        assert_eq!(categorize_dish("Çoban Salata"), Some("SALATA-I".to_string()));
        assert_eq!(categorize_dish("Mevsim Salata"), Some("SALATA-II".to_string()));
        assert_eq!(categorize_dish("Roka Salata"), Some("SALATA-III".to_string()));
        assert_eq!(categorize_dish("Kaşarlı Tost"), Some("KAŞARLI TOST".to_string()));
        assert_eq!(categorize_dish("Susamlı Simit"), Some("SİMİT".to_string()));
        assert_eq!(categorize_dish("Kuşbaşılı Pide"), Some("KUŞBAŞILI VEYA SUCUKLU PİDE".to_string()));
        assert_eq!(categorize_dish("Kıymalı Pide"), Some("KIYMALI PİDE".to_string()));
        assert_eq!(categorize_dish("Lahmacun"), Some("LAHMACUN".to_string()));
    }
}

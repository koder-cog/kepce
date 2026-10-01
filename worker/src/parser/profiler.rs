//! Belge Profilleyici (Document Profiler)
//!
//! Gelen dosyanın metin veya OCR içeriğindeki göstergelere dayanarak
//! belgenin tipini (Tabldot Menü, Resmi Fiyat Panosu, Al Götür veya Hibrit) belirler.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentProfile {
    DailyMenu,
    OfficialPricing,
    TakeawayPackage,
    Compound,
}

impl DocumentProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DailyMenu => "daily_menu",
            Self::OfficialPricing => "official_pricing",
            Self::TakeawayPackage => "takeaway_package",
            Self::Compound => "compound",
        }
    }
}

/// Metin içeriğindeki anahtar kelimelere ve yapısal göstergelere göre belge profilini belirler.
pub fn profile_text(content: &str) -> DocumentProfile {
    let upper = content.to_uppercase();

    let has_pricing_keywords = upper.contains("GRAMAJ VE FİYAT")
        || upper.contains("FİYAT LİSTESİ")
        || upper.contains("FİYAT TARİFESİ")
        || upper.contains("TAVAN FİYAT");

    let has_takeaway_keywords = upper.contains("AL GÖTÜR MENÜ")
        || upper.contains("AL GÖTÜR")
        || (upper.contains("SANDVİÇ") && upper.contains("AYRAN") && upper.contains("MEYVE SUYU"));

    let has_menu_keywords = upper.contains("KAHVALTI")
        || upper.contains("ÖĞLE")
        || upper.contains("AKŞAM")
        || upper.contains("MENÜSÜ")
        || upper.contains("YEMEK LİSTESİ");

    if has_pricing_keywords && has_takeaway_keywords {
        return DocumentProfile::Compound;
    }

    if has_pricing_keywords {
        return DocumentProfile::OfficialPricing;
    }

    if has_takeaway_keywords {
        return DocumentProfile::TakeawayPackage;
    }

    if has_menu_keywords {
        return DocumentProfile::DailyMenu;
    }

    DocumentProfile::DailyMenu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_official_pricing() {
        let sample =
            "İSTANBUL GENÇLİK VE SPOR İL MÜDÜRLÜĞÜ 2026-2027 KAHVALTI GRAMAJ VE FİYAT LİSTESİ";
        assert_eq!(profile_text(sample), DocumentProfile::OfficialPricing);

        let sample2 = "2026-2027 YEMEK GRAMAJ VE FİYAT LİSTESİ";
        assert_eq!(profile_text(sample2), DocumentProfile::OfficialPricing);
    }

    #[test]
    fn test_profile_takeaway_package() {
        let sample = "MENÜ ÇEŞİTLERİ AL GÖTÜR MENÜ 1 1 Adet Kaşarlı Soğuk Sandviç veya 1 Paket Süt";
        assert_eq!(profile_text(sample), DocumentProfile::TakeawayPackage);
    }

    #[test]
    fn test_profile_daily_menu() {
        let sample = "EKİM 2026 AYI KAHVALTI VE AKŞAM YEMEĞİ MENÜSÜ 01.10.2026 Mercimek Çorbası";
        assert_eq!(profile_text(sample), DocumentProfile::DailyMenu);
    }
}

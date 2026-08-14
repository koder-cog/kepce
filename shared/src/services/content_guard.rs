use ammonia;
use rustrict::{Censor, Type};

pub struct ContentGuard;

impl ContentGuard {
    /// Gelen metindeki zararlı HTML etiketlerini temizler (XSS koruması)
    /// Ammonia kütüphanesi güvenli bir whitelist stratejisi kullanır.
    pub fn sanitize_html(input: &str) -> String {
        ammonia::clean(input)
    }

    /// İçerikte küfür, hakaret veya sakıncalı bir metin var mı kontrol eder.
    pub fn contains_profanity(input: &str) -> bool {
        let analysis = Censor::from_str(input).analyze();
        analysis.is(Type::INAPPROPRIATE) || analysis.is(Type::PROFANE) || analysis.is(Type::OFFENSIVE) || analysis.is(Type::SEVERE)
    }

    /// Sakıncalı kelimeleri sansürler (yerlerine yıldız * koyar)
    pub fn censor_profanity(input: &str) -> String {
        Censor::from_str(input).censor()
    }

    /// Basit spam kontrolü (Link tespiti ve anlamsız karakter tekrarı)
    pub fn is_spam(input: &str) -> bool {
        let normalized = input.to_lowercase();
        let link_count = normalized.matches("http://").count()
            + normalized.matches("https://").count()
            + normalized.matches("www.").count()
            + normalized.matches("ftp://").count();
            
        if link_count >= 2 || (link_count >= 1 && input.len() < 50) {
            return true;
        }
        
        let mut max_repeat = 0;
        let mut current_repeat = 1;
        let mut prev_char = '\0';
        
        for c in input.chars() {
            if c.is_alphabetic() {
                if c == prev_char {
                    current_repeat += 1;
                    if current_repeat > max_repeat {
                        max_repeat = current_repeat;
                    }
                } else {
                    current_repeat = 1;
                }
                prev_char = c;
            }
        }
        
        if max_repeat > 10 {
            return true;
        }

        false
    }

    /// Menü ve yemek isimlerindeki bürokrat isimleri, duyurular, kalori bilgileri
    /// ve çöp metinleri tespit eder.
    pub fn is_junk_dish_text(input: &str) -> bool {
        let text_trimmed = input.trim();
        if text_trimmed.is_empty() {
            return true;
        }

        // Türkçe karakter normalizasyonu ile küçük harfe çevirme
        let mut normalized = text_trimmed.to_string();
        normalized = normalized
            .replace(['İ', 'I', 'ı'], "i")
            .replace(['ş', 'Ş'], "s")
            .replace(['ğ', 'Ğ'], "g")
            .replace(['ü', 'Ü'], "u")
            .replace(['ö', 'Ö'], "o")
            .replace(['ç', 'Ç'], "c");
        let normalized = normalized.to_lowercase();

        // Engellenecek kara liste kalıpları
        let blocked_keywords = [
            "il muduru",
            "sube muduru",
            "genclik ve spor",
            "balkanlioglu",
            "muhittin",
            "rektor",
            "daire balkani",
            "daire baskani",
            "valisi",
            "valilik",
            "kaymakam",
            "ramazan ayi",
            "hayirli ramazanlar",
            "afiyet olsun",
            "iyi dersler",
            "not:",
            "not :",
            "duyuru",
            "menude degisiklik",
            "menudegisiklik",
            "tarihinde",
            "yili",
            "kalori",
            "kcal",
            "toplam kalori",
            "besin degerleri",
            "besin degeri",
            "icerik",
            "kullanim kosullari",
            "fiyat listesi",
            "tabldot ucreti",
        ];

        for kw in blocked_keywords {
            if normalized.contains(kw) {
                return true;
            }
        }

        // Harf içermeyen (sadece sayı veya sembol) metinler
        if !text_trimmed.chars().any(|c| c.is_alphabetic()) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let clean = ContentGuard::sanitize_html("Hello <strong>World</strong>!");
        assert!(clean.contains("<strong>World</strong>") || clean.contains("Hello "));

        let dirty = "Hello <script>alert(1)</script><a href='javascript:alert(1)'>click</a><iframe src='x'></iframe>";
        let sanitized = ContentGuard::sanitize_html(dirty);
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("javascript:"));
        assert!(!sanitized.contains("<iframe>"));
    }

    #[test]
    fn test_is_spam() {
        assert!(!ContentGuard::is_spam("Bu yemek çok güzeldi, elinize sağlık."));
        assert!(ContentGuard::is_spam("sitemize gidin: www.example.com"));
        assert!(!ContentGuard::is_spam("Merhaba arkadaşlar, bugün kyk menüsünü inceledim ve şu adreste paylaştım: http://example.com/menu"));
        assert!(ContentGuard::is_spam("Linkler: www.site1.com ve www.site2.com adresleri."));
        assert!(ContentGuard::is_spam("Çooook lezzetliiiiiiiiiii bir yemekti."));
        assert!(!ContentGuard::is_spam("Çooook lezzetliiiii bir yemekti."));
    }

    #[test]
    fn test_is_junk_dish_text() {
        // Bureaucrat names and announcements
        assert!(ContentGuard::is_junk_dish_text("Afyon Gençlik ve Spor İl Müdürü Muhittin BALKANLIOĞLU"));
        assert!(ContentGuard::is_junk_dish_text("İL MÜDÜRÜ"));
        assert!(ContentGuard::is_junk_dish_text("ŞUBE MÜDÜRÜ"));
        assert!(ContentGuard::is_junk_dish_text("DAİRE BAŞKANI"));
        assert!(ContentGuard::is_junk_dish_text("NOT: Ramazan ayı boyunca yemek saatleri 20:00'dir"));
        assert!(ContentGuard::is_junk_dish_text("Afiyet Olsun!"));
        assert!(ContentGuard::is_junk_dish_text("Top Toplam Kalori: 850 kcal"));
        assert!(ContentGuard::is_junk_dish_text("450 kcal"));
        assert!(ContentGuard::is_junk_dish_text("12345"));
        assert!(ContentGuard::is_junk_dish_text(""));

        // Valid dishes
        assert!(!ContentGuard::is_junk_dish_text("Pideli Soslu Izgara Köfte"));
        assert!(!ContentGuard::is_junk_dish_text("Mercimek Çorbası"));
        assert!(!ContentGuard::is_junk_dish_text("Siyah Zeytin"));
        assert!(!ContentGuard::is_junk_dish_text("Tavuk Döner"));
    }
}

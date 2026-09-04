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

        // Site navigasyon/başlık kalıntıları: ok karakteriyle başlayan satırlar
        // (örn. "←İstanbul KYK Menüsü") yemek değildir.
        if text_trimmed.starts_with(['←', '→', '«', '»', '‹', '›']) {
            return true;
        }

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
            // Kaynak sitelerdeki "menü yok" placeholder mesajları
            // (örn. "Veri yok. Menüye sahipseniz ... mail atabilirsiniz")
            // yemek satırı sanılıp veritabanına yutulmasın.
            "veri yok",
            "menuye sahipseniz",
            "sahipseniz",
            "mail atabilirsiniz",
            "eposta",
            "e-posta",
            // Sayfa başlığı/navigasyon kalıntıları (kykmenu.com.tr scrape'ında
            // yemek sanılıp DB'ye yutulmuştu): "Kahvaltı Yemek Listesi",
            // "Gün Menüsü", "←İstanbul KYK Menüsü" vb.
            "yemek listesi",
            "gun menusu",
            "kyk menu",
            "kykmenusu",
            "menu listesi",
            // Reklam, sponsorluk ve site şablonu kalıntıları
            "reklam alani",
            "reklam",
            "sponsor",
            "telif hakki",
            "tum haklari saklidir",
            "iletisim:",
            "web sitemiz",
            "gunun corbasi:",
            "gunun tatlisi:",
            "alternatif menuler",
            // Birleşik öğün adları: parser'a boşlukları yutulmuş başlık
            // kalıntıları ("KahvaltıAkşam" vb.) yemek olarak girmesin.
            "kahvaltiaksam",
            "kahvaltiogle",
            "ogleaksam",
            // Sezon/dönem başı duyuruları (örn. "14 Eylül itibarıyla yeni dönem listeleri...")
            "yeni donem",
            "girilmeye baslanacak",
            "basarilar dileriz",
            "itibariyla",
            "itibariyle",
        ];

        for kw in blocked_keywords {
            if normalized.contains(kw) {
                return true;
            }
        }

        // Yalnızca öğün/başlık kelimelerinden oluşan satırlar (örn.
        // "Kahvaltı Öğle", "Akşam Yemeği") yemek değil, başlık kalıntısıdır.
        let meal_header_words = [
            "kahvalti", "ogle", "aksam", "yemegi", "yemek", "listesi", "menu", "gun",
        ];
        let tokens: Vec<&str> = normalized.split_whitespace().collect();
        if !tokens.is_empty() && tokens.iter().all(|t| meal_header_words.contains(t)) {
            return true;
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
        assert!(ContentGuard::is_junk_dish_text(
            "Veri yok. Menüye sahipseniz destek@kepce.org adresine mail atabilirsiniz, teşekkür ederiz."
        ));
        assert!(ContentGuard::is_junk_dish_text("Menüye sahipseniz bize bildirin"));

        // Site navigasyon/başlık kalıntıları (kykmenu.com.tr scrape kazıntısı)
        assert!(ContentGuard::is_junk_dish_text("←İstanbul KYK Menüsü"));
        assert!(ContentGuard::is_junk_dish_text("→ Kayseri KYK Menüsü"));
        assert!(ContentGuard::is_junk_dish_text("- Kahvaltı Yemek Listesi"));
        assert!(ContentGuard::is_junk_dish_text("- Akşam Yemeği Yemek Listesi"));
        assert!(ContentGuard::is_junk_dish_text("Gün Menüsü"));
        assert!(ContentGuard::is_junk_dish_text(
            "14 Eylül itibarıyla yeni dönem listeleri girilmeye başlanacak. Herkese yeni dönemde başarılar dileriz."
        ));

        // Valid dishes
        assert!(!ContentGuard::is_junk_dish_text("Pideli Soslu Izgara Köfte"));
        assert!(!ContentGuard::is_junk_dish_text("Mercimek Çorbası"));
        assert!(!ContentGuard::is_junk_dish_text("Siyah Zeytin"));
        assert!(!ContentGuard::is_junk_dish_text("Tavuk Döner"));
    }
}

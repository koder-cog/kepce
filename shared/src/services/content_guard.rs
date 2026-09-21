use ammonia;
use std::sync::OnceLock;

const EMBEDDED_SHORTENERS_JSON: &str = include_str!("../../../config/security/url_shorteners.json");
static BLOCKED_URLS: OnceLock<BlockedUrlConfig> = OnceLock::new();

#[derive(Debug, serde::Deserialize)]
struct BlockedUrlConfig {
    shortener_domains: Vec<String>,
    chat_invite_domains: Vec<String>,
    suspicious_tlds: Vec<String>,
}

fn get_blocked_urls() -> &'static BlockedUrlConfig {
    BLOCKED_URLS.get_or_init(|| {
        serde_json::from_str(EMBEDDED_SHORTENERS_JSON).unwrap_or_else(|e| {
            tracing::error!("URL shorteners JSON parse hatası: {:?}", e);
            BlockedUrlConfig {
                shortener_domains: Vec::new(),
                chat_invite_domains: Vec::new(),
                suspicious_tlds: Vec::new(),
            }
        })
    })
}

fn contains_domain(text: &str, domain: &str) -> bool {
    for (pos, _) in text.match_indices(domain) {
        let prefix_ok = if pos == 0 {
            true
        } else {
            let prev_char = text[..pos].chars().last().unwrap();
            !prev_char.is_alphanumeric()
        };
        let after_pos = pos + domain.len();
        let suffix_ok = if after_pos >= text.len() {
            true
        } else {
            let next_char = text[after_pos..].chars().next().unwrap();
            !next_char.is_alphanumeric()
        };
        if prefix_ok && suffix_ok {
            return true;
        }
    }
    false
}

fn contains_suspicious_tld(text: &str, tld: &str) -> bool {
    for (pos, _) in text.match_indices(tld) {
        let prefix_has_alphanumeric = if pos == 0 {
            false
        } else {
            let prev_char = text[..pos].chars().last().unwrap();
            prev_char.is_alphanumeric()
        };
        let after_pos = pos + tld.len();
        let suffix_ok = if after_pos >= text.len() {
            true
        } else {
            let next_char = text[after_pos..].chars().next().unwrap();
            !next_char.is_alphanumeric()
        };
        if prefix_has_alphanumeric && suffix_ok {
            return true;
        }
    }
    false
}

fn extract_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let normalized = text.to_lowercase();
    let mut prefixes: Vec<usize> = Vec::new();

    for (pos, _) in normalized.match_indices("http://") {
        prefixes.push(pos);
    }
    for (pos, _) in normalized.match_indices("https://") {
        prefixes.push(pos);
    }
    for (pos, _) in normalized.match_indices("ftp://") {
        prefixes.push(pos);
    }
    for (pos, _) in normalized.match_indices("www.") {
        if pos < 3 || !normalized[..pos].ends_with("://") {
            prefixes.push(pos);
        }
    }

    prefixes.sort_unstable();
    prefixes.dedup();

    let mut last_end = 0;
    for start in prefixes {
        if start < last_end {
            continue;
        }
        let remainder = &normalized[start..];
        let link_end = remainder
            .find(|c: char| c.is_whitespace())
            .unwrap_or(remainder.len());
        last_end = start + link_end;
        let raw_link = &remainder[..link_end];
        let trimmed =
            raw_link.trim_end_matches(['.', ',', ';', '!', '?', ')', ']', '>', '"', '\'']);
        links.push(trimmed.to_string());
    }

    links
}

pub struct ContentGuard;

impl ContentGuard {
    /// Gelen metindeki zararlı HTML etiketlerini temizler (XSS koruması)
    /// Ammonia kütüphanesi güvenli bir whitelist stratejisi kullanır.
    pub fn sanitize_html(input: &str) -> String {
        ammonia::clean(input)
    }

    /// Gelişmiş spam kontrolü:
    /// 1. Kısaltıcılar (bit.ly, tinyurl vb.) ve sohbet davetleri (t.me, wa.me) tek bir tane dahi olsa engellenir.
    /// 2. Kötü şöhretli TLD'ler (.xyz, .top, .buzz vb.) tek bağlantıda dahi engellenir.
    /// 3. İç linkler (kepce.org) spam sayımından düşülür.
    /// 4. Dış bağlantı sayısı 2 veya daha fazlaysa (veya 1 dış bağlantı varken metin < 50 karakterse) spam sayılır.
    /// 5. 10'dan fazla aynı harf tekrarı engellenir.
    pub fn is_spam(input: &str) -> bool {
        let normalized = input.to_lowercase();
        let config = get_blocked_urls();

        // 1. Kısaltıcılar veya sohbet davetleri doğrudan spam
        for domain in &config.shortener_domains {
            if contains_domain(&normalized, domain) {
                return true;
            }
        }
        for domain in &config.chat_invite_domains {
            if contains_domain(&normalized, domain) {
                return true;
            }
        }

        // 2. Şüpheli TLD'ler (.xyz, .top, .buzz vb.) doğrudan spam
        for tld in &config.suspicious_tlds {
            if contains_suspicious_tld(&normalized, tld) {
                return true;
            }
        }

        // 3. Linkleri çıkar ve iç bağlantıları (kepce.org) muaf tut
        let all_links = extract_links(&normalized);
        let external_links: Vec<&String> = all_links
            .iter()
            .filter(|link| !link.contains("kepce.org") && !link.contains("localhost"))
            .collect();

        if external_links.len() >= 2 || (external_links.len() == 1 && input.len() < 50) {
            return true;
        }

        // 4. Anlamsız karakter tekrarı kontrolü (10'dan fazla aynı harf)
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
            // Scraper kaynak sitelerindeki (kykyemek.com vb.) liste yok placeholder duyuruları
            "elimize ulasir",
            "siteye eklenecektir",
            "elinizde liste",
            "bize iletebilir",
            "odullerden yararlanabilirsiniz",
            "kykyemek",
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
        assert!(!ContentGuard::is_spam(
            "Bu yemek çok güzeldi, elinize sağlık."
        ));
        assert!(ContentGuard::is_spam("sitemize gidin: www.example.com"));
        assert!(!ContentGuard::is_spam("Merhaba arkadaşlar, bugün kyk menüsünü inceledim ve şu adreste paylaştım: http://example.com/menu"));
        assert!(ContentGuard::is_spam(
            "Linkler: www.site1.com ve www.site2.com adresleri."
        ));
        assert!(ContentGuard::is_spam(
            "Çooook lezzetliiiiiiiiiii bir yemekti."
        ));
        assert!(!ContentGuard::is_spam("Çooook lezzetliiiii bir yemekti."));

        // İç linkler (kepce.org) spam sayılmaz, birden fazla olsa dahi izin verilir
        assert!(!ContentGuard::is_spam("Dünkü menü https://kepce.org/istanbul/2026-09-14 ile bugünkü https://www.kepce.org/istanbul/2026-09-15 menüsü çok farklıydı."));
        assert!(!ContentGuard::is_spam(
            "Menü linki: https://kepce.org/istanbul"
        ));

        // URL kısaltıcılar tek başına dahi olsa anında engellenir
        assert!(ContentGuard::is_spam(
            "Burs çekilişi için şu bağlantıya tıklayın: bit.ly/kyk-burs"
        ));
        assert!(ContentGuard::is_spam("Öğrenci indirimleri için tinyurl.com/ogrenci adresini ziyaret edebilirsiniz arkadaslar"));

        // Telegram ve WhatsApp sohbet davetleri engellenir
        assert!(ContentGuard::is_spam(
            "Kyk yemekhane grubumuz açıldı katılın: t.me/kykyemekhane"
        ));
        assert!(ContentGuard::is_spam(
            "Sorular için wa.me/905551234567 numarasından yazabilirsiniz"
        ));

        // Şüpheli TLD'ler (.xyz, .top vb.) engellenir
        assert!(ContentGuard::is_spam(
            "Yeni bir platform açılmış arkadaşlar: https://kykmenu.xyz/giris"
        ));
        assert!(ContentGuard::is_spam(
            "Yemek listesi burada mevcut: menuler.top"
        ));
    }

    #[test]
    fn test_is_junk_dish_text() {
        // Bureaucrat names and announcements
        assert!(ContentGuard::is_junk_dish_text(
            "Afyon Gençlik ve Spor İl Müdürü Muhittin BALKANLIOĞLU"
        ));
        assert!(ContentGuard::is_junk_dish_text("İL MÜDÜRÜ"));
        assert!(ContentGuard::is_junk_dish_text("ŞUBE MÜDÜRÜ"));
        assert!(ContentGuard::is_junk_dish_text("DAİRE BAŞKANI"));
        assert!(ContentGuard::is_junk_dish_text(
            "NOT: Ramazan ayı boyunca yemek saatleri 20:00'dir"
        ));
        assert!(ContentGuard::is_junk_dish_text("Afiyet Olsun!"));
        assert!(ContentGuard::is_junk_dish_text(
            "Top Toplam Kalori: 850 kcal"
        ));
        assert!(ContentGuard::is_junk_dish_text("450 kcal"));
        assert!(ContentGuard::is_junk_dish_text("12345"));
        assert!(ContentGuard::is_junk_dish_text(""));
        assert!(ContentGuard::is_junk_dish_text(
            "Veri yok. Menüye sahipseniz destek@kepce.org adresine mail atabilirsiniz, teşekkür ederiz."
        ));
        assert!(ContentGuard::is_junk_dish_text(
            "Menüye sahipseniz bize bildirin"
        ));

        // Site navigasyon/başlık kalıntıları (kykmenu.com.tr scrape kazıntısı)
        assert!(ContentGuard::is_junk_dish_text("←İstanbul KYK Menüsü"));
        assert!(ContentGuard::is_junk_dish_text("←adana Kyk Menüsü"));
        assert!(ContentGuard::is_junk_dish_text("Kahvaltıakşam"));
        assert!(ContentGuard::is_junk_dish_text("→ Kayseri KYK Menüsü"));
        assert!(ContentGuard::is_junk_dish_text("- Kahvaltı Yemek Listesi"));
        assert!(ContentGuard::is_junk_dish_text(
            "- Akşam Yemeği Yemek Listesi"
        ));
        assert!(ContentGuard::is_junk_dish_text("Gün Menüsü"));
        assert!(ContentGuard::is_junk_dish_text(
            "14 Eylül itibarıyla yeni dönem listeleri girilmeye başlanacak. Herkese yeni dönemde başarılar dileriz."
        ));
        assert!(ContentGuard::is_junk_dish_text(
            "Listeler Elimize Ulaşır Ulaşmaz Siteye Eklenecektir. Elinizde Liste Mevcutsa Üye Olarak Bize İletebilir, Ödüllerden Yararlanabilirsiniz"
        ));

        // Valid dishes
        assert!(!ContentGuard::is_junk_dish_text(
            "Pideli Soslu Izgara Köfte"
        ));
        assert!(!ContentGuard::is_junk_dish_text("Mercimek Çorbası"));
        assert!(!ContentGuard::is_junk_dish_text("Siyah Zeytin"));
        assert!(!ContentGuard::is_junk_dish_text("Tavuk Döner"));
    }
}

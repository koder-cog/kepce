use ammonia;
use std::sync::OnceLock;

const EMBEDDED_SHORTENERS_JSON: &str = include_str!("../../../../config/security/url_shorteners.json");
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

    /// T.C. Kimlik Numarası algoritma doğrulaması:
    /// 1. 11 basamaklı olmalı, ilk hane 0 olamaz, 11. hane çift olmalı.
    /// 2. İlk 9 hanede (payload) tekdüze diziler (11111111110) ve ardışık klavye
    ///    dizileri (12345678950, 98765432102) elenir.
    /// 3. i32 cast ile negatif fark taşması (wrapping_sub) engellenir.
    pub fn is_valid_tckn(digits_str: &str) -> bool {
        let d: Vec<u32> = digits_str.chars().filter_map(|c| c.to_digit(10)).collect();
        if d.len() != 11 || d[0] == 0 || d[10] % 2 != 0 {
            return false;
        }

        // 1. İlk 9 hanede tekdüzelik ve ardışıklık kontrolü
        let payload = &d[0..9];
        let all_same = payload.windows(2).all(|w| w[0] == w[1]);
        let asc_seq = payload.windows(2).all(|w| w[0] + 1 == w[1]);
        let desc_seq = payload.windows(2).all(|w| w[0] == w[1] + 1);

        if all_same || asc_seq || desc_seq {
            return false;
        }

        // 2. Modüler sağlama (i32 ile negatif fark taşması engellenir)
        let odd_sum: i32 = (d[0] + d[2] + d[4] + d[6] + d[8]) as i32;
        let even_sum: i32 = (d[1] + d[3] + d[5] + d[7]) as i32;

        let tenth_check = ((odd_sum * 7) - even_sum).rem_euclid(10) as u32;
        let total_ten_sum: u32 = d[0..10].iter().sum();
        let eleventh_check = total_ten_sum % 10;

        d[9] == tenth_check && d[10] == eleventh_check
    }

    /// Metin içinde geçerli bir TCKN tespiti yapar.
    pub fn contains_tckn(text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        if len < 11 {
            return false;
        }

        // 1. Doğrudan 11 haneli ardışık rakam pencereleri
        for i in 0..=len - 11 {
            let slice: String = chars[i..i + 11].iter().collect();
            if slice.chars().all(|c| c.is_ascii_digit()) {
                let prefix_ok = i == 0 || !chars[i - 1].is_ascii_digit();
                let suffix_ok = i + 11 == len || !chars[i + 11].is_ascii_digit();
                if prefix_ok && suffix_ok && Self::is_valid_tckn(&slice) {
                    return true;
                }
            }
        }

        // 2. Yalnızca rakamlardan oluşan temizlenmiş dizgi (boşluk / tire içeren TCKN'ler)
        let cleaned: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() == 11 && Self::is_valid_tckn(&cleaned) {
            return true;
        }

        false
    }

    /// Fonotaktik kural ve alfabe kümesi izolasyonu ile telefon numarası tespiti:
    /// - Normal telefonlar (05xx, 5xx) ve boşluk/tire ile gizlenmiş halleri yakalanır.
    /// - Leetspeak (o->0, i/ı->1, z->2, e->3, a->4, s->5, b->8, t->7, g->9) ile gizlenmiş
    ///   numaralar taranırken, aday blokta S_turkce_eksik kümesinden harf geçip geçmediği kontrol edilir.
    /// - "Soslu bezelye" gibi doğal Türkçe kelimeler (içinde l, u, y, d vb. geçtiğinden) korunur.
    pub fn contains_phone_number(text: &str) -> bool {
        let lower = turkish_to_lower(text);
        let chars: Vec<char> = lower.chars().collect();
        let len = chars.len();
        if len < 10 {
            return false;
        }

        // Kayan pencere ile mobil başlangıcı arama
        for start in 0..len {
            let start_char = chars[start];
            let mapped_start = map_leetspeak_digit(start_char);
            if mapped_start != Some('0') && mapped_start != Some('5') {
                continue;
            }

            let mut digits = String::with_capacity(11);
            let mut turkce_eksik_count = 0;
            let mut consecutive_same_cons = 1;
            let mut max_consecutive_cons = 1;
            let mut prev_char = '\0';

            for i in start..len {
                let c = chars[i];

                if c.is_whitespace() || matches!(c, '-' | '.' | '_' | '/' | '(' | ')' | '+' | ':') {
                    prev_char = '\0';
                    continue;
                }

                if is_turkce_eksik(c) {
                    turkce_eksik_count += 1;
                }

                if c == prev_char && !is_vowel(c) && c.is_alphabetic() {
                    consecutive_same_cons += 1;
                    if consecutive_same_cons > max_consecutive_cons {
                        max_consecutive_cons = consecutive_same_cons;
                    }
                } else {
                    consecutive_same_cons = 1;
                }
                prev_char = c;

                if let Some(digit) = map_leetspeak_digit(c) {
                    digits.push(digit);

                    let is_mobile_10 = digits.len() == 10
                        && digits.starts_with('5')
                        && matches!(digits.chars().nth(1), Some('0' | '3' | '4' | '5' | '6' | '7' | '8' | '9'));

                    let is_mobile_11 = digits.len() == 11
                        && digits.starts_with("05")
                        && matches!(digits.chars().nth(2), Some('0' | '3' | '4' | '5' | '6' | '7' | '8' | '9'));

                    if (is_mobile_10 || is_mobile_11) && (turkce_eksik_count == 0 || max_consecutive_cons >= 3) {
                        return true;
                    }

                    if digits.len() >= 11 {
                        break;
                    }
                } else if !c.is_whitespace() && is_turkce_eksik(c) {
                    // Türkçe ayırt edici harfe ulaşıldıysa bu pencere doğal metindir
                    break;
                }
            }
        }

        false
    }

    /// IP adresini KYK WiFi / CGNAT mahremiyet standartlarına uygun alt ağa maskeler:
    /// - IPv4: /24 (ilk 3 oktet korunur, örn: 192.168.1.0/24)
    /// - IPv6: /64 (ilk 4 blok korunur, örn: 2001:db8:85a3:8d3::/64)
    pub fn extract_subnet_prefix(ip: &str) -> String {
        let trimmed = ip.trim();
        if let Ok(addr) = trimmed.parse::<std::net::IpAddr>() {
            match addr {
                std::net::IpAddr::V4(ipv4) => {
                    let oct = ipv4.octets();
                    format!("{}.{}.{}.0/24", oct[0], oct[1], oct[2])
                }
                std::net::IpAddr::V6(ipv6) => {
                    let seg = ipv6.segments();
                    format!("{:x}:{:x}:{:x}:{:x}::/64", seg[0], seg[1], seg[2], seg[3])
                }
            }
        } else {
            "0.0.0.0/24".to_string()
        }
    }

    /// Yorum metnini Sıfır Bilgi güvenlik ilkelerine göre doğrular ve temizler.
    pub fn validate_comment_text(input: &str) -> Result<String, &'static str> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("Yorum metni boş olamaz.");
        }
        if trimmed.len() > 1000 {
            return Err("Yorum en fazla 1000 karakter olabilir.");
        }
        if Self::contains_tckn(trimmed) {
            return Err("Yorumunuz T.C. Kimlik Numarası (TCKN) içeremez.");
        }
        if Self::contains_phone_number(trimmed) {
            return Err("Yorumunuz telefon numarası veya iletişim bilgisi içeremez.");
        }
        if Self::is_spam(trimmed) {
            return Err("Yorumunuz spam veya zararlı bağlantı içeremez.");
        }

        let clean = Self::sanitize_html(trimmed);
        Ok(clean)
    }
}

fn turkish_to_lower(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'İ' => 'i',
            'I' => 'ı',
            _ => c.to_ascii_lowercase(),
        })
        .collect()
}

fn map_leetspeak_digit(c: char) -> Option<char> {
    match c {
        '0' | 'o' => Some('0'),
        '1' | 'i' | 'ı' => Some('1'),
        '2' | 'z' => Some('2'),
        '3' | 'e' => Some('3'),
        '4' | 'a' => Some('4'),
        '5' | 's' => Some('5'),
        '6' => Some('6'),
        '7' | 't' => Some('7'),
        '8' | 'b' => Some('8'),
        '9' | 'g' => Some('9'),
        _ => None,
    }
}

fn is_turkce_eksik(c: char) -> bool {
    matches!(
        c,
        'l' | 'm'
            | 'n'
            | 'r'
            | 'k'
            | 'y'
            | 'd'
            | 'c'
            | 'p'
            | 'v'
            | 'u'
            | 'ü'
            | 'ş'
            | 'ç'
            | 'ğ'
            | 'h'
            | 'ö'
            | 'f'
            | 'j'
    )
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü')
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
        assert!(!ContentGuard::is_spam(
            "Merhaba arkadaşlar, bugün kyk menüsünü inceledim ve şu adreste paylaştım: http://example.com/menu"
        ));
        assert!(ContentGuard::is_spam(
            "Linkler: www.site1.com ve www.site2.com adresleri."
        ));
        assert!(ContentGuard::is_spam(
            "Çooook lezzetliiiiiiiiiii bir yemekti."
        ));
        assert!(!ContentGuard::is_spam("Çooook lezzetliiiii bir yemekti."));

        // İç linkler (kepce.org) spam sayılmaz, birden fazla olsa dahi izin verilir
        assert!(!ContentGuard::is_spam(
            "Dünkü menü https://kepce.org/istanbul/2026-09-14 ile bugünkü https://www.kepce.org/istanbul/2026-09-15 menüsü çok farklıydı."
        ));
        assert!(!ContentGuard::is_spam(
            "Menü linki: https://kepce.org/istanbul"
        ));

        // URL kısaltıcılar tek başına dahi olsa anında engellenir
        assert!(ContentGuard::is_spam(
            "Burs çekilişi için şu bağlantıya tıklayın: bit.ly/kyk-burs"
        ));
        assert!(ContentGuard::is_spam(
            "Öğrenci indirimleri için tinyurl.com/ogrenci adresini ziyaret edebilirsiniz arkadaslar"
        ));

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

    #[test]
    fn test_tckn_validation() {
        // Geçerli TCKN'ler
        assert!(ContentGuard::is_valid_tckn("10000000146"));
        // Negatif modüler fark testi (odd_sum*7 < even_sum)
        assert!(ContentGuard::is_valid_tckn("19090909018"));

        // Hileli ve ardışık klavye dizileri (ilk 9 hane payload kontrolü ile elenmeli)
        assert!(!ContentGuard::is_valid_tckn("11111111110"));
        assert!(!ContentGuard::is_valid_tckn("12345678950"));
        assert!(!ContentGuard::is_valid_tckn("98765432102"));

        // Geçersiz uzunluk / 0 ile başlama / tek son hane
        assert!(!ContentGuard::is_valid_tckn("00000000000"));
        assert!(!ContentGuard::is_valid_tckn("1234567890"));
        assert!(!ContentGuard::is_valid_tckn("10000000147"));

        // Metin içinde TCKN tespiti
        assert!(ContentGuard::contains_tckn("Benim numaram 10000000146 lütfen"));
        assert!(ContentGuard::contains_tckn("19090909018"));
        assert!(!ContentGuard::contains_tckn("Bu yemek 10000000145 lira etmez"));
    }

    #[test]
    fn test_phone_obfuscation_and_leetspeak() {
        // Normal telefonlar
        assert!(ContentGuard::contains_phone_number("0555 123 45 67"));
        assert!(ContentGuard::contains_phone_number("5551234567"));
        assert!(ContentGuard::contains_phone_number("+90 532 123 45 67"));

        // Türkçe noktasız 'ı' ve leetspeak içeren gizlenmiş telefonlar
        assert!(ContentGuard::contains_phone_number("555ı234567"));
        assert!(ContentGuard::contains_phone_number("o532 123 45 67"));
        assert!(ContentGuard::contains_phone_number("sss a0e as 72"));

        // Cümle içi telefon tespiti
        assert!(ContentGuard::contains_phone_number("ulaş bana 0555 123 45 67 bekliyorum"));
        assert!(ContentGuard::contains_phone_number("numaram 555ı234567 yaz"));
    }

    #[test]
    fn test_scunthorpe_preservation() {
        // Fonotaktik Türkçe koruma: S_turkce_eksik harflerini barındıran doğal yemek yorumları ASLA engellenmez
        assert!(!ContentGuard::contains_phone_number("soslu bezelye"));
        assert!(!ContentGuard::contains_phone_number("saat 5'te çıkan kakaolu puding"));
        assert!(!ContentGuard::contains_phone_number("bugünkü mercimek çorbası çok lezzetliydi"));
        assert!(!ContentGuard::contains_phone_number("fiyat listesi"));
    }

    #[test]
    fn test_extract_subnet_prefix() {
        assert_eq!(ContentGuard::extract_subnet_prefix("192.168.1.42"), "192.168.1.0/24");
        assert_eq!(ContentGuard::extract_subnet_prefix("10.0.5.99"), "10.0.5.0/24");
        assert_eq!(
            ContentGuard::extract_subnet_prefix("2001:0db8:85a3:08d3:1319:8a2e:0370:7348"),
            "2001:db8:85a3:8d3::/64"
        );
    }

    #[test]
    fn test_validate_comment_text() {
        assert!(ContentGuard::validate_comment_text("Çok güzel bir mercimek çorbasıydı.").is_ok());
        assert!(ContentGuard::validate_comment_text("").is_err());
        assert!(ContentGuard::validate_comment_text("numaram 10000000146").is_err());
        assert!(ContentGuard::validate_comment_text("bana yaz 555ı234567").is_err());
        assert!(ContentGuard::validate_comment_text("sitemize gelin bit.ly/kyk-burs").is_err());
    }
}


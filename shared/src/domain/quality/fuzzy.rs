use crate::domain::quality::normalizer::tr_lowercase;

/// İki karakter arasındaki Türkçe diyakritik ağırlıklı değişim maliyetini hesaplar.
/// Noktasız/noktalı ve yumuşatılmış sesler (ı/i, o/ö, u/ü, c/ç, s/ş, g/ğ) 0.2 maliyet alır.
/// Diğer tüm karakter değişimleri tam (1.0) maliyet alır.
pub fn char_distance(a: char, b: char) -> f64 {
    if a == b {
        return 0.0;
    }
    let is_diacritic_pair = matches!(
        (a, b),
        ('ı', 'i')
            | ('i', 'ı')
            | ('o', 'ö')
            | ('ö', 'o')
            | ('u', 'ü')
            | ('ü', 'u')
            | ('c', 'ç')
            | ('ç', 'c')
            | ('s', 'ş')
            | ('ş', 's')
            | ('g', 'ğ')
            | ('ğ', 'g')
    );
    if is_diacritic_pair { 0.2 } else { 1.0 }
}

/// İki kelime (token) arasındaki ağırlıklı Levenshtein mesafesini hesaplar.
/// Eğer iki kelimenin harfleri birebir aynı multiset'e sahipse (metatez/harf göçüşmesi)
/// ve mesafe 2.0 ise, mesafe 1.0 olarak düzeltilir.
pub fn token_distance(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n as f64;
    }
    if n == 0 {
        return m as f64;
    }

    let mut dp = vec![vec![0.0f64; n + 1]; m + 1];

    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i as f64;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j as f64;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = char_distance(a_chars[i - 1], b_chars[j - 1]);
            let delete = dp[i - 1][j] + 1.0;
            let insert = dp[i][j - 1] + 1.0;
            let substitute = dp[i - 1][j - 1] + cost;

            dp[i][j] = delete.min(insert).min(substitute);
        }
    }

    let raw_dist = dp[m][n];

    // Metatez (harf göçüşmesi) indirimi:
    // İki kelimenin uzunluğu aynı, mesafe tam 2.0 ve harf multiset'leri birebir eşitse mesafeyi 1.0 kabul et.
    if m == n && (raw_dist - 2.0).abs() < 1e-4 {
        let mut a_sorted = a_chars.clone();
        let mut b_sorted = b_chars.clone();
        a_sorted.sort_unstable();
        b_sorted.sort_unstable();
        if a_sorted == b_sorted {
            return 1.0;
        }
    }

    raw_dist
}

/// İki kelimenin (token) benzer olup olmadığını kurallara göre belirler:
/// - 7 harften kısa kelimelerde: weighted_distance <= 1.0 olmalıdır.
/// - 7 harf ve üzeri kelimelerde: benzerlik oranı >= 0.82 olmalıdır.
pub fn is_token_similar(a: &str, b: &str) -> bool {
    let len = a.chars().count().max(b.chars().count());
    if len == 0 {
        return true;
    }
    let dist = token_distance(a, b);
    if len < 7 {
        dist <= 1.0 + 1e-6
    } else {
        let similarity = 1.0 - (dist / len as f64);
        similarity >= 0.82 - 1e-6
    }
}

/// Morfolojik kök ve yapım eki grupları.
/// İki yemek adayı aynı gruptan farklı bir kök taşıyorsa yakınsama kesinlikle reddedilir.
const PROTECTED_STEM_GROUPS: &[&[&str]] = &[
    // Kırmızı et
    &[
        "et",
        "etli",
        "etsiz",
        "kıyma",
        "kıymalı",
        "kıymasız",
        "kiyma",
        "kiymali",
        "kiymasiz",
        "kuşbaşı",
        "kusbasi",
    ],
    // Beyaz et
    &[
        "tavuk",
        "tavuklu",
        "piliç",
        "pilic",
        "hindi",
        "hindili",
        "balık",
        "balıklı",
        "balik",
        "balikli",
    ],
    // Süt ve peynir
    &[
        "peynir",
        "peynirli",
        "kaşar",
        "kaşarlı",
        "kasar",
        "kasarli",
        "lor",
        "lorlu",
        "yoğurt",
        "yoğurtlu",
        "yogurt",
        "yogurtlu",
    ],
    // Sebze ve bakliyat
    &[
        "patates",
        "patatesli",
        "domates",
        "domatesli",
        "mantar",
        "mantarlı",
        "mantarli",
        "nohut",
        "fasulye",
        "mercimek",
        "kavun",
        "kavurma",
    ],
    // Tahıl
    &[
        "pirinç", "pirinc", "bulgur", "makarna", "şehriye", "sehriye", "erişte", "eriste",
    ],
    // Tatlı / Puding ayrımı
    &[
        "sütlaç",
        "sutlac",
        "güllaç",
        "gullac",
        "muhallebi",
        "kazandibi",
    ],
];

/// İki kelime listesinde korumalı belirteç çakışması olup olmadığını kontrol eder.
/// Örneğin biri "domates" içerip diğeri "patates" içeriyorsa `false` döner.
fn check_protected_tokens_guard(tokens_a: &[&str], tokens_b: &[&str]) -> bool {
    for group in PROTECTED_STEM_GROUPS {
        let set_a: Vec<&str> = tokens_a
            .iter()
            .copied()
            .filter(|t| group.contains(t))
            .collect();
        let set_b: Vec<&str> = tokens_b
            .iter()
            .copied()
            .filter(|t| group.contains(t))
            .collect();

        if set_a.is_empty() != set_b.is_empty() {
            return false;
        }
        if set_a.len() != set_b.len() {
            return false;
        }
        for (a, b) in set_a.iter().zip(set_b.iter()) {
            if !is_token_similar(a, b) {
                return false;
            }
        }
    }
    true
}

#[derive(Debug, PartialEq, Eq)]
pub enum MatchResult {
    Exact,
    FuzzyMatch,
    NoMatch,
}

/// İki yemek adı arasında token bazlı güvenli eşleşme yapar.
/// Cümle bazlı `max_len` tuzaklarına düşmeden;
/// 1. Tam eşleşmeyi kontrol eder.
/// 2. Korumalı protein/sebze/tahıl köklerini denetler.
/// 3. Token sayısı eşitliğini ve en fazla 1 farklı token'ın benzerliğini doğrular.
pub fn match_dish_names(candidate: &str, canonical: &str) -> MatchResult {
    let cand_clean = tr_lowercase(candidate.trim());
    let canon_clean = tr_lowercase(canonical.trim());

    if cand_clean == canon_clean {
        return MatchResult::Exact;
    }

    let cand_tokens: Vec<&str> = cand_clean
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|t| !t.is_empty())
        .collect();
    let canon_tokens: Vec<&str> = canon_clean
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|t| !t.is_empty())
        .collect();

    if cand_tokens.is_empty() || canon_tokens.is_empty() {
        return MatchResult::NoMatch;
    }

    if cand_tokens.len() != canon_tokens.len() {
        return MatchResult::NoMatch;
    }

    if !check_protected_tokens_guard(&cand_tokens, &canon_tokens) {
        return MatchResult::NoMatch;
    }

    let mut diff_count = 0;
    let mut all_diffs_similar = true;

    for (a, b) in cand_tokens.iter().zip(canon_tokens.iter()) {
        if a != b {
            diff_count += 1;
            if !is_token_similar(a, b) {
                all_diffs_similar = false;
                break;
            }
        }
    }

    if diff_count == 0 {
        return MatchResult::Exact;
    }

    if all_diffs_similar {
        if diff_count == 1 {
            return MatchResult::FuzzyMatch;
        } else {
            let total_dist: f64 = cand_tokens
                .iter()
                .zip(canon_tokens.iter())
                .filter(|(a, b)| a != b)
                .map(|(a, b)| token_distance(a, b))
                .sum();
            if total_dist <= 1.4 + 1e-6 {
                return MatchResult::FuzzyMatch;
            }
        }
    }

    MatchResult::NoMatch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diacritic_weighted_matching() {
        assert!(is_token_similar("kizartma", "kızartma"));
        assert!(is_token_similar("şinitzel", "şnitzel"));
        assert!(is_token_similar("turlu", "türlü"));
        assert!(is_token_similar("sögüs", "söğüş"));
    }

    #[test]
    fn test_metathesis_vowel_migration() {
        // ezegolin <-> ezogelin: harfler aynı multiset, mesafe 2.0 -> 1.0 indirimi ile benzerlik 0.875 >= 0.82
        assert!(is_token_similar("ezegolin", "ezogelin"));
    }

    #[test]
    fn test_short_word_traps_rejected() {
        // güllaç vs sütlaç: mesafe 2.0 > 1.0 olduğu için reddedilmeli
        assert!(!is_token_similar("güllaç", "sütlaç"));
    }

    #[test]
    fn test_ingredient_traps_rejected() {
        // domatesli vs patatesli: benzerlik 6/9 = 0.66 < 0.82 reddedilmeli
        assert!(!is_token_similar("domatesli", "patatesli"));
        // Cümle seviyesinde korumalı belirteç engeli
        assert_eq!(
            match_dish_names("Domatesli Bulgur Pilavı", "Patatesli Bulgur Pilavı"),
            MatchResult::NoMatch
        );
        assert_eq!(
            match_dish_names("Pirinç Pilavı + Kavun", "Pirinç Pilavı + Kavurma"),
            MatchResult::NoMatch
        );
        assert_eq!(
            match_dish_names("Cevizli Güllaç", "Cevizli Sütlaç"),
            MatchResult::NoMatch
        );
        assert_eq!(
            match_dish_names("Tavuk Sote", "Et Sote"),
            MatchResult::NoMatch
        );
    }

    #[test]
    fn test_safe_dish_fuzzy_matches() {
        assert_eq!(
            match_dish_names("Karisik Kizartma", "Karışık Kızartma"),
            MatchResult::FuzzyMatch
        );
        assert_eq!(
            match_dish_names("Ezegolin Çorbası", "Ezogelin Çorbası"),
            MatchResult::FuzzyMatch
        );
        // Multi-word diacritic float precision test (0.6 + 0.4 = 1.0000000000000002)
        assert_eq!(
            match_dish_names("Kirmizi Mercimek Corbasi", "Kırmızı Mercimek Çorbası"),
            MatchResult::FuzzyMatch
        );
        // Parenthesis and punctuation trimming on tokens
        assert_eq!(
            match_dish_names("Mantı (Yoğurt + Sos)", "Manti (Yogurt + Sos)"),
            MatchResult::FuzzyMatch
        );
        // Protected group clash even inside parenthesis
        assert_eq!(
            match_dish_names("Mantı (Yoğurt)", "Manti (Tavuk)"),
            MatchResult::NoMatch
        );
    }
}

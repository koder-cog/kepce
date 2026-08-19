use std::sync::OnceLock;
use regex::Regex;

/// Turkish Title Casing helper for words.
/// Handles Turkish dotted/dotless I ('i' -> 'İ', 'ı' -> 'I') and preserves lowercase units/conjunctions.
pub fn tr_title_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let lower = word.to_lowercase().replace("i̇", "i").replace('I', "ı");
    
    // Turkish specific known spellings (ASCII to proper Turkish)
    match lower.as_str() {
        "izgara" | "ızgara" => return "Izgara".to_string(),
        "ispanak" | "ıspanak" => return "Ispanak".to_string(),
        "ispanaklı" | "ıspanaklı" => return "Ispanaklı".to_string(),
        "islim" => return "İslim".to_string(),
        "iskender" => return "İskender".to_string(),
        "incik" => return "İncik".to_string(),
        "inegöl" => return "İnegöl".to_string(),
        "izmir" => return "İzmir".to_string(),
        "imam" => return "İmam".to_string(),
        "içli" => return "İçli".to_string(),
        "işkembe" => return "İşkembe".to_string(),
        "iftariyelik" => return "İftariyelik".to_string(),
        _ => {}
    }
    
    // Conjunctions, units, and prepositions stay lowercase unless they are the start of a title
    let preserve_lower = [
        "ve", "veya", "ile", "de", "da", "ml", "g", "gr", "kg", "l", "lt", "kcal", "adet"
    ];
    if preserve_lower.contains(&lower.as_str()) {
        return lower;
    }

    let mut chars = lower.chars();
    if let Some(first) = chars.next() {
        let first_upper = match first {
            'i' => "İ".to_string(),
            'ı' => "I".to_string(),
            c => c.to_uppercase().to_string(),
        };
        let rest: String = chars.collect();
        format!("{}{}", first_upper, rest)
    } else {
        String::new()
    }
}

/// Applies Turkish title casing to a full text string while preserving parentheses, slashes, plus signs.
pub fn turkish_title_case(text: &str) -> String {
    let mut result = Vec::new();
    for token in text.split_whitespace() {
        // Handle parentheses or prefixes like "(100" or "+Patates"
        let trimmed_start = token.trim_start_matches(|c: char| !c.is_alphabetic());
        let prefix = &token[..token.len() - trimmed_start.len()];
        
        let word_only = trimmed_start.trim_end_matches(|c: char| !c.is_alphabetic());
        let suffix = &trimmed_start[word_only.len()..];

        if word_only.is_empty() {
            result.push(token.to_string());
        } else {
            let titled = tr_title_word(word_only);
            result.push(format!("{}{}{}", prefix, titled, suffix));
        }
    }
    result.join(" ")
}

/// Normalizes a single food item name: expands abbreviations, standardizes volume/liquid units,
/// fixes punctuation, double spaces, and Turkish casing.
pub fn normalize_food_name(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return s;
    }

    // 1. Clean bullet characters & leading/trailing hyphens/asterisks
    s = s.trim_start_matches(|c: char| c == '*' || c == '-' || c == '•' || c == '⁃' || c == '+' || c.is_whitespace())
         .trim_end_matches(|c: char| c == '*' || c == '-' || c == '•' || c == '⁃' || c.is_whitespace())
         .to_string();

    // 2. Normalize whitespace (collapse multiple spaces into one)
    static RE_SPACES: OnceLock<Regex> = OnceLock::new();
    let re_spaces = RE_SPACES.get_or_init(|| Regex::new(r"\s+").unwrap());
    s = re_spaces.replace_all(&s, " ").to_string();

    // 3. Normalize liquid and volume units
    static RE_500ML_SU: OnceLock<Regex> = OnceLock::new();
    let re_500ml_su = RE_500ML_SU.get_or_init(|| Regex::new(r"(?i)\b500\s*ml\.?\s*su\b").unwrap());
    s = re_500ml_su.replace_all(&s, "500 ml Su").to_string();

    static RE_200ML_AYRAN: OnceLock<Regex> = OnceLock::new();
    let re_200ml_ayran = RE_200ML_AYRAN.get_or_init(|| Regex::new(r"(?i)\b200\s*ml\.?\s*ayran\b").unwrap());
    s = re_200ml_ayran.replace_all(&s, "200 ml Ayran").to_string();

    static RE_200ML_SUT: OnceLock<Regex> = OnceLock::new();
    let re_200ml_sut = RE_200ML_SUT.get_or_init(|| Regex::new(r"(?i)\b200\s*ml\.?\s*süt\b").unwrap());
    s = re_200ml_sut.replace_all(&s, "200 ml Süt").to_string();

    static RE_200ML_MEYVE: OnceLock<Regex> = OnceLock::new();
    let re_200ml_meyve = RE_200ML_MEYVE.get_or_init(|| Regex::new(r"(?i)\b200\s*ml\.?\s*meyve\s*suyu\b").unwrap());
    s = re_200ml_meyve.replace_all(&s, "200 ml Meyve Suyu").to_string();

    static RE_330ML_SALGAM: OnceLock<Regex> = OnceLock::new();
    let re_330ml_salgam = RE_330ML_SALGAM.get_or_init(|| Regex::new(r"(?i)\b330\s*ml\.?\s*şalgam\b").unwrap());
    s = re_330ml_salgam.replace_all(&s, "330 ml Şalgam").to_string();

    static RE_GENERIC_ML: OnceLock<Regex> = OnceLock::new();
    let re_generic_ml = RE_GENERIC_ML.get_or_init(|| Regex::new(r"(?i)\b(\d+)\s*ml\.?\b").unwrap());
    s = re_generic_ml.replace_all(&s, "$1 ml").to_string();

    // 4. Bread & Gluten-Free standardizations
    static RE_GLUTENSIZ_ROLL: OnceLock<Regex> = OnceLock::new();
    let re_glutensiz_roll = RE_GLUTENSIZ_ROLL.get_or_init(|| Regex::new(r"(?i)\bglutensiz\s+roll(?:\s+ekmek)?\b").unwrap());
    s = re_glutensiz_roll.replace_all(&s, "Glutensiz Roll Ekmek").to_string();

    static RE_CEYREK_EKMEK: OnceLock<Regex> = OnceLock::new();
    let re_ceyrek_ekmek = RE_CEYREK_EKMEK.get_or_init(|| Regex::new(r"(?i)\bçeyrek\s+ekmek\b").unwrap());
    s = re_ceyrek_ekmek.replace_all(&s, "Çeyrek Ekmek").to_string();

    // 5. Expand Common Abbreviations (Most specific first)
    // Pilav abbreviations: "Şeh. Bulgur P.", "Sebzeli Bulgur P.", "Salçalı Bulgur P.", "Bulgur P.", "Pirinç P."
    static RE_SEH_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_seh_bulgur_p = RE_SEH_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bşeh(?:\.|\s+)\s*bulgur(?:\s+p\.?)?\b").unwrap());
    s = re_seh_bulgur_p.replace_all(&s, "Şehriyeli Bulgur Pilavı").to_string();

    static RE_SEBZELI_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_sebzeli_bulgur_p = RE_SEBZELI_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bsebzeli\s+bulgur\s+p\.?\b").unwrap());
    s = re_sebzeli_bulgur_p.replace_all(&s, "Sebzeli Bulgur Pilavı").to_string();

    static RE_SALCALI_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_salcali_bulgur_p = RE_SALCALI_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bsalçalı\s+bulgur\s+p\.?\b").unwrap());
    s = re_salcali_bulgur_p.replace_all(&s, "Salçalı Bulgur Pilavı").to_string();

    static RE_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_bulgur_p = RE_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bbulgur\s+p\.?\b").unwrap());
    s = re_bulgur_p.replace_all(&s, "Bulgur Pilavı").to_string();

    static RE_PIRINC_P: OnceLock<Regex> = OnceLock::new();
    let re_pirinc_p = RE_PIRINC_P.get_or_init(|| Regex::new(r"(?i)\bpirinç\s+p\.?\b").unwrap());
    s = re_pirinc_p.replace_all(&s, "Pirinç Pilavı").to_string();

    // Çorba abbreviations: "Mercimek Ç.", "Ezogelin Ç.", "Domates Ç."
    static RE_CORBA_ABBR: OnceLock<Regex> = OnceLock::new();
    let re_corba_abbr = RE_CORBA_ABBR.get_or_init(|| Regex::new(
        r"(?i)\b(mercimek|ezogelin|domates|yayla|tarhana|tavuk|düğün|şehriye|köz\s*biber|ayran\s*aşı|yeşil\s*mercimek|mahluta|brokoli|dövme|tutmaç)\s+ç\.?\b"
    ).unwrap());
    s = re_corba_abbr.replace_all(&s, "$1 Çorbası").to_string();

    // Yemek abbreviations: "Taze Fasulye Y.", "Etsiz Nohut Y.", "Kurufasulye Y."
    static RE_YEMEK_ABBR: OnceLock<Regex> = OnceLock::new();
    let re_yemek_abbr = RE_YEMEK_ABBR.get_or_init(|| Regex::new(
        r"(?i)\b(taze\s+fasulye|kuru\s+fasulye|kurufasulye|etsiz\s+nohut|nohut|bezelye|pırasa|ispanak|ıspanak|patates|kabak|türlü|kereviz|bamya|semizotu)\s+y\.?\b"
    ).unwrap());
    s = re_yemek_abbr.replace_all(&s, "$1 Yemeği").to_string();

    // Kızartma abbreviations: "Patates Kız.", "Karışık Kız."
    static RE_PATATES_KIZ: OnceLock<Regex> = OnceLock::new();
    let re_patates_kiz = RE_PATATES_KIZ.get_or_init(|| Regex::new(r"(?i)\bpatates\s+kız\.?\b").unwrap());
    s = re_patates_kiz.replace_all(&s, "Patates Kızartması").to_string();

    static RE_KARISIK_KIZ: OnceLock<Regex> = OnceLock::new();
    let re_karisik_kiz = RE_KARISIK_KIZ.get_or_init(|| Regex::new(r"(?i)\bkarışık\s+kız\.?\b").unwrap());
    s = re_karisik_kiz.replace_all(&s, "Karışık Kızartma").to_string();

    // Zeytinyağlı abbreviation: "Z.yağlı", "Z. yağlı"
    static RE_Z_YAGLI: OnceLock<Regex> = OnceLock::new();
    let re_z_yagli = RE_Z_YAGLI.get_or_init(|| Regex::new(r"(?i)\bz\.?\s*yağlı\b").unwrap());
    s = re_z_yagli.replace_all(&s, "Zeytinyağlı").to_string();

    // 6. Clean trailing dots that were leftover from abbreviations (e.g. "Pilavı.")
    s = s.trim_end_matches('.').trim().to_string();

    // 7. Apply proper Turkish Title Casing
    turkish_title_case(&s)
}

/// Splits a raw item string by '/' while smart-resolving orphaned adjectives/prefixes
/// (e.g. "Siyah / Yeşil Zeytin" -> ["Siyah Zeytin", "Yeşil Zeytin"],
/// "Zeytinli / Peynirli Açma" -> ["Zeytinli Açma", "Peynirli Açma"]).
pub fn split_smart_alternatives(raw_item: &str) -> Vec<String> {
    // Split outside parentheses by '/'
    let mut raw_parts = Vec::new();
    let mut current = String::new();
    let mut parens = 0;

    for c in raw_item.chars() {
        match c {
            '(' | '[' => { parens += 1; current.push(c); },
            ')' | ']' => { parens -= 1; current.push(c); },
            '/' if parens == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    raw_parts.push(trimmed.to_string());
                }
                current.clear();
            },
            _ => current.push(c),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        raw_parts.push(trimmed.to_string());
    }

    if raw_parts.len() <= 1 {
        let norm = normalize_food_name(raw_item);
        if norm.is_empty() {
            return Vec::new();
        }
        return vec![norm];
    }

    // Check for smart distribution of head noun across 2 alternatives
    let p1 = raw_parts[0].trim();
    let p2 = raw_parts[1].trim();

    let p1_lower = p1.to_lowercase().replace("i̇", "i").replace('I', "ı");
    let p2_lower = p2.to_lowercase().replace("i̇", "i").replace('I', "ı");

    // Case 1: Olives ("Siyah" / "Yeşil Zeytin" or "Yeşil" / "Siyah Zeytin")
    if p1_lower == "siyah" && p2_lower.contains("zeytin") {
        let n1 = "Siyah Zeytin".to_string();
        let n2 = normalize_food_name(p2);
        return vec![n1, n2];
    }
    if p1_lower == "yeşil" && p2_lower.contains("zeytin") {
        let n1 = "Yeşil Zeytin".to_string();
        let n2 = normalize_food_name(p2);
        return vec![n1, n2];
    }

    // Case 2: Pastries / Açma / Börek / Poğaça
    // ("Zeytinli" / "Peynirli Açma", "Peynirli" / "Patatesli Börek", "Sade" / "Peynirli Poğaça")
    let pastry_adjectives = ["zeytinli", "peynirli", "patatesli", "kaşarlı", "sade", "kıymalı", "ıspanaklı"];
    let pastry_nouns = ["açma", "açması", "börek", "böreği", "poğaça", "poğaçası", "kalem böreği", "sigara böreği", "tepsi böreği"];

    if pastry_adjectives.contains(&p1_lower.as_str()) {
        for noun in &pastry_nouns {
            if p2_lower.contains(noun) {
                let suffix = turkish_title_case(noun);
                let n1 = format!("{} {}", turkish_title_case(p1), suffix);
                let n2 = normalize_food_name(p2);
                return vec![n1, n2];
            }
        }
    }

    // Case 3: Pilavs ("Pirinç" / "Bulgur Pilavı", "Bulgur" / "Pirinç Pilavı")
    if (p1_lower == "pirinç" || p1_lower == "bulgur") && p2_lower.contains("pilav") {
        let n1 = format!("{} Pilavı", turkish_title_case(p1));
        let n2 = normalize_food_name(p2);
        return vec![n1, n2];
    }

    // Case 4: Cheeses ("Örgü" / "Çeçil Peyniri", "Kaşar" / "Beyaz Peynir")
    let cheese_adjectives = ["örgü", "çeçil", "kaşar", "tulum", "lor", "dil", "otlu"];
    if cheese_adjectives.contains(&p1_lower.as_str()) && p2_lower.contains("peynir") {
        let n1 = format!("{} Peyniri", turkish_title_case(p1));
        let n2 = normalize_food_name(p2);
        return vec![n1, n2];
    }

    // Default: normalize all parsed parts
    raw_parts.into_iter().map(|p| normalize_food_name(&p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_units() {
        assert_eq!(normalize_food_name("500 ml. su"), "500 ml Su");
        assert_eq!(normalize_food_name("500ml Su"), "500 ml Su");
        assert_eq!(normalize_food_name("200 ml. ayran"), "200 ml Ayran");
        assert_eq!(normalize_food_name("200ml Ayran"), "200 ml Ayran");
        assert_eq!(normalize_food_name("330 ml. şalgam"), "330 ml Şalgam");
    }

    #[test]
    fn test_normalize_abbreviations() {
        assert_eq!(normalize_food_name("Pirinç P."), "Pirinç Pilavı");
        assert_eq!(normalize_food_name("Bulgur P."), "Bulgur Pilavı");
        assert_eq!(normalize_food_name("Sebzeli Bulgur P."), "Sebzeli Bulgur Pilavı");
        assert_eq!(normalize_food_name("Salçalı Bulgur P."), "Salçalı Bulgur Pilavı");
        assert_eq!(normalize_food_name("Şeh. Bulgur P."), "Şehriyeli Bulgur Pilavı");
        assert_eq!(normalize_food_name("Mercimek Ç."), "Mercimek Çorbası");
        assert_eq!(normalize_food_name("Ezogelin Ç."), "Ezogelin Çorbası");
        assert_eq!(normalize_food_name("Domates Ç."), "Domates Çorbası");
        assert_eq!(normalize_food_name("Taze Fasulye Y."), "Taze Fasulye Yemeği");
        assert_eq!(normalize_food_name("Patates Kız."), "Patates Kızartması");
        assert_eq!(normalize_food_name("Z.yağlı Pırasa"), "Zeytinyağlı Pırasa");
    }

    #[test]
    fn test_smart_splitting_olives() {
        let res = split_smart_alternatives("Siyah / Yeşil Zeytin");
        assert_eq!(res, vec!["Siyah Zeytin", "Yeşil Zeytin"]);
    }

    #[test]
    fn test_smart_splitting_pastries() {
        let res1 = split_smart_alternatives("Zeytinli / Peynirli Açma");
        assert_eq!(res1, vec!["Zeytinli Açma", "Peynirli Açma"]);

        let res2 = split_smart_alternatives("Peynirli / Patatesli Börek");
        assert_eq!(res2, vec!["Peynirli Börek", "Patatesli Börek"]);
    }

    #[test]
    fn test_smart_splitting_pilav() {
        let res = split_smart_alternatives("Pirinç / Bulgur Pilavı");
        assert_eq!(res, vec!["Pirinç Pilavı", "Bulgur Pilavı"]);
    }

    #[test]
    fn test_turkish_title_casing() {
        assert_eq!(normalize_food_name("ıspanaklı börek"), "Ispanaklı Börek");
        assert_eq!(normalize_food_name("izgara köfte"), "Izgara Köfte");
        assert_eq!(normalize_food_name("bal + tereyağ"), "Bal + Tereyağ");
        assert_eq!(normalize_food_name("glutensiz roll"), "Glutensiz Roll Ekmek");
    }
}

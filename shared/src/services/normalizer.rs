use regex::Regex;
use std::sync::OnceLock;

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
        "ve", "veya", "ile", "de", "da", "ml", "g", "gr", "kg", "l", "lt", "kcal", "adet",
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

/// Represents a normalized food item with its canonical name and extracted portion/amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedFoodItem {
    pub name: String,
    pub amount: Option<String>,
}

impl NormalizedFoodItem {
    pub fn new(name: impl Into<String>, amount: Option<String>) -> Self {
        Self {
            name: name.into(),
            amount,
        }
    }
}

/// Splits a string by delimiter only when the delimiter is outside parentheses or brackets.
pub fn split_outside_parens(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut parens = 0;

    for c in s.chars() {
        match c {
            '(' | '[' => {
                parens += 1;
                current.push(c);
            }
            ')' | ']' => {
                if parens > 0 {
                    parens -= 1;
                }
                current.push(c);
            }
            c if c == delimiter && parens == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    parts
}

/// Standardizes a portion or amount string into consistent spacing and unit format.
pub fn standardize_amount(raw: &str) -> String {
    let s = raw.trim().trim_end_matches('.').trim();

    static RE_AMOUNT_SPACING: OnceLock<Regex> = OnceLock::new();
    let re_spacing = RE_AMOUNT_SPACING.get_or_init(|| {
        Regex::new(
            r"(?i)^(\d+(?:[.,]\d+)?)\s*(ml|l|lt|g|gr|kg|adet|paket|porsiyon|dilim|şişe|kutu)$",
        )
        .unwrap()
    });

    if let Some(caps) = re_spacing.captures(s) {
        let val_str = caps.get(1).unwrap().as_str().replace(',', ".");
        let unit = caps.get(2).unwrap().as_str().to_lowercase();

        match unit.as_str() {
            "gr" => return format!("{} g", val_str),
            "l" | "lt" => {
                if let Ok(val) = val_str.parse::<f32>() {
                    if (val - 0.5).abs() < 0.01 {
                        return "500 ml".to_string();
                    } else if (val - 0.2).abs() < 0.01 {
                        return "200 ml".to_string();
                    } else if (val - 0.33).abs() < 0.01 {
                        return "330 ml".to_string();
                    } else if (val - 1.0).abs() < 0.01 {
                        return "1 lt".to_string();
                    }
                }
                return format!("{} lt", val_str);
            }
            _ => return format!("{} {}", val_str, unit),
        }
    }
    s.to_string()
}

/// Extracts bread type and portion/quantity information.
/// Handles cases like "Çeyrek Ekmek" -> (Ekmek, 1/4 adet), "Çeyrek Ekmek (80 g)" -> (Ekmek, 1/4 adet (80 g)).
fn try_extract_bread(raw: &str, existing_gram: Option<String>) -> Option<NormalizedFoodItem> {
    static RE_BREAD: OnceLock<Regex> = OnceLock::new();
    let re_bread = RE_BREAD.get_or_init(|| {
        Regex::new(r"(?i)^\s*(çeyrek|1/4|yarım|1/2|tam|1)\s*(?:adet)?\s*(.*?ekmek.*?)$").unwrap()
    });

    if let Some(caps) = re_bread.captures(raw) {
        let portion_kw = caps.get(1).unwrap().as_str().to_lowercase();
        let portion_str = match portion_kw.as_str() {
            "çeyrek" | "1/4" => "1/4 adet",
            "yarım" | "1/2" => "1/2 adet",
            "tam" | "1" => "1 adet",
            _ => "1 adet",
        };

        let amount = if let Some(gram) = existing_gram {
            Some(format!("{} ({})", portion_str, gram))
        } else {
            Some(portion_str.to_string())
        };

        let bread_name_raw = caps.get(2).unwrap().as_str().trim();
        let canonical_bread_name = if bread_name_raw.eq_ignore_ascii_case("ekmek") {
            "Ekmek".to_string()
        } else {
            turkish_title_case(bread_name_raw)
        };

        return Some(NormalizedFoodItem::new(canonical_bread_name, amount));
    }

    None
}

/// Normalizes a single food item segment, extracting portions, units, and cleaning the title.
fn normalize_single_food_item(raw: &str) -> NormalizedFoodItem {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return NormalizedFoodItem::new("", None);
    }

    // 1. Clean bullet characters & leading/trailing hyphens/asterisks
    s = s
        .trim_start_matches(|c: char| {
            c == '*' || c == '-' || c == '•' || c == '⁃' || c == '+' || c.is_whitespace()
        })
        .trim_end_matches(|c: char| {
            c == '*' || c == '-' || c == '•' || c == '⁃' || c == '+' || c.is_whitespace()
        })
        .to_string();

    if s.is_empty() {
        return NormalizedFoodItem::new("", None);
    }

    let mut extracted_amount = None;

    // 2. Extract numerical portion / contract specification inside parentheses
    // e.g. "(50 g)", "(120 gr)", "(200 ml)", "(90 g Et)", "(60 g Et)", "(100 g Et-izgara Sebze ile)"
    // Descriptive parentheses like "(Yoğurt+Sos)" or "(Göbek Marul+Dilim Limon)" are preserved.
    static RE_PAREN_PORTION: OnceLock<Regex> = OnceLock::new();
    let re_paren_portion = RE_PAREN_PORTION.get_or_init(|| {
        Regex::new(
            r"(?i)\s*\(\s*(\d+(?:[.,]\d+)?\s*(?:g|gr|kg|ml|lt|l|adet|porsiyon|dilim|paket)(?:\s+[A-Za-zğüşıöçĞÜŞİÖÇ]+(?:-[^)]*)?)?)\s*\)",
        )
        .unwrap()
    });

    if let Some(caps) = re_paren_portion.captures(&s) {
        let inside = caps.get(1).unwrap().as_str().trim();
        // Clean contract suffixes like "-izgara Sebze ile"
        let clean_amt = if let Some(dash_idx) = inside.find('-') {
            inside[..dash_idx].trim().to_string()
        } else {
            inside.to_string()
        };
        extracted_amount = Some(standardize_amount(&clean_amt));
        s = re_paren_portion.replace(&s, "").to_string();
    }

    // 3. Check for bread variants after extracting possible parenthetical grams
    if let Some(bread_item) = try_extract_bread(&s, extracted_amount.clone()) {
        return bread_item;
    }

    // 4. Extract leading quantity/package prefixes (e.g. "500 ml. su", "1 Paket Ayran", "1 adet Karışık Tost")
    static RE_PREFIX_AMOUNT: OnceLock<Regex> = OnceLock::new();
    let re_prefix_amount = RE_PREFIX_AMOUNT.get_or_init(|| {
        Regex::new(
            r"(?i)^\s*(\d+(?:[.,]\d+)?\s*(?:ml|l|lt|g|gr|kg|adet|paket|porsiyon|dilim|şişe|kutu)\.?)\s+(.+)$",
        )
        .unwrap()
    });

    if let Some(caps) = re_prefix_amount.captures(&s).map(|c| {
        (
            c.get(1).unwrap().as_str().to_string(),
            c.get(2).unwrap().as_str().to_string(),
        )
    }) {
        let prefix = caps.0;
        let remainder = caps.1.trim();

        // Check if remainder also starts with a quantity (e.g. "1 adet 500 ml Su")
        if let Some(next_caps) = re_prefix_amount.captures(remainder) {
            let next_prefix = next_caps.get(1).unwrap().as_str().to_string();
            let next_rem = next_caps.get(2).unwrap().as_str().trim();
            extracted_amount = Some(standardize_amount(&next_prefix));
            s = next_rem.to_string();
        } else {
            let std_prefix = standardize_amount(&prefix);
            if extracted_amount.is_none() {
                extracted_amount = Some(std_prefix);
            }
            s = remainder.to_string();
        }
    }

    // 5. Extract trailing quantity/volume suffixes (e.g. "Su 500 ml", "Ayran 200 ml", "Yoğurt 200 gr")
    static RE_SUFFIX_AMOUNT: OnceLock<Regex> = OnceLock::new();
    let re_suffix_amount = RE_SUFFIX_AMOUNT.get_or_init(|| {
        Regex::new(r"(?i)^(.+?)\s+(\d+(?:[.,]\d+)?\s*(?:ml|l|lt|g|gr|kg|adet|paket|dilim)\.?)$")
            .unwrap()
    });

    if extracted_amount.is_none()
        && let Some(caps) = re_suffix_amount.captures(&s)
    {
        let name_cand = caps.get(1).unwrap().as_str().trim();
        let amt_cand = caps.get(2).unwrap().as_str().trim();
        if name_cand.chars().any(|c| c.is_alphabetic()) {
            extracted_amount = Some(standardize_amount(amt_cand));
            s = name_cand.to_string();
        }
    }

    // 6. Shifted Cell Guard: If the string is purely a standalone unit or quantity without food name
    static RE_PURE_AMOUNT: OnceLock<Regex> = OnceLock::new();
    let re_pure_amount = RE_PURE_AMOUNT.get_or_init(|| {
        Regex::new(
            r"(?i)^\s*\d+(?:[.,]\d+)?\s*(?:ml|l|lt|g|gr|kg|adet|paket|porsiyon|dilim)?\.?\s*$",
        )
        .unwrap()
    });
    if re_pure_amount.is_match(&s) {
        let std_amt = standardize_amount(&s);
        return NormalizedFoodItem::new("", Some(std_amt));
    }

    // 7. Standardize spacing and punctuation
    static RE_SPACES: OnceLock<Regex> = OnceLock::new();
    let re_spaces = RE_SPACES.get_or_init(|| Regex::new(r"\s+").unwrap());
    s = re_spaces.replace_all(&s, " ").to_string();

    static RE_PLUS: OnceLock<Regex> = OnceLock::new();
    let re_plus = RE_PLUS.get_or_init(|| Regex::new(r"\s*\+\s*").unwrap());
    s = re_plus.replace_all(&s, " + ").to_string();

    // 8. TDK Compound Word Standardizations
    static RE_DEREOTU: OnceLock<Regex> = OnceLock::new();
    let re_dereotu = RE_DEREOTU.get_or_init(|| Regex::new(r"(?i)\bdere\s+ot(u|lu)?\b").unwrap());
    s = re_dereotu.replace_all(&s, "dereot$1").to_string();

    static RE_SEMIZOTU: OnceLock<Regex> = OnceLock::new();
    let re_semizotu = RE_SEMIZOTU.get_or_init(|| Regex::new(r"(?i)\bsemiz\s+ot(u|lu)?\b").unwrap());
    s = re_semizotu.replace_all(&s, "semizot$1").to_string();

    static RE_COREKOTU: OnceLock<Regex> = OnceLock::new();
    let re_corekotu =
        RE_COREKOTU.get_or_init(|| Regex::new(r"(?i)\bçöre[k]?\s+ot(u|lu)?\b").unwrap());
    s = re_corekotu.replace_all(&s, "çöreot$1").to_string();

    static RE_KURUFASULYE: OnceLock<Regex> = OnceLock::new();
    let re_kurufasulye = RE_KURUFASULYE.get_or_init(|| Regex::new(r"(?i)\bkurufasulye\b").unwrap());
    s = re_kurufasulye.replace_all(&s, "kuru fasulye").to_string();

    // 9. Bread standardizations (e.g. Glutensiz Roll)
    static RE_GLUTENSIZ_ROLL: OnceLock<Regex> = OnceLock::new();
    let re_glutensiz_roll = RE_GLUTENSIZ_ROLL
        .get_or_init(|| Regex::new(r"(?i)\bglutensiz\s+roll(?:\s+ekmek)?\b").unwrap());
    s = re_glutensiz_roll
        .replace_all(&s, "Glutensiz Roll Ekmek")
        .to_string();

    // 10. Expand Common Abbreviations (Most specific first)
    static RE_SEH_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_seh_bulgur_p = RE_SEH_BULGUR_P
        .get_or_init(|| Regex::new(r"(?i)\bşeh(?:\.|\s+)\s*bulgur(?:\s+p\.?)?\b").unwrap());
    s = re_seh_bulgur_p
        .replace_all(&s, "Şehriyeli Bulgur Pilavı")
        .to_string();

    static RE_SEBZELI_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_sebzeli_bulgur_p =
        RE_SEBZELI_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bsebzeli\s+bulgur\s+p\.?\b").unwrap());
    s = re_sebzeli_bulgur_p
        .replace_all(&s, "Sebzeli Bulgur Pilavı")
        .to_string();

    static RE_SALCALI_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_salcali_bulgur_p =
        RE_SALCALI_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bsalçalı\s+bulgur\s+p\.?\b").unwrap());
    s = re_salcali_bulgur_p
        .replace_all(&s, "Salçalı Bulgur Pilavı")
        .to_string();

    static RE_BULGUR_P: OnceLock<Regex> = OnceLock::new();
    let re_bulgur_p = RE_BULGUR_P.get_or_init(|| Regex::new(r"(?i)\bbulgur\s+p\.?\b").unwrap());
    s = re_bulgur_p.replace_all(&s, "Bulgur Pilavı").to_string();

    static RE_PIRINC_P: OnceLock<Regex> = OnceLock::new();
    let re_pirinc_p = RE_PIRINC_P.get_or_init(|| Regex::new(r"(?i)\bpirinç\s+p\.?\b").unwrap());
    s = re_pirinc_p.replace_all(&s, "Pirinç Pilavı").to_string();

    // Çorba abbreviations and generative expansions
    static RE_K_MERCIMEK: OnceLock<Regex> = OnceLock::new();
    let re_k_mercimek = RE_K_MERCIMEK
        .get_or_init(|| Regex::new(r"(?i)\bk(?:\.|\s+)\s*mercimek\s*çorba(?:sı)?\b").unwrap());
    s = re_k_mercimek
        .replace_all(&s, "Kırmızı Mercimek Çorbası")
        .to_string();

    static RE_Y_MERCIMEK: OnceLock<Regex> = OnceLock::new();
    let re_y_mercimek = RE_Y_MERCIMEK.get_or_init(|| {
        Regex::new(r"(?i)\b(erişteli\s+)?y(?:\.|\s+)\s*mercimek\s*çorba(?:sı)?\b").unwrap()
    });
    s = re_y_mercimek
        .replace_all(&s, "${1}Yeşil Mercimek Çorbası")
        .to_string();

    // Generative soup expansion: expands any `<Ad> Ç.` or `<Ad> Ç` token into `<Ad> Çorbası`
    static RE_CORBA_ABBR: OnceLock<Regex> = OnceLock::new();
    let re_corba_abbr = RE_CORBA_ABBR.get_or_init(|| {
        Regex::new(
            r"(?i)\b([A-Za-zğüşıöçĞÜŞİÖÇ]{2,}(?:\s+[A-Za-zğüşıöçĞÜŞİÖÇ]{2,})*)\s+[çÇ]\.?(\s*(?:\+|,|$))",
        )
        .unwrap()
    });
    s = re_corba_abbr.replace_all(&s, "$1 Çorbası$2").to_string();

    // Generative soup suffix: standardizes any `<Ad> Çorba` into `<Ad> Çorbası`
    static RE_CORBA_SUFFIX: OnceLock<Regex> = OnceLock::new();
    let re_corba_suffix = RE_CORBA_SUFFIX.get_or_init(|| {
        Regex::new(
            r"(?i)\b([A-Za-zğüşıöçĞÜŞİÖÇ]{2,}(?:\s+[A-Za-zğüşıöçĞÜŞİÖÇ]{2,})*)\s+çorba(\s*(?:\+|,|$))",
        )
        .unwrap()
    });
    s = re_corba_suffix.replace_all(&s, "$1 Çorbası$2").to_string();

    // Salata/Piyaz abbreviations
    static RE_K_FASULYE_PIYAZ: OnceLock<Regex> = OnceLock::new();
    let re_k_fasulye_piyaz = RE_K_FASULYE_PIYAZ
        .get_or_init(|| Regex::new(r"(?i)\bk(?:\.|\s+)\s*fasulye\s*piyazı\b").unwrap());
    s = re_k_fasulye_piyaz
        .replace_all(&s, "Kuru Fasulye Piyazı")
        .to_string();

    static RE_ARPA_SEH_SALATA: OnceLock<Regex> = OnceLock::new();
    let re_arpa_seh_salata = RE_ARPA_SEH_SALATA
        .get_or_init(|| Regex::new(r"(?i)\barpa\s+şeh(?:\.|\s+)\s*salatası\b").unwrap());
    s = re_arpa_seh_salata
        .replace_all(&s, "Arpa Şehriye Salatası")
        .to_string();

    // Yemek abbreviations
    static RE_YEMEK_ABBR: OnceLock<Regex> = OnceLock::new();
    let re_yemek_abbr = RE_YEMEK_ABBR.get_or_init(|| {
        Regex::new(
            r"(?i)\b(taze\s+fasulye|kuru\s+fasulye|kurufasulye|etsiz\s+nohut|nohut|bezelye|pırasa|ispanak|ıspanak|patates|kabak|türlü|kereviz|bamya|semizotu)\s+y\.?\b",
        )
        .unwrap()
    });
    s = re_yemek_abbr.replace_all(&s, "$1 Yemeği").to_string();

    // Kızartma abbreviations
    static RE_PATATES_KIZ: OnceLock<Regex> = OnceLock::new();
    let re_patates_kiz =
        RE_PATATES_KIZ.get_or_init(|| Regex::new(r"(?i)\b(?:patates|pat)\.?\s*kız\.?\b").unwrap());
    s = re_patates_kiz
        .replace_all(&s, "Patates Kızartması")
        .to_string();

    static RE_KARISIK_KIZ: OnceLock<Regex> = OnceLock::new();
    let re_karisik_kiz =
        RE_KARISIK_KIZ.get_or_init(|| Regex::new(r"(?i)\bkarışık\s+kız\.?\b").unwrap());
    s = re_karisik_kiz
        .replace_all(&s, "Karışık Kızartma")
        .to_string();

    // Zeytinyağlı abbreviation
    static RE_Z_YAGLI: OnceLock<Regex> = OnceLock::new();
    let re_z_yagli =
        RE_Z_YAGLI.get_or_init(|| Regex::new(r"(?i)\b(?:z\.?\s*yağlı|zyt\.?|zeyt\.)\s*").unwrap());
    s = re_z_yagli.replace_all(&s, "Zeytinyağlı ").to_string();

    // 11. Clean trailing dots leftover from abbreviations
    s = s.trim_end_matches('.').trim().to_string();

    // 12. Apply proper Turkish Title Casing
    let titled = turkish_title_case(&s);
    NormalizedFoodItem::new(titled, extracted_amount)
}

/// Normalizes a food item, extracting portions and handling compound dishes (Option C).
/// Compound dishes with '+' are split outside parentheses, normalized individually,
/// and recombined without losing component-specific portion information.
pub fn normalize_food_item(raw: &str) -> NormalizedFoodItem {
    let s = raw.trim();
    if s.is_empty() {
        return NormalizedFoodItem::new("", None);
    }

    let parts = split_outside_parens(s, '+');
    if parts.len() <= 1 {
        return normalize_single_food_item(s);
    }

    let normalized_pieces: Vec<NormalizedFoodItem> = parts
        .iter()
        .map(|p| normalize_single_food_item(p))
        .filter(|p| !p.name.is_empty())
        .collect();

    if normalized_pieces.is_empty() {
        return NormalizedFoodItem::new("", None);
    }

    if normalized_pieces.len() == 1 {
        return normalized_pieces.into_iter().next().unwrap();
    }

    let combined_name = normalized_pieces
        .iter()
        .map(|p| p.name.as_str())
        .collect::<Vec<_>>()
        .join(" + ");

    let amounts_count = normalized_pieces
        .iter()
        .filter(|p| p.amount.is_some())
        .count();

    let combined_amount = if amounts_count == 0 {
        None
    } else if amounts_count == normalized_pieces.len() {
        Some(
            normalized_pieces
                .iter()
                .map(|p| p.amount.as_deref().unwrap())
                .collect::<Vec<_>>()
                .join(" + "),
        )
    } else if normalized_pieces.len() == 2
        && normalized_pieces[0].amount.is_none()
        && normalized_pieces[1].amount.is_some()
    {
        // Option C pattern: "Lahana Sarması + Yoğurt (50 g)" -> "+ 50 g Yoğurt"
        Some(format!(
            "+ {} {}",
            normalized_pieces[1].amount.as_deref().unwrap(),
            normalized_pieces[1].name
        ))
    } else if normalized_pieces.len() == 2
        && normalized_pieces[0].amount.is_some()
        && normalized_pieces[1].amount.is_none()
    {
        // Option C pattern: "Lahana Sarması (200 g) + Yoğurt" -> "200 g Lahana Sarması +"
        Some(format!(
            "{} {} +",
            normalized_pieces[0].amount.as_deref().unwrap(),
            normalized_pieces[0].name
        ))
    } else {
        // Multi-component partial amounts
        let formatted = normalized_pieces
            .iter()
            .map(|p| {
                if let Some(ref amt) = p.amount {
                    format!("{} ({})", p.name, amt)
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" + ");
        Some(formatted)
    };

    NormalizedFoodItem::new(combined_name, combined_amount)
}

/// Normalizes a single food item name: expands abbreviations, standardizes volume/liquid units,
/// fixes punctuation, and Turkish casing while returning the canonical name.
pub fn normalize_food_name(raw: &str) -> String {
    normalize_food_item(raw).name
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
            '(' | '[' => {
                parens += 1;
                current.push(c);
            }
            ')' | ']' => {
                parens -= 1;
                current.push(c);
            }
            '/' if parens == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    raw_parts.push(trimmed.to_string());
                }
                current.clear();
            }
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
    let pastry_adjectives = [
        "zeytinli",
        "peynirli",
        "patatesli",
        "kaşarlı",
        "sade",
        "kıymalı",
        "ıspanaklı",
    ];
    let pastry_nouns = [
        "açma",
        "açması",
        "börek",
        "böreği",
        "poğaça",
        "poğaçası",
        "kalem böreği",
        "sigara böreği",
        "tepsi böreği",
    ];

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
    raw_parts
        .into_iter()
        .map(|p| normalize_food_name(&p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_units() {
        assert_eq!(normalize_food_name("500 ml. su"), "Su");
        assert_eq!(normalize_food_name("500ml Su"), "Su");
        assert_eq!(normalize_food_name("200 ml. ayran"), "Ayran");
        assert_eq!(normalize_food_name("200ml Ayran"), "Ayran");
        assert_eq!(normalize_food_name("330 ml. şalgam"), "Şalgam");
        assert_eq!(normalize_food_name("200 gr. Yoğurt"), "Yoğurt");
        assert_eq!(normalize_food_name("1 kg. Elma"), "Elma");
        assert_eq!(normalize_food_name("1 Paket Ayran"), "Ayran");
        assert_eq!(normalize_food_name("1 adet Karışık Tost"), "Karışık Tost");
    }

    #[test]
    fn test_food_item_portions() {
        assert_eq!(
            normalize_food_item("500 ml. su"),
            NormalizedFoodItem::new("Su", Some("500 ml".into()))
        );
        assert_eq!(
            normalize_food_item("500ml Su"),
            NormalizedFoodItem::new("Su", Some("500 ml".into()))
        );
        assert_eq!(
            normalize_food_item("200 ml. ayran"),
            NormalizedFoodItem::new("Ayran", Some("200 ml".into()))
        );
        assert_eq!(
            normalize_food_item("1 Paket Ayran"),
            NormalizedFoodItem::new("Ayran", Some("1 paket".into()))
        );
        assert_eq!(
            normalize_food_item("200 gr. Yoğurt"),
            NormalizedFoodItem::new("Yoğurt", Some("200 g".into()))
        );
        assert_eq!(
            normalize_food_item("1 kg. Elma"),
            NormalizedFoodItem::new("Elma", Some("1 kg".into()))
        );
        assert_eq!(
            normalize_food_item("Fırın Tavuk (120 g)"),
            NormalizedFoodItem::new("Fırın Tavuk", Some("120 g".into()))
        );
        assert_eq!(
            normalize_food_item("Et Sote (90 g Et)"),
            NormalizedFoodItem::new("Et Sote", Some("90 g Et".into()))
        );
        assert_eq!(
            normalize_food_item("Mantı (Yoğurt+Sos)"),
            NormalizedFoodItem::new("Mantı (Yoğurt + Sos)", None)
        );
    }

    #[test]
    fn test_bread_normalization() {
        assert_eq!(
            normalize_food_item("Çeyrek Ekmek"),
            NormalizedFoodItem::new("Ekmek", Some("1/4 adet".into()))
        );
        assert_eq!(
            normalize_food_item("1/4 Ekmek"),
            NormalizedFoodItem::new("Ekmek", Some("1/4 adet".into()))
        );
        assert_eq!(
            normalize_food_item("Çeyrek Ekmek (80 g)"),
            NormalizedFoodItem::new("Ekmek", Some("1/4 adet (80 g)".into()))
        );
        assert_eq!(
            normalize_food_item("1/4 Adet (80 g) Ekmek"),
            NormalizedFoodItem::new("Ekmek", Some("1/4 adet (80 g)".into()))
        );
        assert_eq!(
            normalize_food_item("Yarım Ekmek"),
            NormalizedFoodItem::new("Ekmek", Some("1/2 adet".into()))
        );
        assert_eq!(
            normalize_food_item("Tam Ekmek"),
            NormalizedFoodItem::new("Ekmek", Some("1 adet".into()))
        );
        assert_eq!(
            normalize_food_item("Çeyrek Kepekli Ekmek"),
            NormalizedFoodItem::new("Kepekli Ekmek", Some("1/4 adet".into()))
        );
    }

    #[test]
    fn test_option_c_compound_portions() {
        assert_eq!(
            normalize_food_item("Lahana Sarması (200 g) + Yoğurt (50 g)"),
            NormalizedFoodItem::new("Lahana Sarması + Yoğurt", Some("200 g + 50 g".into()))
        );
        assert_eq!(
            normalize_food_item("Lahana Sarması + Yoğurt (50 g)"),
            NormalizedFoodItem::new("Lahana Sarması + Yoğurt", Some("+ 50 g Yoğurt".into()))
        );
        assert_eq!(
            normalize_food_item("Lahana Sarması (200 g) + Yoğurt"),
            NormalizedFoodItem::new(
                "Lahana Sarması + Yoğurt",
                Some("200 g Lahana Sarması +".into())
            )
        );
        assert_eq!(
            normalize_food_item("Köfte (120 g) + Patates (80 g)"),
            NormalizedFoodItem::new("Köfte + Patates", Some("120 g + 80 g".into()))
        );
    }

    #[test]
    fn test_normalize_abbreviations() {
        assert_eq!(normalize_food_name("Pirinç P."), "Pirinç Pilavı");
        assert_eq!(normalize_food_name("Bulgur P."), "Bulgur Pilavı");
        assert_eq!(
            normalize_food_name("Sebzeli Bulgur P."),
            "Sebzeli Bulgur Pilavı"
        );
        assert_eq!(
            normalize_food_name("Salçalı Bulgur P."),
            "Salçalı Bulgur Pilavı"
        );
        assert_eq!(
            normalize_food_name("Şeh. Bulgur P."),
            "Şehriyeli Bulgur Pilavı"
        );
        assert_eq!(normalize_food_name("Mercimek Ç."), "Mercimek Çorbası");
        assert_eq!(normalize_food_name("Ezogelin Ç."), "Ezogelin Çorbası");
        assert_eq!(normalize_food_name("Domates Ç."), "Domates Çorbası");
        assert_eq!(
            normalize_food_name("Taze Fasulye Y."),
            "Taze Fasulye Yemeği"
        );
        assert_eq!(normalize_food_name("Patates Kız."), "Patates Kızartması");
        assert_eq!(normalize_food_name("Pat. Kız."), "Patates Kızartması");
        assert_eq!(normalize_food_name("Z.yağlı Pırasa"), "Zeytinyağlı Pırasa");
        assert_eq!(normalize_food_name("Zyt. Fasulye"), "Zeytinyağlı Fasulye");
        assert_eq!(normalize_food_name("Zeyt. Pırasa"), "Zeytinyağlı Pırasa");
        assert_eq!(
            normalize_food_name("K. Mercimek Çorba"),
            "Kırmızı Mercimek Çorbası"
        );
        assert_eq!(
            normalize_food_name("Erişteli Y. Mercimek Çorba"),
            "Erişteli Yeşil Mercimek Çorbası"
        );
        assert_eq!(
            normalize_food_name("K. Fasulye Piyazı"),
            "Kuru Fasulye Piyazı"
        );
        assert_eq!(
            normalize_food_name("Arpa Şeh. Salatası"),
            "Arpa Şehriye Salatası"
        );
    }

    #[test]
    fn test_expand_soup_shorthand() {
        assert_eq!(normalize_food_name("Yüksük Ç."), "Yüksük Çorbası");
        assert_eq!(normalize_food_name("Toyga Ç."), "Toyga Çorbası");
        assert_eq!(normalize_food_name("Toyga Ç"), "Toyga Çorbası");
        assert_eq!(normalize_food_name("Arabaşı Ç"), "Arabaşı Çorbası");
        assert_eq!(normalize_food_name("Cennet Ç"), "Cennet Çorbası");
        assert_eq!(normalize_food_name("Ezogelin Çorba"), "Ezogelin Çorbası");
        assert_eq!(normalize_food_name("Mercimek Çorba"), "Mercimek Çorbası");
        assert_eq!(normalize_food_name("Yayla Çorba"), "Yayla Çorbası");
        assert_eq!(normalize_food_name("Tarhana Çorba"), "Tarhana Çorbası");
        assert_eq!(
            normalize_food_name("Tavuk Suyu Çorba"),
            "Tavuk Suyu Çorbası"
        );
        assert_eq!(normalize_food_name("Çeşmi Nigar Ç."), "Çeşmi Nigar Çorbası");
        assert_eq!(
            normalize_food_name("Mercimek Çorba + Salata"),
            "Mercimek Çorbası + Salata"
        );
    }

    #[test]
    fn test_normalize_tight_plus() {
        assert_eq!(normalize_food_name("Bal+tereyağ"), "Bal + Tereyağ");
        assert_eq!(
            normalize_food_name("Tavuk Sote+Pilav"),
            "Tavuk Sote + Pilav"
        );
        assert_eq!(normalize_food_name("+Bal+tereyağ+"), "Bal + Tereyağ");
        assert_eq!(normalize_food_name("Reçel +Tereyağ"), "Reçel + Tereyağ");
    }

    #[test]
    fn test_tdk_rules() {
        assert_eq!(normalize_food_name("Dere Otlu Poğaça"), "Dereotlu Poğaça");
        assert_eq!(normalize_food_name("dere otu"), "Dereotu");
        assert_eq!(
            normalize_food_name("semiz otu salatası"),
            "Semizotu Salatası"
        );
        assert_eq!(normalize_food_name("çörek otlu poğaça"), "Çöreotlu Poğaça");
        assert_eq!(normalize_food_name("Kurufasulye"), "Kuru Fasulye");
        assert_eq!(normalize_food_name("Kurufasulye Y."), "Kuru Fasulye Yemeği");
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
        assert_eq!(
            normalize_food_name("glutensiz roll"),
            "Glutensiz Roll Ekmek"
        );
    }

    #[test]
    fn test_dish_normalization_corpus() {
        let tsv = include_str!("../../../worker/tests/fixtures/dish_normalization.tsv");
        for line in tsv.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split('\t');
            let input = parts.next().expect("input");
            let expected = parts.next().expect("expected");
            let actual = normalize_food_name(input);
            assert_eq!(
                actual, expected,
                "Normalizasyon hatası: input='{}', expected='{}', actual='{}'",
                input, expected, actual
            );
        }
    }
}

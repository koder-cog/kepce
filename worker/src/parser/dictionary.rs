use std::collections::HashSet;
use std::sync::OnceLock;

static DICTIONARY: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub fn get_dictionary() -> &'static HashSet<&'static str> {
    DICTIONARY.get_or_init(|| {
        let mut set = HashSet::new();
        let words = [
            "ekmek", "götür", "menü", "çeyrek", "yeşil", "siyah", "zeytin", "peynir", "çay", "bitki", "çayı", 
            "tavuk", "yumurta", "patates", "pilavı", "haşlanmış", "mercimek", "glutensiz", "roll", "karışık", 
            "beyaz", "domates", "pirinç", "soslu", "yoğurt", "ezogelin", "omlet", "makarna", "adet", "mevsim", 
            "peynirli", "kaşar", "salata", "ayran", "söğüş", "kızartması", "meyve", "kaşarlı", "havuç", "kızartma", 
            "sebzeleri", "dilim", "çikolata", "sürülebilir", "tarhana", "bal", "bulgur", "köfte", "biber", "sade", 
            "şehriyeli", "zyt", "iftariyelik", "hurma", "tereyağ", "yemeği", "poğaça", "sebze", "çeşitleri", "kebabı", 
            "çölyak", "elma", "köz", "örgü", "çeçil", "reçel", "tahinli", "pekmez", "açma", "çikolatalı", "fasulye", 
            "mısır", "izgara", "milföy", "börek", "kavurma", "pizza", "simit", "sucuklu", "sebzeli", "dolma", "menemen", 
            "pilav", "salatalık", "graten", "şehriye", "aysberg", "mücver", "brokoli", "labne", "etsiz", "turşu", 
            "zeytinli", "yöresel", "külbastı", "tava", "aşı", "nohut", "yayla", "dere", "otlu", "kuru", "toyga", "pişi", 
            "helva", "krem", "haydari", "türlü", "düğün", "üstü", "sote", "köftesi", "yüksük", "spagetti", "böreği", 
            "çoban", "limon", "çıtır", "kroket", "cacık", "karnabahar", "sosisli", "muz", "acılı", "ezme", "şakşuka", 
            "tavuklu", "imam", "bayıldı", "köri", "pasta", "piliç", "taze", "ispanak", "mısırlı", "bezelye", "cevizli", 
            "çin", "piyaz", "barbunya", "fajita", "kavurması", "salatası", "kaşık", "çilekli", "sosis", "kokteyl", 
            "salçalı", "kabak", "ratatuy", "erişteli", "hünkar", "beğendi", "yoğurtlu", "çiftlik", "ketçap", "mayonez", 
            "baklava", "usülü", "patlıcan", "tatlısı", "parmak", "sucuk", "burgu", "top", "süt", "tantuni", "dürüm", 
            "çiğköfte", "çökertme", "mozaik", "sultan", "şiş", "lavaş", "karpuz", "kremalı", "şinitzel", "kumpir", 
            "tarator", "beğendili", "orman", "çorba", "paket", "cornflakes", "sütlaç", "pideli", "enginar", "fesleğen", 
            "suyu", "mix", "tereyağı", "triliçe", "tutmaç", "pirzola", "fellah", "yaş", "limonata", "sandal", "supangle", 
            "napoliten", "mantı", "meyvesi", "kabuksuz", "çeşmi", "nigar", "bezelyeli", "bahçe", "hamburger", "göbek", 
            "marul", "kornişon", "patatesli", "tepsi", "kısır", "lahana", "sarma", "sulu", "şekerpare", "tulumba", "çilek", 
            "musakka", "roti", "julyen", "kuşbaşılı", "alaca", "normal", "soğan", "kıbrıs", "falafel", "portakal", "beşamel", 
            "browni", "sigara", "gram", "inegöl", "püresi", "portakallı", "kereviz", "güllaç", "alinazik", "magnolya", 
            "revani", "sefası", "arap", "pembe", "tel", "çiğ", "topkapı", "ekşili", "kalye", "prinç", "kek", "etli", "usulü", 
            "karnıyarık", "terbiyeli", "erişte", "baget", "magnolia", "kalem", "salam", "piknik", "grubu", "sürebilir", 
            "jülyen", "ispanaklı", "çanak", "mercimekli", "kemalpaşa", "kıymalı", "tirit", "trileçe", "kağıt", "ankara", 
            "tatlı", "sinitzel", "pırasa", "izmir", "yumurtalı", "tavukburger", "dondurma", "pankek", "tahin", 
            "semizotu", "kavun", "et", "tas", "fırın", "fırında", "balık", "levrek", "sos", "özel", "harçlı", "somon", "fırınlanmış"
        ];
        for w in words {
            set.insert(w);
        }
        set
    })
}

/// Computes the ratio of dictionary words found in the given item name string.
/// Returns a score between 0.0 and 100.0.
pub fn calculate_match_ratio(item_name: &str) -> f64 {
    let dict = get_dictionary();
    
    // Turkish lowercase conversion and sanitization
    let lower = item_name.to_lowercase().replace("i̇", "i").replace('I', "ı");
    
    // Replace non-alphabetic chars with spaces
    let cleaned: String = lower.chars()
        .map(|c| if c.is_alphabetic() || c.is_whitespace() { c } else { ' ' })
        .collect();

    // Extract words with at least 2 characters (handles Turkish chars correctly)
    let words: Vec<&str> = cleaned.split_whitespace()
        .filter(|w| w.chars().count() >= 2)
        .collect();

    if words.is_empty() {
        return 0.0;
    }

    let mut match_count = 0;
    for word in &words {
        if dict.contains(word) {
            match_count += 1;
        }
    }

    (match_count as f64 / words.len() as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_match_ratio() {
        assert_eq!(calculate_match_ratio("Mercimek Çorbası"), 50.0); // "mercimek" is in dict, "çorbası" is not (only "çorba")
        assert_eq!(calculate_match_ratio("Pilavı (Tavuklu)"), 100.0); // "pilavı", "tavuklu" are in dict
        assert_eq!(calculate_match_ratio("Aşırı saçma sapan bir kelime"), 0.0);
        assert_eq!(calculate_match_ratio("Süt"), 100.0); // 3 char word
        assert_eq!(calculate_match_ratio("Ve pilav"), 50.0); // "ve" is now counted (length 2) but not in dict, so score is 50.0
    }
}

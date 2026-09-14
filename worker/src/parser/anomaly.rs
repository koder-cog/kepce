use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::{OnceLock, Mutex};

static MODEL: OnceLock<Option<Mutex<TextEmbedding>>> = OnceLock::new();
static BASELINE_VECTOR: OnceLock<Option<Vec<f32>>> = OnceLock::new();

pub fn get_model() -> Option<&'static Mutex<TextEmbedding>> {
    MODEL.get_or_init(|| {
        match TextEmbedding::try_new(InitOptions::new(EmbeddingModel::ParaphraseMLMiniLML12V2)) {
            Ok(model) => Some(Mutex::new(model)),
            Err(e) => {
                tracing::warn!("Text embedding model yüklenemedi (çevrimdışı fallback aktif): {:?}", e);
                None
            }
        }
    }).as_ref()
}

pub fn get_baseline_vector() -> Option<&'static Vec<f32>> {
    BASELINE_VECTOR.get_or_init(|| {
        let model_mutex = get_model()?;
        let mut model = model_mutex.lock().ok()?;
        let dict = crate::parser::dictionary::get_dictionary();
        let baseline_text: String = dict.iter().copied().collect::<Vec<_>>().join(" ");
        let mut embeddings = model.embed(vec![baseline_text], None).ok()?;
        embeddings.pop()
    }).as_ref()
}

/// Calculate cosine distance between two vectors. Returns 1.0 - cosine_similarity.
/// Lower value (near 0) means vectors are more similar.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (val_a, val_b) in a.iter().zip(b.iter()) {
        dot_product += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }
    let sim = if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    };
    1.0 - sim
}

/// Model yüklenemediğinde veya ağ çevrimdışıyken çalışan sözlük tabanlı anomali puanlayıcısı.
/// 0.0 ile 1.0 arasında bir mesafe döner; yüksek değer anomaliyi (yemek dışı çöp metin) gösterir.
pub fn calculate_lexical_fallback_distance(text: &str) -> Option<f32> {
    if text.trim().is_empty() {
        return None;
    }
    let match_ratio = crate::parser::dictionary::calculate_match_ratio(text);
    let distance = (1.0 - (match_ratio / 100.0) as f32).clamp(0.0, 1.0);
    tracing::debug!(
        "Çevrimdışı anomali analizi uygulandı (oran: {:.1}%, mesafe: {:.2})",
        match_ratio, distance
    );
    Some(distance)
}

pub fn calculate_menu_distance(text: &str) -> Option<f32> {
    if text.trim().is_empty() {
        return None;
    }

    // Tier 1: Fastembed nöral embedding modeli
    if let (Some(baseline), Some(model_mutex)) = (get_baseline_vector(), get_model()) {
        if let Ok(mut model) = model_mutex.lock() {
            if let Ok(mut embeddings) = model.embed(vec![text.to_string()], None) {
                if let Some(emb) = embeddings.pop() {
                    return Some(cosine_distance(&emb, baseline));
                }
            }
        }
    }

    // Tier 2: Model çevrimdışıyken sözlük eşleşme oranından türetilen deterministik mesafe
    calculate_lexical_fallback_distance(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexical_fallback_normal_menu() {
        let text = "Mercimek Çorbası Tavuk Sote Pirinç Pilavı Ayran Çeyrek Ekmek";
        let dist = calculate_lexical_fallback_distance(text).unwrap();
        // Tanınan yemeklerde anomali mesafesi düşük olmalı (< 0.40)
        assert!(dist < 0.40, "Geçerli yemek menüsü için mesafe düşük olmalı: {}", dist);
    }

    #[test]
    fn test_lexical_fallback_garbage_text() {
        let text = "404 Not Found nginx error gateway timeout connection refused unauthorized";
        let dist = calculate_lexical_fallback_distance(text).unwrap();
        // Çöp ve hata metinlerinde anomali mesafesi yüksek olmalı (> 0.65)
        assert!(dist > 0.65, "Çöp/hata metninde anomali mesafesi yüksek olmalı: {}", dist);
    }

    #[test]
    fn test_lexical_fallback_empty() {
        assert!(calculate_lexical_fallback_distance("").is_none());
        assert!(calculate_lexical_fallback_distance("   ").is_none());
    }
}

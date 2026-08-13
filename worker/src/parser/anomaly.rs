use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::{OnceLock, Mutex};

static MODEL: OnceLock<Mutex<TextEmbedding>> = OnceLock::new();
static BASELINE_VECTOR: OnceLock<Vec<f32>> = OnceLock::new();

pub fn get_model() -> &'static Mutex<TextEmbedding> {
    MODEL.get_or_init(|| {
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::ParaphraseMLMiniLML12V2))
            .expect("Failed to initialize text embedding model");
        Mutex::new(model)
    })
}

pub fn get_baseline_vector() -> &'static Vec<f32> {
    BASELINE_VECTOR.get_or_init(|| {
        let mut model = get_model().lock().unwrap();
        let dict = crate::parser::dictionary::get_dictionary();
        let baseline_text: String = dict.iter().copied().collect::<Vec<_>>().join(" ");
        let mut embeddings = model.embed(vec![baseline_text], None).expect("Failed to compute baseline embedding");
        embeddings.pop().unwrap()
    })
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

pub fn calculate_menu_distance(text: &str) -> Option<f32> {
    if text.trim().is_empty() {
        return None;
    }
    let baseline = get_baseline_vector();
    let mut model = get_model().lock().unwrap();
    
    if let Ok(mut embeddings) = model.embed(vec![text.to_string()], None) {
        if let Some(emb) = embeddings.pop() {
            return Some(cosine_distance(&emb, baseline));
        }
    }
    None
}

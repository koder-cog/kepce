*******************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tracing::{error, info};

const MODEL_BASE_URL: &str = "https://huggingface.co/nanelimon/bert-base-turkish-offensive";

#[derive(Clone)]
pub struct AppState {
    classifier: Arc<BertClassifier>,
}

pub struct BertClassifier {
    bert: BertModel,
    classifier_weight: Tensor,
    classifier_bias: Tensor,
    tokenizer: Tokenizer,
    device: Device,
    labels: Vec<&'static str>,
}

async fn download_file_if_missing(url: &str, destination: &Path) -> anyhow::Result<()> {
    if destination.exists() {
        info!("Found cached file: {:?}", destination);
        return Ok(());
    }

    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    info!("Downloading {} to {:?}...", url, destination);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download {}: HTTP {}", url, response.status());
    }

    let bytes = response.bytes().await?;
    tokio::fs::write(destination, bytes).await?;
    info!("Successfully saved {:?}", destination);
    Ok(())
}

impl BertClassifier {
    pub async fn load() -> anyhow::Result<Self> {
        let device = Device::Cpu;
        let cache_dir = PathBuf::from(std::env::var("MODEL_DIR").unwrap_or_else(|_| "/root/.cache/models/bert-offensive".to_string()));

        let config_path = cache_dir.join("config.json");
        let vocab_path = cache_dir.join("vocab.txt");
        let weights_path = cache_dir.join("model.safetensors");

        // 1. Dosyaları doğrudan indir / doğrula
        download_file_if_missing(&format!("{}/raw/main/config.json", MODEL_BASE_URL), &config_path).await?;
        download_file_if_missing(&format!("{}/raw/main/vocab.txt", MODEL_BASE_URL), &vocab_path).await?;
        download_file_if_missing(&format!("{}/resolve/main/model.safetensors", MODEL_BASE_URL), &weights_path).await?;

        // 2. Config
        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let config: BertConfig = serde_json::from_str(&config_str)?;

        // 3. Tokenizer (WordPiece from vocab.txt)
        let wp = tokenizers::models::wordpiece::WordPiece::from_file(&vocab_path.to_string_lossy())
            .build()
            .map_err(|e| anyhow::anyhow!("Wordpiece build error: {}", e))?;
        let mut tokenizer = Tokenizer::new(wp);
        tokenizer.with_pre_tokenizer(Some(tokenizers::pre_tokenizers::bert::BertPreTokenizer));

        // 4. Weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)?
        };

        let bert = BertModel::load(vb.pp("bert"), &config)?;
        let classifier_weight = vb.get((5, config.hidden_size), "classifier.weight")?;
        let classifier_bias = vb.get(5, "classifier.bias")?;

        let labels = vec!["INSULT", "OTHER", "PROFANITY", "RACIST", "SEXIST"];

        info!("BERT Moderation model loaded successfully into memory.");
        Ok(Self {
            bert,
            classifier_weight,
            classifier_bias,
            tokenizer,
            device,
            labels,
        })
    }

    pub fn predict(&self, text: &str) -> anyhow::Result<ModerationResult> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;

        let token_ids: Vec<u32> = encoding.get_ids().to_vec();
        let token_type_ids: Vec<u32> = encoding.get_type_ids().to_vec();

        let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = Tensor::new(token_type_ids.as_slice(), &self.device)?.unsqueeze(0)?;

        let embeddings = self.bert.forward(&input_ids, &token_type_ids, None)?;
        
        // [CLS] token embedding: shape [hidden_size]
        let cls_emb = embeddings.i((0, 0))?;
        
        // Linear: cls_emb * W^T + b
        let logits = (cls_emb.unsqueeze(0)?.matmul(&self.classifier_weight.t()?)?.squeeze(0)? + &self.classifier_bias)?;
        let probs = candle_nn::ops::softmax(&logits, 0)?;
        let probs_vec: Vec<f32> = probs.to_vec1()?;

        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut max_score = -1.0;
        let mut best_label = "OTHER";

        for (i, label) in self.labels.iter().enumerate() {
            let score = probs_vec.get(i).copied().unwrap_or(0.0);
            scores.insert(label.to_string(), score);
            if score > max_score {
                max_score = score;
                best_label = label;
            }
        }

        let is_toxic = best_label != "OTHER" && max_score >= 0.5;

        Ok(ModerationResult {
            is_toxic,
            label: best_label.to_string(),
            score: max_score,
            scores,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CheckRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ModerationResult {
    pub is_toxic: bool,
    pub label: String,
    pub score: f32,
    pub scores: HashMap<String, f32>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub model: &'static str,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ready",
        model: "nanelimon/bert-base-turkish-offensive",
    })
}

async fn check_text(
    State(state): State<AppState>,
    Json(payload): Json<CheckRequest>,
) -> Result<Json<ModerationResult>, (StatusCode, String)> {
    if payload.text.trim().is_empty() {
        let mut scores = HashMap::new();
        scores.insert("OTHER".to_string(), 1.0);
        return Ok(Json(ModerationResult {
            is_toxic: false,
            label: "OTHER".to_string(),
            score: 1.0,
            scores,
        }));
    }

    match state.classifier.predict(&payload.text) {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("Prediction error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Inference error: {}", e),
            ))
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,moderator=debug".into()),
        )
        .init();

    info!("Starting Kepçe Native Moderator Sidecar...");

    let classifier = BertClassifier::load().await?;
    let state = AppState {
        classifier: Arc::new(classifier),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/check", post(check_text))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8002".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    info!("Moderator listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

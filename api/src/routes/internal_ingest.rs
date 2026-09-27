use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use sha2::{Digest, Sha256};

use crate::{
    config::AppState, dto::internal_ingest::InternalKykyemekIngestRequest,
    services::internal_ingest::InternalIngestService,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ingest/kykyemek", post(ingest_kykyemek))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
}

async fn ingest_kykyemek(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<InternalKykyemekIngestRequest>,
) -> impl IntoResponse {
    let expected_secret = match state.config.internal_ingest_secret.as_ref() {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            tracing::warn!(
                "İç aktarım isteği reddedildi: INTERNAL_INGEST_SECRET sunucuda tanımlanmamış."
            );
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "İç aktarım servisi bu sunucuda aktif değil"
                })),
            )
                .into_response();
        }
    };

    let provided_token = headers
        .get("x-internal-token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim())
        .unwrap_or("");

    // Zamanlama saldırılarını önlemek için SHA-256 hash'leri üzerinden karşılaştır
    let expected_hash = Sha256::digest(expected_secret.as_bytes());
    let provided_hash = Sha256::digest(provided_token.as_bytes());

    if expected_hash != provided_hash {
        tracing::warn!("İç aktarım isteğinde geçersiz token sağlandı.");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Geçersiz veya eksik X-Internal-Token"
            })),
        )
            .into_response();
    }

    match InternalIngestService::ingest_kykyemek(&state.db, payload).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            tracing::error!("İç aktarım işleminde hata: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("İç aktarım başarısız: {}", err)
                })),
            )
                .into_response()
        }
    }
}

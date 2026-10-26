use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use std::path::Path;

use crate::{
    config::AppState, dto::internal_ingest::InternalKykyemekIngestRequest,
    services::internal_ingest::InternalIngestService,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ingest/kykyemek", post(ingest_kykyemek))
        .route("/traffic-logs/pull", get(pull_traffic_logs))
        .route("/traffic-logs/ack", post(ack_traffic_logs))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
}

fn verify_sync_token(headers: &HeaderMap, expected_secret: Option<&str>) -> bool {
    let expected = match expected_secret {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return false,
    };

    let provided = headers
        .get("x-sync-token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim())
        .unwrap_or("");

    let expected_hash = Sha256::digest(expected.as_bytes());
    let provided_hash = Sha256::digest(provided.as_bytes());

    expected_hash.ct_eq(&provided_hash).unwrap_u8() == 1
}

/// 5651 RAM Tamponundan şifreli paketleri çeker (`GET /api/v1/internal/traffic-logs/pull`).
async fn pull_traffic_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !verify_sync_token(&headers, state.config.traffic_log_sync_token.as_deref()) {
        tracing::warn!("Trafik log çekme isteğinde yetkisiz token sağlandı.");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Geçersiz veya eksik X-Sync-Token" })),
        )
            .into_response();
    }

    let ram_path = Path::new(&state.config.traffic_log_ram_dir);
    match crate::core::traffic_logger::pull_log_batch(ram_path, 1000).await {
        Ok(Some((seq_id, hash, bytes))) => {
            let mut response = Response::new(axum::body::Body::from(bytes));
            *response.status_mut() = StatusCode::OK;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            response.headers_mut().insert(
                "x-batch-seq-id",
                seq_id.parse().unwrap(),
            );
            response.headers_mut().insert(
                "x-batch-hash",
                hash.parse().unwrap(),
            );
            response
        }
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("RAM tamponundan log çekilirken hata: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Log çekme hatası: {}", e) })),
            )
                .into_response()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct AckQuery {
    pub up_to_seq_id: String,
}

/// Türkiye sunucusu tarafından çekilen paketleri RAM tamponundan siler (`POST /api/v1/internal/traffic-logs/ack?up_to_seq_id=...`).
async fn ack_traffic_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AckQuery>,
) -> impl IntoResponse {
    if !verify_sync_token(&headers, state.config.traffic_log_sync_token.as_deref()) {
        tracing::warn!("Trafik log ack isteğinde yetkisiz token sağlandı.");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Geçersiz veya eksik X-Sync-Token" })),
        )
            .into_response();
    }

    let ram_path = Path::new(&state.config.traffic_log_ram_dir);
    match crate::core::traffic_logger::ack_log_batch(ram_path, &query.up_to_seq_id).await {
        Ok(deleted_count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "deleted_count": deleted_count
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("RAM tamponundan log silinirken hata: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Log ack hatası: {}", e) })),
            )
                .into_response()
        }
    }
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

    let expected_hash = Sha256::digest(expected_secret.as_bytes());
    let provided_hash = Sha256::digest(provided_token.as_bytes());

    if expected_hash.ct_eq(&provided_hash).unwrap_u8() != 1 {
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

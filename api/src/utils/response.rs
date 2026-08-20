// Kepçe API - Utils: Response & HTTP Caching Helpers
// ====================================================

use axum::{
    body::Body,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use crate::error::AppError;

/// Serializes data to JSON and returns a response with Cache-Control and ETag headers.
/// If the request contains a matching If-None-Match header, returns 304 Not Modified.
pub fn cached_json_response<T: serde::Serialize>(
    headers: &HeaderMap,
    data: &T,
    max_age_secs: u32,
) -> Result<Response, AppError> {
    let json_bytes = serde_json::to_vec(data).map_err(|e| {
        tracing::error!("JSON serialization error: {}", e);
        AppError::Internal("Serialization error".to_string())
    })?;

    // Compute SHA-256 ETag
    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    let hash = hasher.finalize();
    let hex_hash: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    let etag = format!("\"{}\"", hex_hash);

    let cache_control = format!(
        "public, max-age={}, s-maxage=3600, stale-while-revalidate=86400",
        max_age_secs
    );

    // Check If-None-Match conditional request
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
        let clean_match = if_none_match
            .trim()
            .trim_start_matches("W/")
            .trim_matches('"')
            .trim_end_matches("-gzip");
        if clean_match == hex_hash || if_none_match.trim() == "*" {
            return Ok(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header(header::CACHE_CONTROL, cache_control)
                .header(header::ETAG, etag)
                .body(Body::empty())
                .unwrap_or_else(|_| StatusCode::NOT_MODIFIED.into_response()));
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, cache_control)
        .header(header::ETAG, etag)
        .body(Body::from(json_bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}

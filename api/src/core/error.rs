//! Kepçe API birleşik hata yönetim katmanı.
//!
//! Route seviyesinde elle `(StatusCode, String)` türetmek yerine,
//! `IntoResponse` trait'ini uygulayan merkezi `AppError` enum'u kullanılır.
//! Bu sayede tüm endpoint'ler istemciye standart hata şemasında JSON yanıt döner.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
    TooManyRequests(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::TooManyRequests(msg) => write!(f, "Too Many Requests: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, is_not_found) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, true),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, false),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, false),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, false),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, false),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, false),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg, false),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        let mut res = (status, body).into_response();
        if is_not_found {
            res.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=300, s-maxage=600"),
            );
        }
        res
    }
}

impl From<crate::services::user::UserError> for AppError {
    fn from(err: crate::services::user::UserError) -> Self {
        match err {
            crate::services::user::UserError::NotFound => {
                AppError::NotFound("User not found".to_string())
            }
            crate::services::user::UserError::ConsentRequired => AppError::Forbidden(
                "Yurt dışı barındırma onayını tamamlamanız gerekmektedir.".to_string(),
            ),
            crate::services::user::UserError::DatabaseError(e) => {
                tracing::error!("Database error in UserService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

impl From<crate::services::developer::DeveloperError> for AppError {
    fn from(err: crate::services::developer::DeveloperError) -> Self {
        match err {
            crate::services::developer::DeveloperError::NotFound => {
                AppError::NotFound("Bulunamadı".to_string())
            }
            crate::services::developer::DeveloperError::Unauthorized => {
                AppError::Forbidden("Yetkisiz işlem".to_string())
            }
            crate::services::developer::DeveloperError::UnverifiedUser => AppError::Forbidden(
                "Proje veya API anahtarı oluşturmak için e-postanızı onaylamalısınız.".to_string(),
            ),
            crate::services::developer::DeveloperError::InvalidInput(msg) => {
                AppError::BadRequest(msg)
            }
            crate::services::developer::DeveloperError::DatabaseError(e) => {
                tracing::error!("Database error in DeveloperService: {}", e);
                AppError::Internal("Veritabanı hatası".to_string())
            }
        }
    }
}

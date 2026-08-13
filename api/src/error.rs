// Kepçe API — Birleşik Hata Yönetimi
// ====================================
//
// Tüm API genelinde tutarlı hata yanıtları sağlar.
// Her route'ta tekrar tekrar (StatusCode, String) döndürmek yerine,
// merkezi bir AppError enum'u kullanılır.
//
// Avantajları:
//   - Her endpoint aynı hata formatını döndürür
//   - Axum'ın IntoResponse trait'i ile otomatik dönüşüm
//   - Hata loglama tek noktadan yapılır
//
// Örnek:
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
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
            AppError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::TooManyRequests(msg) => write!(f, "Too Many Requests: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<crate::services::user::UserError> for AppError {
    fn from(err: crate::services::user::UserError) -> Self {
        match err {
            crate::services::user::UserError::NotFound => AppError::NotFound("User not found".to_string()),
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
            crate::services::developer::DeveloperError::NotFound => AppError::NotFound("Bulunamadı".to_string()),
            crate::services::developer::DeveloperError::Unauthorized => AppError::Forbidden("Yetkisiz işlem".to_string()),
            crate::services::developer::DeveloperError::UnverifiedUser => AppError::Forbidden("Proje veya API anahtarı oluşturmak için e-postanızı onaylamalısınız.".to_string()),
            crate::services::developer::DeveloperError::InvalidInput(msg) => AppError::BadRequest(msg),
            crate::services::developer::DeveloperError::DatabaseError(e) => {
                tracing::error!("Database error in DeveloperService: {}", e);
                AppError::Internal("Veritabanı hatası".to_string())
            }
        }
    }
}


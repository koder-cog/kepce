// Kepçe API - Extractors: Public API Key
// ========================================
//
// 3. parti geliştiricilerin X-API-Key header'ı ile
// erişim sağlaması için kullanılır.
//
// API anahtarını veritabanından doğrular,
// kullanıcının tier bilgisini (standart/ticari) çözer,
// rate limit ve kullanım istatistiği için bilgi sağlar.

use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::request::Parts,
};
use sha2::{Sha256, Digest};
use shared::entities::{api_keys, prelude::*};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use crate::config::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;

#[derive(Debug)]
pub struct ValidApiKey {
    pub model: api_keys::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for ValidApiKey
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let api_key_header = parts.headers.get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("API Key eksik (X-API-Key header'ı gerekli)".to_string()))?;

        // Hash raw key using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(api_key_header.as_bytes());
        let hash_result = hasher.finalize();
        let key_hash: String = hash_result.iter().map(|b| format!("{:02x}", b)).collect();

        // Query database
        let api_key_model = ApiKeys::find()
            .filter(api_keys::Column::KeyHash.eq(&key_hash))
            .filter(api_keys::Column::IsActive.eq(true))
            .one(&app_state.db)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking API Key: {}", e);
                AppError::Internal("Veritabanı hatası".to_string())
            })?
            .ok_or_else(|| AppError::Unauthorized("Geçersiz veya pasif API Key".to_string()))?;

        // Record request & Check daily rate limits
        app_state.usage_tracker.record_request(
            &app_state.db,
            api_key_model.id,
            &api_key_model.tier,
            false
        ).await.map_err(AppError::TooManyRequests)?;

        Ok(ValidApiKey {
            model: api_key_model,
        })
    }
}

pub enum IngestionAuth {
    User(AuthenticatedUser),
    Developer(api_keys::Model),
}

#[async_trait]
impl<S> FromRequestParts<S> for IngestionAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let has_cookie = parts.headers.contains_key(axum::http::header::COOKIE);
        let has_bearer = parts.headers.contains_key(axum::http::header::AUTHORIZATION);
        let has_api_key = parts.headers.contains_key("X-API-Key");

        // 1. Try JWT (Cookie or Bearer)
        if has_cookie || has_bearer {
            match AuthenticatedUser::from_request_parts(parts, state).await {
                Ok(user) => return Ok(IngestionAuth::User(user)),
                Err(e) => {
                    // Sadece API Key verilmemişse hemen hata dönüyoruz.
                    // Çünkü tarayıcılardan gelen alakasız bir çerez (örn. Cloudflare) 
                    // bu bloğu tetikleyip valid API Key'i engelleyebilir.
                    if !has_api_key {
                        return Err(e);
                    }
                }
            }
        }

        // 2. Try API Key
        if has_api_key {
            match ValidApiKey::from_request_parts(parts, state).await {
                Ok(api_key) => return Ok(IngestionAuth::Developer(api_key.model)),
                Err(e) => return Err(e),
            }
        }

        // 3. Neither present
        Err(AppError::Unauthorized("Bu işlem için giriş yapmalı veya geçerli bir X-API-Key sağlamalısınız.".to_string()))
    }
}

//! Public API anahtarı (X-API-Key) ve veri alım yetkilendirme extractor'ları.
//!
//! Geliştirici API anahtarının SHA-256 özetini veritabanından doğrular,
//! hesap seviyesini (tier) çözer ve günlük kullanım limitlerini denetler.

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

        let mut hasher = Sha256::new();
        hasher.update(api_key_header.as_bytes());
        let hash_result = hasher.finalize();
        let key_hash: String = hash_result.iter().map(|b| format!("{:02x}", b)).collect();

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

        if has_cookie || has_bearer {
            match AuthenticatedUser::from_request_parts(parts, state).await {
                Ok(user) => return Ok(IngestionAuth::User(user)),
                Err(e) => {
                    // Tarayıcıdan gelen alakasız çerezlerin API Key yetkilendirmesini engellemesini önle.
                    if !has_api_key {
                        return Err(e);
                    }
                }
            }
        }

        if has_api_key {
            match ValidApiKey::from_request_parts(parts, state).await {
                Ok(api_key) => return Ok(IngestionAuth::Developer(api_key.model)),
                Err(e) => return Err(e),
            }
        }

        Err(AppError::Unauthorized("Bu işlem için giriş yapmalı veya geçerli bir X-API-Key sağlamalısınız.".to_string()))
    }
}

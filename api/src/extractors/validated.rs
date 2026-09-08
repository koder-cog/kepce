//! Otomatik veri doğrulama (`validator`) destekli JSON extractor'ı.
//!
//! Axum'ın varsayılan `Json<T>` extractor'ına ek olarak DTO tiplerinde tanımlı
//! `#[validate]` kurallarını çalıştırır; geçersiz girdilerde istemciye standart `AppError` döner.

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Geçersiz JSON: {}", e)))?;
        
        if let Err(e) = value.validate() {
            let mut message = String::new();
            if let Some((_, errors)) = e.field_errors().iter().next() {
                if let Some(err) = errors.first() {
                    if let Some(msg) = &err.message {
                        message = msg.to_string();
                    }
                }
            }
            if message.is_empty() {
                message = "Girdi doğrulama hatası".to_string();
            }
            return Err(AppError::BadRequest(message));
        }
        Ok(ValidatedJson(value))
    }
}

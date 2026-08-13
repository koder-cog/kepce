use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::config::AppState;
use crate::dto::user::UserRole;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub role: UserRole,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
    pub rem: bool,
    /// Oturum ID (jti) - user_sessions tablosundaki ID ile eşleşmeli
    pub jti: Uuid,
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        
        // 1. Try Cookie first
        let mut token_opt = parts.headers.get(axum::http::header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str.split(';')
                    .map(|pair| pair.trim())
                    .find(|pair| pair.starts_with("kepce_token="))
                    .map(|pair| &pair["kepce_token=".len()..])
            });

        // 2. Fallback to Authorization Header
        if token_opt.is_none() {
            token_opt = parts.headers.get(axum::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .filter(|s| s.starts_with("Bearer "))
                .map(|s| &s[7..]);
        }

        let token = token_opt.ok_or_else(|| AppError::Unauthorized("Missing or invalid authorization header or cookie".to_string()))?;

        let mut validation = Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-web"]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(app_state.config.jwt_secret.as_bytes()),
            &validation,
        ).map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;



        Ok(AuthenticatedUser {
            id: token_data.claims.sub,
            username: token_data.claims.username,
            role: token_data.claims.role,
        })
    }
}

pub struct OptionalUser(pub Option<AuthenticatedUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await.ok();
        Ok(OptionalUser(user))
    }
}

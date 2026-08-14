#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use axum::{
        extract::{FromRequest, FromRequestParts},
        http::{header::{AUTHORIZATION, COOKIE}, Request},
    };
    use jsonwebtoken::{encode, Header, EncodingKey};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use uuid::Uuid;
    use validator::Validate;

    use crate::config::{AppState, Config};
    use crate::dto::user::UserRole;
    use crate::extractors::auth::{AuthenticatedUser, Claims};
    use crate::extractors::validated::ValidatedJson;
    use crate::extractors::api_key::ValidApiKey;
    use crate::middleware::rate_limiter::RateLimiter;
    use crate::services::usage_tracker::UsageTracker;

    #[derive(Debug, Serialize, Deserialize, Validate)]
    struct DummyDto {
        #[validate(email(message = "Geçersiz e-posta adresi"))]
        email: String,
        #[validate(length(min = 3, message = "Çok kısa"))]
        name: String,
    }

    async fn mock_app_state() -> AppState {
        let config = Config::from_env();
        let db = sea_orm::Database::connect(&config.database_url).await.unwrap();
        AppState {
            db: db.clone(),
            config: Arc::new(config),
            rate_limiter: Arc::new(RateLimiter::new()),
            usage_tracker: Arc::new(UsageTracker::new(db.clone())),
        }
    }

    #[tokio::test]
    async fn test_validated_json_valid() {
        let payload = serde_json::json!({
            "email": "test@kepce.org",
            "name": "Omer"
        });
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(axum::body::Body::from(payload.to_string()))
            .unwrap();

        let state = mock_app_state().await;
        let result = ValidatedJson::<DummyDto>::from_request(req, &state).await;
        assert!(result.is_ok());
        let ValidatedJson(dto) = result.unwrap();
        assert_eq!(dto.email, "test@kepce.org");
        assert_eq!(dto.name, "Omer");
    }

    #[tokio::test]
    async fn test_validated_json_invalid_email() {
        let payload = serde_json::json!({
            "email": "not_an_email",
            "name": "Omer"
        });
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(axum::body::Body::from(payload.to_string()))
            .unwrap();

        let state = mock_app_state().await;
        let result = ValidatedJson::<DummyDto>::from_request(req, &state).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Geçersiz e-posta adresi"));
    }

    #[tokio::test]
    async fn test_validated_json_invalid_name() {
        let payload = serde_json::json!({
            "email": "test@kepce.org",
            "name": "ab"
        });
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(axum::body::Body::from(payload.to_string()))
            .unwrap();

        let state = mock_app_state().await;
        let result = ValidatedJson::<DummyDto>::from_request(req, &state).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Çok kısa"));
    }

    #[tokio::test]
    async fn test_auth_extractor_jwt_header() {
        let state = mock_app_state().await;
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id,
            username: "test_user".to_string(),
            role: UserRole::User,
            exp: 10000000000, // far future
            iss: "kepce".to_string(),
            aud: "kepce-web".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        ).unwrap();

        let req = Request::builder()
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let auth_user = AuthenticatedUser::from_request_parts(&mut parts, &state).await.unwrap();

        assert_eq!(auth_user.id, user_id);
        assert_eq!(auth_user.username, "test_user");
        assert!(matches!(auth_user.role, UserRole::User));
    }

    #[tokio::test]
    async fn test_auth_extractor_jwt_cookie() {
        let state = mock_app_state().await;
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id,
            username: "cookie_monster".to_string(),
            role: UserRole::Admin,
            exp: 10000000000,
            iss: "kepce".to_string(),
            aud: "kepce-web".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        ).unwrap();

        let req = Request::builder()
            .header(COOKIE, format!("kepce_token={}; Path=/; HttpOnly", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let auth_user = AuthenticatedUser::from_request_parts(&mut parts, &state).await.unwrap();

        assert_eq!(auth_user.id, user_id);
        assert_eq!(auth_user.username, "cookie_monster");
        assert!(matches!(auth_user.role, UserRole::Admin));
    }

    #[tokio::test]
    async fn test_auth_extractor_jwt_missing() {
        let state = mock_app_state().await;
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedUser::from_request_parts(&mut parts, &state).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Missing or invalid authorization header or cookie"));
    }

    #[tokio::test]
    async fn test_api_key_extractor_missing() {
        let state = mock_app_state().await;
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let result = ValidApiKey::from_request_parts(&mut parts, &state).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("API Key eksik"));
    }
}

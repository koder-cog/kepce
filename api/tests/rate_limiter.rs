use api::config::{AppState, Config};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use std::sync::Arc;
use tower::util::ServiceExt;

async fn setup_app() -> axum::Router {
    let config = Config {
        database_url: "postgres://mock:mock@localhost/mock".to_string(),
        jwt_secret: "test_jwt_secret_key_12345678901234567890".to_string(),
        cors_origin: "*".to_string(),
        gemini_api_key: None,
        gemini_model: "gemini-flash-latest".to_string(),
        bot_directive: String::new(),
        initial_admin_email: None,
        initial_admin_password: None,
        cookie_secure: false,
        resend_api_key: "mock_key".to_string(),
        base_url: "http://localhost:5173".to_string(),
        google_client_id: None,
        google_client_secret: None,
        google_redirect_uri: None,
        searxng_url: None,
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        internal_ingest_secret: None,
    };
    let db = sea_orm::DatabaseConnection::Disconnected;
    let rate_limiter = Arc::new(api::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(api::services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db,
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    axum::Router::new()
        .route(
            "/api/v1/auth/login",
            axum::routing::post(|| async { StatusCode::OK }),
        )
        .layer(axum::middleware::from_fn_with_state(
            state,
            api::middleware::rate_limiter::rate_limit_middleware,
        ))
}

#[tokio::test]
async fn test_rate_limiter_triggers() {
    let app = setup_app().await;

    let login_payload = serde_json::json!({
        "identifier": "test@kepce.org",
        "password": "password",
        "remember": false
    });

    // Login kategorisi 60 saniyede 5 isteğe izin verir; ilk 5 istek 200 OK dönmeli
    for _ in 0..5 {
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/auth/login")
            .header(http::header::CONTENT_TYPE, "application/json")
            .header("x-client-id", "test-client-123")
            .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // 6. istek rate limiter tarafından 429 Too Many Requests ile engellenmeli
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-client-id", "test-client-123")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

use api::{
    build_cors, build_router,
    config::{AppState, Config},
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::Database;
use std::sync::Arc;
use tower::util::ServiceExt;

async fn setup_app() -> axum::Router {
    dotenvy::dotenv().ok();
    let config = Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://mock:mock@localhost/mock".to_string()),
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
    let db = match Database::connect(&config.database_url).await {
        Ok(conn) => conn,
        Err(_) => sea_orm::DatabaseConnection::Disconnected,
    };
    let cors = build_cors(&config.cors_origin).unwrap();
    let rate_limiter = Arc::new(api::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(api::services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db,
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    build_router(state, cors)
}

#[tokio::test]
async fn test_rate_limiter_triggers() {
    let app = setup_app().await;

    // Login Category allows 5 requests per 60 seconds (6th should fail)
    let login_payload = serde_json::json!({
        "identifier": "nonexistent@kepce.org",
        "password": "wrong_password",
        "remember": false
    });

    // Make 5 requests - all should pass rate limiter (not 429)
    for _ in 0..5 {
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/auth/login")
            .header(http::header::CONTENT_TYPE, "application/json")
            // Use static client id to identify the same "device"
            .header("x-client-id", "test-client-123")
            .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    // 6th request should be rate limited with 429 Too Many Requests
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

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::Database;
use std::sync::Arc;
use tower::util::ServiceExt;
use api::{
    config::{AppState, Config},
    build_cors, build_router,
};

async fn setup_app() -> axum::Router {
    let config = Config::from_env();
    let db = Database::connect(&config.database_url).await.unwrap();
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

    // Make 5 requests - all should fail with 401 Unauthorized (because of wrong password) but NOT 429
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
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

use std::sync::Arc;

use api::{
    build_cors, build_router,
    config::{AppState, Config},
    dto::user::UserRole,
    services::auth::AuthService,
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::Database;
use tower::util::ServiceExt;
use uuid::Uuid;

async fn setup_test_app() -> (axum::Router, Arc<Config>) {
    dotenvy::dotenv().ok();
    let config = Config::from_env();
    let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
    opt.connect_timeout(std::time::Duration::from_millis(200));
    let db = match Database::connect(opt).await {
        Ok(conn) => conn,
        Err(_) => sea_orm::DatabaseConnection::Disconnected,
    };
    let cors = build_cors(&config.cors_origin).unwrap();
    let rate_limiter = Arc::new(api::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(api::services::usage_tracker::UsageTracker::new(db.clone()));
    let config_arc = Arc::new(config);
    let state = AppState {
        db,
        config: config_arc.clone(),
        rate_limiter,
        usage_tracker,
    };
    (build_router(state, cors), config_arc)
}

#[tokio::test]
async fn test_on_behalf_of_anonymous_forbidden() {
    let (app, _) = setup_test_app().await;

    let boundary = "testboundary123";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"city_slug\"\r\n\r\nistanbul\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"year\"\r\n\r\n2026\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"month\"\r\n\r\n10\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"on_behalf_of\"\r\n\r\nfaik\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"files\"; filename=\"test.png\"\r\nContent-Type: image/png\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13]);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/ingestion/submit")
        .header(
            http::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_on_behalf_of_normal_user_forbidden() {
    let (app, config) = setup_test_app().await;

    let user_token = AuthService::generate_token(
        Uuid::new_v4(),
        "normal_user",
        &UserRole::User,
        &config.jwt_secret,
    )
    .unwrap();

    let boundary = "testboundary456";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"city_slug\"\r\n\r\nistanbul\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"year\"\r\n\r\n2026\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"month\"\r\n\r\n10\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"on_behalf_of\"\r\n\r\nfaik\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"files\"; filename=\"test.png\"\r\nContent-Type: image/png\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13]);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/ingestion/submit")
        .header(
            http::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(http::header::AUTHORIZATION, format!("Bearer {user_token}"))
        .body(Body::from(body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

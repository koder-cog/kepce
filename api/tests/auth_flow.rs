use api::{
    build_cors, build_router,
    config::{AppState, Config},
    services::auth::AuthService,
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::{ColumnTrait, Database, EntityTrait, QueryFilter};
use shared::entities::{prelude::Users, users};
use std::sync::Arc;
use tower::util::ServiceExt; // the correct oneshot trait

async fn setup_app() -> Option<(axum::Router, AppState)> {
    let mut config = Config::from_env();
    config.resend_api_key = "mock_key".to_string(); // Test ortamında gerçek e-posta gönderimini engelle
    let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
    opt.connect_timeout(std::time::Duration::from_secs(5));
    opt.acquire_timeout(std::time::Duration::from_secs(5));
    let db = Database::connect(opt).await.ok()?;
    api::services::migration::run_migrations(&db)
        .await
        .expect("Migration çalıştırma başarısız oldu!");
    let cors = build_cors(&config.cors_origin).unwrap();
    let rate_limiter = Arc::new(api::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(api::services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db: db.clone(),
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    let app = build_router(state.clone(), cors);
    Some((app, state))
}

#[tokio::test]
async fn test_full_auth_flow() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let Some((app, state)) = setup_app().await else {
        println!("Veritabanı bağlantısı kurulamadığı için test atlandı.");
        return;
    };

    // Clean up any test user from previous run
    let test_email = "flow_test@kepce.org";
    users::Entity::delete_many()
        .filter(users::Column::Email.eq(test_email))
        .exec(&state.db)
        .await
        .expect("Test öncesi kullanıcı temizliği başarısız oldu!");

    // 1. Register User
    let register_payload = serde_json::json!({
        "email": test_email,
        "password": "SuperSecretPassword123",
        "username": "flow_test"
    });

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .extension(axum::extract::connect_info::ConnectInfo(
            std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
        ))
        .body(Body::from(
            serde_json::to_string(&register_payload).unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    println!(
        "STATUS: {}, BODY: {:?}",
        status,
        String::from_utf8_lossy(&body_bytes)
    );
    assert_eq!(status, StatusCode::OK);

    // Verify user created but not verified in DB
    let user_db = Users::find()
        .filter(users::Column::Email.eq(test_email))
        .one(&state.db)
        .await
        .unwrap()
        .expect("User should exist in database");
    assert!(!user_db.is_verified);

    // 2. Generate verification token & verify
    let verification_token =
        AuthService::generate_verification_token(user_db.id, &state.config.jwt_secret).unwrap();
    let verify_req = Request::builder()
        .method(http::Method::GET)
        .uri(format!("/api/v1/auth/verify?token={}", verification_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check DB that user is now verified
    let user_db_verified = Users::find()
        .filter(users::Column::Email.eq(test_email))
        .one(&state.db)
        .await
        .unwrap()
        .expect("User should exist");
    assert!(user_db_verified.is_verified);

    // 3. Login
    let login_payload = serde_json::json!({
        "identifier": test_email,
        "password": "SuperSecretPassword123",
        "remember": false
    });

    let login_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .extension(axum::extract::connect_info::ConnectInfo(
            std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
        ))
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Extract Bearer token from response body to use for authenticated endpoints
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(login_res.get("user").is_some());

    let jwt_token = login_res
        .get("token")
        .and_then(|t| t.as_str())
        .expect("login response must include token for non-browser clients");

    // 4. Access protected profile route
    let me_req = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/auth/me")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Clean up test user
    let _ = users::Entity::delete_many()
        .filter(users::Column::Email.eq(test_email))
        .exec(&state.db)
        .await;
}

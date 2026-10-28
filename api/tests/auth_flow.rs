use api::{
    build_cors, build_router,
    config::{AppState, Config},
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::{ColumnTrait, Database, EntityTrait, QueryFilter};
use shared::entities::{prelude::Users, users};
use std::sync::Arc;
use tower::util::ServiceExt;

async fn setup_app() -> Option<(axum::Router, AppState)> {
    if std::env::var("DATABASE_URL").is_err() {
        return None;
    }
    let mut config = Config::from_env();
    config.resend_api_key = "mock_key".to_string();
    config.smtp_host = None;
    config.smtp_username = None;
    config.smtp_password = None;
    let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
    opt.connect_timeout(std::time::Duration::from_millis(200));
    opt.acquire_timeout(std::time::Duration::from_millis(200));
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
async fn test_auth_flow() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let Some((app, state)) = setup_app().await else {
        println!("Veritabanı bağlantısı kurulamadığı için test atlandı.");
        return;
    };

    let run_id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let test_nickname = format!("flow_{}", run_id);

    // 1. Kayıt (Register User - Sadece Rumuz ve Şifre)
    let register_payload = serde_json::json!({
        "nickname": test_nickname,
        "password": "SuperSecretPassword123"
    });

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .extension(axum::extract::connect_info::ConnectInfo(
            std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
        ))
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);

    let reg_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let recovery_key = reg_res
        .get("recovery_key")
        .and_then(|k| k.as_str())
        .expect("Register must return plaintext recovery key")
        .to_string();

    let jwt_token = reg_res
        .get("token")
        .and_then(|t| t.as_str())
        .expect("Register must return access token")
        .to_string();

    // Veritabanında kullanıcının oluştuğunu ve is_verified = false olduğunu doğrula
    let user_db = Users::find()
        .filter(users::Column::Nickname.eq(&test_nickname))
        .one(&state.db)
        .await
        .unwrap()
        .expect("User should exist in database");
    assert!(!user_db.is_verified);
    assert!(user_db.recovery_key_hash.is_some());

    // 2.a Yanlış Son Blok Teyidi Denemesi (400 Bad Request dönmeli)
    let wrong_payload = serde_json::json!({
        "last_block": "wrong"
    });
    let wrong_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/confirm-recovery")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::from(serde_json::to_string(&wrong_payload).unwrap()))
        .unwrap();
    let wrong_resp = app.clone().oneshot(wrong_req).await.unwrap();
    assert_eq!(wrong_resp.status(), StatusCode::BAD_REQUEST);

    // 2.b Doğru Kurtarma Anahtarı Son Blok Teyidi (Confirm Recovery)
    let last_block = recovery_key
        .split('-')
        .last()
        .unwrap_or(&recovery_key)
        .to_string();

    let confirm_payload = serde_json::json!({
        "last_block": last_block
    });

    let confirm_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/confirm-recovery")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::from(serde_json::to_string(&confirm_payload).unwrap()))
        .unwrap();

    let confirm_resp = app.clone().oneshot(confirm_req).await.unwrap();
    assert_eq!(confirm_resp.status(), StatusCode::OK);

    // DB'de is_verified = true olduğunu doğrula
    let user_db_verified = Users::find_by_id(user_db.id)
        .one(&state.db)
        .await
        .unwrap()
        .expect("User should exist");
    assert!(user_db_verified.is_verified);

    // 3. E-posta ile Giriş Denemesi (Kullanıcı alışkanlık uyarısı - 400 Bad Request)
    let email_login_payload = serde_json::json!({
        "identifier": "ornek@kepce.org",
        "password": "SuperSecretPassword123"
    });

    let email_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .extension(axum::extract::connect_info::ConnectInfo(
            std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
        ))
        .body(Body::from(serde_json::to_string(&email_login_payload).unwrap()))
        .unwrap();

    let email_resp = app.clone().oneshot(email_req).await.unwrap();
    assert_eq!(email_resp.status(), StatusCode::BAD_REQUEST);

    // 4. Rumuz ile Başarılı Giriş (Login)
    let login_payload = serde_json::json!({
        "identifier": test_nickname,
        "password": "SuperSecretPassword123"
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

    let login_resp = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);

    // 5. Kurtarma Kodu ile Şifre Sıfırlama (Password Reset With Recovery Key)
    let reset_payload = serde_json::json!({
        "nickname": test_nickname,
        "recovery_key": recovery_key,
        "new_password": "NewSecretPassword456"
    });

    let reset_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/password-reset")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&reset_payload).unwrap()))
        .unwrap();

    let reset_resp = app.clone().oneshot(reset_req).await.unwrap();
    assert_eq!(reset_resp.status(), StatusCode::OK);

    // 6. Profil Bilgisi Çekme (/me)
    let me_req = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/auth/me")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap();

    let me_resp = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_resp.status(), StatusCode::OK);

    // Test kullanıcısını temizle
    let _ = shared::entities::user_sessions::Entity::delete_many()
        .filter(shared::entities::user_sessions::Column::UserId.eq(user_db.id))
        .exec(&state.db)
        .await;
    let _ = users::Entity::delete_by_id(user_db.id)
        .exec(&state.db)
        .await;
}

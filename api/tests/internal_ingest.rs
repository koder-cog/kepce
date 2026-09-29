use std::sync::Arc;

use api::{
    build_cors, build_router,
    config::{AppState, Config},
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sea_orm::{ColumnTrait, Database, EntityTrait, ModelTrait, QueryFilter};
use tower::util::ServiceExt;

async fn setup_test_app(secret: Option<&str>) -> (axum::Router, sea_orm::DatabaseConnection) {
    dotenvy::dotenv().ok();
    let mut config = Config::from_env();
    config.internal_ingest_secret = secret.map(|s| s.to_string());
    let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
    opt.connect_timeout(std::time::Duration::from_millis(200));
    let db = match Database::connect(opt).await {
        Ok(conn) => conn,
        Err(_) => sea_orm::DatabaseConnection::Disconnected,
    };
    let cors = build_cors(&config.cors_origin).unwrap();
    let rate_limiter = Arc::new(api::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(api::services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db: db.clone(),
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    (build_router(state, cors), db)
}

#[tokio::test]
async fn test_internal_ingest_auth_guards() {
    let (app_without_secret, _) = setup_test_app(None).await;

    // 1. Sunucuda secret ayarlı değilse 403 Forbidden dönmeli
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-internal-token", "herhangi_bir_token")
        .body(Body::from(r#"{"menus": []}"#))
        .unwrap();

    let resp = app_without_secret.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let (app_with_secret, _) = setup_test_app(Some("gizli_test_anahtari_12345")).await;

    // 2. Token başlığı yoksa 401 Unauthorized dönmeli
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"menus": []}"#))
        .unwrap();

    let resp = app_with_secret.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 3. Yanlış token verilmişse 401 Unauthorized dönmeli
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-internal-token", "yanlis_anahtar")
        .body(Body::from(r#"{"menus": []}"#))
        .unwrap();

    let resp = app_with_secret.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_internal_ingest_success_and_idempotency() {
    let test_secret = "test_ingest_token_super_secret_987";
    let (app, db) = setup_test_app(Some(test_secret)).await;
    if matches!(db, sea_orm::DatabaseConnection::Disconnected) {
        return;
    }

    let test_date = "2026-10-15";
    let payload = serde_json::json!({
        "menus": [
            {
                "city_slug": "ankara",
                "serve_date": test_date,
                "meal_type": "dinner",
                "dishes": [
                    [{"name": "Mercimek Çorbası", "calories": "150"}],
                    [{"name": "Orman Kebabı", "calories": "380"}],
                    [{"name": "Pirinç Pilavı", "calories": "250"}],
                    [{"name": "Ayran", "calories": "75"}]
                ],
                "celiac_dishes": [],
                "takeaways": [],
                "calorie_range_min": 800,
                "calorie_range_max": 950
            }
        ],
        "source_type": "kykyemek"
    });

    // İlk istek: menü veritabanına eklenmeli (total_inserted: 1)
    let req1 = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-internal-token", test_secret)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    let body_bytes1 = axum::body::to_bytes(resp1.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let res1: serde_json::Value = serde_json::from_slice(&body_bytes1).unwrap();
    eprintln!("res1: {:#?}", res1);
    assert_eq!(res1["total_received"], 1);
    assert_eq!(res1["total_inserted"], 1);
    assert_eq!(res1["total_updated"], 0);
    assert_eq!(res1["total_skipped"], 0);

    // İkinci istek (birebir aynı içerik): Değişiklik olmamalı (idempotency, total_skipped: 1)
    let req2 = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-internal-token", test_secret)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let body_bytes2 = axum::body::to_bytes(resp2.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let res2: serde_json::Value = serde_json::from_slice(&body_bytes2).unwrap();
    assert_eq!(res2["total_received"], 1);
    assert_eq!(res2["total_inserted"], 0);
    assert_eq!(res2["total_updated"], 0);
    assert_eq!(res2["total_skipped"], 1);

    // Temizlik: Test menüsünü sil
    let parsed_date = chrono::NaiveDate::parse_from_str(test_date, "%Y-%m-%d").unwrap();
    if let Ok(Some(m)) = shared::entities::menus::Entity::find()
        .filter(shared::entities::menus::Column::ServeDate.eq(parsed_date))
        .filter(shared::entities::menus::Column::CityId.eq(6))
        .one(&db)
        .await
    {
        let _ = shared::entities::menu_dishes::Entity::delete_many()
            .filter(shared::entities::menu_dishes::Column::MenuId.eq(m.id))
            .exec(&db)
            .await;
        let _ = m.delete(&db).await;
    }
}

#[tokio::test]
async fn test_internal_ingest_junk_menu_rejected() {
    let test_secret = "test_ingest_junk_secret_333";
    let (app, db) = setup_test_app(Some(test_secret)).await;
    if matches!(db, sea_orm::DatabaseConnection::Disconnected) {
        return;
    }

    // Yalnızca 1 yemek içeren ve çöp metin olan menü
    let payload = serde_json::json!({
        "menus": [
            {
                "city_slug": "ankara",
                "serve_date": "2026-10-20",
                "meal_type": "dinner",
                "dishes": [
                    [{"name": "İletişim İçin Tıklayınız", "calories": null}]
                ],
                "celiac_dishes": [],
                "takeaways": []
            }
        ]
    });

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/internal/ingest/kykyemek")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-internal-token", test_secret)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(res["total_received"], 1);
    assert_eq!(res["total_inserted"], 0);
    assert_eq!(res["total_skipped"], 1); // Çöp filtreye takılıp atlandı
}

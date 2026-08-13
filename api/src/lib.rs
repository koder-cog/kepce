// Kepçe API — Kütüphane Giriş Noktası
// ====================================

pub mod config;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod utils;

use std::sync::Arc;
use axum::Router;
use sea_orm::Database;
use tower_http::cors::CorsLayer;
use http::Method;

use config::{AppState, Config};

pub fn build_cors(cors_origin: &str) -> anyhow::Result<CorsLayer> {
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION, http::header::ACCEPT]);

    let cors_origin_trimmed = cors_origin.trim();
    if cors_origin_trimmed == "*" {
        cors = cors.allow_origin(tower_http::cors::AllowOrigin::any()).allow_credentials(false);
    } else if cors_origin_trimmed.contains(',') {
        let origins: Result<Vec<http::HeaderValue>, _> = cors_origin_trimmed
            .split(',')
            .map(|s| s.trim().parse::<http::HeaderValue>())
            .collect();
        cors = cors.allow_origin(tower_http::cors::AllowOrigin::list(origins?)).allow_credentials(true);
    } else {
        cors = cors.allow_origin(cors_origin_trimmed.parse::<http::HeaderValue>()?).allow_credentials(true);
    }
    Ok(cors)
}

pub fn build_router(state: AppState, cors: CorsLayer) -> Router {
    use tower_http::trace::TraceLayer;
    Router::new()
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/menus", routes::menus::router())
        .nest("/api/v1/comments", routes::comments::router())
        .nest("/api/v1/profile", routes::profile::router())
        .nest("/api/v1/moderation", routes::moderation::router())
        .nest("/api/v1/statistics", routes::statistics::router())
        .nest("/api/v1/public", routes::public_api::router())
        .nest("/api/v1/system", routes::system::router())
        .nest("/api/v1/ingestion", routes::ingestion::router())
        .nest("/api/v1/admin", routes::admin::router())
        .nest("/api/v1/reports", routes::reports::router())
        .nest("/api/v1/public/contact", routes::contact::router())

        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::rate_limiter::rate_limit_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

pub async fn run() -> anyhow::Result<()> {
    // .env dosyasından ortam değişkenlerini yükle
    let config = Config::from_env();
    
    // Structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Kepçe API v2 başlatılıyor...");

    // Veritabanı bağlantısı
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Veritabanı bağlantısı başarılı.");

    // Veritabanı Şema Migrasyonu
    if let Err(e) = services::migration::run_migrations(&db).await {
        tracing::error!("Veritabanı migrasyonu başarısız oldu: {:?}", e);
        return Err(e);
    }

    // Fiyatlandırma Bilgilerini Veritabanından Belleğe Yükle
    if let Err(e) = services::pricing::load_pricing_from_db(&db).await {
        tracing::warn!("Fiyatlandırma verileri yüklenirken uyarı: {:?}", e);
    }

    // Admin Bootstrap
    if let Err(e) = services::admin::bootstrap_admin(&db, &config).await {
        tracing::error!("Admin bootstrap işlemi sırasında hata oluştu: {:?}", e);
    }

    // Repair NULL dish IDs in dish_aliases
    if let Err(e) = services::admin::repair_null_dish_ids(&db).await {
        tracing::error!("Yemek eşleşme onarımı sırasında hata oluştu: {:?}", e);
    }

    // CORS yapılandırması
    let cors = build_cors(&config.cors_origin)?;

    // Paylaşılan uygulama durumu
    let rate_limiter = Arc::new(middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db,
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    // static/avatars dizinini oluştur
    tokio::fs::create_dir_all("static/avatars").await?;

    let app = build_router(state, cors);

    // Sunucuyu başlat
    let addr = "0.0.0.0:8000";
    tracing::info!("Sunucu dinleniyor: {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;

    Ok(())
}

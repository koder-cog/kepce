//! Kepçe REST API kütüphane çekirdeği.
//!
//! Axum tabanlı HTTP yönlendirmeleri, middleware katmanları, veritabanı bağlantı
//! yönetimi ve servis bootstrap adımlarını içerir.

pub mod core;
pub mod modules;

// Geriye dönük uyumluluk ve doğrudan modül erişim köprüleri:
pub use crate::core::config;
pub use crate::core::error;
pub use crate::core::extractors;
pub use crate::core::middleware;
pub use crate::core::utils;

pub mod services {
    pub use crate::modules::community::comment_service as comment;
    pub use crate::modules::community::notification_service as notification;
    pub use crate::modules::community::push_service as push;
    pub use crate::modules::community::reaction_service as reaction;
    pub use crate::modules::community::vote_service as vote;
    pub use crate::modules::identity::auth_service as auth;
    pub use crate::modules::identity::email_service as email;
    pub use crate::modules::identity::user_service as user;
    pub use crate::modules::ingestion::bot_service as bot;
    pub use crate::modules::ingestion::ingest_service as ingestion;
    pub use crate::modules::ingestion::internal_service as internal_ingest;
    pub use crate::modules::menus::city_service as city;
    pub use crate::modules::menus::menu_service as menu;
    pub use crate::modules::menus::og_image_service as og_image;
    pub use crate::modules::menus::pricing_service as pricing;
    pub use crate::modules::moderation::admin_service as admin;
    pub use crate::modules::moderation::migration_service as migration;
    pub use crate::modules::moderation::moderation_service as moderation;
    pub use crate::modules::platform::developer_service as developer;
    pub use crate::modules::platform::statistics_service as statistics;
    pub use crate::modules::platform::system_service as system;
    pub use crate::modules::platform::usage_tracker;
}

pub mod routes {
    pub use crate::modules::community::comment_routes as comments;
    pub use crate::modules::community::push_routes as push;
    pub use crate::modules::identity::auth_routes as auth;
    pub use crate::modules::identity::profile_routes as profile;
    pub use crate::modules::ingestion::internal_routes as internal_ingest;
    pub use crate::modules::ingestion::public_routes as ingestion;
    pub use crate::modules::menus::og_routes as og;
    pub use crate::modules::menus::routes as menus;
    pub use crate::modules::moderation::admin_routes as admin;
    pub use crate::modules::moderation::db_admin_routes as database_admin;
    pub use crate::modules::moderation::moderation_routes as moderation;
    pub use crate::modules::moderation::report_routes as reports;
    pub use crate::modules::platform::contact_routes as contact;
    pub use crate::modules::platform::public_api_routes as public_api;
    pub use crate::modules::platform::statistics_routes as statistics;
    pub use crate::modules::platform::system_routes as system;
}

pub mod dto {
    pub use crate::modules::community::dto::comment;
    pub use crate::modules::community::dto::reaction;
    pub use crate::modules::identity::dto as user;
    pub use crate::modules::identity::dto as auth;
    pub use crate::modules::ingestion::dto as internal_ingest;
    pub use crate::modules::menus::dto as menu;
    pub use crate::modules::moderation::dto::admin;
    pub use crate::modules::moderation::dto::moderation;
    pub use crate::modules::platform::dto::developer;
    pub use crate::modules::platform::dto::pagination;
    pub use crate::modules::platform::dto::statistics;
    pub use crate::modules::platform::dto::system;
}

use axum::Router;
use http::Method;
use sea_orm::Database;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use core::config::{AppState, Config};

pub fn build_cors(cors_origin: &str) -> anyhow::Result<CorsLayer> {
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
        ]);

    let cors_origin_trimmed = cors_origin.trim();
    if cors_origin_trimmed == "*" {
        cors = cors
            .allow_origin(tower_http::cors::AllowOrigin::any())
            .allow_credentials(false);
    } else if cors_origin_trimmed.contains(',') {
        let origins: Result<Vec<http::HeaderValue>, _> = cors_origin_trimmed
            .split(',')
            .map(|s| s.trim().parse::<http::HeaderValue>())
            .collect();
        cors = cors
            .allow_origin(tower_http::cors::AllowOrigin::list(origins?))
            .allow_credentials(true);
    } else {
        cors = cors
            .allow_origin(cors_origin_trimmed.parse::<http::HeaderValue>()?)
            .allow_credentials(true);
    }
    Ok(cors)
}

pub fn build_router(state: AppState, cors: CorsLayer) -> Router {
    use tower_http::trace::TraceLayer;
    Router::new()
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/menus", routes::menus::router())
        .route(
            "/api/v1/cities",
            axum::routing::get(|| async {
                axum::response::Redirect::permanent("/api/v1/public/cities")
            }),
        )
        .route(
            "/api/v1/cities/detect",
            axum::routing::get(|| async {
                axum::response::Redirect::permanent("/api/v1/public/cities/detect")
            }),
        )
        .nest("/api/v1/comments", routes::comments::router())
        .nest("/api/v1/profile", routes::profile::router())
        .nest("/api/v1/moderation", routes::moderation::router())
        .nest("/api/v1/statistics", routes::statistics::router())
        .nest("/api/v1/public", routes::public_api::router())
        .nest("/api/v1/system", routes::system::router())
        .nest("/api/v1/ingestion", routes::ingestion::router())
        .nest("/api/v1/internal", routes::internal_ingest::router())
        .nest("/api/v1/admin", routes::admin::router())
        .nest("/api/v1/reports", routes::reports::router())
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            core::middleware::rate_limiter::rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

pub async fn run() -> anyhow::Result<()> {
    let config = Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Kepçe API v2 başlatılıyor...");

    let pool_max: u32 = std::env::var("DATABASE_POOL_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let mut db_opts = sea_orm::ConnectOptions::new(&config.database_url);
    db_opts
        .max_connections(pool_max)
        .min_connections(2)
        .connect_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .sqlx_logging(false);
    let db = Database::connect(db_opts).await?;
    tracing::info!("Veritabanı bağlantısı başarılı.");

    if let Err(e) = services::migration::run_migrations(&db).await {
        tracing::error!("Veritabanı migrasyonu başarısız oldu: {:?}", e);
        return Err(e);
    }

    if let Err(e) = services::pricing::load_pricing_from_db(&db).await {
        tracing::warn!("Fiyatlandırma verileri yüklenirken uyarı: {:?}", e);
    }

    if let Err(e) = services::admin::bootstrap_admin(&db, &config).await {
        tracing::error!("Admin bootstrap işlemi sırasında hata oluştu: {:?}", e);
    }

    if let Err(e) = services::admin::repair_null_dish_ids(&db).await {
        tracing::error!("Yemek eşleşme onarımı sırasında hata oluştu: {:?}", e);
    }

    let cors = build_cors(&config.cors_origin)?;

    let rate_limiter = Arc::new(core::middleware::rate_limiter::RateLimiter::new());
    let usage_tracker = Arc::new(services::usage_tracker::UsageTracker::new(db.clone()));
    let state = AppState {
        db,
        config: Arc::new(config),
        rate_limiter,
        usage_tracker,
    };
    tokio::fs::create_dir_all("static/avatars").await?;

    let app = build_router(state, cors);

    let addr = "0.0.0.0:8000";
    tracing::info!("Sunucu dinleniyor: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

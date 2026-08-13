use sea_orm::DatabaseConnection;
use std::env;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origin: String,
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
    pub bot_directive: String,
    pub initial_admin_email: Option<String>,
    pub initial_admin_password: Option<String>,
    pub cookie_secure: bool,
    pub resend_api_key: String,
    pub base_url: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("KEPCE_SECRET_KEY").expect("KEPCE_SECRET_KEY must be set"),
            cors_origin: env::var("KEPCE_CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            gemini_api_key: env::var("GEMINI_API_KEY").ok(),
            gemini_model: env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-flash-latest".to_string()),
            bot_directive: load_bot_directive(),
            initial_admin_email: env::var("INITIAL_ADMIN_EMAIL").ok(),
            initial_admin_password: env::var("INITIAL_ADMIN_PASSWORD").ok(),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            resend_api_key: env::var("RESEND_API_KEY").unwrap_or_else(|_| "mock_key".to_string()),
            base_url: env::var("KEPCE_BASE_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI").ok(),
        }

    }
}

/// Bot sistem direktifi: derleme zamanında `prompts/kepce_bot.md` dosyasından gömülür.
/// Çalışma zamanında `BOT_DIRECTIVE_PATH` env değişkeniyle override edilebilir;
/// dosya yoksa veya okunamazsa gömülü varsayılana düşer (sessiz fallback).
fn load_bot_directive() -> String {
    const DEFAULT_DIRECTIVE: &str = include_str!("../prompts/kepce_bot.md");
    match env::var("BOT_DIRECTIVE_PATH") {
        Ok(path) => std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "BOT_DIRECTIVE_PATH='{}' okunamadı ({}), gömülü varsayılan direktif kullanılıyor.",
                    path, e
                );
                DEFAULT_DIRECTIVE.to_string()
            }),
        Err(_) => DEFAULT_DIRECTIVE.to_string(),
    }
}

use axum::extract::FromRef;

/// Tüm route'lara paylaştırılacak (inject edilecek) ortak durum (state).
#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<crate::middleware::rate_limiter::RateLimiter>,
    pub usage_tracker: Arc<crate::services::usage_tracker::UsageTracker>,
}

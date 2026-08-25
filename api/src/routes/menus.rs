// Kepçe API - Routes: Menü Endpoint'leri
// ========================================
//
// İnce zarf. İş mantığı yok - MenuService'e delege eder.
//
use axum::{
    routing::get,
    Router,
    extract::{State, Path, Query},
    Json,
};
use serde::Deserialize;
use chrono::{NaiveDate, Utc};
use crate::services::menu::{MenuService, MenuError};
use crate::services::vote::{VoteService, VoteError};
use crate::error::AppError;
use crate::extractors::auth::{OptionalUser, AuthenticatedUser};
use shared::entities::sea_orm_active_enums::SentimentEnum;
pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", get(get_menus))
        .route("/today", get(get_today))
        .route("/today/:city_slug", get(get_today_city))
        .route("/archive/years", get(get_archive_years))
        .route("/:menu_id", get(get_menu))
        .route("/:menu_id/vote", axum::routing::post(vote_menu))
}

impl From<MenuError> for AppError {
    fn from(err: MenuError) -> Self {
        match err {
            MenuError::NotFound => AppError::NotFound("Menu not found".to_string()),
            MenuError::DatabaseError(e) => {
                tracing::error!("Database error in MenuService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

impl From<VoteError> for AppError {
    fn from(err: VoteError) -> Self {
        match err {
            VoteError::MenuNotFound => AppError::NotFound("Menu not found".to_string()),
            VoteError::UnverifiedUser => AppError::Forbidden("Oy vermek için e-postanızı onaylamalısınız.".to_string()),
            VoteError::DatabaseError(e) => {
                tracing::error!("Database error in VoteService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

async fn get_today(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Query(filter): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let today = match filter.date.as_deref() {
        Some("today") | None => Utc::now().date_naive(),
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Geçersiz tarih formatı. YYYY-MM-DD veya 'today' kullanılmalıdır.".to_string()))?,
    };
    let user_id = user.map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(
        &db,
        filter.city,
        Some(today),
        filter.dietary_type,
        None,
        None,
        user_id,
    ).await?;
    crate::utils::response::cached_json_response(&headers, &menus, 300)
}

#[derive(Deserialize)]
pub struct MenuFilterQueryDto {
    pub city: Option<String>,
    pub date: Option<String>,
    pub dietary_type: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

async fn get_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let user_id = user.map(|u| u.id);
    let parsed_date = match query.date.as_deref() {
        Some("today") => Some(Utc::now().date_naive()),
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => return Err(AppError::BadRequest("Geçersiz tarih formatı. YYYY-MM-DD veya 'today' kullanılmalıdır.".to_string())),
        },
        None => None,
    };

    let menus = MenuService::get_menus_by_filter(
        &db,
        query.city,
        parsed_date,
        query.dietary_type,
        query.year,
        query.month,
        user_id,
    ).await?;
    crate::utils::response::cached_json_response(&headers, &menus, 300)
}

async fn get_today_city(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Path(city_slug): Path<String>,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let today = Utc::now().date_naive();
    let user_id = user.map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(&db, Some(city_slug), Some(today), query.dietary_type, None, None, user_id).await?;
    crate::utils::response::cached_json_response(&headers, &menus, 300)
}

#[derive(Deserialize)]
pub struct ArchiveYearsQuery {
    pub city: Option<String>,
}

async fn get_archive_years(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: http::HeaderMap,
    Query(query): Query<ArchiveYearsQuery>,
) -> Result<axum::response::Response, AppError> {
    let years = MenuService::get_archive_years(&db, query.city).await?;
    crate::utils::response::cached_json_response(&headers, &years, 3600)
}

#[derive(Deserialize)]
pub struct MenuDetailQueryDto {
    pub dietary_type: Option<String>,
}

async fn get_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Path(menu_id): Path<i32>,
    Query(query): Query<MenuDetailQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let user_id = user.map(|u| u.id);
    let menu = MenuService::get_menu_with_items(&db, menu_id, query.dietary_type, user_id).await?;
    crate::utils::response::cached_json_response(&headers, &menu, 300)
}

#[derive(Deserialize)]
pub struct VoteMenuDto {
    pub sentiment: String,
}

async fn vote_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    Json(payload): Json<VoteMenuDto>,
) -> Result<Json<()>, AppError> {
    let sentiment = match payload.sentiment.to_lowercase().as_str() {
        "positive" => SentimentEnum::Positive,
        "negative" => SentimentEnum::Negative,
        "neutral" | "" => SentimentEnum::Neutral,
        _ => return Err(AppError::BadRequest("Geçersiz oy türü. Yalnızca 'positive', 'negative' veya 'neutral' kabul edilir.".to_string())),
    };

    VoteService::vote_menu(&db, menu_id, user.id, sentiment).await?;
    Ok(Json(()))
}

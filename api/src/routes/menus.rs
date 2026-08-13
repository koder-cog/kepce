// Kepçe API — Routes: Menü Endpoint'leri
// ========================================
//
// İnce zarf. İş mantığı yok — MenuService'e delege eder.
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
use crate::dto::menu::MenuResponseDto;
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
    Query(filter): Query<MenuFilterQueryDto>,
) -> Result<Json<Vec<MenuResponseDto>>, AppError> {
    let today = filter.date.unwrap_or_else(|| Utc::now().date_naive());
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
    Ok(Json(menus))
}

#[derive(Deserialize)]
pub struct MenuFilterQueryDto {
    pub city: Option<String>,
    pub date: Option<NaiveDate>,
    pub dietary_type: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

async fn get_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<Json<Vec<MenuResponseDto>>, AppError> {
    let user_id = user.map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(
        &db,
        query.city,
        query.date,
        query.dietary_type,
        query.year,
        query.month,
        user_id,
    ).await?;
    Ok(Json(menus))
}

async fn get_today_city(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    Path(city_slug): Path<String>,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<Json<Vec<MenuResponseDto>>, AppError> {
    let today = Utc::now().date_naive();
    let user_id = user.map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(&db, Some(city_slug), Some(today), query.dietary_type, None, None, user_id).await?;
    Ok(Json(menus))
}

#[derive(Deserialize)]
pub struct ArchiveYearsQuery {
    pub city: Option<String>,
}

async fn get_archive_years(
    State(db): State<sea_orm::DatabaseConnection>,
    Query(query): Query<ArchiveYearsQuery>,
) -> Result<Json<Vec<i32>>, AppError> {
    let years = MenuService::get_archive_years(&db, query.city).await?;
    Ok(Json(years))
}

#[derive(Deserialize)]
pub struct MenuDetailQueryDto {
    pub dietary_type: Option<String>,
}

async fn get_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    Path(menu_id): Path<i32>,
    Query(query): Query<MenuDetailQueryDto>,
) -> Result<Json<MenuResponseDto>, AppError> {
    let user_id = user.map(|u| u.id);
    let menu = MenuService::get_menu_with_items(&db, menu_id, query.dietary_type, user_id).await?;
    Ok(Json(menu))
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
        "neutral" => SentimentEnum::Neutral,
        _ => return Err(AppError::BadRequest("Invalid sentiment".to_string())),
    };

    VoteService::vote_menu(&db, menu_id, user.id, sentiment).await?;
    Ok(Json(()))
}

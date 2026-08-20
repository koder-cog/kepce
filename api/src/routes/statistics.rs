// Kepçe API - Routes: İstatistik Endpoint'leri
// ==============================================
//
// İnce zarf. StatisticsService kullanır.
//
use axum::{
    routing::get,
    Router,
    extract::{State, Query},
    Json,
};
use crate::services::statistics::{StatisticsService, StatsError};
use crate::services::comment::CommentService;
use crate::dto::statistics::{TopDishDto, ModerationStatsDto, TrendingTagDto, HumanityStatsDto};
use crate::dto::comment::CommentResponseDto;

use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/top-dishes", get(get_top_dishes))
        .route("/worst-dishes", get(get_worst_dishes))
        .route("/moderation", get(get_moderation_stats))
        .route("/trending-tags", get(get_trending_tags))
        .route("/humanity", get(get_humanity_stats))
        .route("/dish/:dish_id/tags", get(get_dish_tags))
        .route("/comments/top", get(get_global_top_comments))
        .route("/comments/recent", get(get_global_recent_comments))
}

impl From<StatsError> for AppError {
    fn from(err: StatsError) -> Self {
        match err {
            StatsError::DatabaseError(e) => {
                tracing::error!("Database error in StatisticsService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}


#[derive(serde::Deserialize)]
pub struct LimitQuery {
    pub limit: Option<u64>,
    pub city_slug: Option<String>,
    pub timeframe: Option<String>,
}

async fn get_top_dishes(
    State(db): State<sea_orm::DatabaseConnection>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<TopDishDto>>, AppError> {
    let limit = query.limit.unwrap_or(10).min(100);
    let dishes = StatisticsService::get_dish_leaderboard(&db, limit, true, query.city_slug, query.timeframe).await?;
    Ok(Json(dishes))
}

async fn get_worst_dishes(
    State(db): State<sea_orm::DatabaseConnection>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<TopDishDto>>, AppError> {
    let limit = query.limit.unwrap_or(10).min(100);
    let dishes = StatisticsService::get_dish_leaderboard(&db, limit, false, query.city_slug, query.timeframe).await?;
    Ok(Json(dishes))
}

async fn get_moderation_stats(
    State(db): State<sea_orm::DatabaseConnection>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<ModerationStatsDto>, AppError> {
    let stats = StatisticsService::get_moderation_stats(&db, query.timeframe).await?;
    Ok(Json(stats))
}

async fn get_trending_tags(
    State(db): State<sea_orm::DatabaseConnection>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<TrendingTagDto>>, AppError> {
    let limit = query.limit.unwrap_or(10).min(100);
    let tags = StatisticsService::get_trending_tags(&db, limit).await?;
    Ok(Json(tags))
}

async fn get_humanity_stats(
    State(db): State<sea_orm::DatabaseConnection>,
) -> Result<Json<HumanityStatsDto>, AppError> {
    let stats = StatisticsService::get_humanity_stats(&db).await?;
    Ok(Json(stats))
}

async fn get_dish_tags(
    State(db): State<sea_orm::DatabaseConnection>,
    axum::extract::Path(dish_id): axum::extract::Path<i32>,
) -> Result<Json<Vec<TrendingTagDto>>, AppError> {
    let tags = StatisticsService::get_dish_tags(&db, dish_id).await?;
    Ok(Json(tags))
}

async fn get_global_top_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: Option<AuthenticatedUser>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<CommentResponseDto>>, AppError> {
    let limit = query.limit.unwrap_or(10).min(100);
    let current_user_id = user.map(|u| u.id);
    let comments = CommentService::get_top_comments(&db, current_user_id, limit, query.timeframe).await?;
    Ok(Json(comments))
}

async fn get_global_recent_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: Option<AuthenticatedUser>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<CommentResponseDto>>, AppError> {
    let limit = query.limit.unwrap_or(15).min(100);
    let current_user_id = user.map(|u| u.id);
    let comments = CommentService::get_recent_comments(&db, current_user_id, limit).await?;
    Ok(Json(comments))
}

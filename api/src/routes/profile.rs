// Kepçe API — Routes: Profil Endpoint'leri
// ==========================================
//
// İnce zarf. UserService + CommentService kullanır.
//
use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path, Query},
    Json,
};
use uuid::Uuid;
use crate::services::user::UserService;
use crate::services::comment::CommentService;
use crate::services::moderation::{ModerationService, ModerationError};
use crate::dto::user::UserProfileDto;
use crate::dto::comment::CommentResponseDto;
use crate::error::AppError;

use crate::extractors::auth::{AuthenticatedUser, OptionalUser};

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/me", get(get_my_profile))
        .route("/:username", get(get_profile))
        .route("/:username/comments", get(get_profile_comments))
        .route("/:username/stats/dashboard", get(get_profile_dashboard_stats))
        .route("/block/:blocked_id", post(block_user).delete(unblock_user))
}

impl From<ModerationError> for AppError {
    fn from(err: ModerationError) -> Self {
        match err {
            ModerationError::UserNotFound => AppError::NotFound("User not found".to_string()),
            ModerationError::CommentNotFound => AppError::NotFound("Comment not found".to_string()),
            ModerationError::SelfBlockNotAllowed => AppError::BadRequest("Cannot block yourself".to_string()),
            ModerationError::SelfReportNotAllowed => AppError::BadRequest("Cannot report yourself".to_string()),
            ModerationError::AlreadyReported => AppError::BadRequest("Already reported".to_string()),
            ModerationError::AlreadyBlocked => AppError::BadRequest("Already blocked".to_string()),
            ModerationError::CommentAlreadyDeleted => AppError::BadRequest("Comment already deleted".to_string()),
            ModerationError::DatabaseError(e) => {
                tracing::error!("Database error in ModerationService: {}", e);
                AppError::Internal("Database error".to_string())
            }
            ModerationError::CityNotFound => AppError::NotFound("Şehir bulunamadı".to_string()),
            ModerationError::NoMenusForMonth => {
                AppError::NotFound("Bu ay için onaylı menü bulunamadı".to_string())
            }
            ModerationError::InvalidMonth(m) => {
                AppError::BadRequest(format!("Geçersiz ay formatı: {}", m))
            }
            ModerationError::DateParseError(d) => {
                AppError::BadRequest(format!("Tarih çözümlenemedi: {}", d))
            }
        }
    }
}

async fn get_my_profile(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<UserProfileDto>, AppError> {
    let profile = UserService::get_user_profile_by_id(&db, user.id).await?;
    Ok(Json(profile))
}

async fn get_profile(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(username): Path<String>,
) -> Result<Json<UserProfileDto>, AppError> {
    let profile = UserService::get_user_profile_by_username(&db, &username).await?;
    Ok(Json(profile))
}

use crate::dto::pagination::PaginationQuery;

async fn get_profile_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: OptionalUser,
    Path(username): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<crate::dto::pagination::PaginatedResponse<CommentResponseDto>>, AppError> {
    let limit = query.limit_num();
    let offset = query.offset();
    
    // First get the user id
    let profile = UserService::get_user_profile_by_username(&db, &username).await?;
    
    let current_user_id = user.0.map(|u| u.id);
    
    let comments = CommentService::get_user_comments(&db, profile.id, current_user_id, limit, offset).await?;
    Ok(Json(comments))
}

use crate::dto::moderation::BlockUserDto;

async fn block_user(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(blocked_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    ModerationService::block_user(&db, user.id, BlockUserDto { blocked_user_id: blocked_id }).await?;
    Ok(Json(()))
}

async fn unblock_user(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(blocked_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    ModerationService::unblock_user(&db, user.id, blocked_id).await?;
    Ok(Json(()))
}

async fn get_profile_dashboard_stats(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(username): Path<String>,
) -> Result<Json<crate::dto::user::UserDashboardStatsDto>, AppError> {
    // Profilin varlığını kontrol et ve user_id al
    let profile = UserService::get_user_profile_by_username(&db, &username).await?;
    
    let stats = UserService::get_dashboard_stats(&db, profile.id).await?;
    Ok(Json(stats))
}

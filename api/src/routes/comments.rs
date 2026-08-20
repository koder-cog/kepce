// Kepçe API - Routes: Yorum ve Reaksiyon Endpoint'leri
// =====================================================
//
// İnce zarf. CommentService (okuma) ve ReactionService (yazma) kullanır.
//
use axum::{
    routing::{get, post, put},
    Router,
    extract::{State, Path, Query},
    Json,
};
use uuid::Uuid;
use crate::services::comment::{CommentService, CommentError};
use crate::services::reaction::{ReactionService, ReactionError};
use crate::dto::comment::{CreateCommentDto, UpdateCommentDto, CommentResponseDto};
use crate::dto::reaction::ReactionRequestDto;
use crate::error::AppError;
use crate::extractors::validated::ValidatedJson;

use crate::extractors::auth::{AuthenticatedUser, OptionalUser};

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/recent", get(get_recent_comments))
        .route("/menu/:menu_id", get(get_comments))
        .route("/", post(create_comment))
        .route("/react", post(toggle_reaction))
        .route("/:hash", put(update_comment).delete(delete_comment))
}

impl From<CommentError> for AppError {
    fn from(err: CommentError) -> Self {
        match err {
            CommentError::MenuNotFound => AppError::NotFound("Menü bulunamadı ya da zaten hiç var olmamıştı.".to_string()),
            CommentError::UserNotFound => AppError::NotFound("Kullanıcı bulunamadı.".to_string()),
            CommentError::UnverifiedUser => AppError::Forbidden("Yorum yapmak için e-postanızı onaylamalısınız.".to_string()),
            CommentError::DishNotFound => AppError::NotFound("Yemek bulunamadı".to_string()),
            CommentError::DishNotInMenu => AppError::BadRequest("Menüde bu yemek mevcut değil".to_string()),
            CommentError::ParentCommentNotFound => AppError::NotFound("Baş yorum bulunamadı".to_string()),
            CommentError::InvalidOperation => AppError::BadRequest("Geçersiz işlem".to_string()),
            CommentError::SpamDetected => AppError::BadRequest("Bu içerik spam olarak işaretlendi.".to_string()),
            CommentError::DatabaseError(e) => {
                tracing::error!("CommentService'te veritabanı hatası: {}", e);
                AppError::Internal("Veritabanı hatası".to_string())
            }
        }
    }
}

impl From<ReactionError> for AppError {
    fn from(err: ReactionError) -> Self {
        match err {
            ReactionError::CommentNotFound => AppError::NotFound("Comment not found".to_string()),
            ReactionError::Unauthorized => AppError::Unauthorized("Unauthorized to modify this comment".to_string()),
            ReactionError::UnverifiedUser => AppError::Forbidden("Oy kullanmak için e-postanızı onaylamalısınız.".to_string()),
            ReactionError::InvalidOperation => AppError::BadRequest("Bu içerik üzerinde işlem yapılamaz.".to_string()),
            ReactionError::DatabaseError(e) => {
                tracing::error!("Database error in ReactionService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CommentTreeQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

async fn get_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: OptionalUser,
    Path(menu_id): Path<i32>,
    Query(query): Query<CommentTreeQuery>,
) -> Result<Json<Vec<CommentResponseDto>>, AppError> {
    let current_user_id = user.0.map(|u| u.id);
    let comments = CommentService::get_menu_comment_tree(
        &db,
        menu_id,
        current_user_id,
        query.limit,
        query.offset,
    ).await?;
    Ok(Json(comments))
}

async fn create_comment(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreateCommentDto>,
) -> Result<Json<CommentResponseDto>, AppError> {
    let parent_id = payload.parent_id;
    let comment = CommentService::create_comment(&db, user.id, user.username, payload, parent_id).await?;
    Ok(Json(comment))
}

async fn toggle_reaction(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<ReactionRequestDto>,
) -> Result<Json<()>, AppError> {
    ReactionService::toggle_reaction(&db, user.id, payload.vote_id, payload.reaction).await?;
    Ok(Json(()))
}

async fn update_comment(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(hash): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCommentDto>,
) -> Result<Json<CommentResponseDto>, AppError> {
    let comment = CommentService::update_comment(&db, user.id, hash, payload.comment).await?;
    Ok(Json(comment))
}

async fn delete_comment(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(hash): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    ReactionService::delete_comment(&db, user.id, &user.role, hash).await?;
    Ok(Json(()))
}

#[derive(serde::Deserialize)]
struct RecentQuery {
    limit: Option<u64>,
}

async fn get_recent_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: OptionalUser,
    Query(query): Query<RecentQuery>,
) -> Result<Json<Vec<CommentResponseDto>>, AppError> {
    let current_user_id = user.0.map(|u| u.id);
    let limit = query.limit.unwrap_or(10).min(50);
    let comments = CommentService::get_recent_comments(&db, current_user_id, limit).await?;
    Ok(Json(comments))
}

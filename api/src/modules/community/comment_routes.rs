//! Menü yorumları, duygu oyları ve kullanıcı reaksiyon endpoint'leri.
use crate::dto::comment::{CommentResponseDto, CreateCommentDto, UpdateCommentDto};
use crate::dto::reaction::ReactionRequestDto;
use crate::error::AppError;
use crate::extractors::validated::ValidatedJson;
use crate::services::comment::{CommentError, CommentService};
use crate::services::reaction::{ReactionError, ReactionService};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::extractors::auth::{AuthenticatedUser, OptionalUser};
use shared::domain::quality::content_guard::ContentGuard;

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
            CommentError::MenuNotFound => {
                AppError::NotFound("Menü bulunamadı ya da zaten hiç var olmamıştı.".to_string())
            }
            CommentError::UserNotFound => AppError::NotFound("Kullanıcı bulunamadı.".to_string()),
            CommentError::UnverifiedUser => {
                AppError::Forbidden("Yorum yapmak için e-postanızı onaylamalısınız.".to_string())
            }
            CommentError::DishNotFound => AppError::NotFound("Yemek bulunamadı".to_string()),
            CommentError::DishNotInMenu => {
                AppError::BadRequest("Menüde bu yemek mevcut değil".to_string())
            }
            CommentError::ParentCommentNotFound => {
                AppError::NotFound("Baş yorum bulunamadı".to_string())
            }
            CommentError::InvalidOperation => AppError::BadRequest("Geçersiz işlem".to_string()),
            CommentError::SpamDetected => {
                AppError::BadRequest("Bu içerik spam olarak işaretlendi.".to_string())
            }
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
            ReactionError::Unauthorized => {
                AppError::Unauthorized("Unauthorized to modify this comment".to_string())
            }
            ReactionError::UnverifiedUser => {
                AppError::Forbidden("Oy kullanmak için e-postanızı onaylamalısınız.".to_string())
            }
            ReactionError::InvalidOperation => {
                AppError::BadRequest("Bu içerik üzerinde işlem yapılamaz.".to_string())
            }
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
    )
    .await?;
    Ok(Json(comments))
}

async fn create_comment(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    user: AuthenticatedUser,
    ValidatedJson(mut payload): ValidatedJson<CreateCommentDto>,
) -> Result<Json<CommentResponseDto>, AppError> {
    // 1. TCKN, telefon ve spam kontrolleri
    let payload_bytes = if let Some(ref text) = payload.comment {
        let validated_text = ContentGuard::validate_comment_text(text)
            .map_err(|msg| AppError::BadRequest(msg.to_string()))?;
        let bytes = validated_text.len();
        payload.comment = Some(validated_text);
        bytes
    } else {
        0
    };

    let parent_id = payload.parent_id;
    let comment =
        CommentService::create_comment(&db, user.id, user.username, payload.clone(), parent_id)
            .await?;

    // 2. 5651 Sayılı Kanun gereği RAM'de X25519 ile mühürlenen trafik kaydı
    if let Some(ref pk_hex) = config.traffic_log_public_key {
        let client_ip = headers
            .get("cf-connecting-ip")
            .or_else(|| headers.get("x-real-ip"))
            .or_else(|| headers.get("x-forwarded-for"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let client_port = headers
            .get("x-real-port")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);

        let traffic_payload = serde_json::json!({
            "ip": client_ip,
            "port": client_port,
            "user_id": user.id,
            "reference_id": comment.id,
            "service_type": "comment",
            "payload_bytes": payload_bytes,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Ok(sealed_hex) =
            crate::core::zero_knowledge::seal_traffic_log(pk_hex, &traffic_payload.to_string())
        {
            use sea_orm::{ActiveModelTrait, Set};
            let log_entry = shared::entities::traffic_logs::ActiveModel {
                id: Set(Uuid::new_v4()),
                key_id: Set(config.traffic_log_key_id),
                service_type: Set("comment".to_string()),
                reference_id: Set(Some(comment.id)),
                encrypted_payload: Set(sealed_hex),
                created_at: Set(Some(chrono::Utc::now().into())),
            };
            let _ = log_entry.insert(&db).await;
        }
    }

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
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    user: AuthenticatedUser,
    Path(hash): Path<Uuid>,
    ValidatedJson(mut payload): ValidatedJson<UpdateCommentDto>,
) -> Result<Json<CommentResponseDto>, AppError> {
    let validated_text = ContentGuard::validate_comment_text(&payload.comment)
        .map_err(|msg| AppError::BadRequest(msg.to_string()))?;
    let payload_bytes = validated_text.len();
    payload.comment = validated_text;

    let comment = CommentService::update_comment(&db, user.id, hash, payload.comment).await?;

    if let Some(ref pk_hex) = config.traffic_log_public_key {
        let client_ip = headers
            .get("cf-connecting-ip")
            .or_else(|| headers.get("x-real-ip"))
            .or_else(|| headers.get("x-forwarded-for"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let client_port = headers
            .get("x-real-port")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);

        let traffic_payload = serde_json::json!({
            "ip": client_ip,
            "port": client_port,
            "user_id": user.id,
            "reference_id": comment.id,
            "service_type": "comment_update",
            "payload_bytes": payload_bytes,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Ok(sealed_hex) =
            crate::core::zero_knowledge::seal_traffic_log(pk_hex, &traffic_payload.to_string())
        {
            use sea_orm::{ActiveModelTrait, Set};
            let log_entry = shared::entities::traffic_logs::ActiveModel {
                id: Set(Uuid::new_v4()),
                key_id: Set(config.traffic_log_key_id),
                service_type: Set("comment_update".to_string()),
                reference_id: Set(Some(comment.id)),
                encrypted_payload: Set(sealed_hex),
                created_at: Set(Some(chrono::Utc::now().into())),
            };
            let _ = log_entry.insert(&db).await;
        }
    }

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

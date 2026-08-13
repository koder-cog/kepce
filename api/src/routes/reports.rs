use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path},
};
use uuid::Uuid;

use crate::{
    error::AppError,
    extractors::auth::AuthenticatedUser,
    extractors::validated::ValidatedJson,
};
use shared::entities::{prelude::*, reports, sea_orm_active_enums::ReportStatusEnum};
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use serde::Deserialize;
use validator::Validate;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", post(submit_report))
        .route("/", get(get_reports))
        .route("/contact", get(get_contact_messages))
        .route("/contact/:id", axum::routing::patch(update_contact_status).delete(delete_contact))
        .route("/:id", axum::routing::patch(update_report_status).delete(delete_report))
}

#[derive(Deserialize, Validate)]
pub struct SubmitReportDto {
    pub target_type: String,
    pub target_id: String,
    pub reason: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateReportStatusDto {
    pub status: String,
}

async fn submit_report(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<SubmitReportDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    if (payload.reason == "other" || payload.reason == "bot_other" || payload.reason == "Diğer")
        && payload.description.as_ref().is_none_or(|d| d.trim().is_empty()) {
            return Err(AppError::BadRequest("Lütfen detaylı açıklama giriniz.".to_string()));
        }

    let mut report = reports::ActiveModel {
        reporter_id: Set(user.id),
        reason: Set(Some(payload.reason)),
        description: Set(payload.description),
        status: Set(ReportStatusEnum::Pending),
        ..Default::default()
    };

    match payload.target_type.as_str() {
        "comment" => {
            let comment_id = Uuid::parse_str(&payload.target_id).map_err(|_| AppError::BadRequest("Geçersiz yorum ID".into()))?;
            let comment_exists = Comments::find_by_id(comment_id).one(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?.is_some();
            if !comment_exists { return Err(AppError::NotFound("Yorum bulunamadı.".into())); }
            
            let existing_report = Reports::find()
                .filter(reports::Column::ReporterId.eq(user.id))
                .filter(reports::Column::ReportedCommentId.eq(comment_id))
                .filter(reports::Column::Status.eq(ReportStatusEnum::Pending))
                .one(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
            if existing_report.is_some() { return Err(AppError::BadRequest("Zaten şikayetiniz var.".into())); }
            
            report.reported_comment_id = Set(Some(comment_id));
            report.r#type = Set(Some("comment".into()));
        },
        "menu" => {
            let m_id: i32 = payload.target_id.parse().map_err(|_| AppError::BadRequest("Geçersiz menü ID".into()))?;
            report.menu_id = Set(Some(m_id));
            report.r#type = Set(Some("menu".into()));
        },
        "bot" => {
            let m_id: i32 = payload.target_id.parse().map_err(|_| AppError::BadRequest("Geçersiz menü ID".into()))?;
            report.menu_id = Set(Some(m_id));
            report.r#type = Set(Some("bot".into()));
        },
        "user" => {
            let reported_user_id = Uuid::parse_str(&payload.target_id).map_err(|_| AppError::BadRequest("Geçersiz kullanıcı ID".into()))?;
            report.reported_user_id = Set(Some(reported_user_id));
            report.r#type = Set(Some("user".into()));
        },
        _ => return Err(AppError::BadRequest("Bilinmeyen hedef türü.".into())),
    }

    report.insert(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "message": "Report submitted successfully" })))
}

async fn get_reports(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    let reports_list = Reports::find()
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    let mut result = Vec::new();
    for r in reports_list {
        result.push(serde_json::json!({
            "id": r.id,
            "reporter_id": r.reporter_id,
            "reported_comment_id": r.reported_comment_id,
            "reported_user_id": r.reported_user_id,
            "menu_id": r.menu_id,
            "type": r.r#type,
            "reason": r.reason,
            "description": r.description,
            "status": match r.status {
                ReportStatusEnum::Pending => "pending",
                ReportStatusEnum::Resolved => "resolved",
                ReportStatusEnum::Dismissed => "dismissed",
            },
            "created_at": r.created_at,
            "resolved_at": r.resolved_at
        }));
    }
    Ok(Json(serde_json::json!(result)))
}

async fn update_report_status(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(report_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateReportStatusDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    let mut report: reports::ActiveModel = Reports::find_by_id(report_id)
        .one(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Report not found".into()))?.into();
        
    let new_status = match payload.status.as_str() {
        "pending" => ReportStatusEnum::Pending,
        "resolved" => ReportStatusEnum::Resolved,
        "dismissed" => ReportStatusEnum::Dismissed,
        _ => return Err(AppError::BadRequest("Invalid status".to_string())),
    };
    report.status = Set(new_status);
    report.update(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({ "message": "Report status updated" })))
}

async fn get_contact_messages(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    
    let messages = shared::entities::contact_messages::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    let mut result = Vec::new();
    for m in messages {
        result.push(serde_json::json!({
            "id": m.id,
            "user_id": m.user_id,
            "email": m.email,
            "category": m.category,
            "subject": m.subject,
            "message": m.message,
            "status": match m.status {
                ReportStatusEnum::Pending => "pending",
                ReportStatusEnum::Resolved => "resolved",
                ReportStatusEnum::Dismissed => "dismissed",
            },
            "created_at": m.created_at,
            "resolved_at": m.resolved_at
        }));
    }
    Ok(Json(serde_json::json!(result)))
}

async fn delete_report(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(report_id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    Reports::delete_by_id(report_id).exec(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "message": "Report deleted" })))
}

async fn update_contact_status(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateReportStatusDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    let mut msg: shared::entities::contact_messages::ActiveModel = shared::entities::contact_messages::Entity::find_by_id(id)
        .one(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Contact message not found".into()))?.into();
        
    let new_status = match payload.status.as_str() {
        "pending" => ReportStatusEnum::Pending,
        "resolved" => ReportStatusEnum::Resolved,
        "dismissed" => ReportStatusEnum::Dismissed,
        _ => return Err(AppError::BadRequest("Invalid status".to_string())),
    };
    let is_resolved = new_status == ReportStatusEnum::Resolved;
    msg.status = Set(new_status);
    if is_resolved {
        msg.resolved_at = Set(Some(chrono::Utc::now().into()));
    } else {
        msg.resolved_at = Set(None);
    }
    msg.update(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({ "message": "Contact message status updated" })))
}

async fn delete_contact(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }
    shared::entities::contact_messages::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "message": "Contact message deleted" })))
}

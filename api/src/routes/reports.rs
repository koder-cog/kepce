use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    extractors::auth::{AuthenticatedUser, OptionalUser},
    extractors::validated::ValidatedJson,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;
use shared::entities::{prelude::*, reports, sea_orm_active_enums::ReportStatusEnum};
use validator::Validate;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", post(submit_report))
        .route("/", get(get_reports))
        .route("/contact", get(get_contact_messages))
        .route(
            "/contact/:id",
            axum::routing::patch(update_contact_status).delete(delete_contact),
        )
        .route("/contact/:id/reply", post(reply_contact_message))
        .route("/contact/:id/replies", get(get_contact_replies))
        .route(
            "/:id",
            axum::routing::patch(update_report_status).delete(delete_report),
        )
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
    user: OptionalUser,
    ValidatedJson(payload): ValidatedJson<SubmitReportDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    if (payload.reason == "other" || payload.reason == "bot_other" || payload.reason == "Diğer")
        && payload
            .description
            .as_ref()
            .is_none_or(|d| d.trim().is_empty())
    {
        return Err(AppError::BadRequest(
            "Lütfen detaylı açıklama giriniz.".to_string(),
        ));
    }

    let reporter_id = user.0.as_ref().map(|u| u.id);

    let mut report = reports::ActiveModel {
        reporter_id: Set(reporter_id),
        reason: Set(Some(payload.reason)),
        description: Set(payload.description),
        status: Set(ReportStatusEnum::Pending),
        ..Default::default()
    };

    match payload.target_type.as_str() {
        "comment" => {
            let comment_id = Uuid::parse_str(&payload.target_id)
                .map_err(|_| AppError::BadRequest("Geçersiz yorum ID".into()))?;
            let comment_exists = Comments::find_by_id(comment_id)
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .is_some();
            if !comment_exists {
                return Err(AppError::NotFound("Yorum bulunamadı.".into()));
            }

            if let Some(uid) = reporter_id {
                let existing_report = Reports::find()
                    .filter(reports::Column::ReporterId.eq(uid))
                    .filter(reports::Column::ReportedCommentId.eq(comment_id))
                    .filter(reports::Column::Status.eq(ReportStatusEnum::Pending))
                    .one(&state.db)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                if existing_report.is_some() {
                    return Err(AppError::BadRequest("Zaten şikayetiniz var.".into()));
                }
            }

            report.reported_comment_id = Set(Some(comment_id));
            report.r#type = Set(Some("comment".into()));
        }
        "menu" => {
            let m_id: i32 = payload
                .target_id
                .parse()
                .map_err(|_| AppError::BadRequest("Geçersiz menü ID".into()))?;
            report.menu_id = Set(Some(m_id));
            report.r#type = Set(Some("menu".into()));
        }
        "bot" => {
            let m_id: i32 = payload
                .target_id
                .parse()
                .map_err(|_| AppError::BadRequest("Geçersiz menü ID".into()))?;
            report.menu_id = Set(Some(m_id));
            report.r#type = Set(Some("bot".into()));
        }
        "user" => {
            let reported_user_id = Uuid::parse_str(&payload.target_id)
                .map_err(|_| AppError::BadRequest("Geçersiz kullanıcı ID".into()))?;
            report.reported_user_id = Set(Some(reported_user_id));
            report.r#type = Set(Some("user".into()));
        }
        _ => return Err(AppError::BadRequest("Bilinmeyen hedef türü.".into())),
    }

    report
        .insert(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "message": "Report submitted successfully" }),
    ))
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
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Report not found".into()))?
        .into();

    let new_status = match payload.status.as_str() {
        "pending" => ReportStatusEnum::Pending,
        "resolved" => ReportStatusEnum::Resolved,
        "dismissed" => ReportStatusEnum::Dismissed,
        _ => return Err(AppError::BadRequest("Invalid status".to_string())),
    };
    report.status = Set(new_status);
    report
        .update(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "message": "Report status updated" }),
    ))
}

#[derive(Deserialize, Validate)]
pub struct ReplyContactDto {
    #[validate(length(
        min = 2,
        max = 5000,
        message = "Yanıt 2 ile 5000 karakter arasında olmalıdır."
    ))]
    pub reply_body: String,
}

async fn get_contact_messages(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }

    let messages = shared::entities::contact_messages::Entity::find()
        .order_by_desc(shared::entities::contact_messages::Column::CreatedAt)
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
            "source": m.source,
            "page_url": m.page_url,
            "user_agent": m.user_agent,
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

async fn reply_contact_message(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<ReplyContactDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }

    let mut msg: shared::entities::contact_messages::ActiveModel =
        shared::entities::contact_messages::Entity::find_by_id(id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound("İletişim mesajı bulunamadı".into()))?
            .into();

    let target_email = msg.email.as_ref().clone();
    let original_subject = msg.subject.as_ref().clone();

    let reply_model = shared::entities::contact_message_replies::ActiveModel {
        contact_message_id: Set(id),
        responder_id: Set(Some(user.id)),
        reply_body: Set(payload.reply_body.clone()),
        created_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    };
    reply_model
        .insert(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    msg.status = Set(ReportStatusEnum::Resolved);
    msg.resolved_at = Set(Some(chrono::Utc::now().into()));
    msg.update(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Send email using EmailService
    let email_service = crate::services::email::EmailService::from_config(&state.config);
    let email_subject = format!("Re: {} - Kepçe Destek", original_subject);
    let safe_body = payload
        .reply_body
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    let email_html = format!(
        r#"<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #222; max-width: 600px; margin: 0 auto; padding: 24px; border: 1px solid #eaeaea; border-radius: 8px;">
            <h2 style="color: #e5533d; margin-top: 0;">Kepçe Destek Ekibi</h2>
            <p>Merhaba,</p>
            <p>İlettiğiniz mesaja istinaden yanıtımız aşağıdadır:</p>
            <div style="background: #f8f9fa; border-left: 4px solid #e5533d; padding: 16px; margin: 20px 0; border-radius: 4px; white-space: pre-wrap; font-size: 15px;">{}</div>
            <hr style="border: none; border-top: 1px solid #eee; margin: 24px 0;" />
            <p style="font-size: 12px; color: #777; margin-bottom: 0;">Bu e-posta Kepçe Destek Sistemi üzerinden iletilmiştir.</p>
        </div>"#,
        safe_body
    );

    if let Err(e) = email_service
        .send_email(&target_email, &email_subject, email_html)
        .await
    {
        tracing::error!(
            "İletişim yanıt e-postası gönderilemedi ({}): {:?}",
            target_email,
            e
        );
    }

    Ok(Json(
        serde_json::json!({ "message": "Yanıt gönderildi ve kaydedildi." }),
    ))
}

async fn get_contact_replies(
    State(state): State<crate::config::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role != crate::dto::user::UserRole::Admin {
        return Err(AppError::Forbidden("Admins only".to_string()));
    }

    let replies = shared::entities::contact_message_replies::Entity::find()
        .filter(shared::entities::contact_message_replies::Column::ContactMessageId.eq(id))
        .order_by_asc(shared::entities::contact_message_replies::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for r in replies {
        let responder_username = if let Some(uid) = r.responder_id {
            shared::entities::users::Entity::find_by_id(uid)
                .one(&state.db)
                .await
                .ok()
                .flatten()
                .map(|u| u.username)
        } else {
            None
        };

        result.push(serde_json::json!({
            "id": r.id,
            "contact_message_id": r.contact_message_id,
            "responder_id": r.responder_id,
            "responder_username": responder_username,
            "reply_body": r.reply_body,
            "created_at": r.created_at
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
    Reports::delete_by_id(report_id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
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
    let mut msg: shared::entities::contact_messages::ActiveModel =
        shared::entities::contact_messages::Entity::find_by_id(id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound("Contact message not found".into()))?
            .into();

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
    msg.update(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "message": "Contact message status updated" }),
    ))
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
    Ok(Json(
        serde_json::json!({ "message": "Contact message deleted" }),
    ))
}

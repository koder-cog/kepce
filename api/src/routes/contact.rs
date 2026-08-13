use axum::{
    routing::post,
    Router, Json, extract::State,
};
use crate::{
    error::AppError,
    extractors::auth::OptionalUser,
    extractors::validated::ValidatedJson,
};
use shared::entities::{contact_messages, sea_orm_active_enums::ReportStatusEnum};
use sea_orm::{Set, ActiveModelTrait};
use serde::Deserialize;
use validator::Validate;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", post(submit_contact_form))
}

#[derive(Deserialize, Validate)]
pub struct SubmitContactDto {
    #[validate(email(message = "Geçerli bir e-posta giriniz."))]
    pub email: String,
    pub report_type: String,
    #[validate(length(min = 3, max = 150, message = "Konu 3 ile 150 karakter arasında olmalıdır."))]
    pub subject: String,
    #[validate(length(min = 10, max = 2000, message = "Mesaj en az 10, en fazla 2000 karakter olmalıdır."))]
    pub description: String,
}

async fn submit_contact_form(
    State(state): State<crate::config::AppState>,
    user: OptionalUser,
    ValidatedJson(payload): ValidatedJson<SubmitContactDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    
    let contact_model = contact_messages::ActiveModel {
        user_id: Set(user.0.map(|u| u.id)),
        email: Set(payload.email),
        category: Set(payload.report_type),
        subject: Set(payload.subject),
        message: Set(payload.description),
        status: Set(ReportStatusEnum::Pending),
        created_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    };

    contact_model.insert(&state.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({ "message": "Contact message submitted successfully" })))
}

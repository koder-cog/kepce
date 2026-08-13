use axum::{
    routing::get,
    Router,
    extract::State,
    Json,
};
use crate::error::AppError;
use crate::services::system::SystemService;
use crate::dto::system::{VerifyTreeResponseDto, SystemStatusDto, ComponentHistoryDto, SystemHealthResponseDto};
use crate::extractors::auth::AuthenticatedUser;
use crate::dto::user::UserRole;

/// SA-11: Bütünlük doğrulama ve geçmiş sorguları iç altyapı bilgisi sızdırır
/// ve DB ağırlığı yaratır; yalnızca adminlere açıktır.
fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }
    Ok(())
}

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(system_status))
        .route("/verify", get(verify_tree))
        .route("/status/history", get(get_status_history))
}

async fn health_check(State(db): State<sea_orm::DatabaseConnection>) -> Result<Json<SystemHealthResponseDto>, AppError> {
    let result = SystemService::get_system_health(&db).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}

async fn system_status(State(db): State<sea_orm::DatabaseConnection>) -> Result<Json<SystemStatusDto>, AppError> {
    let result = SystemService::get_system_status(&db).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}

async fn verify_tree(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<VerifyTreeResponseDto>, AppError> {
    require_admin(&user)?;
    let result = SystemService::verify_chain(&db).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}

async fn get_status_history(
    State(db): State<sea_orm::DatabaseConnection>,
) -> Result<Json<Vec<ComponentHistoryDto>>, AppError> {
    let result = SystemService::get_status_history(&db).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}

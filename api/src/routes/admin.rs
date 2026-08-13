use axum::{
    extract::{Path, State},
    routing::{post, put, delete, get},
    Json, Router,
};
use crate::extractors::validated::ValidatedJson;
use crate::{
    config::AppState,
    dto::admin::{CreateDishDto, UpdateDishDto, MergeDishesDto, SplitDishDto, DetachDishDto},
    services::admin as admin_service,
};
use crate::dto::user::UserRole;
use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dishes/stats", get(get_dish_stats))
        .route("/dishes", post(create_dish))
        .route("/dishes/:id", put(update_dish))
        .route("/dishes/:id", delete(delete_dish))
        .route("/dishes/merge", post(merge_dishes))
        .route("/dishes/split", post(split_dish))
        .route("/dishes/detach", post(detach_dish))
}

fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem için yetkiniz yok.".into()));
    }
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct DishStatsQuery {
    pub search: Option<String>,
}

async fn get_dish_stats(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Query(query): axum::extract::Query<DishStatsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let dtos = admin_service::get_dish_stats(&state.db, query.search)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!(dtos)))
}

async fn create_dish(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreateDishDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let dish = admin_service::create_dish(&state.db, payload).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ 
        "success": true, 
        "dish": {
            "id": dish.id,
            "name": dish.name,
            "category": dish.category,
            "is_celiac": dish.is_celiac,
            "is_vegan": dish.is_vegan,
            "is_vegetarian": dish.is_vegetarian,
        }
    })))
}

async fn update_dish(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateDishDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let dish = admin_service::update_dish(&state.db, id, payload).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ 
        "success": true, 
        "dish": {
            "id": dish.id,
            "name": dish.name,
            "category": dish.category,
            "is_celiac": dish.is_celiac,
            "is_vegan": dish.is_vegan,
            "is_vegetarian": dish.is_vegetarian,
        }
    })))
}

async fn delete_dish(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    admin_service::delete_dish(&state.db, id).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn merge_dishes(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<MergeDishesDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    admin_service::merge_dishes(&state.db, payload).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn split_dish(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<SplitDishDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let dish = admin_service::split_dish(&state.db, payload).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ 
        "success": true, 
        "dish": {
            "id": dish.id,
            "name": dish.name,
            "category": dish.category,
            "is_celiac": dish.is_celiac,
            "is_vegan": dish.is_vegan,
            "is_vegetarian": dish.is_vegetarian,
        }
    })))
}

async fn detach_dish(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DetachDishDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    admin_service::detach_dish(&state.db, payload).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

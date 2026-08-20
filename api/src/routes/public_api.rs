use axum::{
    routing::get,
    Router,
    extract::{State, Path},
    http::HeaderMap,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use shared::entities::cities;
use crate::error::AppError;
use crate::services::menu::MenuService;
use crate::services::city::CityService;

#[derive(serde::Serialize)]
pub struct CityResponseDto {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub has_celiac: bool,
}

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/cities", get(get_cities))
        .route("/cities/detect", get(detect_city))
        .route("/menus/today/:city_id", get(get_today_menu))
        .route("/menus/:id", get(get_single_menu))
}

pub fn cities_router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", get(get_cities))
        .route("/detect", get(detect_city))
}

pub async fn get_cities(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let cities_data = CityService::get_active_cities(&db)
        .await
        .map_err(|e| {
            tracing::error!("CityService Error: {:?}", e);
            AppError::Internal("Database error".to_string())
        })?;

    let response: Vec<CityResponseDto> = cities_data
        .into_iter()
        .map(|(c, has_celiac)| CityResponseDto {
            id: c.id,
            name: c.name,
            slug: c.slug,
            has_celiac,
        })
        .collect();

    crate::utils::response::cached_json_response(&headers, &response, 3600)
}

use axum::extract::Query;

#[derive(serde::Deserialize)]
pub struct MenuQuery {
    pub dietary_type: Option<String>,
}

async fn get_today_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: crate::extractors::api_key::ValidApiKey,
    headers: HeaderMap,
    Path(city_id): Path<i32>,
    Query(query): Query<MenuQuery>,
) -> Result<axum::response::Response, AppError> {
    let today = Utc::now().date_naive();
    let menus = MenuService::get_daily_menus(&db, city_id, today, query.dietary_type, None).await?;
    crate::utils::response::cached_json_response(&headers, &menus, 300)
}

/// GET /cities/detect
/// Cloudflare header bilgisinden şehir tahmini yapar.
pub async fn detect_city(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    // 1) Cloudflare CF-IPCity header'ını kontrol et
    if let Some(cf_city) = headers.get("CF-IPCity").and_then(|v| v.to_str().ok()) {
        let slug = cf_city.to_lowercase().replace(' ', "-");
        let city = cities::Entity::find()
            .filter(cities::Column::Slug.eq(&slug))
            .one(&db)
            .await
            .map_err(|e| {
                tracing::error!("DB error detecting city by CF header: {}", e);
                AppError::Internal("DB Error".to_string())
            })?;

        if let Some(c) = city {
            return Ok(Json(json!({"city_slug": c.slug, "city_name": c.name, "source": "cloudflare"})));
        }
    }

    // 2) Fallback - ilk şehri döndür
    let first = cities::Entity::find()
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching fallback city: {}", e);
            AppError::Internal("DB Error".to_string())
        })?;

    match first {
        Some(c) => Ok(Json(json!({"city_slug": c.slug, "city_name": c.name, "source": "fallback"}))),
        None => Ok(Json(json!({"city_slug": serde_json::Value::Null, "source": "none"}))),
    }
}

async fn get_single_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: crate::extractors::api_key::ValidApiKey,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Query(query): Query<MenuQuery>,
) -> Result<axum::response::Response, AppError> {
    let menu = MenuService::get_menu_with_items(&db, id, query.dietary_type, None).await?;
    crate::utils::response::cached_json_response(&headers, &menu, 300)
}

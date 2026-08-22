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
        .route("/menus/months", get(get_menu_months))
        .route("/menus/index", get(get_menu_index))
        .route("/menus/today/:city", get(get_today_menu))
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

/// GET /api/v1/public/menus/today/:city
/// Kanonik anahtar slug'dır; geriye dönük uyumluluk için sayısal id de kabul edilir.
async fn get_today_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: crate::extractors::api_key::ValidApiKey,
    headers: HeaderMap,
    Path(city): Path<String>,
    Query(query): Query<MenuQuery>,
) -> Result<axum::response::Response, AppError> {
    // Slug kanonik; sayısal id gelirse olduğu gibi kullanılır.
    let city_id: i32 = match city.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            let c = cities::Entity::find()
                .filter(cities::Column::Slug.eq(city.to_lowercase()))
                .one(&db)
                .await
                .map_err(|e| {
                    tracing::error!("DB error resolving city slug: {}", e);
                    AppError::Internal("DB Error".to_string())
                })?
                .ok_or_else(|| AppError::NotFound("Şehir bulunamadı.".to_string()))?;
            c.id
        }
    };

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

// ============================================================
// Sitemap veri kaynakları — aylık bölünmüş sitemap index yapısı
// Yalnızca id + serve_date döner; item join'i YOKTUR (ucuz sorgu).
// ============================================================

/// GET /api/v1/public/menus/months
/// Onaylı menülerin bulunduğu aylar (YYYY-MM), yeniden eskiden.
pub async fn get_menu_months(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    use sea_orm::{ConnectionTrait, Statement, DatabaseBackend};
    let rows = db
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT DISTINCT TO_CHAR(serve_date, 'YYYY-MM') AS month \
             FROM menus WHERE status = 'approved' ORDER BY month DESC",
        ))
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching menu months: {}", e);
            AppError::Internal("DB Error".to_string())
        })?;

    let months: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String>("", "month").ok())
        .collect();

    crate::utils::response::cached_json_response(&headers, &months, 3600)
}

#[derive(serde::Deserialize)]
pub struct MenuIndexQuery {
    /// YYYY-MM biçiminde ay
    pub month: String,
}

/// GET /api/v1/public/menus/index?month=YYYY-MM
/// Belirtilen aydaki onaylı menülerin { id, serve_date } listesi.
pub async fn get_menu_index(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: HeaderMap,
    Query(query): Query<MenuIndexQuery>,
) -> Result<axum::response::Response, AppError> {
    use shared::entities::{menus, sea_orm_active_enums::MenuStatusEnum};
    use sea_orm::QuerySelect;

    let parts: Vec<&str> = query.month.split('-').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Ay formatı YYYY-MM olmalıdır.".to_string()));
    }
    let year: i32 = parts[0].parse().map_err(|_| AppError::BadRequest("Geçersiz yıl.".to_string()))?;
    let month: u32 = parts[1].parse().map_err(|_| AppError::BadRequest("Geçersiz ay.".to_string()))?;
    if !(1..=12).contains(&month) {
        return Err(AppError::BadRequest("Geçersiz ay.".to_string()));
    }

    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::BadRequest("Geçersiz ay.".to_string()))?;
    let (next_y, next_m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let end = chrono::NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .ok_or_else(|| AppError::BadRequest("Geçersiz ay.".to_string()))?;

    let items = menus::Entity::find()
        .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
        .filter(menus::Column::ServeDate.gte(start))
        .filter(menus::Column::ServeDate.lt(end))
        .select_only()
        .column(menus::Column::Id)
        .column_as(menus::Column::ServeDate, "serve_date")
        .into_json()
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching menu index: {}", e);
            AppError::Internal("DB Error".to_string())
        })?;

    // Geçmiş aylar değişmez → uzun cache; güncel ay kısa cache
    let now_month = chrono::Utc::now().format("%Y-%m").to_string();
    let ttl: u32 = if query.month == now_month { 3600 } else { 86400 };
    crate::utils::response::cached_json_response(&headers, &items, ttl)
}

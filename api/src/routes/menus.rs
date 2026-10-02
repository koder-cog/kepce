//! Şehir ve gün bazlı KYK menü sorgulama endpoint'leri.
use crate::error::AppError;
use crate::extractors::api_key::OptionalApiKey;
use crate::extractors::auth::{AuthenticatedUser, OptionalUser};
use crate::services::menu::{MenuError, MenuService};
use crate::services::user::UserService;
use crate::services::vote::{VoteError, VoteService};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use chrono::NaiveDate;
use serde::Deserialize;
use shared::entities::sea_orm_active_enums::SentimentEnum;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/", get(get_menus))
        .route("/today", get(get_today))
        .route("/today/:city", get(get_today_city))
        .route("/range", get(get_menus_range))
        .route("/months", get(crate::routes::public_api::get_menu_months))
        .route("/days", get(crate::routes::public_api::get_menu_days))
        .route("/index", get(crate::routes::public_api::get_menu_index))
        .route("/archive/years", get(get_archive_years))
        .route("/archive/highlights", get(get_archive_highlights))
        .route("/:menu_id", get(get_menu))
        .route("/:menu_id/vote", axum::routing::post(vote_menu))
}

impl From<MenuError> for AppError {
    fn from(err: MenuError) -> Self {
        match err {
            MenuError::NotFound => AppError::NotFound("Menu not found".to_string()),
            MenuError::DatabaseError(e) => {
                tracing::error!("Database error in MenuService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

impl From<VoteError> for AppError {
    fn from(err: VoteError) -> Self {
        match err {
            VoteError::MenuNotFound => AppError::NotFound("Menu not found".to_string()),
            VoteError::UnverifiedUser => {
                AppError::Forbidden("Oy vermek için e-postanızı onaylamalısınız.".to_string())
            }
            VoteError::DatabaseError(e) => {
                tracing::error!("Database error in VoteService: {}", e);
                AppError::Internal("Database error".to_string())
            }
        }
    }
}

async fn get_today(
    State(db): State<sea_orm::DatabaseConnection>,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Query(filter): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let today = match filter.date.as_deref() {
        Some("today") | None => crate::utils::time::istanbul_today(),
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest(
                "Geçersiz tarih formatı. YYYY-MM-DD veya 'today' kullanılmalıdır.".to_string(),
            )
        })?,
    };
    let user_id = user.as_ref().map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(
        &db,
        filter.city,
        Some(today),
        None,
        filter.dietary_type,
        None,
        None,
        user_id,
    )
    .await?;
    let is_private = user.is_some();
    crate::utils::response::cached_json_response_with_privacy(&headers, &menus, 300, is_private)
}

#[derive(Deserialize)]
pub struct MenuFilterQueryDto {
    pub city: Option<String>,
    pub date: Option<String>,
    pub dietary_type: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

async fn get_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: OptionalApiKey,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let user_id = user.as_ref().map(|u| u.id);
    let parsed_date = match query.date.as_deref() {
        Some("today") => Some(crate::utils::time::istanbul_today()),
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                return Err(AppError::BadRequest(
                    "Geçersiz tarih formatı. YYYY-MM-DD veya 'today' kullanılmalıdır.".to_string(),
                ));
            }
        },
        None => None,
    };

    let menus = MenuService::get_menus_by_filter(
        &db,
        query.city,
        parsed_date,
        None,
        query.dietary_type,
        query.year,
        query.month,
        user_id,
    )
    .await?;
    let is_private = user.is_some();
    crate::utils::response::cached_json_response_with_privacy(&headers, &menus, 300, is_private)
}

async fn get_today_city(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: OptionalApiKey,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Path(city): Path<String>,
    Query(query): Query<MenuFilterQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let today = crate::utils::time::istanbul_today();
    let user_id = user.as_ref().map(|u| u.id);
    let menus = MenuService::get_menus_by_filter(
        &db,
        Some(city),
        Some(today),
        None,
        query.dietary_type,
        None,
        None,
        user_id,
    )
    .await?;
    let is_private = user.is_some();
    crate::utils::response::cached_json_response_with_privacy(&headers, &menus, 300, is_private)
}

#[derive(Deserialize)]
pub struct RangeQueryDto {
    pub city: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub dietary_type: Option<String>,
}

pub fn validate_range_params(
    query: &RangeQueryDto,
) -> Result<(String, NaiveDate, NaiveDate), AppError> {
    let city_slug = match query.city.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return Err(AppError::BadRequest(
                "city parametresi zorunludur.".to_string(),
            ));
        }
    };

    let start_date_str = match query.start_date.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Err(AppError::BadRequest(
                "start_date parametresi zorunludur.".to_string(),
            ));
        }
    };

    let end_date_str = match query.end_date.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Err(AppError::BadRequest(
                "end_date parametresi zorunludur.".to_string(),
            ));
        }
    };

    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("Geçersiz start_date formatı. YYYY-MM-DD kullanılmalıdır.".to_string())
    })?;

    let end_date = NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("Geçersiz end_date formatı. YYYY-MM-DD kullanılmalıdır.".to_string())
    })?;

    if start_date > end_date {
        return Err(AppError::BadRequest(
            "start_date, end_date tarihinden sonra olamaz.".to_string(),
        ));
    }

    let diff_days = (end_date - start_date).num_days();
    if diff_days > 31 {
        return Err(AppError::BadRequest(
            "Tarih aralığı en fazla 31 gün olabilir.".to_string(),
        ));
    }

    Ok((city_slug, start_date, end_date))
}

async fn get_menus_range(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: OptionalApiKey,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Query(query): Query<RangeQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let (city_slug, start_date, end_date) = validate_range_params(&query)?;

    let user_id = user.as_ref().map(|u| u.id);
    let menus = MenuService::get_menus_by_range(
        &db,
        &city_slug,
        start_date,
        end_date,
        query.dietary_type,
        user_id,
    )
    .await?;

    let is_private = user.is_some();
    let today = crate::utils::time::istanbul_today();
    let cache_ttl = if end_date < today { 3600 } else { 300 };

    crate::utils::response::cached_json_response_with_privacy(
        &headers, &menus, cache_ttl, is_private,
    )
}

#[derive(Deserialize)]
pub struct ArchiveYearsQuery {
    pub city: Option<String>,
}

async fn get_archive_years(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: http::HeaderMap,
    Query(query): Query<ArchiveYearsQuery>,
) -> Result<axum::response::Response, AppError> {
    let years = MenuService::get_archive_years(&db, query.city).await?;
    crate::utils::response::cached_json_response(&headers, &years, 3600)
}

#[derive(Deserialize)]
pub struct ArchiveHighlightsQuery {
    pub limit: Option<u64>,
}

async fn get_archive_highlights(
    State(db): State<sea_orm::DatabaseConnection>,
    headers: http::HeaderMap,
    Query(query): Query<ArchiveHighlightsQuery>,
) -> Result<axum::response::Response, AppError> {
    let limit = query.limit.unwrap_or(4);
    let highlights = MenuService::get_archive_highlights(&db, limit).await?;
    crate::utils::response::cached_json_response(&headers, &highlights, 300)
}

#[derive(Deserialize)]
pub struct MenuDetailQueryDto {
    pub dietary_type: Option<String>,
}

async fn get_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    _key: OptionalApiKey,
    OptionalUser(user): OptionalUser,
    headers: http::HeaderMap,
    Path(menu_id): Path<i32>,
    Query(query): Query<MenuDetailQueryDto>,
) -> Result<axum::response::Response, AppError> {
    let user_id = user.as_ref().map(|u| u.id);
    let menu = MenuService::get_menu_with_items(&db, menu_id, query.dietary_type, user_id).await?;
    let is_private = user.is_some();
    crate::utils::response::cached_json_response_with_privacy(&headers, &menu, 300, is_private)
}

#[derive(Deserialize)]
pub struct VoteMenuDto {
    pub sentiment: String,
}

async fn vote_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    Json(payload): Json<VoteMenuDto>,
) -> Result<Json<()>, AppError> {
    UserService::ensure_cross_border_consent(&db, user.id).await?;
    let sentiment =
        match payload.sentiment.to_lowercase().as_str() {
            "positive" => SentimentEnum::Positive,
            "negative" => SentimentEnum::Negative,
            "neutral" | "" => SentimentEnum::Neutral,
            _ => return Err(AppError::BadRequest(
                "Geçersiz oy türü. Yalnızca 'positive', 'negative' veya 'neutral' kabul edilir."
                    .to_string(),
            )),
        };

    VoteService::vote_menu(&db, menu_id, user.id, sentiment).await?;
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range_missing_city() {
        let q = RangeQueryDto {
            city: None,
            start_date: Some("2026-09-01".to_string()),
            end_date: Some("2026-09-15".to_string()),
            dietary_type: None,
        };
        let err = validate_range_params(&q).unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert_eq!(msg, "city parametresi zorunludur."),
            _ => panic!("Expected BadRequest"),
        }

        let q_empty = RangeQueryDto {
            city: Some("   ".to_string()),
            start_date: Some("2026-09-01".to_string()),
            end_date: Some("2026-09-15".to_string()),
            dietary_type: None,
        };
        let err_empty = validate_range_params(&q_empty).unwrap_err();
        match err_empty {
            AppError::BadRequest(msg) => assert_eq!(msg, "city parametresi zorunludur."),
            _ => panic!("Expected BadRequest"),
        }
    }

    #[test]
    fn test_validate_range_missing_dates() {
        let q_no_start = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: None,
            end_date: Some("2026-09-15".to_string()),
            dietary_type: None,
        };
        assert!(matches!(
            validate_range_params(&q_no_start).unwrap_err(),
            AppError::BadRequest(_)
        ));

        let q_no_end = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: Some("2026-09-01".to_string()),
            end_date: None,
            dietary_type: None,
        };
        assert!(matches!(
            validate_range_params(&q_no_end).unwrap_err(),
            AppError::BadRequest(_)
        ));
    }

    #[test]
    fn test_validate_range_invalid_date_format() {
        let q = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: Some("01-09-2026".to_string()),
            end_date: Some("2026-09-15".to_string()),
            dietary_type: None,
        };
        let err = validate_range_params(&q).unwrap_err();
        match err {
            AppError::BadRequest(msg) => {
                assert!(msg.contains("Geçersiz start_date formatı"));
            }
            _ => panic!("Expected BadRequest"),
        }
    }

    #[test]
    fn test_validate_range_start_after_end() {
        let q = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: Some("2026-09-20".to_string()),
            end_date: Some("2026-09-10".to_string()),
            dietary_type: None,
        };
        let err = validate_range_params(&q).unwrap_err();
        match err {
            AppError::BadRequest(msg) => {
                assert_eq!(msg, "start_date, end_date tarihinden sonra olamaz.");
            }
            _ => panic!("Expected BadRequest"),
        }
    }

    #[test]
    fn test_validate_range_exceeds_31_days() {
        let q = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: Some("2026-09-01".to_string()),
            end_date: Some("2026-10-03".to_string()), // 32 days
            dietary_type: None,
        };
        let err = validate_range_params(&q).unwrap_err();
        match err {
            AppError::BadRequest(msg) => {
                assert_eq!(msg, "Tarih aralığı en fazla 31 gün olabilir.");
            }
            _ => panic!("Expected BadRequest"),
        }
    }

    #[test]
    fn test_validate_range_valid_31_days() {
        let q = RangeQueryDto {
            city: Some("istanbul".to_string()),
            start_date: Some("2026-09-01".to_string()),
            end_date: Some("2026-10-02".to_string()), // Exactly 31 days
            dietary_type: Some("normal".to_string()),
        };
        let (city, start, end) = validate_range_params(&q).unwrap();
        assert_eq!(city, "istanbul");
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 10, 2).unwrap());
    }
}

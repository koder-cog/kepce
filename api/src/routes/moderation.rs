//! Topluluk denetimi, yorum moderasyonu ve rapor yönetim endpoint'leri.
//!
//! Yalnızca yetkili moderatör veya yönetici rolündeki kullanıcılar erişebilir.
use crate::dto::moderation::{
    BotExportMonthlyQuery, BotExportMonthlyResponseDto, BotGenerateRequestDto,
    BotGenerateResponseDto, BulkUpdateMenuStatusDto, BulkUpdateMenuStatusResponseDto,
    CreateMenuDto, InjectBotCommentsDto, InjectBotCommentsResponseDto, ReportCommentRequestDto,
    ResolveReportDto, SubmissionItemDto, UpdateSubmissionStatusDto, UpdateUserStatusDto,
    WarnUserDto,
};
use crate::dto::user::UserRole;
use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;
use crate::extractors::validated::ValidatedJson;
use crate::services::bot::{BotError, BotService};
use crate::services::moderation::ModerationService;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/submissions", get(get_submissions))
        .route(
            "/submissions/:submission_id/status",
            post(update_submission_status),
        )
        .route("/report/:hash", post(report_comment))
        .route("/bot/generate", post(generate_bot_comment))
        .route("/bot/export-monthly", get(export_monthly_menu_for_bot))
        .route("/bot/inject", post(inject_bot_comments))
        .route("/users/:user_id/status", put(update_user_status))
        .route("/users/:user_id", put(update_user))
        .route("/users/:user_id/ban", post(ban_user))
        .route("/users/:user_id/warn", post(warn_user))
        .route("/reports/:report_id/resolve", post(resolve_report))
        .route("/pending", get(get_pending_menus))
        .route("/menus", get(get_menus).post(create_menu))
        .route("/menus/bulk-status", post(bulk_update_menu_status))
        .route("/:menu_id/approve", post(approve_menu))
        .route("/:menu_id/reject", post(reject_menu))
        .route("/menus/:menu_id/commentary", put(update_menu_commentary))
        .route(
            "/:menu_id/items",
            get(get_menu_items).put(update_menu_items),
        )
        .route("/votes/pending", get(get_pending_votes))
        .route("/votes/all", get(get_all_votes))
        .route("/votes/complaints", get(get_complaints))
        .route("/votes/:vote_id/approve", post(approve_vote))
        .route("/votes/:vote_id/reject", post(reject_vote))
        .route("/votes/:vote_id/reset", post(reset_vote))
        .route("/votes/:vote_id/purge", delete(purge_vote))
        .route("/users", get(get_users))
        .route("/tags", get(get_tags).post(create_tag))
        .route("/tags/:tag_id", put(update_tag).delete(delete_tag))
        .route("/kitchen/coverage", get(get_kitchen_coverage))
        .nest("/database", crate::routes::database_admin::router())
        .route("/incidents", get(get_incidents).post(create_incident))
        .route(
            "/incidents/:incident_id",
            put(update_incident).delete(delete_incident),
        )
}

impl From<BotError> for AppError {
    fn from(err: BotError) -> Self {
        match err {
            BotError::NetworkError(e) => {
                tracing::error!("Bot Network Error: {}", e);
                AppError::Internal("AI yorum üretilemedi".to_string())
            }
            BotError::ApiError(e) => {
                tracing::error!("Bot API Error: {}", e);
                AppError::Internal("AI yorum üretilemedi".to_string())
            }
        }
    }
}

// NOT: From<ModerationError> for AppError trait impl'i routes/profile.rs'te tanımlı.
// Rust'ta trait impl'leri crate-wide geçerli olduğu için burada ? operatörü doğrudan çalışır.

async fn report_comment(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(hash): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ReportCommentRequestDto>,
) -> Result<Json<()>, AppError> {
    ModerationService::report_comment(&db, user.id, hash, payload.reason).await?;
    Ok(Json(()))
}

/// Admin-only: Gemini API üzerinden AI yorum üretir
async fn generate_bot_comment(
    State(config): State<std::sync::Arc<crate::config::Config>>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<BotGenerateRequestDto>,
) -> Result<Json<BotGenerateResponseDto>, AppError> {
    // Sadece admin tetikleyebilir
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }

    // API anahtarı yapılandırılmış mı?
    let api_key = config
        .gemini_api_key
        .as_deref()
        .ok_or_else(|| AppError::Internal("AI servisi yapılandırılmamış".to_string()))?;

    let client = reqwest::Client::new();
    let context = format!(
        "Yemek: {}\nSentiment: {}",
        payload.dish_name, payload.sentiment
    );
    let generated = BotService::generate_ai_comment(
        &client,
        api_key,
        &config.gemini_model,
        &config.bot_directive,
        &context,
    )
    .await?;

    Ok(Json(BotGenerateResponseDto {
        generated_comment: generated,
    }))
}

/// Admin-only: Aylık menüyü bot girdisi (prompt) + şema olarak dışa aktarır.
async fn export_monthly_menu_for_bot(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<Arc<crate::config::Config>>,
    user: AuthenticatedUser,
    Query(query): Query<BotExportMonthlyQuery>,
) -> Result<Json<BotExportMonthlyResponseDto>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }

    let menu_text =
        ModerationService::export_monthly_menu_for_bot(&db, &query.city_slug, &query.month).await?;

    let schema: serde_json::Value = serde_json::from_str(BotService::BOT_OUTPUT_SCHEMA)
        .map_err(|e| AppError::Internal(format!("Bot şeması çözümlenemedi: {}", e)))?;

    let directive = config.bot_directive.clone();
    let menu_data = menu_text.trim().to_string();
    let prompt = format!("{}\n\n{}", directive, menu_data);

    Ok(Json(BotExportMonthlyResponseDto {
        directive,
        menu_data,
        prompt,
        schema,
    }))
}

/// Admin-only: Bot yorumlarını menülere yazar (gün bazlı, tüm öğünler).
async fn inject_bot_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<InjectBotCommentsDto>,
) -> Result<Json<InjectBotCommentsResponseDto>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }

    let updated =
        ModerationService::inject_bot_comments(&db, &payload.city_slug, &payload.comments).await?;

    Ok(Json(InjectBotCommentsResponseDto {
        updated_count: updated,
    }))
}

async fn update_user_status(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserStatusDto>,
) -> Result<Json<()>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }
    ModerationService::update_user_status(&db, user_id, &payload.status)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

async fn update_user(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateUserDto>,
) -> Result<Json<()>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }
    ModerationService::update_user(
        &db,
        user_id,
        payload.is_verified,
        payload.is_admin,
        payload.is_banned,
    )
    .await
    .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

async fn resolve_report(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(report_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<ResolveReportDto>,
) -> Result<Json<()>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }
    ModerationService::resolve_report(&db, report_id, payload.action_taken)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

// --- IMPLEMENTED MODERATION ENDPOINTS ---

use sea_orm::QueryOrder;
use sea_orm::*;
use shared::entities::{comments, menus, prelude::*, tags, users};

fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Bu işlem yalnızca yöneticilere açıktır".to_string(),
        ));
    }
    Ok(())
}

async fn ban_user(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    ModerationService::update_user_status(&db, user_id, "banned")
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

async fn warn_user(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<WarnUserDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    use shared::entities::user_warnings;

    let warning = user_warnings::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        user_id: Set(user_id),
        message: Set(payload.message.clone()),
        created_at: Set(Some(chrono::Utc::now().into())),
    };

    warning
        .insert(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::services::notification::NotificationService::send_notification(
        &db,
        user_id,
        "moderation",
        "Moderasyon Uyarısı",
        &payload.message,
        None,
        None,
    )
    .await;

    Ok(Json(()))
}

async fn get_pending_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::MenuModerationResponseDto>>, AppError> {
    require_admin(&user)?;
    let menus_with_cities = Menus::find()
        .filter(
            menus::Column::Status
                .eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending),
        )
        .find_also_related(shared::entities::cities::Entity)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for (m, city_opt) in menus_with_cities {
        result.push(crate::dto::moderation::MenuModerationResponseDto {
            id: m.id,
            date: m.serve_date.to_string(),
            meal_type: match m.meal_type {
                shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => {
                    "breakfast".to_string()
                }
                shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => {
                    "dinner".to_string()
                }
            },
            status: "pending".to_string(),
            source_type: m.source_type,
            notice: m.notice,
            bot_commentary: m.bot_commentary,
            city: city_opt.map(|c| crate::dto::moderation::MenuModerationCityDto { name: c.name }),
        });
    }
    Ok(Json(result))
}

async fn get_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Query(query): Query<crate::dto::moderation::GetMenusQuery>,
) -> Result<Json<Vec<crate::dto::moderation::MenuModerationResponseDto>>, AppError> {
    require_admin(&user)?;

    use shared::entities::sea_orm_active_enums::MenuStatusEnum;

    let mut condition = sea_orm::Condition::all();

    if let Some(status) = &query.status {
        if !status.is_empty() {
            let status_enum = match status.as_str() {
                "pending" => MenuStatusEnum::Pending,
                "approved" => MenuStatusEnum::Approved,
                "rejected" => MenuStatusEnum::Rejected,
                _ => return Err(AppError::BadRequest("Invalid status".to_string())),
            };
            condition = condition.add(menus::Column::Status.eq(status_enum));
        }
    }

    if let Some(month) = &query.month {
        if !month.is_empty() {
            if month.len() == 7 {
                // YYYY-MM
                let parts: Vec<&str> = month.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(y), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                        let next_y = if m == 12 { y + 1 } else { y };
                        let next_m = if m == 12 { 1 } else { m + 1 };

                        if let (Some(start_date), Some(end_date)) = (
                            chrono::NaiveDate::from_ymd_opt(y, m, 1),
                            chrono::NaiveDate::from_ymd_opt(next_y, next_m, 1),
                        ) {
                            condition = condition.add(menus::Column::ServeDate.gte(start_date));
                            condition = condition.add(menus::Column::ServeDate.lt(end_date));
                        }
                    }
                }
            } else if month.len() == 4 {
                // YYYY
                if let Ok(y) = month.parse::<i32>() {
                    if let (Some(start_date), Some(end_date)) = (
                        chrono::NaiveDate::from_ymd_opt(y, 1, 1),
                        chrono::NaiveDate::from_ymd_opt(y + 1, 1, 1),
                    ) {
                        condition = condition.add(menus::Column::ServeDate.gte(start_date));
                        condition = condition.add(menus::Column::ServeDate.lt(end_date));
                    }
                }
            }
        }
    }

    let mut select = Menus::find().filter(condition);

    if let Some(city_slug) = &query.city_slug {
        if !city_slug.is_empty() {
            let city = shared::entities::cities::Entity::find()
                .filter(shared::entities::cities::Column::Slug.eq(city_slug))
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some(c) = city {
                select = select.filter(menus::Column::CityId.eq(c.id));
            } else {
                return Ok(Json(Vec::new())); // City not found, return empty
            }
        }
    }

    let menus_with_cities = select
        .find_also_related(shared::entities::cities::Entity)
        .order_by_desc(menus::Column::ServeDate)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for (m, city_opt) in menus_with_cities {
        result.push(crate::dto::moderation::MenuModerationResponseDto {
            id: m.id,
            date: m.serve_date.to_string(),
            meal_type: match m.meal_type {
                shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => {
                    "breakfast".to_string()
                }
                shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => {
                    "dinner".to_string()
                }
            },
            status: match m.status {
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending => {
                    "pending".to_string()
                }
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved => {
                    "approved".to_string()
                }
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected => {
                    "rejected".to_string()
                }
            },
            source_type: m.source_type,
            notice: m.notice,
            bot_commentary: m.bot_commentary,
            city: city_opt.map(|c| crate::dto::moderation::MenuModerationCityDto { name: c.name }),
        });
    }
    Ok(Json(result))
}

async fn create_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(dto): ValidatedJson<CreateMenuDto>,
) -> Result<Json<crate::dto::moderation::MenuModerationResponseDto>, AppError> {
    require_admin(&user)?;

    let menu = crate::services::moderation::create_menu(&db, dto, user.id).await?;

    let city = shared::entities::cities::Entity::find_by_id(menu.city_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let meal_type_str = match menu.meal_type {
        shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast".to_string(),
        shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch".to_string(),
        shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner".to_string(),
    };

    let status_str = match menu.status {
        shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending => "pending".to_string(),
        shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved => "approved".to_string(),
        shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected => "rejected".to_string(),
    };

    Ok(Json(crate::dto::moderation::MenuModerationResponseDto {
        id: menu.id,
        date: menu.serve_date.to_string(),
        meal_type: meal_type_str,
        status: status_str,
        source_type: menu.source_type,
        notice: menu.notice,
        bot_commentary: menu.bot_commentary,
        city: city.map(|c| crate::dto::moderation::MenuModerationCityDto { name: c.name }),
    }))
}

async fn approve_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let original_menu = Menus::find_by_id(menu_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?;
    let submitter_id = original_menu.submitted_by;
    let mut menu: menus::ActiveModel = original_menu.into();
    menu.status = Set(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved);
    menu.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    shared::services::immutable_store::ImmutableStore::write_menu_hash(&db, menu_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(sub_id) = submitter_id {
        let action_url = format!("/menu/{}", menu_id);
        let _ = crate::services::notification::NotificationService::send_notification(
            &db,
            sub_id,
            "moderation",
            "Menü Gönderin Onaylandı",
            "Gönderdiğin menü moderatörler tarafından incelendi ve yayına alındı.",
            Some("Menüyü Gör"),
            Some(&action_url),
        )
        .await;
    }

    Ok(Json(()))
}

async fn reject_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let original_menu = Menus::find_by_id(menu_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?;
    let submitter_id = original_menu.submitted_by;
    let mut menu: menus::ActiveModel = original_menu.into();
    menu.status = Set(shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected);
    menu.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(sub_id) = submitter_id {
        let _ = crate::services::notification::NotificationService::send_notification(
            &db,
            sub_id,
            "moderation",
            "Menü Gönderin Reddedildi",
            "Gönderdiğin menü inceleme sonucunda uygun bulunmadı.",
            None,
            None,
        )
        .await;
    }

    Ok(Json(()))
}

async fn bulk_update_menu_status(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<BulkUpdateMenuStatusDto>,
) -> Result<Json<BulkUpdateMenuStatusResponseDto>, AppError> {
    require_admin(&user)?;

    let target_status = match payload.status.to_lowercase().as_str() {
        "approved" => shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved,
        "rejected" => shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected,
        _ => {
            return Err(AppError::BadRequest(
                "Geçersiz menü durumu (yalnızca 'approved' veya 'rejected')".to_string(),
            ))
        }
    };

    let target_menus = Menus::find()
        .filter(menus::Column::Id.is_in(payload.menu_ids.clone()))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut updated_count = 0;
    for original_menu in target_menus {
        let menu_id = original_menu.id;
        let submitter_id = original_menu.submitted_by;
        let mut active: menus::ActiveModel = original_menu.into();
        active.status = Set(target_status.clone());
        active
            .update(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        updated_count += 1;

        if target_status == shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved {
            let _ =
                shared::services::immutable_store::ImmutableStore::write_menu_hash(&db, menu_id)
                    .await;
            if let Some(sub_id) = submitter_id {
                let action_url = format!("/menu/{}", menu_id);
                let _ = crate::services::notification::NotificationService::send_notification(
                    &db,
                    sub_id,
                    "moderation",
                    "Menü Gönderin Onaylandı",
                    "Gönderdiğin menü moderatörler tarafından incelendi ve yayına alındı.",
                    Some("Menüyü Gör"),
                    Some(&action_url),
                )
                .await;
            }
        } else if target_status == shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected
        {
            if let Some(sub_id) = submitter_id {
                let _ = crate::services::notification::NotificationService::send_notification(
                    &db,
                    sub_id,
                    "moderation",
                    "Menü Gönderin Reddedildi",
                    "Gönderdiğin menü inceleme sonucunda uygun bulunmadı.",
                    None,
                    None,
                )
                .await;
            }
        }
    }

    Ok(Json(BulkUpdateMenuStatusResponseDto { updated_count }))
}

async fn update_menu_commentary(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateMenuCommentaryDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let mut menu: menus::ActiveModel = Menus::find_by_id(menu_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?
        .into();
    let sanitized_commentary =
        shared::services::content_guard::ContentGuard::sanitize_html(&payload.content);
    menu.bot_commentary = Set(Some(sanitized_commentary));
    menu.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn update_menu_items(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateMenuItemsDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    ModerationService::update_menu_items(&db, menu_id, payload)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;

    let menu = Menus::find_by_id(menu_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(m) = menu {
        if m.status == shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved {
            shared::services::immutable_store::ImmutableStore::write_menu_hash(&db, menu_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    Ok(Json(()))
}

async fn get_menu_items(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
) -> Result<Json<Vec<crate::dto::moderation::MenuDishItemDto>>, AppError> {
    require_admin(&user)?;
    let menu_dishes = shared::entities::menu_dishes::Entity::find()
        .filter(shared::entities::menu_dishes::Column::MenuId.eq(menu_id))
        .order_by_asc(shared::entities::menu_dishes::Column::OrderIndex)
        .order_by_asc(shared::entities::menu_dishes::Column::IsAlternative)
        .find_also_related(shared::entities::dish_aliases::Entity)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut dish_ids = Vec::new();
    for (_, alias_opt) in &menu_dishes {
        if let Some(alias) = alias_opt {
            if let Some(id) = alias.dish_id {
                dish_ids.push(id);
            }
        }
    }

    let dishes = if !dish_ids.is_empty() {
        shared::entities::dishes::Entity::find()
            .filter(shared::entities::dishes::Column::Id.is_in(dish_ids))
            .all(&db)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    let dishes_map: std::collections::HashMap<i32, shared::entities::dishes::Model> =
        dishes.into_iter().map(|d| (d.id, d)).collect();

    let mut result = Vec::new();
    for (md, alias_opt) in menu_dishes {
        if let Some(alias) = alias_opt {
            if let Some(d_id) = alias.dish_id {
                if let Some(dish) = dishes_map.get(&d_id) {
                    result.push(crate::dto::moderation::MenuDishItemDto {
                        id: dish.id,
                        name: dish.name.clone(),
                        order_index: md.order_index,
                        is_alternative: md.is_alternative,
                        package_name: md.package_name,
                        category: dish.category.clone(),
                    });
                }
            }
        }
    }

    Ok(Json(result))
}

async fn get_pending_votes(
    State(_db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::VoteModerationResponseDto>>, AppError> {
    require_admin(&user)?;
    // With URM, all comments are published by default. We can return an empty list or unresolved reports here
    let result: Vec<crate::dto::moderation::VoteModerationResponseDto> = Vec::new();
    Ok(Json(result))
}

async fn get_all_votes(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Query(query): Query<crate::dto::pagination::PaginationQuery>,
) -> Result<
    Json<
        crate::dto::pagination::PaginatedResponse<
            crate::dto::moderation::VoteModerationResponseDto,
        >,
    >,
    AppError,
> {
    require_admin(&user)?;

    let limit = query.limit_num();
    let offset = query.offset();

    let total = Comments::find().count(&db).await.unwrap_or(0);

    let comments_list = Comments::find()
        .order_by_desc(comments::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for c in comments_list {
        let user_dto = if let Some(uid) = c.user_id {
            let u = Users::find_by_id(uid)
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            u.map(|user| crate::dto::moderation::VoteModerationUserDto {
                username: user.username,
            })
        } else {
            None
        };

        use shared::entities::sea_orm_active_enums::ReactionTypeEnum;
        use shared::entities::vote_reactions;
        let reactions = vote_reactions::Entity::find()
            .filter(vote_reactions::Column::CommentId.eq(c.id))
            .all(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let up = reactions
            .iter()
            .filter(|r| r.reaction_type == ReactionTypeEnum::Upvote)
            .count() as i32;
        let down = reactions
            .iter()
            .filter(|r| r.reaction_type == ReactionTypeEnum::Downvote)
            .count() as i32;

        let created_at_str = c.created_at.map(|dt| dt.to_rfc3339());

        let sentiment_str = match c.sentiment {
            shared::entities::sea_orm_active_enums::SentimentEnum::Positive => {
                "positive".to_string()
            }
            shared::entities::sea_orm_active_enums::SentimentEnum::Negative => {
                "negative".to_string()
            }
            shared::entities::sea_orm_active_enums::SentimentEnum::Neutral => "neutral".to_string(),
        };

        result.push(crate::dto::moderation::VoteModerationResponseDto {
            id: c.id,
            comment: c.content.unwrap_or_default(),
            is_deleted: c.is_deleted,
            user: user_dto,
            created_at: created_at_str,
            reaction_summary: crate::dto::moderation::VoteModerationReactionSummaryDto { up, down },
            status: "published".to_string(),
            sentiment: sentiment_str,
        });
    }

    let total_pages = ((total as f64) / (limit as f64)).ceil().max(1.0) as u64;
    let current_page = (offset / limit) + 1;

    Ok(Json(crate::dto::pagination::PaginatedResponse {
        items: result,
        total_items: total,
        total_pages,
        current_page,
    }))
}

async fn get_complaints(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::ReportModerationResponseDto>>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use shared::entities::reports;

    let pending_reports = Reports::find()
        .filter(
            reports::Column::Status
                .eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending),
        )
        .filter(reports::Column::ReportedCommentId.is_not_null())
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for r in pending_reports {
        let comment_id = r.reported_comment_id.unwrap();

        let comment_opt = Comments::find_by_id(comment_id)
            .one(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let comment_content = comment_opt.as_ref().and_then(|c| c.content.clone());
        let comment_author_id = comment_opt.as_ref().and_then(|c| c.user_id);

        let author_username = if let Some(author_uid) = comment_author_id {
            Users::find_by_id(author_uid)
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .map(|u| u.username)
        } else {
            None
        };

        let reporter_username = if let Some(rid) = r.reporter_id {
            Users::find_by_id(rid)
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .map(|u| u.username)
        } else {
            None
        };

        let created_at_str = r.created_at.map(|dt| dt.to_rfc3339());

        result.push(crate::dto::moderation::ReportModerationResponseDto {
            id: comment_id,
            reason: r.reason.clone(),
            reported_comment_id: Some(comment_id),
            status: "pending".to_string(),
            comment: comment_content,
            author_id: author_username.or_else(|| Some("Anonim".to_string())),
            user_id: reporter_username.or_else(|| Some("Kullanıcı".to_string())),
            created_at: created_at_str,
            tags: r.reason.clone(),
            report_count: Some(1),
        });
    }

    Ok(Json(result))
}

async fn approve_vote(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(vote_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    use shared::entities::reports;

    let comment = Comments::find_by_id(vote_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    let mut active_comment: comments::ActiveModel = comment.into();
    active_comment.is_deleted = Set(false);
    active_comment
        .update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .filter(
            reports::Column::Status
                .eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending),
        )
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status =
            Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Dismissed);
        active_report.resolved_at = Set(Some(chrono::Utc::now().into()));
        active_report
            .update(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(()))
}

async fn reject_vote(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(vote_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    use shared::entities::reports;

    crate::services::reaction::ReactionService::delete_comment(&db, user.id, &user.role, vote_id)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;

    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .filter(
            reports::Column::Status
                .eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending),
        )
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status =
            Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Resolved);
        active_report.resolved_at = Set(Some(chrono::Utc::now().into()));
        active_report
            .update(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(()))
}

async fn reset_vote(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(vote_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    use shared::entities::reports;

    let comment = Comments::find_by_id(vote_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    let mut active_comment: comments::ActiveModel = comment.into();
    active_comment.is_deleted = Set(false);
    active_comment
        .update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status =
            Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending);
        active_report
            .update(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(()))
}

async fn purge_vote(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(vote_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use shared::entities::{reports, vote_reactions};

    vote_reactions::Entity::delete_many()
        .filter(vote_reactions::Column::CommentId.eq(vote_id))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    reports::Entity::delete_many()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Comments::delete_by_id(vote_id)
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(()))
}

async fn get_users(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::UserModerationResponseDto>>, AppError> {
    require_admin(&user)?;
    let users_list = Users::find()
        .order_by_desc(users::Column::CreatedAt)
        .limit(50)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for u in users_list {
        let is_admin = u.role == shared::entities::sea_orm_active_enums::UserRoleEnum::Admin;
        let is_banned =
            u.account_status == shared::entities::sea_orm_active_enums::AccountStatusEnum::Banned;
        let created_at_time = u
            .created_at
            .unwrap_or_else(|| chrono::Utc::now().into())
            .into();
        result.push(crate::dto::moderation::UserModerationResponseDto {
            id: u.id,
            username: u.username,
            email: u.email,
            role: match u.role {
                shared::entities::sea_orm_active_enums::UserRoleEnum::Admin => "admin".to_string(),
                shared::entities::sea_orm_active_enums::UserRoleEnum::User => "user".to_string(),
                _ => "unknown".to_string(),
            },
            status: match u.account_status {
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Active => {
                    "active".to_string()
                }
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Suspended => {
                    "suspended".to_string()
                }
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Banned => {
                    "banned".to_string()
                }
            },
            is_admin,
            is_verified: u.is_verified,
            is_banned,
            created_at: created_at_time,
            avatar_url: u.avatar_url,
        });
    }
    Ok(Json(result))
}

async fn get_tags(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::TagResponseDto>>, AppError> {
    require_admin(&user)?;
    let tags_list = Tags::find()
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for t in tags_list {
        result.push(crate::dto::moderation::TagResponseDto {
            id: t.id,
            name: t.name,
            category: t.category,
        });
    }
    Ok(Json(result))
}

async fn create_tag(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::CreateTagDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let tag = tags::ActiveModel {
        name: Set(payload.name),
        category: Set(payload.category),
        sort_order: Set(payload.sort_order),
        ..Default::default()
    };
    tag.insert(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn update_tag(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(tag_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::CreateTagDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let mut tag: tags::ActiveModel = Tags::find_by_id(tag_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Tag not found".into()))?
        .into();
    tag.name = Set(payload.name);
    tag.category = Set(payload.category);
    tag.sort_order = Set(payload.sort_order);
    tag.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn delete_tag(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(tag_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    Tags::delete_by_id(tag_id)
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn get_incidents(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::IncidentAdminDto>>, AppError> {
    require_admin(&user)?;
    let incidents = shared::entities::system_incidents::Entity::find()
        .order_by_desc(shared::entities::system_incidents::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = incidents
        .into_iter()
        .map(|i| crate::dto::moderation::IncidentAdminDto {
            id: i.id,
            component: i.component,
            title: i.title,
            message: i.message,
            status: i.status,
            impact: i.impact,
            created_at: i.created_at.map(|d| d.to_rfc3339()),
            resolved_at: i.resolved_at.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(Json(result))
}

async fn create_incident(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::CreateIncidentDto>,
) -> Result<Json<i32>, AppError> {
    require_admin(&user)?;
    let id = ModerationService::create_incident(&db, payload)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(id))
}

async fn update_incident(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(incident_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateIncidentDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    ModerationService::update_incident(&db, incident_id, payload)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

async fn delete_incident(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(incident_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    ModerationService::delete_incident(&db, incident_id)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

#[derive(Debug, serde::Deserialize)]
pub struct SubmissionsQuery {
    pub status: Option<String>,
}

async fn get_submissions(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Query(params): Query<SubmissionsQuery>,
) -> Result<Json<Vec<SubmissionItemDto>>, AppError> {
    require_admin(&user)?;
    let items = ModerationService::get_submissions(&db, params.status.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(items))
}

async fn update_submission_status(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(submission_id): Path<i32>,
    ValidatedJson(dto): ValidatedJson<UpdateSubmissionStatusDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let updated = ModerationService::update_submission_status(&db, submission_id, &dto.status)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(serde_json::json!({
        "success": true,
        "id": updated.id,
        "status": updated.status
    })))
}

#[derive(serde::Deserialize)]
pub struct KitchenCoverageQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
}

async fn get_kitchen_coverage(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Query(query): Query<KitchenCoverageQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;

    let now = chrono::Utc::now();
    let year = query
        .year
        .unwrap_or_else(|| now.format("%Y").to_string().parse().unwrap_or(2026));
    let month = query
        .month
        .unwrap_or_else(|| now.format("%m").to_string().parse().unwrap_or(9));

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    use sea_orm::{ConnectionTrait, Statement};
    let sql = r#"
        SELECT 
            c.id AS city_id,
            c.name AS city_name,
            c.slug AS city_slug,
            m.id AS menu_id,
            TO_CHAR(m.date, 'YYYY-MM-DD') AS menu_date,
            EXTRACT(DAY FROM m.date)::int AS day_num,
            m.meal_type::text AS meal_type,
            m.is_approved AS is_approved,
            m.source AS source,
            (m.bot_commentary IS NOT NULL AND m.bot_commentary != '') AS has_bot_commentary,
            (SELECT COUNT(*) FROM menu_dishes md WHERE md.menu_id = m.id)::int AS dish_count
        FROM cities c
        LEFT JOIN menus m ON m.city_id = c.id 
            AND EXTRACT(YEAR FROM m.date) = $1 
            AND EXTRACT(MONTH FROM m.date) = $2
        ORDER BY c.id ASC, m.date ASC;
    "#;

    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        vec![year.into(), month.into()],
    );

    let rows = db
        .query_all(stmt)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    use std::collections::BTreeMap;
    struct CityItem {
        id: i32,
        name: String,
        slug: String,
        days: BTreeMap<i32, serde_json::Map<String, serde_json::Value>>,
    }

    let mut cities_map: BTreeMap<i32, CityItem> = BTreeMap::new();

    for row in rows {
        let city_id: i32 = row.try_get("", "city_id").unwrap_or(0);
        let city_name: String = row.try_get("", "city_name").unwrap_or_default();
        let city_slug: String = row.try_get("", "city_slug").unwrap_or_default();

        let entry = cities_map.entry(city_id).or_insert_with(|| CityItem {
            id: city_id,
            name: city_name,
            slug: city_slug,
            days: BTreeMap::new(),
        });

        if let Ok(Some(d)) = row.try_get::<Option<i32>>("", "day_num") {
            let menu_id: Option<i32> = row.try_get("", "menu_id").ok();
            let meal_type: Option<String> = row.try_get("", "meal_type").ok();
            let is_approved: Option<bool> = row.try_get("", "is_approved").ok();
            let source: Option<String> = row.try_get("", "source").ok();
            let has_bot_commentary: Option<bool> = row.try_get("", "has_bot_commentary").ok();
            let dish_count: Option<i32> = row.try_get("", "dish_count").ok();

            if let Some(m_type) = meal_type {
                let day_entry = entry.days.entry(d).or_default();
                day_entry.insert(
                    m_type.to_lowercase(),
                    serde_json::json!({
                        "menu_id": menu_id,
                        "is_approved": is_approved.unwrap_or(false),
                        "source": source,
                        "has_bot_commentary": has_bot_commentary.unwrap_or(false),
                        "dish_count": dish_count.unwrap_or(0),
                    }),
                );
            }
        }
    }

    let cities_list: Vec<serde_json::Value> = cities_map
        .into_values()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "slug": c.slug,
                "days": c.days,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "year": year,
        "month": month,
        "days_in_month": days_in_month,
        "cities": cities_list,
    })))
}

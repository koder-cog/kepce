// Kepçe API - Routes: Moderasyon Endpoint'leri
// ==============================================
//
// İnce zarf. ModerationService kullanır.
// Tüm endpoint'ler kimlik doğrulaması gerektirir (AuthenticatedUser).
//
use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{State, Path, Query},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::services::moderation::ModerationService;
use crate::services::bot::{BotService, BotError};
use crate::dto::moderation::{ReportCommentRequestDto, BotGenerateRequestDto, BotGenerateResponseDto, UpdateUserStatusDto, ResolveReportDto, WarnUserDto, BotExportMonthlyQuery, BotExportMonthlyResponseDto, InjectBotCommentsDto, InjectBotCommentsResponseDto};
use crate::dto::user::UserRole;
use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;
use crate::extractors::validated::ValidatedJson;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
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
        .route("/menus", get(get_menus))
        .route("/:menu_id/approve", post(approve_menu))
        .route("/:menu_id/reject", post(reject_menu))
        .route("/menus/:menu_id/commentary", put(update_menu_commentary))
        .route("/:menu_id/items", get(get_menu_items).put(update_menu_items))
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
        .route("/incidents", get(get_incidents).post(create_incident))
        .route("/incidents/:incident_id", put(update_incident).delete(delete_incident))
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
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }

    // API anahtarı yapılandırılmış mı?
    let api_key = config.gemini_api_key.as_deref()
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
    ).await?;

    Ok(Json(BotGenerateResponseDto { generated_comment: generated }))
}

/// Admin-only: Aylık menüyü bot girdisi (prompt) + şema olarak dışa aktarır.
async fn export_monthly_menu_for_bot(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<Arc<crate::config::Config>>,
    user: AuthenticatedUser,
    Query(query): Query<BotExportMonthlyQuery>,
) -> Result<Json<BotExportMonthlyResponseDto>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }

    let menu_text = ModerationService::export_monthly_menu_for_bot(&db, &query.city_slug, &query.month).await?;

    let schema: serde_json::Value = serde_json::from_str(BotService::BOT_OUTPUT_SCHEMA)
        .map_err(|e| AppError::Internal(format!("Bot şeması çözümlenemedi: {}", e)))?;

    let instruction = "\n\nYukarıdaki aylık menü verisini kullanarak HER GÜN için, yönergelerdeki Kepçe Bot kişiliğine uygun tek bir yorum üret. Çıktını belirtilen JSON şemasına göre formatla: tarih alanı ISO 8601 (YYYY-MM-DD), yorum alanı ham Türkçe metin (Markdown/madde işareti yok).";

    let prompt = format!("{}{}{}", config.bot_directive, instruction, menu_text);

    Ok(Json(BotExportMonthlyResponseDto { prompt, schema }))
}

/// Admin-only: Bot yorumlarını menülere yazar (gün bazlı, tüm öğünler).
async fn inject_bot_comments(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<InjectBotCommentsDto>,
) -> Result<Json<InjectBotCommentsResponseDto>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }

    let updated = ModerationService::inject_bot_comments(&db, &payload.city_slug, &payload.comments).await?;

    Ok(Json(InjectBotCommentsResponseDto { updated_count: updated }))
}

async fn update_user_status(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserStatusDto>,
) -> Result<Json<()>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
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
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }
    ModerationService::update_user(&db, user_id, payload.is_verified, payload.is_admin, payload.is_banned)
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
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
    }
    ModerationService::resolve_report(&db, report_id, payload.action_taken)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
    Ok(Json(()))
}

// --- IMPLEMENTED MODERATION ENDPOINTS ---

use shared::entities::{prelude::*, menus, comments, users, tags};
use sea_orm::*; use sea_orm::QueryOrder;

fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Bu işlem yalnızca yöneticilere açıktır".to_string()));
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
    
    use sea_orm::{ActiveValue::Set, ActiveModelTrait};
    use shared::entities::user_warnings;
    
    let warning = user_warnings::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        user_id: Set(user_id),
        message: Set(payload.message.clone()),
        created_at: Set(Some(chrono::Utc::now().into())),
    };
    
    warning.insert(&db)
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
    ).await;

    Ok(Json(()))
}

async fn get_pending_menus(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<crate::dto::moderation::MenuModerationResponseDto>>, AppError> {
    require_admin(&user)?;
    let menus_with_cities = Menus::find()
        .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending))
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
                shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner".to_string(),
            },
            status: "pending".to_string(),
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
            if month.len() == 7 { // YYYY-MM
                let parts: Vec<&str> = month.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(y), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                        let next_y = if m == 12 { y + 1 } else { y };
                        let next_m = if m == 12 { 1 } else { m + 1 };
                        
                        if let (Some(start_date), Some(end_date)) = (
                            chrono::NaiveDate::from_ymd_opt(y, m, 1),
                            chrono::NaiveDate::from_ymd_opt(next_y, next_m, 1)
                        ) {
                            condition = condition.add(menus::Column::ServeDate.gte(start_date));
                            condition = condition.add(menus::Column::ServeDate.lt(end_date));
                        }
                    }
                }
            } else if month.len() == 4 { // YYYY
                if let Ok(y) = month.parse::<i32>() {
                    if let (Some(start_date), Some(end_date)) = (
                        chrono::NaiveDate::from_ymd_opt(y, 1, 1),
                        chrono::NaiveDate::from_ymd_opt(y + 1, 1, 1)
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
                shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch".to_string(),
                shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner".to_string(),
            },
            status: match m.status {
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending => "pending".to_string(),
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved => "approved".to_string(),
                shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected => "rejected".to_string(),
            },
            bot_commentary: m.bot_commentary,
            city: city_opt.map(|c| crate::dto::moderation::MenuModerationCityDto { name: c.name }),
        });
    }
    Ok(Json(result))
}

async fn approve_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let original_menu = Menus::find_by_id(menu_id)
        .one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?;
    let submitter_id = original_menu.submitted_by;
    let mut menu: menus::ActiveModel = original_menu.into();
    menu.status = Set(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved);
    menu.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;

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
        ).await;
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
        .one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?;
    let submitter_id = original_menu.submitted_by;
    let mut menu: menus::ActiveModel = original_menu.into();
    menu.status = Set(shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected);
    menu.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(sub_id) = submitter_id {
        let _ = crate::services::notification::NotificationService::send_notification(
            &db,
            sub_id,
            "moderation",
            "Menü Gönderin Reddedildi",
            "Gönderdiğin menü inceleme sonucunda uygun bulunmadı.",
            None,
            None,
        ).await;
    }

    Ok(Json(()))
}

async fn update_menu_commentary(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateMenuCommentaryDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    let mut menu: menus::ActiveModel = Menus::find_by_id(menu_id)
        .one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Menu not found".into()))?.into();
    let sanitized_commentary = shared::services::content_guard::ContentGuard::sanitize_html(&payload.content);
    menu.bot_commentary = Set(Some(sanitized_commentary));
    menu.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn update_menu_items(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(menu_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::dto::moderation::UpdateMenuItemsDto>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    ModerationService::update_menu_items(&db, menu_id, payload.dish_ids)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;

    let menu = Menus::find_by_id(menu_id)
        .one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
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
    
    let mut result = Vec::new();
    for dish in dishes {
        result.push(crate::dto::moderation::MenuDishItemDto {
            id: dish.id,
            name: dish.name.clone(),
        });
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
) -> Result<Json<crate::dto::pagination::PaginatedResponse<crate::dto::moderation::VoteModerationResponseDto>>, AppError> {
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
            let u = Users::find_by_id(uid).one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
            u.map(|user| crate::dto::moderation::VoteModerationUserDto {
                username: user.username,
            })
        } else {
            None
        };

        use shared::entities::vote_reactions;
        use shared::entities::sea_orm_active_enums::ReactionTypeEnum;
        let reactions = vote_reactions::Entity::find()
            .filter(vote_reactions::Column::CommentId.eq(c.id))
            .all(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
            
        let up = reactions.iter().filter(|r| r.reaction_type == ReactionTypeEnum::Upvote).count() as i32;
        let down = reactions.iter().filter(|r| r.reaction_type == ReactionTypeEnum::Downvote).count() as i32;

        let created_at_str = c.created_at.map(|dt| dt.to_rfc3339());
        
        let sentiment_str = match c.sentiment {
            shared::entities::sea_orm_active_enums::SentimentEnum::Positive => "positive".to_string(),
            shared::entities::sea_orm_active_enums::SentimentEnum::Negative => "negative".to_string(),
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
    
    use sea_orm::{QueryFilter, ColumnTrait, EntityTrait};
    use shared::entities::reports;
    
    let pending_reports = Reports::find()
        .filter(reports::Column::Status.eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending))
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
        
        let reporter_username = Users::find_by_id(r.reporter_id)
            .one(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .map(|u| u.username);
            
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
    
    use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter, EntityTrait};
    use shared::entities::reports;
    
    let comment = Comments::find_by_id(vote_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
        
    let mut active_comment: comments::ActiveModel = comment.into();
    active_comment.is_deleted = Set(false);
    active_comment.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .filter(reports::Column::Status.eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status = Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Dismissed);
        active_report.resolved_at = Set(Some(chrono::Utc::now().into()));
        active_report.update(&db)
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
    
    use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter, EntityTrait};
    use shared::entities::reports;
    
    crate::services::reaction::ReactionService::delete_comment(&db, user.id, &user.role, vote_id)
        .await
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;
        
    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .filter(reports::Column::Status.eq(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status = Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Resolved);
        active_report.resolved_at = Set(Some(chrono::Utc::now().into()));
        active_report.update(&db)
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
    
    use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter, EntityTrait};
    use shared::entities::reports;
    
    let comment = Comments::find_by_id(vote_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
        
    let mut active_comment: comments::ActiveModel = comment.into();
    active_comment.is_deleted = Set(false);
    active_comment.update(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    let comment_reports = Reports::find()
        .filter(reports::Column::ReportedCommentId.eq(vote_id))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    for r in comment_reports {
        let mut active_report: reports::ActiveModel = r.into();
        active_report.status = Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending);
        active_report.update(&db)
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
    
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
    use shared::entities::{vote_reactions, reports};
    
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
        let is_banned = u.account_status == shared::entities::sea_orm_active_enums::AccountStatusEnum::Banned;
        let created_at_time = u.created_at.unwrap_or_else(|| chrono::Utc::now().into()).into();
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
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Active => "active".to_string(),
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Suspended => "suspended".to_string(),
                shared::entities::sea_orm_active_enums::AccountStatusEnum::Banned => "banned".to_string(),
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
    tag.insert(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
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
        .one(&db).await.map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("Tag not found".into()))?.into();
    tag.name = Set(payload.name);
    tag.category = Set(payload.category);
    tag.sort_order = Set(payload.sort_order);
    tag.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(()))
}

async fn delete_tag(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(tag_id): Path<i32>,
) -> Result<Json<()>, AppError> {
    require_admin(&user)?;
    Tags::delete_by_id(tag_id).exec(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
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
        
    let result = incidents.into_iter().map(|i| crate::dto::moderation::IncidentAdminDto {
        id: i.id,
        component: i.component,
        title: i.title,
        message: i.message,
        status: i.status,
        impact: i.impact,
        created_at: i.created_at.map(|d| d.to_rfc3339()),
        resolved_at: i.resolved_at.map(|d| d.to_rfc3339()),
    }).collect();
    
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

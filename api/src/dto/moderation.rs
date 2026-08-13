use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct GetMenusQuery {
    pub status: Option<String>,
    pub city_slug: Option<String>,
    pub month: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReportCommentRequestDto {
    #[validate(length(min = 1, max = 500, message = "Şikayet nedeni 1 ile 500 karakter arasında olmalıdır"))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BlockUserDto {
    pub blocked_user_id: Uuid,
}

/// Moderasyon panelinden tetiklenen AI yorum üretme isteği
#[derive(Debug, Deserialize, Validate)]
pub struct BotGenerateRequestDto {
    #[validate(length(min = 1, max = 200, message = "Yemek adı 1 ile 200 karakter arasında olmalıdır"))]
    pub dish_name: String,
    #[validate(length(min = 1, max = 20, message = "Sentiment geçersiz"))]
    pub sentiment: String,
}

/// Bot yanıtı
#[derive(Serialize)]
pub struct BotGenerateResponseDto {
    pub generated_comment: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMenuCommentaryDto {
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MenuDishItemDto {
    pub id: i32,
    pub name: String,
}

// TODO: Wire up in update_menu_items handler when menu-item editing is implemented
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMenuItemsDto {
    pub dish_ids: Vec<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    pub is_verified: Option<bool>,
    pub is_admin: Option<bool>,
    pub is_banned: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserStatusDto {
    pub status: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResolveReportDto {
    #[validate(length(min = 1, max = 255))]
    pub action_taken: String,
}

#[derive(Debug, Serialize)]
pub struct MenuModerationCityDto {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MenuModerationResponseDto {
    pub id: i32,
    pub date: String,
    pub meal_type: String,
    pub status: String,
    pub bot_commentary: Option<String>,
    pub city: Option<MenuModerationCityDto>,
}

#[derive(Debug, Serialize)]
pub struct VoteModerationUserDto {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct VoteModerationReactionSummaryDto {
    pub up: i32,
    pub down: i32,
}

#[derive(Debug, Serialize)]
pub struct VoteModerationResponseDto {
    pub id: Uuid,
    pub comment: String,
    pub is_deleted: bool,
    pub user: Option<VoteModerationUserDto>,
    pub created_at: Option<String>,
    pub reaction_summary: VoteModerationReactionSummaryDto,
    pub status: String,
    pub sentiment: String,
}

#[derive(Debug, Serialize)]
pub struct ReportModerationResponseDto {
    pub id: Uuid,
    pub reason: Option<String>,
    pub reported_comment_id: Option<Uuid>,
    pub status: String,
    pub comment: Option<String>,
    pub author_id: Option<String>,
    pub user_id: Option<String>,
    pub created_at: Option<String>,
    pub tags: Option<String>,
    pub report_count: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UserModerationResponseDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub is_admin: bool,
    pub is_verified: bool,
    pub is_banned: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TagResponseDto {
    pub id: i32,
    pub name: String,
    pub category: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateIncidentDto {
    #[validate(length(min = 1, max = 100))]
    pub component: String,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub message: String,
    #[validate(length(min = 1, max = 50))]
    pub impact: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateIncidentDto {
    #[validate(length(min = 1, max = 50))]
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct IncidentAdminDto {
    pub id: i32,
    pub component: String,
    pub title: String,
    pub message: String,
    pub status: String,
    pub impact: String,
    pub created_at: Option<String>,
    pub resolved_at: Option<String>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct WarnUserDto {
    #[validate(length(min = 1, max = 1000, message = "Uyarı mesajı 1 ile 1000 karakter arasında olmalıdır"))]
    pub message: String,
}

/// GET /moderation/bot/export-monthly sorgu parametreleri
#[derive(Debug, Deserialize)]
pub struct BotExportMonthlyQuery {
    pub city_slug: String,
    pub month: String,
}

/// export-monthly yanıtı
#[derive(Debug, Serialize)]
pub struct BotExportMonthlyResponseDto {
    pub prompt: String,
    pub schema: serde_json::Value,
}

/// POST /moderation/bot/inject isteği
#[derive(Debug, Deserialize, Validate)]
pub struct InjectBotCommentsDto {
    #[validate(length(min = 1, message = "Şehir slug boş olamaz"))]
    pub city_slug: String,
    pub comments: Vec<InjectBotCommentEntryDto>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct InjectBotCommentEntryDto {
    #[validate(length(min = 1, message = "Tarih boş olamaz"))]
    pub date: String,
    #[validate(length(min = 1, max = 2000, message = "Yorum 1-2000 karakter arası olmalı"))]
    pub commentary: String,
}

/// inject yanıtı
#[derive(Debug, Serialize)]
pub struct InjectBotCommentsResponseDto {
    pub updated_count: usize,
}

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct NotificationDto {
    pub id: i32,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub action_label: Option<String>,
    pub action_href: Option<String>,
    pub created_at: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, validator::Validate)]
pub struct NotificationMarkReadDto {
    pub id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
    SystemBot,
}

/// Sadece giriş yaparken (Login) kullanılır.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequestDto {
    #[validate(length(min = 3, message = "Kullanıcı adı en az 3 karakter olmalıdır"))]
    pub identifier: String,

    #[validate(length(min = 1, max = 128, message = "Lütfen şifrenizi giriniz"))]
    pub password: String,

    pub remember: Option<bool>,
}

/// Boş string ("" veya "   ") geldiğinde None olarak ayrıştıran özel deserializer
pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.and_then(|s| {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }))
}

/// Yeni kayıt (Register) isteği.
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequestDto {
    #[serde(alias = "username")]
    #[validate(length(
        min = 3,
        max = 25,
        message = "Kullanıcı adı 3-25 karakter arasında olmalıdır"
    ))]
    pub nickname: String,

    #[validate(length(min = 8, max = 72, message = "Şifre 8-72 karakter arasında olmalıdır"))]
    pub password: String,

    pub default_city_slug: Option<String>,
    pub diet_mode: Option<String>,
}

/// Kurtarma anahtarı aktif hatırlama doğrulama isteği
#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmRecoveryDto {
    #[validate(length(
        min = 5,
        max = 5,
        message = "Kurtarma kodunun son 5 karakteri girilmelidir"
    ))]
    pub last_block: String,
}

/// Onaysız hesaplar için şifre teyidiyle yeni kurtarma anahtarı üretme isteği
#[derive(Debug, Deserialize, Validate)]
pub struct RegenerateRecoveryDto {
    #[validate(length(min = 1, max = 128, message = "Lütfen mevcut şifrenizi giriniz"))]
    pub password: String,
}

/// Kurtarma anahtarı ile şifre sıfırlama isteği
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordWithRecoveryDto {
    #[validate(length(min = 3, max = 50, message = "Geçersiz kullanıcı adı"))]
    pub nickname: String,

    #[validate(length(min = 20, max = 40, message = "Geçersiz kurtarma anahtarı formatı"))]
    pub recovery_key: String,

    #[validate(length(
        min = 8,
        max = 72,
        message = "Yeni şifre 8-72 karakter arasında olmalıdır"
    ))]
    pub new_password: String,
}

/// Dışarıya (Frontend'e) verilecek güvenli kullanıcı profili
#[derive(Debug, Serialize, Clone)]
pub struct LevelProgressDto {
    pub level: i32,
    pub title: String,
    pub progress_percent: i32,
    pub karma_in_level: i32,
    pub karma_for_next: i32,
}

#[derive(Debug, Serialize)]
pub struct UserProfileDto {
    pub id: Uuid,

    pub nickname: String,
    // Geriye dönük frontend bileşen uyumluluğu için alias:
    pub username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserRole>,

    pub karma_score: i32,
    pub is_verified: bool,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub avatar_url: Option<String>,
    pub default_city_slug: Option<String>,
    pub level: i32,
    pub level_progress: LevelProgressDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    pub badge_count: i32,
    pub total_badges: i32,

    pub badges: Vec<UserBadgeDto>,
    pub pinned_badges: Vec<String>,
    pub opt_out_statistics: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_replies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_interactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_system: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_breakfast_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_breakfast_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_dinner_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notif_dinner_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blocked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blocked_by: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserBadgeDto {
    pub slug: String,
    pub name: String,
    pub icon: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub category: String,
    pub tier: String,
    pub is_hidden: bool,
    pub awarded_at: Option<DateTime<Utc>>,
    pub unlocked: bool,
    pub karma_reward: i32,
    pub count: i32,
    pub is_repeatable: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePinnedBadgesDto {
    #[validate(length(max = 5, message = "En fazla 5 rozet sabitleyebilirsiniz"))]
    pub pinned_badges: Vec<String>,
}

/// Kimlik doğrulama yanıtı: JWT token + kullanıcı profili + açık kurtarma kodu (varsa)
#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub user: UserProfileDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SimpleDishDto {
    pub dish_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct FavoriteAuthorDto {
    pub username: String,
    pub favorite_count: i32,
    pub avatar_url: Option<String>,
    pub level: Option<i32>,
    pub karma_score: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UserDashboardStatsDto {
    pub favorite_meals: Vec<SimpleDishDto>,
    pub pinned_meals: Vec<SimpleDishDto>,
    pub favorite_comments: Vec<crate::dto::comment::CommentResponseDto>,
    pub favorite_authors: Vec<FavoriteAuthorDto>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateProfileDto {
    #[serde(default, alias = "username", deserialize_with = "empty_string_as_none")]
    #[validate(length(
        min = 3,
        max = 25,
        message = "Kullanıcı adı 3-25 karakter arasında olmalıdır"
    ))]
    pub nickname: Option<String>,

    pub current_password: Option<String>,

    #[validate(length(
        min = 8,
        max = 72,
        message = "Yeni şifre 8-72 karakter arasında olmalıdır"
    ))]
    pub password: Option<String>,

    pub default_city_slug: Option<String>,
    pub opt_out_statistics: Option<bool>,
    pub notif_replies: Option<bool>,
    pub notif_interactions: Option<bool>,
    pub notif_system: Option<bool>,
    pub notif_breakfast_enabled: Option<bool>,
    #[validate(custom(function = "validate_time_format"))]
    pub notif_breakfast_time: Option<String>,
    pub notif_dinner_enabled: Option<bool>,
    #[validate(custom(function = "validate_time_format"))]
    pub notif_dinner_time: Option<String>,
}

fn validate_time_format(time_str: &str) -> Result<(), validator::ValidationError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(validator::ValidationError::new("invalid_time_format"));
    }
    if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
        && h < 24
        && m < 60
        && parts[0].len() == 2
        && parts[1].len() == 2
    {
        return Ok(());
    }
    Err(validator::ValidationError::new("invalid_time_format"))
}

/// Hesap silme isteği - step-up auth: mevcut şifre zorunlu (SA-12)
#[derive(Debug, Deserialize, validator::Validate)]
pub struct DeleteAccountDto {
    #[validate(length(min = 1, max = 128, message = "Lütfen şifrenizi giriniz"))]
    pub password: String,
}

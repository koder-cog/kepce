use chrono::{DateTime, Utc, FixedOffset};
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
    // E-posta veya Kullanıcı Adı girilebilir. Bu yüzden tipini identifier koyduk ve email validasyonunu kaldırdık.
    #[validate(length(min = 3, message = "Kullanıcı adı veya e-posta en az 3 karakter olmalıdır"))]
    pub identifier: String,
    
    #[validate(length(min = 1, max = 128, message = "Lütfen şifrenizi giriniz"))]
    pub password: String,

    pub remember: Option<bool>,
}

/// Şifresiz giriş linki istemek için
#[derive(Debug, Deserialize, Validate)]
pub struct PasswordlessRequestDto {
    #[validate(email(message = "Geçerli bir e-posta adresi giriniz"))]
    pub email: String,
}

/// E-postaya gelen token ile giriş yapmak için
#[derive(Debug, Deserialize, Validate)]
pub struct PasswordlessLoginDto {
    pub token: String,
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

/// Sadece yeni kayıt olurken (Register) kullanılır.
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequestDto {
    // Kullanıcı adı artık opsiyonel. Girilmezse AuthService e-postadan türetecek.
    #[serde(default, deserialize_with = "empty_string_as_none")]
    #[validate(length(min = 3, max = 30, message = "Kullanıcı adı 3-30 karakter arasında olmalıdır"))]
    pub username: Option<String>,

    #[validate(email(message = "Geçerli bir e-posta adresi giriniz"))]
    pub email: String,
    
    #[validate(length(min = 8, max = 128, message = "Şifre 8-128 karakter arasında olmalıdır"))]
    pub password: String,
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
    
    pub username: String,
    
    // SA-13: Rol yalnızca kullanıcının KENDİ profilinde (include_private) döner.
    // Public profillerde admin hesaplarının enumeration'ını önlemek için gizlenir.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserRole>,
    
    pub karma_score: i32,
    
    pub is_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub default_city_slug: Option<String>,
    pub level: i32,
    pub level_progress: LevelProgressDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    pub badge_count: i32,
    pub total_badges: i32,
    
    // Kullanıcının kazandığı rozetlerin listesi
    pub badges: Vec<UserBadgeDto>,
    pub opt_out_statistics: bool,
}

#[derive(Debug, Serialize)]
pub struct UserBadgeDto {
    pub name: String,
    pub icon: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub category: String,
    pub awarded_at: Option<DateTime<Utc>>,
    pub unlocked: bool,
    pub karma_reward: i32,
    pub count: i32,
    pub is_repeatable: bool,
}

/// Kimlik doğrulama yanıtı: JWT token + kullanıcı profili
#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub user: UserProfileDto,
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
    #[serde(default, deserialize_with = "empty_string_as_none")]
    #[validate(length(min = 3, max = 30, message = "Kullanıcı adı 3-30 karakter arasında olmalıdır"))]
    pub username: Option<String>,
    
    #[validate(email(message = "Geçerli bir e-posta adresi giriniz"))]
    pub email: Option<String>,
    
    pub current_password: Option<String>,
    
    #[validate(length(min = 8, max = 128, message = "Yeni şifre 8-128 karakter arasında olmalıdır"))]
    pub password: Option<String>,

    pub bio: Option<String>,
    pub default_city_slug: Option<String>,
    pub opt_out_statistics: Option<bool>,
}

/// Hesap silme isteği — step-up auth: mevcut şifre zorunlu (SA-12)
#[derive(Debug, Deserialize, validator::Validate)]
pub struct DeleteAccountDto {
    #[validate(length(min = 1, max = 128, message = "Lütfen şifrenizi giriniz"))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_register_request_empty_username_deserializes_to_none() {
        let json_empty = r#"{"email":"test@example.com","password":"password123","username":""}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json_empty).unwrap();
        assert_eq!(dto.username, None);
        assert!(dto.validate().is_ok());

        let json_whitespace = r#"{"email":"test@example.com","password":"password123","username":"   "}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json_whitespace).unwrap();
        assert_eq!(dto.username, None);
        assert!(dto.validate().is_ok());

        let json_null = r#"{"email":"test@example.com","password":"password123","username":null}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json_null).unwrap();
        assert_eq!(dto.username, None);
        assert!(dto.validate().is_ok());

        let json_omitted = r#"{"email":"test@example.com","password":"password123"}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json_omitted).unwrap();
        assert_eq!(dto.username, None);
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_register_request_valid_username() {
        let json = r#"{"email":"test@example.com","password":"password123","username":"ahmet"}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.username, Some("ahmet".to_string()));
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_register_request_short_username_fails_validation() {
        let json = r#"{"email":"test@example.com","password":"password123","username":"ab"}"#;
        let dto: RegisterRequestDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.username, Some("ab".to_string()));
        assert!(dto.validate().is_err());
    }
}

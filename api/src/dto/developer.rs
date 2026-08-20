use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 4, max = 30, message = "Proje ismi 4-30 karakter arasında olmalıdır"))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponseDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateApiKeyDto {
    pub project_id: Uuid,
    #[validate(length(min = 1, max = 30, message = "Anahtar ismi 1-30 karakter arasında olmalıdır"))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponseDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUsageDto {
    pub date: String,
    pub requests: i32,
    pub errors: i32,
}

// MenuSubmissionRequestDto kaldırıldı - multipart parse doğrudan route katmanında yapılır.

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuSubmissionResponseDto {
    pub id: i32,
    pub city_slug: String,
    pub year: i32,
    pub month: i32,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

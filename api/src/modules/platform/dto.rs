pub mod developer {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    pub struct CreateProjectDto {
        #[validate(length(
            min = 4,
            max = 30,
            message = "Proje ismi 4-30 karakter arasında olmalıdır"
        ))]
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
        #[validate(length(
            min = 1,
            max = 30,
            message = "Anahtar ismi 1-30 karakter arasında olmalıdır"
        ))]
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
}

pub mod pagination {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PaginatedResponse<T> {
        pub items: Vec<T>,
        pub total_items: u64,
        pub total_pages: u64,
        pub current_page: u64,
    }

    #[derive(Debug, Deserialize)]
    pub struct PaginationQuery {
        pub page: Option<u64>,
        pub limit: Option<u64>,
        pub offset: Option<u64>,
    }

    impl PaginationQuery {
        pub fn page_num(&self) -> u64 {
            if let Some(p) = self.page {
                return p.max(1);
            }
            if let Some(off) = self.offset {
                let lim = self.limit_num();
                return (off / lim) + 1;
            }
            1
        }

        pub fn limit_num(&self) -> u64 {
            // En fazla 100, varsayılan 20
            self.limit.unwrap_or(20).clamp(1, 100)
        }

        pub fn offset(&self) -> u64 {
            if let Some(off) = self.offset {
                return off;
            }
            (self.page_num() - 1) * self.limit_num()
        }
    }
}

pub mod statistics {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct TopDishDto {
        pub dish_id: i32,
        pub name: String,
        // Skor (Örn: %85 beğeni veya +40 net oy)
        pub score: i32,
        pub total_votes: i32,
        pub average_rating: Option<f64>,
    }

    #[derive(Debug, Serialize)]
    pub struct ModerationCategorySliceDto {
        pub category: String,
        pub count: i64,
        pub percentage: f64,
        pub color: String,
    }

    #[derive(Debug, Serialize)]
    pub struct RecentActionDto {
        pub action: String,
        pub category: String,
        pub action_type: String,
        pub created_at: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ModerationStatsDto {
        pub total_reports: i64,
        pub resolved_reports: i64,
        pub pending_reports: i64,
        pub resolution_rate: Option<i32>,
        pub deleted_comments: i64,
        pub category_distribution: Vec<ModerationCategorySliceDto>,
        pub recent_actions: Vec<RecentActionDto>,
    }

    #[derive(Debug, Serialize)]
    pub struct TrendingTagDto {
        pub name: String,
        pub count: i64,
        pub category: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ContributorDto {
        pub nickname: String,
        pub avatar_url: Option<String>,
        pub resolved_count: i32,
    }

    #[derive(Debug, Serialize)]
    pub struct HumanityStatsDto {
        pub resolved_reports: i64,
        pub pending_reports: i64,
        pub total_reports: i64,
        pub resolution_rate: Option<i32>,
        pub contributors: Vec<ContributorDto>,
    }
}

pub mod system {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct VerifyTreeResponseDto {
        pub is_valid: bool,
        pub node_count: i32,
        pub corrupted_count: i32,
        pub corrupted_hashes: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct HeadDto {
        pub key: String,
        pub hash: String,
    }

    #[derive(Debug, Serialize)]
    pub struct SystemHealthResponseDto {
        pub status: String,
        pub global_fingerprint: String,
        pub node_counts: std::collections::HashMap<String, i64>,
        pub heads: Vec<HeadDto>,
        pub total_nodes: i64,
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct IncidentDto {
        pub id: Option<i32>,
        pub component: String,
        pub title: String,
        pub message: String,
        pub started_at: String,
        pub ended_at: Option<String>,
        pub resolved_at: Option<String>,
        pub status: String,
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct SystemStatusDto {
        pub status: String,
        pub last_activity: Option<String>,
        pub incidents: Vec<IncidentDto>,
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct StatusDayDto {
        pub date: String,
        pub status: String,
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct ComponentHistoryDto {
        pub name: String,
        pub days: Vec<StatusDayDto>,
    }
}

pub use developer::*;
pub use pagination::*;
pub use statistics::*;
pub use system::*;

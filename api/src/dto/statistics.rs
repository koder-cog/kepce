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

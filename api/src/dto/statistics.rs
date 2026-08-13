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
pub struct ModerationStatsDto {
    pub pending_reports_count: i32,
    pub resolved_reports_count: i32,
    pub auto_dismissed_count: i32,
    pub active_bans_count: i32,
    pub deleted_comments: i64,
    pub recent_actions: Vec<RecentActionDto>,
}

#[derive(Debug, Serialize)]
pub struct TrendingTagDto {
    pub name: String,
    pub count: i64,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct RecentActionDto {
    pub nickname: String,
    pub action: String,
    pub action_type: String,
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentDto {
    pub menu_id: i32,
    pub dish_id: Option<i32>,
    #[validate(length(max = 500, message = "Yorum 500 karakterden uzun olamaz"))]
    pub comment: Option<String>,
    pub sentiment: Sentiment,
    pub parent_id: Option<Uuid>,
    pub is_tabldot: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReactionTypeDto {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReactionSummaryDto {
    pub up: i32,
    pub down: i32,
    pub my_vote: Option<ReactionTypeDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommentResponseDto {
    pub id: Uuid,
    pub comment: Option<String>,
    pub sentiment: Sentiment,
    pub is_tabldot: bool,
    pub user: UserSummaryDto,
    pub reaction_summary: ReactionSummaryDto,
    pub children: Vec<CommentResponseDto>,
    pub created_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deletion_type: Option<String>,
}

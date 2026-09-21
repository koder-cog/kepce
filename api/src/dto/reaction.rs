use crate::dto::comment::ReactionTypeDto;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ReactionRequestDto {
    pub vote_id: uuid::Uuid,
    #[serde(rename = "type")]
    pub reaction: ReactionTypeDto,
}

#[derive(Debug, Serialize)]
pub struct ReactionSummaryDto {
    pub upvotes: i32,
    pub downvotes: i32,
    pub my_vote: Option<ReactionTypeDto>,
}

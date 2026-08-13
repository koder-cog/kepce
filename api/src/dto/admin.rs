use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UpdateDishDto {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub category: Option<String>,
    pub is_celiac: Option<bool>,
    pub is_vegan: Option<bool>,
    pub is_vegetarian: Option<bool>,
}

#[derive(Deserialize, Validate)]
pub struct CreateDishDto {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub category: Option<String>,
    pub is_celiac: Option<bool>,
    pub is_vegan: Option<bool>,
    pub is_vegetarian: Option<bool>,
}

#[derive(Deserialize)]
pub struct MergeDishesDto {
    pub source_dish_id: i32,
    pub target_dish_id: i32,
}

#[derive(Deserialize, Validate)]
pub struct SplitDishDto {
    pub dish_id: i32,
    #[validate(length(min = 1, max = 50))]
    pub delimiter: String,
}

#[derive(Deserialize)]
pub struct DetachDishDto {
    pub alias_id: i32,
}

#[derive(Serialize)]
pub struct DishAliasDto {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct DishModerationStatsDto {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub constraints: Vec<String>,
    pub usage_count: i64,
    pub aliases: Vec<DishAliasDto>,
}

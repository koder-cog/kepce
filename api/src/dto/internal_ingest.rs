use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalDishItemDto {
    pub name: String,
    #[serde(default)]
    pub calories: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalTakeawayDto {
    pub package_name: String,
    pub dishes: Vec<Vec<InternalDishItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalMenuDto {
    pub city_slug: String,
    pub serve_date: String,
    pub meal_type: String,
    pub dishes: Vec<Vec<InternalDishItemDto>>,
    #[serde(default)]
    pub celiac_dishes: Vec<Vec<InternalDishItemDto>>,
    #[serde(default)]
    pub takeaways: Vec<InternalTakeawayDto>,
    #[serde(default)]
    pub calorie_range_min: Option<i32>,
    #[serde(default)]
    pub calorie_range_max: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalKykyemekIngestRequest {
    pub menus: Vec<InternalMenuDto>,
    #[serde(default)]
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalIngestResponseDto {
    pub total_received: usize,
    pub total_inserted: usize,
    pub total_updated: usize,
    pub total_skipped: usize,
    pub errors: Vec<String>,
}

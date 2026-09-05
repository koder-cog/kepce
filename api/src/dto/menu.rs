use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

/// Menünün dışarıya verilen ana yanıt yapısı
#[derive(Debug, Serialize)]
pub struct MenuResponseDto {
    pub id: i32,
    pub city_name: String,
    pub city_slug: String,
    pub serve_date: NaiveDate,
    pub meal_type: MealType,
    pub source_type: String,
    pub status: String,
    pub bot_commentary: Option<String>,
    pub items: Vec<MenuItemDto>, // Menüdeki standart yemeklerin listesi
    pub takeaways: Vec<TakeawayMenuDto>, // Al Götür menüleri
    pub comment_count: i32,
    pub rating_sum: i32,
    pub vote_count: i32,
    pub my_vote: Option<String>,
    pub calorie_range_min: Option<i32>,
    pub calorie_range_max: Option<i32>,
    pub calorie_range: Option<String>, // Menü kalori aralığı (örn: "600 - 800 kcal")
    pub calculated_calories: Option<i32>, // Yemeklerin estimated_calories toplamı
}

/// Al Götür paketinin detayı
#[derive(Debug, Serialize)]
pub struct TakeawayMenuDto {
    pub name: String,
    pub items: Vec<MenuItemDto>,
}

/// Menü içindeki her bir yemeğin detayı
#[derive(Debug, Serialize)]
pub struct MenuItemDto {
    pub order_index: i32,
    
    // Botun getirdiği ham (raw) isim
    pub raw_name: String, 
    
    // Yemeğin alternatif bir seçenek olup olmadığını gösterir
    pub is_alternative: bool,
    
    pub amount: Option<String>,
    pub calories: Option<i32>,
    pub price: Option<String>,
    pub category: Option<String>,
    
    // Eğer yemek admin tarafından resmi listeyle (master) eşleştirildiyse bu obje dolu gelir.
    pub master_data: Option<DishMasterDataDto>,
}

/// Eşleşmiş yemeğin kesin, onaylanmış verileri
#[derive(Debug, Serialize, Clone)]
pub struct DishMasterDataDto {
    pub dish_id: i32,
    pub name: String,
    pub is_celiac: bool,
    pub is_vegan: bool,
    pub is_vegetarian: bool,
    pub estimated_calories: Option<i32>,
    pub total_votes: i32,
    pub positive_votes: i32,
    pub negative_votes: i32,
    pub dislike_ratio: Option<f64>,
    pub like_ratio: Option<f64>,
}

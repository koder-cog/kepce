use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

/// Alternatif kaynaktan gelen menü bilgisi
#[derive(Debug, Serialize, Clone)]
pub struct AlternativeMenuDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub source_type: String,
    pub meal_type: MealType,
    pub items: Vec<MenuItemDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_commentary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
    pub items: Vec<MenuItemDto>, // Menüdeki standart yemeklerin listesi
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub takeaways: Vec<TakeawayMenuDto>, // Al Götür menüleri
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<AlternativeMenuDto>, // Başka kaynakların dedikleri
    pub comment_count: i32,
    pub rating_sum: i32,
    pub vote_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_vote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calorie_range_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calorie_range_max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculated_calories: Option<i32>, // Yemeklerin estimated_calories toplamı
}

/// Al Götür paketinin detayı
#[derive(Debug, Serialize, Clone)]
pub struct TakeawayMenuDto {
    pub name: String,
    pub items: Vec<MenuItemDto>,
}

/// Menü içindeki her bir yemeğin detayı
#[derive(Debug, Serialize, Clone)]
pub struct MenuItemDto {
    pub order_index: i32,
    /// Temizlenmiş ve onaylanmış nihai gösterim ismi (master varsa master.name, yoksa raw_name)
    pub name: String,
    /// Sadece menüdeki ham isim nihai isimden farklı olduğunda serileştirilir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<String>,
    /// Yemeğin alternatif bir seçenek olup olmadığını gösterir
    pub is_alternative: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    /// Tekil efektif kalori değeri (menüde yazan veya katalog tahmini)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Eğer yemek resmi listeyle (master) eşleştirildiyse bu obje dolu gelir
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_calories: Option<i32>,
    pub total_votes: i32,
    pub positive_votes: i32,
    pub negative_votes: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dislike_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_ratio: Option<f64>,
}

/// Arşiv sayfasında onaylı menüsü olan şehirlerin son menü yıl-ay bilgisi
#[derive(Debug, Clone, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ArchiveHighlightDto {
    pub city_slug: String,
    pub city_name: String,
    pub year: i32,
    pub month: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_dto_serialization_omits_none_and_empty_fields() {
        let dto = MenuResponseDto {
            id: 1,
            city_name: "İstanbul".to_string(),
            city_slug: "istanbul".to_string(),
            serve_date: NaiveDate::from_ymd_opt(2026, 9, 17).unwrap(),
            meal_type: MealType::Dinner,
            source_type: "kykyemek".to_string(),
            status: "approved".to_string(),
            bot_commentary: None,
            notice: None,
            items: vec![MenuItemDto {
                order_index: 0,
                name: "Ezogelin Çorbası".to_string(),
                raw_name: None,
                is_alternative: false,
                amount: None,
                calories: Some(180),
                price: None,
                category: None,
                master_data: None,
            }],
            takeaways: Vec::new(),
            alternatives: Vec::new(),
            comment_count: 0,
            rating_sum: 0,
            vote_count: 0,
            my_vote: None,
            calorie_range_min: Some(1100),
            calorie_range_max: Some(1500),
            calculated_calories: None,
        };

        let json_str = serde_json::to_string(&dto).expect("JSON serialize edilmeli");
        let v: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON olmalı");

        // null alanlar serileşmemeli
        assert!(v.get("bot_commentary").is_none());
        assert!(v.get("notice").is_none());
        assert!(v.get("my_vote").is_none());
        assert!(v.get("calculated_calories").is_none());
        assert!(v.get("takeaways").is_none());
        assert!(v.get("alternatives").is_none());
        assert!(v.get("calorie_range").is_none());

        // items doğrulaması
        let item = &v["items"][0];
        assert_eq!(item["name"], "Ezogelin Çorbası");
        assert_eq!(item["calories"], 180);
        assert!(item.get("raw_name").is_none());
        assert!(item.get("amount").is_none());
        assert!(item.get("price").is_none());
        assert!(item.get("category").is_none());
        assert!(item.get("master_data").is_none());
    }

    #[test]
    fn test_menu_item_serializes_raw_name_only_when_different() {
        let item_same = MenuItemDto {
            order_index: 0,
            name: "Mercimek Çorbası".to_string(),
            raw_name: None,
            is_alternative: false,
            amount: None,
            calories: None,
            price: None,
            category: None,
            master_data: None,
        };
        let json_same = serde_json::to_string(&item_same).unwrap();
        assert!(!json_same.contains("raw_name"));

        let item_diff = MenuItemDto {
            order_index: 0,
            name: "Ezogelin Çorbası".to_string(),
            raw_name: Some("EZOGELİN ÇORBA".to_string()),
            is_alternative: false,
            amount: None,
            calories: None,
            price: None,
            category: None,
            master_data: None,
        };
        let json_diff = serde_json::to_string(&item_diff).unwrap();
        assert!(json_diff.contains("\"raw_name\":\"EZOGELİN ÇORBA\""));
    }
}


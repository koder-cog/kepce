use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuComponent {
    pub name: String,
    pub amount: Option<String>,
    pub calories: Option<String>,
    pub category: Option<String>,
}

impl PartialEq<&str> for MenuComponent {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

impl PartialEq<String> for MenuComponent {
    fn eq(&self, other: &String) -> bool {
        self.name == *other
    }
}

impl From<&str> for MenuComponent {
    fn from(s: &str) -> Self {
        MenuComponent {
            name: s.to_string(),
            amount: None,
            calories: None,
            category: None,
        }
    }
}

impl From<String> for MenuComponent {
    fn from(s: String) -> Self {
        MenuComponent {
            name: s,
            amount: None,
            calories: None,
            category: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MenuItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takeaway_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alternatives: Vec<MenuComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DailyMenu {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakfast_kcal: Option<String>,
    #[serde(default)]
    pub breakfast: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lunch_kcal: Option<String>,
    #[serde(default)]
    pub lunch: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dinner_kcal: Option<String>,
    #[serde(default)]
    pub dinner: Vec<MenuItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DayMetadata {
    pub trust_score: u8,
    pub status: String, // "approved" or "needs_review"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DayData {
    #[serde(default)]
    pub normal: DailyMenu,
    #[serde(default)]
    pub colyak: DailyMenu,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DayMetadata>,
}

// Map of Date (YYYY-MM-DD) to DayData
pub type MenuDatabase = HashMap<String, DayData>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PricingCategoryItem {
    pub meal_type: String,
    pub category_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portion_amount: Option<String>,
    pub price: sea_orm::prelude::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OfficialPricingData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub academic_year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_start: Option<chrono::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_end: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub items: Vec<PricingCategoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TakeawayItemData {
    pub dish_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portion: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TakeawaySlotData {
    pub slot_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_title: Option<String>,
    #[serde(default = "default_true")]
    pub is_required: bool,
    #[serde(default)]
    pub items: Vec<TakeawayItemData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TakeawayPackageData {
    pub package_name: String,
    #[serde(default)]
    pub slots: Vec<TakeawaySlotData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TakeawayData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub academic_year: Option<String>,
    #[serde(default)]
    pub packages: Vec<TakeawayPackageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ParsedDocumentPayload {
    DailyMenu(MenuDatabase),
    OfficialPricing(OfficialPricingData),
    Takeaway(TakeawayData),
    Compound {
        #[serde(skip_serializing_if = "Option::is_none")]
        menu: Option<MenuDatabase>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pricing: Option<OfficialPricingData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        takeaway: Option<TakeawayData>,
    },
}

impl ParsedDocumentPayload {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::DailyMenu(db) => db.is_empty(),
            Self::OfficialPricing(p) => p.items.is_empty(),
            Self::Takeaway(t) => t.packages.is_empty(),
            Self::Compound {
                menu,
                pricing,
                takeaway,
            } => {
                menu.as_ref().is_none_or(|m| m.is_empty())
                    && pricing.as_ref().is_none_or(|p| p.items.is_empty())
                    && takeaway.as_ref().is_none_or(|t| t.packages.is_empty())
            }
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takeaway_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alternatives: Vec<MenuComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyMenu {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakfast_kcal: Option<String>,
    #[serde(default)]
    pub breakfast: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dinner_kcal: Option<String>,
    #[serde(default)]
    pub dinner: Vec<MenuItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayMetadata {
    pub trust_score: u8,
    pub status: String, // "approved" or "needs_review"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

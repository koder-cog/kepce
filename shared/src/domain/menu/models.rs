//! Menü ortak domain modelleri
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuDishItem {
    pub name: String,
    pub calories: Option<i32>,
    pub category: Option<String>,
}

//! Ingest karar motoru ve moderasyon eşik değerlendirmesi
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReport {
    pub file_name: String,
    pub status: String, // "approved", "pending", "rejected"
    pub cqs_score: u32,
    pub human_diagnostic: String,
}

impl DecisionReport {
    pub fn new(file_name: &str, status: &str, cqs_score: u32, diag: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            status: status.to_string(),
            cqs_score,
            human_diagnostic: diag.to_string(),
        }
    }
}

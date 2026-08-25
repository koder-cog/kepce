use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct VerifyTreeResponseDto {
    pub is_valid: bool,
    pub node_count: i32,
    pub corrupted_count: i32,
    pub corrupted_hashes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HeadDto {
    pub key: String,
    pub hash: String,
}

#[derive(Debug, Serialize)]
pub struct SystemHealthResponseDto {
    pub status: String,
    pub global_fingerprint: String,
    pub node_counts: std::collections::HashMap<String, i64>,
    pub heads: Vec<HeadDto>,
    pub total_nodes: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct IncidentDto {
    pub id: Option<i32>,
    pub component: String,
    pub title: String,
    pub message: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub resolved_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemStatusDto {
    pub status: String,
    pub last_activity: Option<String>,
    pub incidents: Vec<IncidentDto>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatusDayDto {
    pub date: String,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComponentHistoryDto {
    pub name: String,
    pub days: Vec<StatusDayDto>,
}

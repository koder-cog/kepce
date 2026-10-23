use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

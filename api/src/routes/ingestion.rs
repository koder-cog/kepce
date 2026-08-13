use axum::{
    routing::post,
    Router,
    extract::{State, Multipart},
    Json,
};
use crate::error::AppError;
use crate::extractors::api_key::IngestionAuth;
use crate::dto::developer::MenuSubmissionResponseDto;
use crate::services::ingestion::{IngestionService, IngestionError, MenuSubmissionInput, IngestedFile};

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/submit", post(submit_menu))
}

impl From<IngestionError> for AppError {
    fn from(err: IngestionError) -> Self {
        match err {
            IngestionError::CityNotFound => AppError::BadRequest("Geçersiz şehir seçimi".to_string()),
            IngestionError::InvalidInput(msg) => AppError::BadRequest(msg),
            IngestionError::FileTooLarge => AppError::BadRequest("Dosya boyutu çok büyük (Maks 20MB)".to_string()),
            IngestionError::TooManyFiles => AppError::BadRequest("En fazla 5 dosya gönderilebilir".to_string()),
            IngestionError::InvalidFileType(name) => AppError::BadRequest(format!("{}: Geçersiz dosya formatı veya içeriği", name)),
            IngestionError::IoError(e) => {
                tracing::error!("IO error in IngestionService: {}", e);
                AppError::Internal("Dosya kaydedilemedi".to_string())
            }
            IngestionError::DatabaseError(e) => {
                tracing::error!("Database error in IngestionService: {}", e);
                AppError::Internal("Veritabanı hatası".to_string())
            }
        }
    }
}

async fn submit_menu(
    State(db): State<sea_orm::DatabaseConnection>,
    auth: IngestionAuth,
    mut multipart: Multipart,
) -> Result<Json<MenuSubmissionResponseDto>, AppError> {
    let mut city_slug: Option<String> = None;
    let mut year: Option<i32> = None;
    let mut month: Option<i32> = None;
    let mut notes: Option<String> = None;
    let mut files: Vec<IngestedFile> = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "city_slug" => {
                city_slug = Some(field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
            "year" => {
                let val = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                year = Some(val.parse::<i32>().map_err(|_| AppError::BadRequest("Geçersiz yıl değeri".to_string()))?);
            }
            "month" => {
                let val = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                month = Some(val.parse::<i32>().map_err(|_| AppError::BadRequest("Geçersiz ay değeri".to_string()))?);
            }
            "notes" => {
                notes = Some(field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
            "files" => {
                let file_name = field.file_name().unwrap_or("unnamed").to_string();
                let content_type = field.content_type().map(|ct| ct.to_string());
                let mut data = Vec::new();
                while let Some(chunk) = field.chunk().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
                    data.extend_from_slice(&chunk);
                    if data.len() > 20 * 1024 * 1024 {
                        return Err(AppError::BadRequest("Her bir dosya boyutu 20MB'tan büyük olamaz.".to_string()));
                    }
                }
                files.push(IngestedFile {
                    name: file_name,
                    content_type,
                    data,
                });
            }
            _ => {}
        }
    }

    let city_slug = city_slug.ok_or_else(|| AppError::BadRequest("Şehir seçimi zorunludur".to_string()))?;
    let year = year.ok_or_else(|| AppError::BadRequest("Yıl seçimi zorunludur".to_string()))?;
    let month = month.ok_or_else(|| AppError::BadRequest("Ay seçimi zorunludur".to_string()))?;

    let user_id = match auth {
        IngestionAuth::User(user) => Some(user.id),
        IngestionAuth::Developer(key) => Some(key.user_id),
    };

    let input = MenuSubmissionInput {
        city_slug,
        year,
        month,
        notes,
        files,
    };

    let result = IngestionService::submit_menu(&db, user_id, input).await?;
    Ok(Json(result))
}

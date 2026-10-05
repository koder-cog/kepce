use crate::dto::developer::MenuSubmissionResponseDto;
use crate::dto::user::UserRole;
use crate::error::AppError;
use crate::extractors::api_key::IngestionAuth;
use crate::services::ingestion::{
    IngestedFile, IngestionError, IngestionService, MenuSubmissionInput,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    routing::post,
};
use sea_orm::{EntityTrait, QueryFilter};
use shared::entities::{prelude::*, users};
use uuid::Uuid;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/submit", post(submit_menu))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
}

impl From<IngestionError> for AppError {
    fn from(err: IngestionError) -> Self {
        match err {
            IngestionError::CityNotFound => {
                AppError::BadRequest("Geçersiz şehir seçimi".to_string())
            }
            IngestionError::InvalidInput(msg) => AppError::BadRequest(msg),
            IngestionError::FileTooLarge => {
                AppError::BadRequest("Dosya boyutu çok büyük (Maks 20MB)".to_string())
            }
            IngestionError::TooManyFiles => {
                AppError::BadRequest("En fazla 5 dosya gönderilebilir".to_string())
            }
            IngestionError::InvalidFileType(name) => {
                AppError::BadRequest(format!("{}: Geçersiz dosya formatı veya içeriği", name))
            }
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
    let mut on_behalf_of: Option<String> = None;
    let mut files: Vec<IngestedFile> = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "city_slug" => {
                city_slug = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            "year" => {
                let val = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                year = Some(
                    val.parse::<i32>()
                        .map_err(|_| AppError::BadRequest("Geçersiz yıl değeri".to_string()))?,
                );
            }
            "month" => {
                let val = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                month = Some(
                    val.parse::<i32>()
                        .map_err(|_| AppError::BadRequest("Geçersiz ay değeri".to_string()))?,
                );
            }
            "notes" => {
                notes = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            "on_behalf_of" => {
                let val = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                let trimmed = val.trim();
                if !trimmed.is_empty() {
                    on_behalf_of = Some(trimmed.to_string());
                }
            }
            "files" => {
                let file_name = field.file_name().unwrap_or("unnamed").to_string();
                let content_type = field.content_type().map(|ct| ct.to_string());
                let mut data = Vec::new();
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                {
                    data.extend_from_slice(&chunk);
                    if data.len() > 20 * 1024 * 1024 {
                        return Err(AppError::BadRequest(
                            "Her bir dosya boyutu 20MB'tan büyük olamaz.".to_string(),
                        ));
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

    let city_slug =
        city_slug.ok_or_else(|| AppError::BadRequest("Şehir seçimi zorunludur".to_string()))?;
    let year = year.ok_or_else(|| AppError::BadRequest("Yıl seçimi zorunludur".to_string()))?;
    let month = month.ok_or_else(|| AppError::BadRequest("Ay seçimi zorunludur".to_string()))?;

    let user_id = if let Some(ref target_identifier) = on_behalf_of {
        let admin_user = match auth {
            IngestionAuth::User(ref u) if u.role == UserRole::Admin => u,
            _ => {
                return Err(AppError::Forbidden(
                    "Yalnızca yöneticiler başkası adına menü gönderebilir.".to_string(),
                ));
            }
        };

        let target_user = if let Ok(uid) = Uuid::parse_str(target_identifier) {
            Users::find_by_id(uid)
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            let lower = target_identifier.to_lowercase();
            Users::find()
                .filter(
                    sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                        sea_orm::sea_query::Expr::col(users::Column::Username),
                    ))
                    .eq(&lower),
                )
                .one(&db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        };

        let target = target_user.ok_or_else(|| {
            AppError::BadRequest("Vekaleten gönderilmek istenen kullanıcı bulunamadı.".to_string())
        })?;

        let audit_tag = format!(
            "[Vekaleten Gönderim: @{} -> @{}]",
            admin_user.username, target.username
        );
        notes = Some(match notes {
            Some(existing) if !existing.trim().is_empty() => {
                format!("{}\n{}", audit_tag, existing.trim())
            }
            _ => audit_tag,
        });

        Some(target.id)
    } else {
        match auth {
            IngestionAuth::User(user) => Some(user.id),
            IngestionAuth::Developer(key) => Some(key.user_id),
            IngestionAuth::Anonymous => None,
        }
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

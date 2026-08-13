use sea_orm::*;
use uuid::Uuid;
use shared::entities::{
    prelude::*, menu_submissions, cities
};
use crate::dto::developer::MenuSubmissionResponseDto;

use chrono::{Datelike, Utc};

const MAX_FILE_SIZE: usize = 20 * 1024 * 1024; // 20MB
const MAX_FILES: usize = 5;
const ALLOWED_EXTENSIONS: &[&str] = &["xlsx", "xls", "pdf", "png", "jpg", "jpeg"];
const ALLOWED_MIME_TYPES: &[&str] = &[
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-excel",
    "application/pdf",
    "image/png",
    "image/jpeg",
    "image/pjpeg",
];

#[derive(Debug)]
pub enum IngestionError {
    CityNotFound,
    InvalidInput(String),
    FileTooLarge,
    TooManyFiles,
    InvalidFileType(String),
    IoError(std::io::Error),
    DatabaseError(DbErr),
}

impl From<DbErr> for IngestionError {
    fn from(err: DbErr) -> Self {
        IngestionError::DatabaseError(err)
    }
}

impl From<std::io::Error> for IngestionError {
    fn from(err: std::io::Error) -> Self {
        IngestionError::IoError(err)
    }
}

pub struct IngestedFile {
    pub name: String,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// Input parsed from multipart form data.
pub struct MenuSubmissionInput {
    pub city_slug: String,
    pub year: i32,
    pub month: i32,
    pub notes: Option<String>,
    pub files: Vec<IngestedFile>,
}

fn verify_file_signature(data: &[u8], ext: &str) -> bool {
    match ext {
        "png" => data.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]),
        "jpg" | "jpeg" => data.starts_with(&[0xFF, 0xD8]),
        "pdf" => data.starts_with(&[0x25, 0x50, 0x44, 0x46]),
        "xlsx" => data.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "xls" => data.starts_with(&[0xD0, 0xCF, 17, 224, 161, 177, 26, 225]),
        _ => false,
    }
}

pub struct IngestionService;

impl IngestionService {
    pub async fn submit_menu(
        db: &DatabaseConnection,
        user_id: Option<Uuid>,
        input: MenuSubmissionInput,
    ) -> Result<MenuSubmissionResponseDto, IngestionError> {
        // 1. Validate fields (Ocak 2026'dan bulunulan tarihin 1 ay sonrasına kadar)
        let now = Utc::now().date_naive();
        let max_date = now + chrono::Months::new(1);
        let max_year = max_date.year();
        let max_month = max_date.month() as i32;

        if input.year < 2026 || input.year > max_year || (input.year == max_year && input.month > max_month) {
            return Err(IngestionError::InvalidInput(format!(
                "Menü tarihi 2026-01 ile {}-{:02} arasında olmalıdır",
                max_year, max_month
            )));
        }
        if input.month < 1 || input.month > 12 {
            return Err(IngestionError::InvalidInput("Geçersiz ay (1-12 arasında olmalıdır)".to_string()));
        }
        if let Some(ref n) = input.notes {
            if n.len() > 1000 {
                return Err(IngestionError::InvalidInput("Notlar en fazla 1000 karakter olabilir".to_string()));
            }
        }

        // 2. Validate files
        if input.files.len() > MAX_FILES {
            return Err(IngestionError::TooManyFiles);
        }
        if input.files.is_empty() {
            return Err(IngestionError::InvalidInput("En az bir dosya gönderilmelidir".to_string()));
        }
        
        let mut validated_files = Vec::new();

        for file in &input.files {
            if file.data.len() > MAX_FILE_SIZE {
                return Err(IngestionError::FileTooLarge);
            }
            let ext = file.name.rsplit('.').next().unwrap_or("").to_lowercase();
            if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
                return Err(IngestionError::InvalidFileType(file.name.clone()));
            }
            
            // MIME Type Check
            if let Some(ref ct) = file.content_type {
                if !ALLOWED_MIME_TYPES.contains(&ct.as_str()) {
                    return Err(IngestionError::InvalidFileType(format!("{}: Geçersiz MIME tipi ({})", file.name, ct)));
                }
            } else {
                return Err(IngestionError::InvalidFileType(format!("{}: MIME tipi eksik", file.name)));
            }

            // Magic Bytes verification
            if !verify_file_signature(&file.data, &ext) {
                return Err(IngestionError::InvalidFileType(format!("{}: Dosya içeriği doğrulaması başarısız (Magic Bytes uyuşmazlığı)", file.name)));
            }

            // Sanitize: reject path traversal
            let basename = std::path::Path::new(&file.name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if basename.is_empty() || basename.contains("..") {
                return Err(IngestionError::InvalidFileType(file.name.clone()));
            }

            // Filename sanitization (ASCII alphanumeric, dots, hyphens, underscores only)
            let sanitized_name: String = basename.chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
                .collect();

            validated_files.push((sanitized_name, &file.data));
        }

        // 3. Verify city_slug
        let city_exists = Cities::find()
            .filter(cities::Column::Slug.eq(&input.city_slug))
            .one(db)
            .await?
            .is_some();

        if !city_exists {
            return Err(IngestionError::CityNotFound);
        }

        // 4. Save files to disk
        // SA-9: Pending dosyalar web root DIŞINDA karantinada tutulur. `static/`
        // ServeDir ile public olduğu için moderasyon öncesi dosyalar oraya yazılmaz;
        // onay sonrası taşıma/serbest bırakma akışı ayrıca tasarlanmalıdır.
        let submission_id = Uuid::new_v4();
        let upload_dir = format!(
            "uploads/quarantine/menus/{}/{}/{}",
            input.city_slug, input.year, submission_id
        );
        tokio::fs::create_dir_all(&upload_dir).await?;

        let mut saved_names: Vec<String> = Vec::new();
        for (safe_name, data) in &validated_files {
            let dest = format!("{}/{}", upload_dir, safe_name);
            tokio::fs::write(&dest, data).await?;
            saved_names.push(safe_name.to_string());
        }

        // 5. Build notes with file list
        let mut final_notes = input.notes.unwrap_or_default();
        let files_str = saved_names.join(", ");
        if final_notes.trim().is_empty() {
            final_notes = format!("Dosyalar: {}", files_str);
        } else {
            final_notes = format!("{}\nDosyalar: {}", final_notes, files_str);
        }

        // 6. Create database entry
        let sanitized_notes = shared::services::content_guard::ContentGuard::sanitize_html(&final_notes);
        let new_sub = menu_submissions::ActiveModel {
            user_id: Set(user_id),
            city_slug: Set(input.city_slug),
            year: Set(input.year),
            month: Set(input.month),
            notes: Set(Some(sanitized_notes)),
            status: Set("pending".to_string()),
            ..Default::default()
        };

        let inserted = new_sub.insert(db).await?;

        Ok(MenuSubmissionResponseDto {
            id: inserted.id,
            city_slug: inserted.city_slug,
            year: inserted.year,
            month: inserted.month,
            notes: inserted.notes,
            status: inserted.status,
            created_at: inserted.created_at.map(|t| t.into()),
        })
    }
}

//! Dijital PDF doğrudan metin ve tablo çıkarma modülü
use crate::domain::models::MenuDatabase;
use anyhow::Result;
use std::path::Path;

pub fn extract_digital_pdf_text(_path: &Path, _db: &mut MenuDatabase) -> Result<bool> {
    // Dijital PDF doğrudan metin çıkarma şimdilik VLM hattına delege eder.
    Ok(false)
}

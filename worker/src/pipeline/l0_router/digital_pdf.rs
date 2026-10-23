//! Dijital PDF doğrudan metin ve tablo çıkarma modülü
use anyhow::Result;
use std::path::Path;
use crate::domain::models::MenuDatabase;

pub fn extract_digital_pdf_text(_path: &Path, _db: &mut MenuDatabase) -> Result<bool> {
    // Dijital PDF doğrudan metin çıkarma şimdilik VLM hattına delege eder.
    Ok(false)
}

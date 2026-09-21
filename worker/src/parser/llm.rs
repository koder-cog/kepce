use crate::parser::models::MenuDatabase;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use serde_json::json;
use std::path::Path;

fn detect_mime_type(path: &Path, bytes: &[u8]) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "heic" | "heif" => "image/heic",
        _ => {
            if bytes.starts_with(&[0x25, 0x50, 0x44, 0x46]) {
                "application/pdf"
            } else if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
                "image/png"
            } else if bytes.starts_with(&[0xFF, 0xD8]) {
                "image/jpeg"
            } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
                "image/webp"
            } else {
                "application/pdf"
            }
        }
    }
}

pub fn menu_response_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "description": "Turkish university and dormitory monthly menu structure.",
        "properties": {
            "city": {
                "type": "string",
                "description": "City name if identifiable from document (e.g. 'istanbul', 'ankara')."
            },
            "period": {
                "type": "string",
                "description": "Menu period or month-year (e.g. 'Eylül 2026')."
            },
            "meal_type": {
                "type": "string",
                "description": "Default meal type: 'breakfast' for Kahvaltı, 'dinner' for Akşam Yemeği, 'lunch' for Öğle Yemeği."
            },
            "is_colyak": {
                "type": "boolean",
                "description": "True if this is specifically a Celiac (Glutensiz/Çölyak) menu, false otherwise."
            },
            "default_calories": {
                "type": "string",
                "description": "Overall target calories (e.g. '850-1000 kcal')."
            },
            "days": {
                "type": "array",
                "description": "List of daily menus extracted from the document.",
                "items": {
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD or DD.MM.YYYY format."
                        },
                        "meal_type": {
                            "type": "string",
                            "description": "Meal type for this day ('breakfast', 'dinner', 'lunch')."
                        },
                        "calories": {
                            "type": "string",
                            "description": "Calories for this day (e.g. '950 kcal')."
                        },
                        "takeaway": {
                            "type": "string",
                            "description": "Al Götür package name or id if specified (e.g. 'Al Götür 1')."
                        },
                        "items": {
                            "type": "array",
                            "description": "Dishes and food items served on this day.",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "description": "Name of the dish (e.g. 'Mercimek Çorbası')."
                                    },
                                    "amount": {
                                        "type": "string",
                                        "description": "Portion size or weight if listed (e.g. '200 gr', '1 adet')."
                                    },
                                    "calories": {
                                        "type": "string",
                                        "description": "Calories for this specific item if listed."
                                    },
                                    "alternatives": {
                                        "type": "array",
                                        "description": "Alternative choices if separated by '/' or 'veya' (e.g. ['Pirinç Pilavı', 'Bulgur Pilavı']).",
                                        "items": {
                                            "type": "string"
                                        }
                                    }
                                },
                                "required": ["name"]
                            }
                        }
                    },
                    "required": ["date", "items"]
                }
            }
        },
        "required": ["days"]
    })
}

fn extract_response_text(json_res: &serde_json::Value) -> Option<String> {
    if let Some(t) = json_res.get("output_text").and_then(|t| t.as_str()) {
        if !t.trim().is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(t) = json_res.get("text").and_then(|t| t.as_str()) {
        if !t.trim().is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(outputs) = json_res.get("outputs").and_then(|o| o.as_array()) {
        for output in outputs.iter().rev() {
            if let Some(t) = output.get("text").and_then(|t| t.as_str()) {
                if !t.trim().is_empty() {
                    return Some(t.to_string());
                }
            }
        }
    }
    if let Some(steps) = json_res.get("steps").and_then(|s| s.as_array()) {
        for step in steps.iter().rev() {
            if let Some(t) = step.get("text").and_then(|t| t.as_str()) {
                if !t.trim().is_empty() {
                    return Some(t.to_string());
                }
            }
            if let Some(parts) = step.get("parts").and_then(|p| p.as_array()) {
                if let Some(t) = parts
                    .first()
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                {
                    if !t.trim().is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
            if let Some(output) = step.get("output") {
                if let Some(t) = output.as_str() {
                    if !t.trim().is_empty() {
                        return Some(t.to_string());
                    }
                }
                if let Some(t) = output.get("text").and_then(|t| t.as_str()) {
                    if !t.trim().is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
    }
    if let Some(candidates) = json_res.get("candidates").and_then(|c| c.as_array()) {
        if let Some(t) = candidates
            .first()
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .and_then(|a| a.first())
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
        {
            if !t.trim().is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn clean_json_markdown(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json") {
        if let Some(inner) = stripped.strip_suffix("```") {
            return inner.trim();
        }
    }
    if let Some(stripped) = trimmed.strip_prefix("```") {
        if let Some(inner) = stripped.strip_suffix("```") {
            return inner.trim();
        }
    }
    trimmed
}

pub async fn parse_document_with_llm(
    client: &Client,
    api_key: &str,
    file_path: &Path,
) -> Result<MenuDatabase> {
    tracing::info!(
        "Belge Gemini Interactions API ile ayrıştırılıyor: {:?}",
        file_path
    );

    let metadata = tokio::fs::metadata(file_path)
        .await
        .context(format!("Dosya metadata'sı okunamadı: {:?}", file_path))?;
    if metadata.len() > 50 * 1024 * 1024 {
        anyhow::bail!("Dosya boyutu limitini aşıyor (max 50MB): {:?}", file_path);
    }

    let file_bytes = tokio::fs::read(file_path)
        .await
        .context(format!("Dosya okunamadı: {:?}", file_path))?;

    let base64_data = BASE64.encode(&file_bytes);
    let mime_type = detect_mime_type(file_path, &file_bytes);

    let prompt = "You are a precise data extraction engine for Turkish university and dormitory dining hall menus (KYK menüleri).
Extract all daily menus, dates, meal types (Kahvaltı -> breakfast, Akşam Yemeği -> dinner, Öğle -> lunch), food items, portions/weights, calories, and alternatives from the provided document or image.
Ensure every day present in the document is extracted into the 'days' array with accurate dates (YYYY-MM-DD or DD.MM.YYYY).
If multiple dish options are offered for a slot (separated by '/', 'veya', or alternate lines), include them in the 'alternatives' list.
Output strictly conforming to the requested JSON schema.";

    let model_name = std::env::var("GEMINI_MODEL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "gemini-flash".to_string());

    let payload = json!({
        "model": model_name,
        // KVKK veri minimizasyonu: Interactions API varsayılan olarak istekleri
        // sunucu tarafında saklar (store=true). Kullanıcı menü belgelerinin
        // Google tarafında tutulmaması için stateless mod zorunlu kılınır.
        "store": false,
        "input": [
            {"text": prompt},
            {"inlineData": {"mimeType": mime_type, "data": base64_data}}
        ],
        "response_format": {
            "type": "text",
            "mime_type": "application/json",
            "schema": menu_response_schema()
        }
    });

    let url = "https://generativelanguage.googleapis.com/v1beta/interactions";

    let max_retries = 2;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        tracing::info!(
            "  Interactions API Denemesi {}/{} (Model: {})...",
            attempt,
            max_retries,
            model_name
        );

        match client
            .post(url)
            .header("x-goog-api-key", api_key)
            .json(&payload)
            .send()
            .await
        {
            Ok(res) => {
                let status = res.status();
                let text_res = res.text().await.unwrap_or_default();

                if !status.is_success() {
                    let err_msg = format!("API Error ({}): {}", status, text_res);
                    tracing::warn!("  Hata: {}", err_msg);
                    last_error = err_msg;
                    continue;
                }

                if let Ok(json_res) = serde_json::from_str::<serde_json::Value>(&text_res) {
                    let extracted =
                        extract_response_text(&json_res).unwrap_or_else(|| text_res.clone());
                    let cleaned = clean_json_markdown(&extracted);

                    tracing::debug!("LLM Raw Response: {}", cleaned);

                    let file_name_hint = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document_llm");

                    match crate::parser::json::parse_json_str(cleaned, file_name_hint) {
                        Ok(mut db) => {
                            for day_data in db.values_mut() {
                                crate::parser::validation::finalize_day_metadata(day_data);
                            }

                            tracing::info!("  Başarıyla ayrıştırıldı: {} gün bulundu.", db.len());
                            return Ok(db);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "  JSON Ingest ayrıştırma hatası (deneme {}): {}",
                                attempt,
                                e
                            );
                            last_error = format!("IngestMenuJson error: {}", e);
                            continue;
                        }
                    }
                } else {
                    last_error = format!("Failed to parse response as JSON: {}", text_res);
                }
            }
            Err(e) => {
                tracing::warn!("  İstek atılamadı: {}", e);
                last_error = format!("Request failed: {}", e);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Tüm LLM denemeleri başarısız oldu. Son hata: {}",
        last_error
    ))
}

pub use parse_document_with_llm as parse_pdf_with_llm;

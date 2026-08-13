use anyhow::{Context, Result};
use crate::parser::models::MenuDatabase;
use crate::parser::core::{SheetGrid, parse_grid};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Deserialize, Serialize)]
struct LlmMenuResponse {
    sheets: Vec<SheetGrid>,
}

fn menu_response_schema() -> serde_json::Value {
    json!({
        "type": "OBJECT",
        "properties": {
            "sheets": {
                "type": "ARRAY",
                "description": "Each menu table/page from the document.",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "name": {
                            "type": "STRING",
                            "description": "Name of the sheet, describing the meal type and date (e.g., 'Nisan 2026 Kahvaltı')"
                        },
                        "rows": {
                            "type": "ARRAY",
                            "description": "2D array of grid cells representing the table",
                            "items": {
                                "type": "ARRAY",
                                "items": {
                                    "type": "STRING"
                                }
                            }
                        }
                    },
                    "required": ["name", "rows"]
                }
            }
        },
        "required": ["sheets"]
    })
}

pub async fn parse_pdf_with_llm(client: &Client, api_key: &str, pdf_path: &Path) -> Result<MenuDatabase> {
    tracing::info!("PDF dosyası Gemini Interactions API ile okunuyor: {:?}", pdf_path);

    let metadata = tokio::fs::metadata(pdf_path).await
        .context(format!("PDF dosyası metadata'sı okunamadı: {:?}", pdf_path))?;
    if metadata.len() > 50 * 1024 * 1024 {
        anyhow::bail!("Dosya boyutu limitini aşıyor (max 50MB): {:?}", pdf_path);
    }

    let pdf_bytes = tokio::fs::read(pdf_path).await
        .context(format!("PDF dosyası okunamadı: {:?}", pdf_path))?;
        
    let base64_pdf = BASE64.encode(&pdf_bytes);

    let prompt = "You are a precise data extraction engine. Extract the tables from this PDF exactly as they appear visually.
Output the data as a 2D grid of raw text.
Preserve the exact character sequence, including all symbols like slashes (/) and asterisks (*).
Maintain the exact row and column structure.
For the sheet 'name', combine the month, year and the meal type (e.g. 'Nisan 2026 Kahvaltı' or 'Nisan 2026 Akşam Yemeği'). If it's a Celiac (Çölyak) menu, include 'Çölyak' in the name.";

    let model_name = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-flash-latest".to_string());
    
    // GenerateContent API Payload
    let payload = json!({
        "contents": [{
            "parts": [
                {"text": prompt},
                {"inlineData": {"mimeType": "application/pdf", "data": base64_pdf}}
            ]
        }],
        "generationConfig": {
            "temperature": 0.1,
            "responseMimeType": "application/json",
            "responseSchema": menu_response_schema()
        }
    });

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model_name);

    let max_retries = 2;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        tracing::info!("  Interactions API Denemesi {}/{}...", attempt, max_retries);

        match client.post(&url)
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

                // Parse Interactions API response
                if let Ok(json_res) = serde_json::from_str::<serde_json::Value>(&text_res) {
                    let text = if let Some(t) = json_res.get("text").and_then(|t| t.as_str()) {
                         t.to_string()
                    } else if let Some(parts) = json_res.pointer("/output/parts") {
                        if let Some(t) = parts.as_array().and_then(|a| a.first()).and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
                            t.to_string()
                        } else {
                            last_error = format!("Unexpected LLM response structure: {}", text_res);
                            tracing::warn!("  Hata: {}", last_error);
                            continue;
                        }
                    } else if let Some(candidates) = json_res.get("candidates") {
                         if let Some(t) = candidates.as_array().and_then(|a| a.first())
                            .and_then(|c| c.get("content"))
                            .and_then(|c| c.get("parts"))
                            .and_then(|a| a.as_array())
                            .and_then(|a| a.first())
                            .and_then(|p| p.get("text"))
                            .and_then(|t| t.as_str()) {
                             t.to_string()
                         } else {
                             last_error = format!("Unexpected fallback structure: {}", text_res);
                             continue;
                         }
                    } else {
                        text_res.clone()
                    };

                    tracing::info!("LLM Raw Response: {}", text);

                    match serde_json::from_str::<LlmMenuResponse>(&text) {
                        Ok(response) => {
                            let mut db = MenuDatabase::new();
                            let file_name_hint = pdf_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown_pdf").to_string();

                            for sheet in response.sheets {
                                parse_grid(&sheet, &mut db, &file_name_hint);
                            }
                            
                            // Calculate trust scores and anomaly distances for all parsed days
                            for day_data in db.values_mut() {
                                crate::parser::validation::finalize_day_metadata(day_data);
                            }

                            return Ok(db);
                        }
                        Err(e) => {
                            if let Ok(response) = serde_json::from_str::<LlmMenuResponse>(&text_res) {
                                let mut db = MenuDatabase::new();
                                let file_name_hint = pdf_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown_pdf").to_string();

                                for sheet in response.sheets {
                                    parse_grid(&sheet, &mut db, &file_name_hint);
                                }
                                
                                for day_data in db.values_mut() {
                                    crate::parser::validation::finalize_day_metadata(day_data);
                                }
                                return Ok(db);
                            }
                            
                            tracing::warn!("  Deserialization failed (attempt {}): {}", attempt, e);
                            last_error = format!("Deserialization error: {}", e);
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

    Err(anyhow::anyhow!("Tüm LLM denemeleri başarısız oldu. Son hata: {}", last_error))
}

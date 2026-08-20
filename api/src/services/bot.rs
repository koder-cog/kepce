// Kepçe API - Service: Bot Servisi
// ==================================
//
// İsteğe bağlı AI yorum üretici.
// Moderasyon panelinden manuel tetiklenebilir.
// Ayrı bir API üzerinden de çağrılabilir (Gemini/GPT).
//
// AppState'te ağırlık oluşturmaz.
// Sadece moderasyon servisi tarafından çağrılır.
// Yapılandırma yoksa sessizce devre dışı kalır.

use reqwest::Client;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum BotError {
    NetworkError(String),
    ApiError(String),
}

pub struct BotService;

impl BotService {
    /// Gemini yapılandırılmış-çıktı şeması: gün bazlı yorum dizisi.
    /// `export-monthly` uç noktası ve otomatik üretimde kullanılır.
    pub const BOT_OUTPUT_SCHEMA: &'static str = r#"{
      "type": "object",
      "properties": {
        "yorum_listesi": {
          "type": "array",
          "description": "Günlük yemek yorumlarının listesi",
          "items": {
            "type": "object",
            "properties": {
              "tarih": {
                "type": "string",
                "description": "Yemek günü, ISO 8601 formatında: YYYY-MM-DD (örn. 2026-04-01)"
              },
              "yorum": {
                "type": "string",
                "description": "O güne ait Kepçe Bot yorumu."
              }
            },
            "required": ["tarih", "yorum"]
          }
        }
      },
      "required": ["yorum_listesi"]
    }"#;

    /// Moderasyon panelinden tetiklendiğinde Gemini API üzerinden yorum üretir.
    /// `directive` = sistem talimatı (persona + kurallar + çıktı formatı), dosyadan gelir.
    /// `context` = günlük menü verisi gibi dinamik içerik (kullanıcı mesajı).
    pub async fn generate_ai_comment(
        client: &Client,
        api_key: &str,
        model: &str,
        directive: &str,
        context: &str,
    ) -> Result<String, BotError> {
        // SA-6: API key URL query'sinde değil, header'da taşınır (log sızıntısı önlemi).
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model
        );

        // Sistem direktifi ayrı bir systemInstruction olarak verilir; günlük menü
        // verisi ise kullanıcı mesajı (context) olarak gönderilir.
        let payload = json!({
            "systemInstruction": {
                "parts": [{ "text": directive }]
            },
            "contents": [{
                "parts": [{ "text": context }]
            }]
        });

        let response = client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| BotError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::ApiError(format!("Gemini API Error: {}", error_text)));
        }

        let result_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BotError::NetworkError(e.to_string()))?;

        if let Some(generated_text) = result_json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            Ok(generated_text.trim().to_string())
        } else {
            // Hata yutmamak için logluyoruz
            tracing::error!("Gemini JSON parsing failed. Raw response: {}", result_json.to_string());
            Err(BotError::ApiError("Beklenmeyen JSON formatı".into()))
        }
    }

    /// Yapılandırılmış (JSON) çıktı için Gemini çağrısı.
    ///
    /// `response_schema` verilirse `responseMimeType: application/json` zorlanır
    /// ve dönen metin JSON olarak çözümlenip `serde_json::Value` döndürülür.
    /// Aylık batch (export-monthly) ve otomatik üretim senaryolarında kullanılır.
    pub async fn generate_structured(
        client: &Client,
        api_key: &str,
        model: &str,
        directive: &str,
        context: &str,
        response_schema: &Value,
    ) -> Result<Value, BotError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model
        );

        let payload = json!({
            "systemInstruction": {
                "parts": [{ "text": directive }]
            },
            "contents": [{
                "parts": [{ "text": context }]
            }],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": response_schema
            }
        });

        let response = client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| BotError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::ApiError(format!("Gemini API Error: {}", error_text)));
        }

        let result_json: Value = response
            .json()
            .await
            .map_err(|e| BotError::NetworkError(e.to_string()))?;

        let text = result_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| {
                tracing::error!("Gemini JSON parsing failed. Raw response: {}", result_json);
                BotError::ApiError("Beklenmeyen JSON formatı".into())
            })?;

        serde_json::from_str::<Value>(text)
            .map_err(|e| BotError::ApiError(format!("JSON çözümleme hatası: {}", e)))
    }
}

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
            // Interactions API: model çıktısı `steps[].content[].text` alanındadır
            // (adım tipi `model_output`). Bu şekil eskiden kontrol edilmediği için
            // metin çıkarılamıyor ve ham zarf ayrıştırıcıya veriliyordu ("0 gün").
            if let Some(content) = step.get("content").and_then(|c| c.as_array()) {
                for part in content.iter().rev() {
                    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                        if !t.trim().is_empty() {
                            return Some(t.to_string());
                        }
                    }
                }
            }
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

/// Yanıt zarfının şekli bilinmese bile içinde `days` anahtarı taşıyan ilk JSON
/// nesnesini bulur. `extract_response_text()` bilinen şekilleri çözemediğinde
/// son çare olarak kullanılır; böylece API zarfı değişse de ayrıştırma kırılmaz.
fn find_menu_json_deep(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            if map.contains_key("days") {
                return Some(serde_json::Value::Object(map.clone()).to_string());
            }
            map.values().find_map(find_menu_json_deep)
        }
        serde_json::Value::Array(items) => items.iter().find_map(find_menu_json_deep),
        // Menü JSON'u bazı yanıtlarda string olarak gömülü olabilir.
        serde_json::Value::String(s) => {
            let cleaned = clean_json_markdown(s);
            serde_json::from_str::<serde_json::Value>(cleaned)
                .ok()
                .and_then(|parsed| find_menu_json_deep(&parsed))
        }
        _ => None,
    }
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

/// Birincil Gemini modeli (tek kaynak / single source of truth).
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-flash-latest";

/// Yasaklı model adı kalıpları.
///
/// `gemini-flash-lite-latest` ve türevleri menü verisini **uydurma/hatalı**
/// üretiyor (canlı gözlem). Bu modeller fallback olarak bile kullanılmamalıdır;
/// hatalı veri üretmektense dosyayı kuyrukta bekletmek yeğdir.
const FORBIDDEN_GEMINI_MODEL_MARKERS: [&str; 2] = ["flash-lite", "flash_lite"];

/// Model adı yasaklı mı? (lite ve türevleri)
fn is_forbidden_gemini_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    FORBIDDEN_GEMINI_MODEL_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

/// Model listesini temizler: yasaklı (lite türevi) modelleri `WARN` ile eler.
///
/// Saf fonksiyon (env okumaz) — birim testi kolay olsun diye ayrıldı.
fn sanitize_gemini_models(models: Vec<String>) -> Vec<String> {
    models
        .into_iter()
        .filter(|m| {
            if is_forbidden_gemini_model(m) {
                tracing::warn!(
                    "Gemini modeli '{}' yasaklı (lite türevi: uydurma/hatalı menü üretiyor), zincirden çıkarıldı.",
                    m
                );
                false
            } else {
                true
            }
        })
        .collect()
}

/// `GEMINI_MODEL` ortam değişkenini sıralı bir model zincirine çözer.
///
/// Virgülle ayrılmış liste desteklenir (örn. `birincil,yedek`). **Otomatik yedek
/// EKLENMEZ**: tek model verilirse zincir yalnızca o modeldir. Birincil model
/// kotaya takılırsa dosya `bekleyen`'de kalır ve sonraki döngüde yeniden denenir
/// — hatalı veri üreten bir modele sessizce düşmek yerine.
pub fn resolve_gemini_models() -> Vec<String> {
    let raw = std::env::var("GEMINI_MODEL").unwrap_or_default();
    let models: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().trim_start_matches("models/").to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let models = sanitize_gemini_models(models);
    if models.is_empty() {
        vec![DEFAULT_GEMINI_MODEL.to_string()]
    } else {
        models
    }
}

/// Birincil modeli döndürür (başlangıç erişilebilirlik kontrolü için).
pub fn resolve_gemini_model() -> String {
    resolve_gemini_models()
        .into_iter()
        .next()
        .unwrap_or_else(|| DEFAULT_GEMINI_MODEL.to_string())
}

/// Muhakeme (reasoning/thinking) seviyesi.
///
/// Canlıda doğrulanan resmî alan: `generation_config.thinking_level`
/// (`high` -> HTTP 200 ve thought token üretimi; üst seviye `thinking_level`
/// ve `reasoning_effort` ise "Unknown parameter" ile 400 döner).
pub fn resolve_thinking_level() -> String {
    std::env::var("GEMINI_THINKING_LEVEL")
        .ok()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "high".to_string())
}

/// Kota/yoğunluk kaynaklı hatalarda modeli değiştirmek gerekir.
///
/// Aynı modeli tekrar denemek 20 istek/gün gibi kotalarda boşa harcamadır;
/// bu hatalarda zincirdeki sonraki modele geçilir.
fn is_model_switchable_error(err_msg: &str) -> bool {
    let msg = err_msg.to_lowercase();
    const MARKERS: [&str; 8] = [
        "429",
        "too_many_requests",
        "rate limit",
        "resource_exhausted",
        "503",
        "service_unavailable",
        "high demand",
        "overloaded",
    ];
    MARKERS.iter().any(|marker| msg.contains(marker))
}

/// LLM sağlayıcı sırası. Varsayılan: OpenRouter (birincil) -> Gemini (yedek).
///
/// Gemini Free Tier üretim kotası model başına 20 istek/gün olduğundan
/// OpenRouter birincil sağlayıcıdır; Gemini kotası/kullanılabilirliği
/// yetersiz kaldığında devreye girer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    OpenRouter,
    Gemini,
}

/// OpenRouter varsayılan modeli. `~` öneki "Gemini Flash ailesinin en güncel
/// modeline yönlendir" anlamına gelir (canlıda `google/gemini-3.8-flash`'a çözülür).
pub const DEFAULT_OPENROUTER_MODEL: &str = "~google/gemini-flash-latest";

/// `LLM_PROVIDER_ORDER` ortam değişkenini sağlayıcı sırasına çözer.
pub fn resolve_provider_order() -> Vec<LlmProvider> {
    let raw = std::env::var("LLM_PROVIDER_ORDER").unwrap_or_default();
    let mut out: Vec<LlmProvider> = raw
        .split(',')
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "openrouter" | "or" => Some(LlmProvider::OpenRouter),
            "gemini" | "google" => Some(LlmProvider::Gemini),
            _ => None,
        })
        .collect();
    if out.is_empty() {
        out = vec![LlmProvider::OpenRouter, LlmProvider::Gemini];
    }
    out
}

fn openrouter_api_key() -> Option<String> {
    std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn openrouter_model() -> String {
    std::env::var("OPENROUTER_MODEL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_OPENROUTER_MODEL.to_string())
}

fn openrouter_reasoning_effort() -> String {
    std::env::var("OPENROUTER_REASONING_EFFORT")
        .ok()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "low".to_string())
}

/// LLM ayrıştırma için en az bir sağlayıcı yapılandırılmış mı?
pub fn llm_available(gemini_api_key: Option<&str>) -> bool {
    gemini_api_key.is_some() || openrouter_api_key().is_some()
}

/// LLM metnini menü veritabanına çevirir; boş sonuç hata sayılır.
///
/// Boş menü "başarı" sayılırsa dosya vault'a taşınır ve veri sessizce
/// kaybolur; bu yüzden 0 gün dönen sonuç hata olarak ele alınır.
fn parse_and_finalize(text: &str, file_name_hint: &str) -> Result<MenuDatabase> {
    let cleaned = clean_json_markdown(text);
    let mut db = crate::parser::json::parse_json_str(cleaned, file_name_hint)?;
    if db.is_empty() {
        anyhow::bail!(
            "Ayrıştırma 0 gün döndürdü (boş menü). Ham yanıt (ilk 500 karakter): {}",
            cleaned.chars().take(500).collect::<String>()
        );
    }
    for day_data in db.values_mut() {
        crate::parser::validation::finalize_day_metadata(day_data);
    }
    Ok(db)
}

/// Sağlayıcıdan bağımsız istek verisi (istem metni + belge).
struct LlmRequest<'a> {
    prompt: &'a str,
    mime_type: &'a str,
    base64_data: &'a str,
    /// Gemini Interactions API için `document` | `image`.
    input_type: &'a str,
}

/// OpenRouter `/chat/completions` çağrısı (OpenAI uyumlu şema).
///
/// Canlıda doğrulandı: base64 `data:` URI ile PDF ve JPEG kabul edilir,
/// `response_format.json_schema` (strict) ve `reasoning.effort` desteklenir.
async fn call_openrouter(
    client: &Client,
    api_key: &str,
    model: &str,
    effort: &str,
    req: &LlmRequest<'_>,
) -> Result<String> {
    let base_url = std::env::var("OPENROUTER_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Muhakeme token'ları bu bütçeden düşer (doküman uyarısı). effort=medium/high
    // ile çok günlü menülerde 16000 YETMEZ ve yanıt finish_reason='length' ile
    // KESİLİR; bu yüzden cömert bir varsayılan ve env ile ayarlanabilirlik.
    let max_tokens: u64 = std::env::var("OPENROUTER_MAX_TOKENS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(32000);

    let mut payload = json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": req.prompt },
                {
                    "type": "image_url",
                    "image_url": { "url": format!("data:{};base64,{}", req.mime_type, req.base64_data) }
                }
            ]
        }],
        "response_format": {
            "type": "json_schema",
            "json_schema": { "name": "menu", "strict": true, "schema": menu_response_schema() }
        },
        // NOT: `provider.require_parameters: true` BİLİNÇLİ OLARAK KULLANILMAZ.
        // Canlıda doğrulandı: bu bayrak, akışı 502 "provider_unavailable" ile
        // bozan bir uç noktaya yönlendiriyor ve KESİK JSON döndürüyor
        // (menu_bursa.pdf -> 1 gün / 0 kayıt). Bayrak kaldırıldığında aynı dosya
        // 28-31 gün olarak eksiksiz ayrıştırılıyor.
        // Muhakeme token'ları bu bütçeden düşer; cömert tutulur.
        "max_tokens": max_tokens
    });

    if !effort.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("reasoning".to_string(), json!({ "effort": effort }));
        }
    }

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("OpenRouter isteği atılamadı")?;

    let status = res.status();
    let text_res = res.text().await.unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("OpenRouter API Error ({}): {}", status, text_res);
    }

    let json_res: serde_json::Value =
        serde_json::from_str(&text_res).context("OpenRouter yanıtı JSON değil")?;

    let choice = json_res
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first());

    let finish = choice
        .and_then(|c| c.get("finish_reason"))
        .and_then(|f| f.as_str())
        .unwrap_or("");
    let content = choice
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    // Sağlayıcı akışı bozulursa (502 provider_unavailable) içerik KESİK JSON
    // olarak döner. finish_reason "error"/"length" ise içerik güvenilmezdir;
    // yarım menü kaydedilmemesi için hata döndürülür (sonraki deneme/model).
    if finish == "error" || finish == "length" {
        anyhow::bail!(
            "OpenRouter yanıtı tamamlanmadı (finish_reason='{}'); içerik kesik olabilir.",
            finish
        );
    }

    if content.trim().is_empty() {
        // Doküman uyarısı: muhakeme token'ları max_tokens'ı tüketirse içerik boş
        // döner (finish_reason="length") ve yine faturalanır.
        anyhow::bail!(
            "OpenRouter boş içerik döndürdü (finish_reason='{}'). Muhakeme bütçesi tükenmiş olabilir.",
            finish
        );
    }

    Ok(content.to_string())
}

/// Gemini Interactions API çağrısı; yanıt zarfından menü metnini çıkarır.
async fn call_gemini(
    client: &Client,
    api_key: &str,
    model: &str,
    thinking_level: &str,
    req: &LlmRequest<'_>,
) -> Result<String> {
    let url = "https://generativelanguage.googleapis.com/v1beta/interactions";

    let mut payload = json!({
        "model": model,
        "store": false,
        "input": [
            { "type": "text", "text": req.prompt },
            { "type": req.input_type, "mime_type": req.mime_type, "data": req.base64_data }
        ],
        "response_format": {
            "type": "text",
            "mime_type": "application/json",
            "schema": menu_response_schema()
        }
    });

    // Yüksek muhakeme. Alan adı canlıda doğrulandı:
    // generation_config.thinking_level = "high" -> HTTP 200 + thought token.
    if !thinking_level.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "generation_config".to_string(),
                json!({ "thinking_level": thinking_level }),
            );
        }
    }

    let res = client
        .post(url)
        .header("x-goog-api-key", api_key)
        .json(&payload)
        .send()
        .await
        .context("Gemini isteği atılamadı")?;

    let status = res.status();
    let text_res = res.text().await.unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("Gemini API Error ({}): {}", status, text_res);
    }

    let json_res: serde_json::Value =
        serde_json::from_str(&text_res).context("Gemini yanıtı JSON değil")?;

    extract_response_text(&json_res)
        .or_else(|| find_menu_json_deep(&json_res))
        .ok_or_else(|| anyhow::anyhow!("Gemini yanıtından metin çıkarılamadı"))
}

/// Belge/görsel menü çıkarım prompt'u (tek kaynak / single source of truth).
///
/// Çok bölümlü belgelerde (ör. ayrı Kahvaltı ve Akşam Yemeği tabloları) modelin
/// ikinci tabloyu atlaması **sessiz veri kaybına** yol açar (canlıda gözlendi:
/// aynı PDF bazen 31 gün, bazen yalnızca akşam 14 gün olarak çıkarıldı). Bu yüzden
/// "her tabloyu ayrı ayrı çıkar, ilk tablodan sonra durma" talimatı açıkça verilir.
pub const MENU_EXTRACTION_PROMPT: &str = "You are a precise data extraction engine for Turkish university and dormitory dining hall menus (KYK menüleri).
Extract all daily menus, dates, meal types (Kahvaltı -> breakfast, Akşam Yemeği -> dinner, Öğle -> lunch), food items, portions/weights, calories, and alternatives from the provided document or image.
The document may contain MULTIPLE separate tables for different meal types (e.g. a Kahvaltı/breakfast table and an Akşam Yemeği/dinner table). Extract EVERY table as separate entries in 'days', each tagged with its own meal_type. Do not stop after the first table.
Ensure every day present in the document is extracted into the 'days' array with accurate dates (YYYY-MM-DD or DD.MM.YYYY).
If multiple dish options are offered for a slot (separated by '/', 'veya', or alternate lines), include them in the 'alternatives' list.
Output strictly conforming to the requested JSON schema.";

pub async fn parse_document_with_llm(
    client: &Client,
    gemini_api_key: Option<&str>,
    file_path: &Path,
) -> Result<MenuDatabase> {
    tracing::info!("Belge LLM ile ayrıştırılıyor: {:?}", file_path);

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

    let prompt = MENU_EXTRACTION_PROMPT;

    let input_type = if mime_type == "application/pdf" {
        "document"
    } else {
        "image"
    };

    let file_name_hint = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document_llm");

    let llm_req = LlmRequest {
        prompt,
        mime_type,
        base64_data: &base64_data,
        input_type,
    };

    let providers = resolve_provider_order();
    let openrouter_key = openrouter_api_key();
    let openrouter_model = openrouter_model();
    let openrouter_effort = openrouter_reasoning_effort();
    let gemini_models = resolve_gemini_models();
    let thinking_level = resolve_thinking_level();

    // Model başına deneme sayısı. Kota/doluluk hatasında aynı modeli zorlamak
    // kotayı boşa harcar; bu durumda hemen sonraki modele/sağlayıcıya geçilir.
    const MAX_ATTEMPTS_PER_MODEL: usize = 2;
    let mut last_error = String::new();
    let mut attempted_any = false;

    for provider in providers {
        match provider {
            LlmProvider::OpenRouter => {
                let Some(key) = openrouter_key.as_deref() else {
                    tracing::warn!("OpenRouter atlanıyor: OPENROUTER_API_KEY ayarlanmamış.");
                    continue;
                };
                attempted_any = true;

                for attempt in 1..=MAX_ATTEMPTS_PER_MODEL {
                    tracing::info!(
                        "  OpenRouter Denemesi {}/{} (Model: {}, effort: {})...",
                        attempt,
                        MAX_ATTEMPTS_PER_MODEL,
                        openrouter_model,
                        openrouter_effort
                    );

                    match call_openrouter(
                        client,
                        key,
                        &openrouter_model,
                        &openrouter_effort,
                        &llm_req,
                    )
                    .await
                    {
                        Ok(text) => match parse_and_finalize(&text, file_name_hint) {
                            Ok(db) => {
                                tracing::info!(
                                    "  Başarıyla ayrıştırıldı: {} gün bulundu (sağlayıcı: openrouter, model: {}).",
                                    db.len(),
                                    openrouter_model
                                );
                                return Ok(db);
                            }
                            Err(e) => {
                                tracing::warn!("  Ayrıştırma hatası (deneme {}): {}", attempt, e);
                                last_error = format!("{}", e);
                            }
                        },
                        Err(e) => {
                            let msg = format!("{:?}", e);
                            tracing::warn!("  Hata: {}", msg);
                            last_error = msg.clone();
                            if is_model_switchable_error(&msg) {
                                break;
                            }
                        }
                    }
                }
            }
            LlmProvider::Gemini => {
                let Some(key) = gemini_api_key else {
                    tracing::warn!("Gemini atlanıyor: GEMINI_API_KEY ayarlanmamış.");
                    continue;
                };
                attempted_any = true;

                for (model_idx, model_name) in gemini_models.iter().enumerate() {
                    let is_last_model = model_idx + 1 == gemini_models.len();

                    for attempt in 1..=MAX_ATTEMPTS_PER_MODEL {
                        tracing::info!(
                            "  Gemini Denemesi {}/{} (Model: {}, thinking: {})...",
                            attempt,
                            MAX_ATTEMPTS_PER_MODEL,
                            model_name,
                            thinking_level
                        );

                        match call_gemini(client, key, model_name, &thinking_level, &llm_req).await
                        {
                            Ok(text) => match parse_and_finalize(&text, file_name_hint) {
                                Ok(db) => {
                                    tracing::info!(
                                        "  Başarıyla ayrıştırıldı: {} gün bulundu (sağlayıcı: gemini, model: {}).",
                                        db.len(),
                                        model_name
                                    );
                                    return Ok(db);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "  Ayrıştırma hatası (deneme {}): {}",
                                        attempt,
                                        e
                                    );
                                    last_error = format!("{}", e);
                                }
                            },
                            Err(e) => {
                                let msg = format!("{:?}", e);
                                tracing::warn!("  Hata: {}", msg);
                                last_error = msg.clone();
                                if is_model_switchable_error(&msg) {
                                    break;
                                }
                            }
                        }
                    }

                    if !is_last_model {
                        tracing::warn!(
                            "  '{}' modeli başarısız oldu, zincirdeki sonraki modele geçiliyor: '{}'",
                            model_name,
                            gemini_models[model_idx + 1]
                        );
                    }
                }
            }
        }
    }

    if !attempted_any {
        anyhow::bail!(
            "Hiçbir LLM sağlayıcısı yapılandırılmamış (OPENROUTER_API_KEY / GEMINI_API_KEY)."
        );
    }

    Err(anyhow::anyhow!(
        "Tüm LLM denemeleri başarısız oldu. Son hata: {}",
        last_error
    ))
}

pub use parse_document_with_llm as parse_pdf_with_llm;

#[cfg(test)]
mod tests {
    use super::*;

    /// Gerçek Interactions API zarfları `steps[].content[].text` içinde metin taşır.
    /// Bu test, kaydedilmiş canlı yanıt şeklini (21.09.2026 canlı çalıştırma) taklit eder.
    #[test]
    fn test_extract_text_from_interactions_envelope() {
        let envelope = serde_json::json!({
            "status": "completed",
            "object": "interaction",
            "model": "gemini-flash-lite-latest",
            "steps": [
                { "type": "thought", "signature": "abc123" },
                {
                    "type": "model_output",
                    "content": [
                        {
                            "type": "text",
                            "text": "{\"days\":[{\"date\":\"14.09.2026\",\"meal_type\":\"dinner\",\"items\":[{\"name\":\"Mercimek Çorbası\",\"amount\":\"250 gr\"}]}]}"
                        }
                    ]
                }
            ]
        });

        let text = extract_response_text(&envelope).expect("metin çıkarılabilmeli");
        assert!(text.contains("Mercimek Çorbası"));

        let db = crate::parser::json::parse_json_str(&text, "test.pdf").expect("parse edilmeli");
        assert_eq!(db.len(), 1, "bir gün ayrıştırılmalı");
    }

    /// Zarf şekli tanınmasa bile derin arama `days` nesnesini bulmalı.
    #[test]
    fn test_deep_search_fallback_finds_days_object() {
        let odd = serde_json::json!({
            "unexpected": { "nested": [ { "days": [ { "date": "2026-09-14", "items": [ { "name": "X" } ] } ] } ] }
        });

        assert!(extract_response_text(&odd).is_none());
        let found = find_menu_json_deep(&odd).expect("days nesnesi bulunmalı");
        assert!(found.contains("\"days\""));
        assert!(crate::parser::json::parse_json_str(&found, "test.pdf").is_ok());
    }

    /// Kota/yoğunluk hatalarında model değiştirilmeli; içerik/söz dizimi
    /// hatalarında ise aynı model denenmeye devam edilmeli.
    #[test]
    fn test_model_switchable_error_detection() {
        assert!(is_model_switchable_error(
            "API Error (429 Too Many Requests): {\"code\":\"too_many_requests\"}"
        ));
        assert!(is_model_switchable_error(
            "API Error (503 Service Unavailable): high demand"
        ));
        assert!(is_model_switchable_error("resource_exhausted"));
        assert!(!is_model_switchable_error(
            "API Error (400 Bad Request): Unknown parameter 'thinking_budget'"
        ));
        assert!(!is_model_switchable_error(
            "Ayrıştırma 0 gün döndürdü (boş menü)"
        ));
    }

    /// Lite ve türevleri zincirden elenmeli; diğer modeller korunmalı.
    #[test]
    fn test_sanitize_gemini_models_drops_lite() {
        let input = vec![
            "gemini-flash-latest".to_string(),
            "gemini-flash-lite-latest".to_string(),
            "gemini-3.5-flash-lite".to_string(),
            "gemini-2.5-flash".to_string(),
        ];
        let out = sanitize_gemini_models(input);
        assert_eq!(
            out,
            vec![
                "gemini-flash-latest".to_string(),
                "gemini-2.5-flash".to_string()
            ]
        );

        assert!(is_forbidden_gemini_model("gemini-flash-lite-latest"));
        assert!(is_forbidden_gemini_model("GEMINI-FLASH-LITE-LATEST"));
        assert!(is_forbidden_gemini_model("gemini-3.5-flash-lite"));
        assert!(!is_forbidden_gemini_model("gemini-flash-latest"));
        assert!(!is_forbidden_gemini_model("gemini-2.5-flash"));
    }

    /// Çoklu-tablo talimatı prompt'ta bulunmalı (sessiz veri kaybı regresyon kalkanı).
    ///
    /// Canlıda aynı PDF bazen 31 (kahvaltı + akşam), bazen yalnız akşam 14 gün
    /// olarak çıkarıldı; ikinci tablonun atlanmaması için bu talimat zorunludur.
    #[test]
    fn test_menu_extraction_prompt_requests_multiple_tables() {
        assert!(
            MENU_EXTRACTION_PROMPT.contains("MULTIPLE separate tables"),
            "prompt çoklu-tablo ifadesini içermeli"
        );
        assert!(
            MENU_EXTRACTION_PROMPT.contains("Do not stop after the first table"),
            "prompt 'ilk tablodan sonra durma' talimatını içermeli"
        );
        assert!(
            MENU_EXTRACTION_PROMPT.contains("each tagged with its own meal_type"),
            "prompt her tablonun meal_type ile etiketlenmesini istemeli"
        );
    }

    /// `.env`'i (repo kökü) yükler; başarısızlık sessizce yok sayılır.
    ///
    /// cargo test cwd'yi paket köküne (`worker/`) ayarladığı için repo kökü
    /// `../.env` olur; her ihtimale karşı bir üst dizin de denenir.
    fn load_probe_env() {
        let _ = dotenvy::dotenv();
        let _ = dotenvy::from_filename("../.env");
        let _ = dotenvy::from_filename("../../.env");
    }

    /// Probe için örnek belge yolunu çözer: `KEPCE_PROBE_PDF` veya bilinen adaylar.
    fn probe_pdf_path() -> Option<std::path::PathBuf> {
        if let Ok(p) = std::env::var("KEPCE_PROBE_PDF") {
            let pb = std::path::PathBuf::from(&p);
            if pb.exists() {
                return Some(pb);
            }
            eprintln!(
                "UYARI: KEPCE_PROBE_PDF bulunamadı, adaylara düşülüyor: {}",
                p
            );
        }
        const CANDIDATES: [&str; 4] = [
            "../data/menuler/admin/bekleyen/Nisan_2026_Kahvaltı.pdf",
            "data/menuler/admin/bekleyen/Nisan_2026_Kahvaltı.pdf",
            "../data/menuler/admin/bekleyen/Nisan_2026_Akşam_Yemeği.pdf",
            "data/menuler/admin/bekleyen/Nisan_2026_Akşam_Yemeği.pdf",
        ];
        CANDIDATES
            .iter()
            .map(std::path::PathBuf::from)
            .find(|p| p.exists())
    }

    /// Canlı LLM probe'u: gerçek bir API çağrısıyla prompt'un çıkarım kalitesini ölçer.
    ///
    /// `#[ignore]` — CI'da ÇALIŞMAZ (ağ + API anahtarı gerektirir). Yerelde:
    /// ```text
    /// KEPCE_PROBE_PDF="data/menuler/admin/bekleyen/Nisan_2026_Kahvaltı.pdf" \
    ///   cargo test -p worker probe_llm_extraction -- --ignored --nocapture
    /// ```
    /// Anahtar `.env`'den (repo kökü) okunur. `KEPCE_PROBE_MIN_DAYS` ile en az gün
    /// sayısı doğrulanır. Çıktı, gün sayısı + öğün kırılımıdır (sessiz kayıp kontrolü).
    #[tokio::test]
    #[ignore = "Canlı LLM probe: ağ + OPENROUTER_API_KEY/GEMINI_API_KEY gerektirir"]
    async fn probe_llm_extraction() {
        load_probe_env();

        let Some(path) = probe_pdf_path() else {
            eprintln!("PROBE atlandı: örnek belge bulunamadı (KEPCE_PROBE_PDF ile yol verin).");
            return;
        };

        let gemini_key = std::env::var("GEMINI_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());
        if !llm_available(gemini_key.as_deref()) {
            eprintln!("PROBE atlandı: OPENROUTER_API_KEY/GEMINI_API_KEY ayarlı değil.");
            return;
        }

        // Zaman aşımları yapılandırılabilir: büyük taranmış PDF'lerde (ör. ~4.5 MB)
        // model işlemesi dakikalar sürebilir; `KEPCE_PROBE_TIMEOUT_SECS` ile ayarlanır.
        let timeout_secs = std::env::var("KEPCE_PROBE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(300);
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("reqwest client kurulamadı");
        eprintln!("PROBE: belge={:?} | timeout={}s", path, timeout_secs);

        let db = parse_document_with_llm(&client, gemini_key.as_deref(), &path)
            .await
            .expect("LLM ayrıştırma başarılı olmalı");

        let days = db.len();
        let count = |meal: fn(&crate::parser::models::DailyMenu) -> bool| {
            db.values()
                .filter(|d| meal(&d.normal) || meal(&d.colyak))
                .count()
        };
        let breakfast = count(|m| !m.breakfast.is_empty());
        let lunch = count(|m| !m.lunch.is_empty());
        let dinner = count(|m| !m.dinner.is_empty());

        eprintln!(
            "PROBE SONUCU: dosya={:?} | {} gün | kahvaltı {} | öğle {} | akşam {}",
            path, days, breakfast, lunch, dinner
        );

        assert!(days > 0, "en az bir gün çıkarılmalı");

        if let Some(min) = std::env::var("KEPCE_PROBE_MIN_DAYS")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
        {
            assert!(days >= min, "beklenen en az {} gün, {} bulundu", min, days);
        }
    }
}

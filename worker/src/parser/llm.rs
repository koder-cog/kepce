use crate::parser::models::MenuDatabase;
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde_json::json;
use std::path::Path;

pub fn detect_mime_type(path: &Path, bytes: &[u8]) -> &'static str {
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

pub fn unified_document_response_schema() -> serde_json::Value {
    let alternative_schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Name of the alternative dish."
            },
            "amount": {
                "type": "string",
                "description": "Portion size or weight for this alternative dish alone (e.g. '200 g')."
            },
            "calories": {
                "type": "string",
                "description": "Calories for this alternative dish alone if listed (e.g. '164 kcal')."
            }
        },
        "required": ["name"]
    });

    let item_schema = json!({
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
                "description": "Alternative dish choices for the same slot.",
                "items": alternative_schema
            }
        },
        "required": ["name"]
    });

    let day_schema = json!({
        "type": "object",
        "properties": {
            "date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Date in strict ISO 8601 YYYY-MM-DD format."
            },
            "date_raw": {
                "type": "string",
                "description": "The date EXACTLY as written in the source document, verbatim (e.g. '01.04.2026')."
            },
            "meal_type": {
                "type": "string",
                "description": "Meal type ('breakfast', 'dinner', 'lunch')."
            },
            "is_colyak": {
                "type": "boolean",
                "description": "True if this meal is specifically a Celiac (Çölyak / Glutensiz) diet menu, false otherwise."
            },
            "calories": {
                "type": "string",
                "description": "Calories for this day (e.g. '950 kcal')."
            },
            "takeaway": {
                "type": "string",
                "description": "Takeaway package name or id if specified."
            },
            "items": {
                "type": "array",
                "description": "Dishes served on this day.",
                "items": item_schema
            }
        },
        "required": ["date", "items"]
    });

    let pricing_item_schema = json!({
        "type": "object",
        "properties": {
            "meal_type": {
                "type": "string",
                "enum": ["breakfast", "dinner", "lunch"],
                "description": "'breakfast' for Kahvalti, 'dinner' for Aksam/Yemek, 'lunch' for Ogle"
            },
            "category_name": {
                "type": "string",
                "description": "Dish, product, or category name (e.g. 'ZEYTİN', 'KAŞARLI TOST', 'ÇORBA', 'PİLAV', 'ET DÖNER')"
            },
            "portion_amount": {
                "type": "string",
                "description": "Grammage or portion amount (e.g. '30 g', '100 g Kaşar', '150 g', '1 Porsiyon')"
            },
            "price": {
                "type": "number",
                "description": "Official ceiling price in Turkish Lira (e.g. 11.0, 65.0, 120.0)"
            }
        },
        "required": ["meal_type", "category_name", "price"]
    });

    let pricing_board_schema = json!({
        "type": "object",
        "description": "Official ceiling price and grammage board data. Include ONLY if official ceiling prices in TL are explicitly present in the source document. Do NOT include if source is a dining menu without monetary prices.",
        "properties": {
            "period_start": {
                "type": "string",
                "description": "Valid from date (YYYY-MM-DD), usually 'YYYY-09-01'."
            },
            "period_end": {
                "type": "string",
                "description": "Valid to date (YYYY-MM-DD), usually 'YYYY-08-31'."
            },
            "items": {
                "type": "array",
                "description": "All price items listed on the board.",
                "items": pricing_item_schema
            }
        },
        "required": ["items"]
    });

    let takeaway_item_schema = json!({
        "type": "object",
        "properties": {
            "dish_name": {
                "type": "string",
                "description": "Item or food choice (e.g. 'Kaşarlı Soğuk Sandviç', 'Ayran (300 ml)', 'Elma')"
            },
            "portion": {
                "type": "string",
                "description": "Portion override or size if specified (e.g. '100 g', '200 ml')"
            }
        },
        "required": ["dish_name"]
    });

    let takeaway_slot_schema = json!({
        "type": "object",
        "properties": {
            "slot_index": {
                "type": "integer",
                "description": "Slot index (1, 2, 3, 4, 5)"
            },
            "slot_title": {
                "type": "string",
                "description": "Slot description (e.g. 'Ana Sandviç', 'İçecek', 'Meyve/Tatlı', 'Çay / Su')"
            },
            "is_required": {
                "type": "boolean",
                "description": "Whether this slot is required."
            },
            "items": {
                "type": "array",
                "description": "Available choices for this slot.",
                "items": takeaway_item_schema
            }
        },
        "required": ["slot_index", "items"]
    });

    let takeaway_package_schema = json!({
        "type": "object",
        "properties": {
            "package_name": {
                "type": "string",
                "description": "Package name (e.g. 'Al Götür Menü 1', 'Al Götür Menü 2')"
            },
            "slots": {
                "type": "array",
                "description": "Selection slots in this package.",
                "items": takeaway_slot_schema
            }
        },
        "required": ["package_name", "slots"]
    });

    let takeaway_schema = json!({
        "type": "object",
        "description": "Al Götür packages and choice slots. Include ONLY if takeaway packages are explicitly present in the source document. Do NOT include if source is a dining menu without takeaway packages.",
        "properties": {
            "packages": {
                "type": "array",
                "description": "List of takeaway packages.",
                "items": takeaway_package_schema
            }
        },
        "required": ["packages"]
    });

    json!({
        "type": "object",
        "description": "Polymorphic schema for Turkish dining menus, price lists, and takeaway packages.",
        "properties": {
            "document_type": {
                "type": "string",
                "enum": ["daily_menu", "official_pricing", "takeaway_package", "compound"],
                "description": "Classification of the document."
            },
            "is_colyak": {
                "type": "boolean",
                "description": "True if the entire document is specifically a Celiac (Çölyak / Glutensiz) diet menu."
            },
            "city": {
                "type": "string",
                "description": "City name if identifiable (e.g. 'istanbul', 'ankara')."
            },
            "academic_year": {
                "type": "string",
                "description": "Academic year if mentioned (e.g. '2026-2027')."
            },
            "period": {
                "type": "string",
                "description": "Menu period or month-year (e.g. 'Ekim 2026')."
            },
            "days": {
                "type": "array",
                "description": "MANDATORY when document_type is 'daily_menu' or 'compound'. Must contain all calendar dates and meal rows found in the table. Must never be empty if a menu table is present.",
                "items": day_schema
            },
            "pricing_board": pricing_board_schema,
            "takeaway": takeaway_schema
        },
        "required": ["document_type"]
    })
}

fn extract_response_text(json_res: &serde_json::Value) -> Option<String> {
    if let Some(t) = json_res.get("output_text").and_then(|t| t.as_str())
        && !t.trim().is_empty()
    {
        return Some(t.to_string());
    }
    if let Some(t) = json_res.get("text").and_then(|t| t.as_str())
        && !t.trim().is_empty()
    {
        return Some(t.to_string());
    }
    if let Some(outputs) = json_res.get("outputs").and_then(|o| o.as_array()) {
        for output in outputs.iter().rev() {
            if let Some(t) = output.get("text").and_then(|t| t.as_str())
                && !t.trim().is_empty()
            {
                return Some(t.to_string());
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
                    if let Some(t) = part.get("text").and_then(|t| t.as_str())
                        && !t.trim().is_empty()
                    {
                        return Some(t.to_string());
                    }
                }
            }
            if let Some(t) = step.get("text").and_then(|t| t.as_str())
                && !t.trim().is_empty()
            {
                return Some(t.to_string());
            }
            if let Some(parts) = step.get("parts").and_then(|p| p.as_array())
                && let Some(t) = parts
                    .first()
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                && !t.trim().is_empty()
            {
                return Some(t.to_string());
            }
            if let Some(output) = step.get("output") {
                if let Some(t) = output.as_str()
                    && !t.trim().is_empty()
                {
                    return Some(t.to_string());
                }
                if let Some(t) = output.get("text").and_then(|t| t.as_str())
                    && !t.trim().is_empty()
                {
                    return Some(t.to_string());
                }
            }
        }
    }
    if let Some(candidates) = json_res.get("candidates").and_then(|c| c.as_array())
        && let Some(t) = candidates
            .first()
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .and_then(|a| a.first())
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
        && !t.trim().is_empty()
    {
        return Some(t.to_string());
    }
    None
}

/// Yanıt zarfının şekli bilinmese bile içinde hedef veri anahtarları taşıyan ilk JSON
/// nesnesini bulur (`document_type`, `days`, `pricing_board`, `takeaway`).
fn find_payload_json_deep(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            if map.contains_key("document_type")
                || map.contains_key("days")
                || map.contains_key("pricing_board")
                || map.contains_key("takeaway")
            {
                return Some(serde_json::Value::Object(map.clone()).to_string());
            }
            map.values().find_map(find_payload_json_deep)
        }
        serde_json::Value::Array(items) => items.iter().find_map(find_payload_json_deep),
        // Bazı yanıtlarda string olarak gömülü olabilir.
        serde_json::Value::String(s) => {
            let cleaned = clean_json_markdown(s);
            serde_json::from_str::<serde_json::Value>(cleaned)
                .ok()
                .and_then(|parsed| find_payload_json_deep(&parsed))
        }
        _ => None,
    }
}

fn find_menu_json_deep(value: &serde_json::Value) -> Option<String> {
    find_payload_json_deep(value)
}

fn clean_json_markdown(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json")
        && let Some(inner) = stripped.strip_suffix("```")
    {
        return inner.trim();
    }
    if let Some(stripped) = trimmed.strip_prefix("```")
        && let Some(inner) = stripped.strip_suffix("```")
    {
        return inner.trim();
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
const FORBIDDEN_GEMINI_MODEL_MARKERS: [&str; 4] =
    ["flash-lite", "flash_lite", "2.5-flash", "2.5_flash"];

/// Model adı yasaklı mı? (lite ve türevleri)
fn is_forbidden_gemini_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    FORBIDDEN_GEMINI_MODEL_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

/// Model listesini temizler: yasaklı (lite türevi) modelleri `WARN` ile eler.
///
/// Saf fonksiyon (env okumaz), birim testi kolay olsun diye ayrıldı.
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
/// kotaya takılırsa dosya `bekleyen`'de kalır ve sonraki döngüde yeniden denenir.
/// Böylece hatalı veri üreten bir modele sessizce düşülmez.
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
        .unwrap_or_else(|| "low".to_string())
}

/// Kota/yoğunluk kaynaklı hatalarda modeli değiştirmek gerekir.
///
/// Aynı modeli tekrar denemek 20 istek/gün gibi kotalarda boşa harcamadır;
/// bu hatalarda zincirdeki sonraki modele geçilir.
fn is_model_switchable_error(err_msg: &str) -> bool {
    let msg = err_msg.to_lowercase();
    // Yoğunluk/kota ve geçici sunucu hataları: aynı modeli zorlamak yerine
    // zincirdeki sonraki modele, ardından sonraki sağlayıcıya geçilir.
    const MARKERS: [&str; 12] = [
        "429",
        "too_many_requests",
        "rate limit",
        "resource_exhausted",
        "503",
        "502",
        "504",
        "unavailable",
        "high demand",
        "overloaded",
        "deadline exceeded",
        "timeout",
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

/// Tüm zincir başarısız olduğunda zincirin kaç kez baştan deneneceği.
///
/// `503 "high demand"` gibi yoğunluk hataları saniyeler içinde geçebilir; tek
/// turda pes etmek dosyayı gereksiz yere kuyrukta bekletir.
fn chain_passes() -> usize {
    std::env::var("LLM_CHAIN_PASSES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|p| *p > 0)
        .map(|p| p.min(5))
        .unwrap_or(2)
}

/// Zincir turları arasındaki taban bekleme (ms); tur başına doğrusal artar.
fn chain_retry_delay_ms() -> u64 {
    std::env::var("LLM_CHAIN_RETRY_BASE_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|d| d.min(30_000))
        .unwrap_or(2_000)
}

/// Yoğunluk hatasından sonra sonraki modele geçmeden önceki bekleme (ms).
fn model_switch_delay_ms() -> u64 {
    std::env::var("LLM_MODEL_SWITCH_DELAY_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|d| d.min(30_000))
        .unwrap_or(1_500)
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
/// İkinci değer, `date_raw` çapraz doğrulama uyuşmazlıklarını taşır (Faz 3.4).
#[allow(dead_code)]
fn parse_and_finalize(text: &str, file_name_hint: &str) -> Result<(MenuDatabase, Vec<String>)> {
    let (payload, mismatches) = parse_and_finalize_polymorphic(text, file_name_hint)?;
    match payload {
        crate::parser::models::ParsedDocumentPayload::DailyMenu(db) => Ok((db, mismatches)),
        _ => anyhow::bail!("Ayrıştırma sonucu günlük tabldot menüsü değil."),
    }
}

/// LLM yanıtını çok biçimli (Polymorphic) veri yüküne (`ParsedDocumentPayload`) çevirir.
/// Günlük menü, tavan fiyat panosu veya Al Götür paketi başarıyla yapılandırılır.
pub fn parse_and_finalize_polymorphic(
    text: &str,
    file_name_hint: &str,
) -> Result<(crate::parser::models::ParsedDocumentPayload, Vec<String>)> {
    let cleaned = clean_json_markdown(text);
    let val: serde_json::Value =
        serde_json::from_str(cleaned).context("Ayrıştırma sonucu geçerli bir JSON değil")?;

    let doc_type = val
        .get("document_type")
        .and_then(|t| t.as_str())
        .unwrap_or("");

    // 1. Resmi Fiyat Panosu
    if doc_type == "official_pricing" || (doc_type.is_empty() && val.get("pricing_board").is_some())
    {
        let pricing_val = val.get("pricing_board").unwrap_or(&val);
        let items_val = pricing_val.get("items").and_then(|i| i.as_array());
        if let Some(items_arr) = items_val
            && !items_arr.is_empty()
        {
            let mut pricing_items = Vec::new();
            for item in items_arr {
                let meal_type = item
                    .get("meal_type")
                    .and_then(|m| m.as_str())
                    .unwrap_or("dinner")
                    .to_lowercase();
                let category_name = item
                    .get("category_name")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_uppercase();
                let portion_amount = item
                    .get("portion_amount")
                    .and_then(|p| p.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let price_num = item.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                if !category_name.is_empty() {
                    pricing_items.push(crate::parser::models::PricingCategoryItem {
                        meal_type,
                        category_name,
                        portion_amount,
                        price: sea_orm::prelude::Decimal::from_f64_retain(price_num)
                            .unwrap_or_default(),
                    });
                }
            }

            let period_start = pricing_val
                .get("period_start")
                .and_then(|s| s.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let period_end = pricing_val
                .get("period_end")
                .and_then(|s| s.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let academic_year = val
                .get("academic_year")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let city_slug = val
                .get("city")
                .and_then(|s| s.as_str())
                .map(|s| s.to_lowercase());

            return Ok((
                crate::parser::models::ParsedDocumentPayload::OfficialPricing(
                    crate::parser::models::OfficialPricingData {
                        city_slug,
                        academic_year,
                        period_start,
                        period_end,
                        items: pricing_items,
                    },
                ),
                Vec::new(),
            ));
        }
    }

    // 2. Çok Biçimli Menü / Al Götür / Compound Ayrıştırması
    let takeaway_opt = parse_takeaway_from_value(&val);
    let menu_opt = if let Ok((mut db, mismatches)) =
        crate::parser::json::parse_json_str_with_diagnostics(cleaned, file_name_hint)
    {
        if !db.is_empty() {
            for day_data in db.values_mut() {
                crate::parser::validation::finalize_day_metadata(day_data);
            }
            Some((db, mismatches))
        } else {
            None
        }
    } else {
        None
    };

    if doc_type == "compound" || (takeaway_opt.is_some() && menu_opt.is_some()) {
        let (db, mismatches) = menu_opt.unwrap_or_default();
        return Ok((
            crate::parser::models::ParsedDocumentPayload::Compound {
                menu: if db.is_empty() { None } else { Some(db) },
                pricing: None,
                takeaway: takeaway_opt,
            },
            mismatches,
        ));
    }

    if let Some(takeaway_data) = takeaway_opt {
        return Ok((
            crate::parser::models::ParsedDocumentPayload::Takeaway(takeaway_data),
            Vec::new(),
        ));
    }

    if let Some((db, mismatches)) = menu_opt {
        return Ok((
            crate::parser::models::ParsedDocumentPayload::DailyMenu(db),
            mismatches,
        ));
    }

    anyhow::bail!(
        "Ayrıştırma sonucu geçerli bir veri (günlük menü, fiyat panosu veya al götür) içermiyor. Ham yanıt: {}",
        cleaned.chars().take(300).collect::<String>()
    );
}

fn parse_takeaway_from_value(
    val: &serde_json::Value,
) -> Option<crate::parser::models::TakeawayData> {
    let takeaway_val = val.get("takeaway").unwrap_or(val);
    let pkgs_val = takeaway_val.get("packages").and_then(|p| p.as_array())?;
    if pkgs_val.is_empty() {
        return None;
    }

    let mut packages = Vec::new();
    for pkg in pkgs_val {
        let package_name = pkg
            .get("package_name")
            .and_then(|n| n.as_str())
            .unwrap_or("Al Götür")
            .trim()
            .to_string();
        let mut slots = Vec::new();
        if let Some(slots_arr) = pkg.get("slots").and_then(|s| s.as_array()) {
            for slot in slots_arr {
                let slot_index =
                    slot.get("slot_index").and_then(|i| i.as_i64()).unwrap_or(1) as i32;
                let slot_title = slot
                    .get("slot_title")
                    .and_then(|t| t.as_str())
                    .map(|s| s.trim().to_string());
                let is_required = slot
                    .get("is_required")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(true);
                let mut items = Vec::new();
                if let Some(items_arr) = slot.get("items").and_then(|i| i.as_array()) {
                    for item in items_arr {
                        let dish_name = item
                            .get("dish_name")
                            .or_else(|| item.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        let portion = item
                            .get("portion")
                            .or_else(|| item.get("amount"))
                            .and_then(|p| p.as_str())
                            .map(|s| s.trim().to_string());
                        if !dish_name.is_empty() {
                            items.push(crate::parser::models::TakeawayItemData {
                                dish_name,
                                portion,
                            });
                        }
                    }
                }
                slots.push(crate::parser::models::TakeawaySlotData {
                    slot_index,
                    slot_title,
                    is_required,
                    items,
                });
            }
        }
        packages.push(crate::parser::models::TakeawayPackageData {
            package_name,
            slots,
        });
    }

    let academic_year = val
        .get("academic_year")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());
    let city_slug = val
        .get("city")
        .and_then(|s| s.as_str())
        .map(|s| s.to_lowercase());

    Some(crate::parser::models::TakeawayData {
        city_slug,
        academic_year,
        packages,
    })
}

/// Sağlayıcıdan bağımsız istek verisi (istem metni + belge + şema).
pub struct LlmRequest<'a> {
    pub prompt: &'a str,
    pub mime_type: &'a str,
    pub base64_data: &'a str,
    /// Gemini Interactions API için `document` | `image`.
    pub input_type: &'a str,
    /// İsteğe bağlı yanıt şeması (`None` ise modelden ham düz metin istenir).
    pub schema: Option<&'a serde_json::Value>,
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

    let mut content = Vec::new();
    if !req.base64_data.is_empty() {
        // OpenAI / OpenRouter vision standardı: taranmış tablolarda düşük çözünürlüğe
        // düşmemesi için detail="high" zorunlu; görsel parça talimattan önce gelir.
        content.push(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", req.mime_type, req.base64_data),
                "detail": "high"
            }
        }));
    }
    content.push(json!({
        "type": "text",
        "text": req.prompt
    }));

    let mut payload = json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": content
        }],
        "max_tokens": max_tokens
    });

    if let Some(schema) = req.schema {
        payload["response_format"] = json!({
            "type": "json_schema",
            "json_schema": { "name": "document", "strict": true, "schema": schema }
        });
    }

    if !effort.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("reasoning".to_string(), json!({ "effort": effort }));
    }

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://kepce.org")
        .header("X-Title", "Kepce Menu Worker")
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

    let mut input = Vec::new();
    if !req.base64_data.is_empty() {
        input.push(json!({
            "type": req.input_type,
            "mime_type": req.mime_type,
            "data": req.base64_data,
            "media_resolution": "high",
        }));
    }
    input.push(json!({
        "type": "text",
        "text": req.prompt
    }));

    let response_format = if let Some(schema) = req.schema {
        json!({
            "type": "text",
            "mime_type": "application/json",
            "schema": schema
        })
    } else {
        json!({
            "type": "text",
            "mime_type": "text/plain"
        })
    };

    let mut payload = json!({
        "model": model,
        "store": false,
        "input": input,
        "response_format": response_format
    });

    if !thinking_level.is_empty() {
        let gen_config = json!({
            "thinking_level": thinking_level
        });
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("generation_config".to_string(), gen_config);
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
Ensure every day present in the document is extracted into the 'days' array. The 'date' field MUST be strict ISO 8601 (YYYY-MM-DD). Additionally, for EVERY day set 'date_raw' to the date EXACTLY as written in the source document, verbatim (e.g. '01.04.2026'). Never swap day and month yourself; Turkish documents use DD.MM.YYYY order.
If multiple dish options are offered for a slot (separated by '/', 'veya', or alternate lines), put the first option into the item's own name/amount/calories fields and the rest into the 'alternatives' array. Each alternative is an object with its own name, amount and calories fields. Do not combine multiple values into one field with a '/' separator.
Garnishes, sides, and sauces served alongside a main dish (e.g. garnitür havuç-kabak, patates püresi, sos) are part of the main dish, not separate alternatives.
If bread (ekmek, çeyrek ekmek) appears multiple times in the same meal from both a table row and a footnote, include it only once.
Output strictly conforming to the requested JSON schema.";

pub const UNIFIED_EXTRACTION_PROMPT: &str = "You are a precise data extraction engine for Turkish university dining, dormitories (KYK), cafeteria ceiling price boards, and takeaway packages.
Examine the provided image or document carefully and determine its 'document_type':
- 'daily_menu': Monthly, weekly, or daily dining hall tabldot menus with specific dates (Kahvaltı, Öğle, Akşam Yemeği).
- 'official_pricing': Official ceiling price and grammage board lists (Gramaj ve Fiyat Listesi, Tavan Fiyat) containing meal types (Kahvaltı, Yemek/Öğle/Akşam), categories/items, portion weights, and ceiling prices in TL.
- 'takeaway_package': Takeaway meal packages (Al Götür Menü 1, 2, 3...) with customizable selection slots (Sandviç, İçecek, Meyve/Tatlı, Su vb.).
- 'compound': Contains multiple distinct data types (e.g. daily menus alongside takeaway slots or price lists).

Important classification rules:
- If a document contains a table/grid of meals across days or weeks with portion amounts (e.g. 250g, 200g, 100g) but without explicit monetary prices in TL, it is a 'daily_menu', NOT 'official_pricing'.
- Official pricing boards must specify monetary prices (Ücret, Fiyat, TL). An item with 0 TL is not a valid pricing item unless explicitly labeled as free.
- If the document is rotated sideways or upside down, first observe the orientation and attempt to read the content in its correct reading direction. If the table layout contains dates or days of the week, extract it as 'daily_menu'.

MANDATORY EXTRACTION CONSTRAINTS:
1. If daily_menu: You MUST extract EVERY SINGLE DAY from the table into the 'days' array with ISO dates (YYYY-MM-DD), date_raw, meal_type, food items, amounts, calories, and alternatives. You must NEVER omit or return an empty 'days' array when a meal table is visible. Do NOT include 'takeaway' or 'pricing_board' keys if the document does not contain them.
2. If official_pricing: Extract every single row into 'pricing_board.items' with meal_type ('breakfast' for Kahvalti, 'dinner' for Aksam/Yemek, 'lunch' for Ogle), category_name (item name in uppercase), portion_amount, and numeric price in TL. If dates or academic year are stated (e.g. 2026-2027), extract period_start and period_end. Do NOT include 'days' or 'takeaway'.
3. If takeaway_package: Extract each package into 'takeaway.packages', including slots (index, title, is_required) and alternative item names and portions. Do NOT include 'days' or 'pricing_board'.
4. If compound: Extract both 'days' and 'takeaway' / 'pricing_board' as present.
Output strictly conforming to the requested JSON schema.";

pub const TABLE_GRID_EXTRACTION_PROMPT: &str = "Bu menü tablosunu CSV formatına dönüştür.
Sütunlar: Tarih,Gün,Yemek / Ürün,Gramaj,Enerji
Kurallar:
1. Her yemek kalemini kendi satırında göster.
2. Tablodaki tüm günleri (1'den 31'e kadar) eksiksiz aktar.
3. Varsa tablonun altındaki dipnot ve idari kuralları en sona ekle.
4. Sadece CSV tablosunu sun.";

pub const TAGGED_GRID_EXTRACTION_PROMPT: &str = r#"Bu sayfadaki menü, paket veya fiyat tablosunu yapılandırılmış etiketli metin blokları olarak aktar.

1. Sayfanın en başına belge türünü belirten tek satırlık başlık koy:
   [BELGE_TURU: AKŞAM] veya [BELGE_TURU: KAHVALTI] veya [BELGE_TURU: AL_GÖTÜR] veya [BELGE_TURU: FIYAT_LISTESI]

2. Standart menü tablosu varsa [TABLO] bloğu altına CSV formatında yaz:
   [TABLO]
   Tarih,Gün,Öğün,Yemek / Ürün,Gramaj,Kalori / Enerji,Günlük Toplam Kalori
   - Tablodaki her bir yemek/ürün satırını ayrı satır olarak yaz.
   - Alternatifli yemekleri varsa '/' ile veya 'veya' ile ayrıldığı gibi yaz.
   - Yemeklerin kendi gramaj ve kalorisi varsa Gramaj ve Kalori sütununa yaz.
   - Günlük toplam kalori belirtilmişse (ör. 1178 kcal) Günlük Toplam Kalori sütununa yaz.

3. Al götür / paket menü tanımları (Paket reçeteleri / Menü 1-4 vb.) varsa [PAKETLER] bloğu altına yaz:
   [PAKETLER]
   Paket No / Adı: ...
   İçerik / Yuvalar:
   - 1. Ürün / Seçenekler: ... (Gramaj)
   - 2. Ürün / Seçenekler: ... (Gramaj)

4. Paket menünün takvim dağılımı (hangi gün hangi paket verilecek) varsa [TAKVİM] bloğu altına CSV olarak yaz:
   [TAKVİM]
   Tarih,Gün,Paket Adı / No,Ekstralar / Notlar

5. Sayfada yer alan dipnotlar, idari kurallar, piknik paket veya diyet notları varsa [DIPNOTLAR] bloğu altına yaz:
   [DIPNOTLAR]
   - ...

Kurallar:
- Sadece bu etiketli blokları ([BELGE_TURU], [TABLO], [PAKETLER], [TAKVİM], [DIPNOTLAR]) kullan.
- Sohbet veya açıklama metni ekleme."#;

pub fn build_csv_to_menu_prompt(csv_content: &str) -> String {
    format!(
        r#"Aşağıdaki menü CSV tablosunu belirtilen JSON şemasına dönüştür.

CSV Verisi:
```csv
{}
```

Kurallar:
1. Her günü 'days' dizisinde ISO formatında (YYYY-MM-DD) tarih, date_raw, meal_type ve o güne ait yemek kalemleri ('items': name, amount, calories, alternatives) ile listele. Tablodaki tüm günleri eksiksiz aktar, hiçbir günü atlama.
2. Belge kahvaltı menüsü ise meal_type='breakfast', akşam yemeği ise meal_type='dinner' yap.
3. Varsa dipnotlardaki alternatif kurallarını (örn. tulum peynirine alternatif beyaz peynir) ilgili günün yemek kalemlerine alternatif olarak ekle.
4. Sadece JSON çıktısı ver."#,
        csv_content
    )
}

pub fn build_tagged_grid_to_payload_prompt(grid_content: &str) -> String {
    format!(
        r#"Aşağıdaki yapılandırılmış etiketli menü/fiyat verisini belirtilen JSON şemasına dönüştür.

Girdi Verisi:
```text
{}
```

Kurallar:
1. Belge Türü Tespiti (document_type):
   - Girdide [TABLO] bloğu varsa ve parasal fiyat içermeyen yemek listesi ise document_type KESİNLİKLE 'daily_menu' (veya paketler de varsa 'compound') olmalıdır.
   - Eğer sadece paket/al götür reçeteleri varsa 'takeaway_package'.
   - Eğer fiyat listesi / gramaj panosu ise 'official_pricing'.

2. Günlük Menü Zorunluluğu (days dizisi - KRİTİK):
   - document_type 'daily_menu' veya 'compound' ise 'days' dizisi KESİNLİKLE ZORUNLUDUR. ASLA boş dizi [] veya null bırakılamaz.
   - Tablodaki TÜM günleri (1. günden son güne kadar, 30/31 gün) eksiksiz olarak 'days' dizisine aktaracaksın. Herhangi bir günü veya satırı atlamak kesinlikle yasaktır.
   - Her günü ISO 8601 YYYY-MM-DD formatında 'date', kaynak metindeki haliyle 'date_raw' ve 'meal_type' ('breakfast' / 'dinner' / 'lunch') ile aktar.
   - Tabloda veya dipnotta o güne ait günlük toplam kalori varsa 'calories' alanına yaz (ör. '1178 kcal').
   - Her yemek kalemini 'items' dizisinde: 'name', 'amount' (gramaj), 'calories' (yemek kalorisi) olarak ayıkla.
   - Alternatif seçenekler varsa (örn. Çorba alternatifi veya Meyve/Tatlı alternatifi) 'alternatives' dizisine ekle.

3. Olmayan Alanları Dahil Etmeme (Negatif Kısıt):
   - Girdide al götür / paket reçetesi yoksa 'takeaway' alanını JSON çıktısına HİÇ EKLEME (boş nesne veya boş dizi olarak dahi koyma).
   - Girdide parasal tavan fiyat listesi yoksa 'pricing_board' alanını JSON çıktısına HİÇ EKLEME.

4. Sadece JSON formatında çıktı ver."#,
        grid_content
    )
}

pub fn daily_menu_table_response_schema() -> serde_json::Value {
    let item_schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Standardized Turkish name of the food item."
            },
            "amount": {
                "type": "string",
                "description": "Portion size or grammage if explicitly present in table (e.g. '250 g', '1 Adet')."
            },
            "calories": {
                "type": "string",
                "description": "Calories if explicitly present in table (e.g. '350 kcal', '120-160')."
            },
            "alternatives": {
                "type": "array",
                "description": "List of alternative options if this dish offers choices (e.g. 'Tavuk Sote / Kuru Fasulye').",
                "items": { "type": "string" }
            }
        },
        "required": ["name"]
    });

    let day_schema = json!({
        "type": "object",
        "properties": {
            "date": {
                "type": "string",
                "description": "Strict ISO 8601 date string (YYYY-MM-DD)."
            },
            "date_raw": {
                "type": "string",
                "description": "Verbatim date text as written in the source table (e.g. '01.09.2026')."
            },
            "meal_type": {
                "type": "string",
                "enum": ["breakfast", "dinner", "lunch"],
                "description": "'breakfast' for Kahvalti, 'dinner' for Aksam/Yemek, 'lunch' for Ogle."
            },
            "items": {
                "type": "array",
                "description": "Dishes served on this day.",
                "items": item_schema
            }
        },
        "required": ["date", "items"]
    });

    json!({
        "type": "object",
        "description": "Schema for daily menu table extraction.",
        "properties": {
            "document_type": {
                "type": "string",
                "enum": ["daily_menu"]
            },
            "period": {
                "type": "string",
                "description": "Period or month name if present (e.g. 'Eylül 2026')."
            },
            "is_colyak": {
                "type": "boolean",
                "description": "True only if this is specifically a Celiac/Glutensiz menu."
            },
            "days": {
                "type": "array",
                "description": "Every single row of daily meals extracted from the CSV table.",
                "items": day_schema
            }
        },
        "required": ["document_type", "days"]
    })
}

fn is_chatter_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    let trimmed = lower.trim();
    if trimmed.is_empty() {
        return true;
    }
    trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.contains("umarım")
        || trimmed.contains("yardımcı")
        || trimmed.contains("rica ederim")
        || trimmed.contains("iyi çalışmalar")
        || trimmed.contains("kolay gelsin")
        || trimmed.contains("afiyet olsun")
        || trimmed.contains("başka bir sorunuz")
        || trimmed.contains("herhangi bir sorunuz")
        || trimmed.contains("lütfen iletin")
        || trimmed.contains("sorunuz olursa")
        || trimmed.contains("tablo aşağıda")
        || trimmed.contains("işte tablonun")
        || trimmed.contains("işte menü")
        || trimmed.contains("aşağıdaki tablo")
        || trimmed.contains("tabloyu inceledim")
        || trimmed.contains("aktarılmıştır")
        || trimmed.contains("dönüştürülmüştür")
        || trimmed.contains("hope this helps")
        || trimmed.contains("let me know")
        || trimmed.contains("feel free")
        || trimmed.contains("here is the")
        || trimmed.contains("certainly")
        || trimmed.contains("as requested")
}

fn is_table_header_or_first_row(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() || is_chatter_line(t) {
        return false;
    }

    let has_delim = t.contains(',') || t.contains(';') || t.contains('\t') || t.contains('|');
    if !has_delim {
        return false;
    }

    let lower = t.to_lowercase();
    lower.contains("tarih")
        || lower.contains("gün")
        || lower.contains("gun")
        || lower.contains("yemek")
        || lower.contains("ürün")
        || lower.contains("urun")
        || lower.contains("kahvalt")
        || lower.contains("akşam")
        || lower.contains("aksam")
        || lower.contains("öğle")
        || lower.contains("ogle")
        || lower.contains("menü")
        || lower.contains("menu")
        || lower.contains("kalori")
        || lower.contains("enerji")
        || lower.contains("gramaj")
        || lower.contains("miktar")
        || lower.contains("porsiyon")
        || lower.contains("fiyat")
        || lower.contains("pazartesi")
        || lower.contains("perşembe")
        || lower.contains("cuma")
        || t.contains(".202")
        || t.contains("-202")
        || t.contains("/202")
}

fn is_table_row_or_footnote(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() || is_chatter_line(t) {
        return false;
    }

    if t.starts_with("---")
        || t.starts_with('*')
        || t.starts_with('•')
        || t.starts_with("- ")
        || t.to_lowercase().starts_with("not:")
        || t.to_lowercase().starts_with("not :")
        || t.to_lowercase().starts_with("dipnot:")
        || t.to_lowercase().starts_with("dipnot :")
    {
        return true;
    }

    let comma_count = t.chars().filter(|&c| c == ',').count();
    let semi_count = t.chars().filter(|&c| c == ';').count();
    let tab_count = t.chars().filter(|&c| c == '\t').count();
    let pipe_count = t.chars().filter(|&c| c == '|').count();

    if comma_count >= 2 || semi_count >= 2 || tab_count >= 2 || pipe_count >= 3 {
        return true;
    }

    let has_date = t.contains(".202") || t.contains("-202") || t.contains("/202");
    if has_date && (comma_count >= 1 || semi_count >= 1 || tab_count >= 1) {
        return true;
    }

    false
}

pub fn clean_csv_markdown(raw: &str) -> &str {
    let trimmed = raw.trim();

    // 1. Kod bloğu ayıklama: Çıktıda ``` bloğu varsa, bloğun içindeki metni aday olarak seç.
    let candidate = if let Some(start_pos) = trimmed.find("```") {
        let after_fence = &trimmed[start_pos + 3..];
        let content_start = if let Some(nl) = after_fence.find('\n') {
            let tag = after_fence[..nl].trim().to_lowercase();
            if tag.is_empty() || tag == "csv" || tag == "tsv" || tag == "text" || tag == "markdown"
            {
                nl + 1
            } else {
                0
            }
        } else {
            0
        };
        let inner = &after_fence[content_start..];
        if let Some(end_pos) = inner.find("```") {
            inner[..end_pos].trim()
        } else {
            inner.trim()
        }
    } else {
        trimmed
    };

    // 2. Tablo sınırlarını bul: Başlangıç başlık/veri satırı ile son geçerli veri/dipnot satırı arasındaki bloğu kes.
    let lines: Vec<&str> = candidate.lines().collect();
    if let Some(first_idx) = lines.iter().position(|l| is_table_header_or_first_row(l)) {
        let table_slice = &lines[first_idx..];
        if let Some(last_offset) = table_slice
            .iter()
            .rposition(|l| is_table_row_or_footnote(l))
        {
            let start_byte = lines[first_idx].as_ptr() as usize - candidate.as_ptr() as usize;
            let end_line = table_slice[last_offset];
            let end_byte =
                (end_line.as_ptr() as usize + end_line.len()) - candidate.as_ptr() as usize;
            if start_byte <= end_byte && end_byte <= candidate.len() {
                return candidate[start_byte..end_byte].trim();
            }
        }
    }

    candidate
}

/// Çıkarılan ham CSV ızgarasının geçerli bir tablo olup olmadığını doğrular.
pub fn validate_table_grid(csv_content: &str) -> Result<()> {
    let table_part = csv_content
        .split("---DIPNOTLAR---")
        .next()
        .unwrap_or(csv_content);

    let lines: Vec<&str> = table_part
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if lines.len() < 2 {
        anyhow::bail!(
            "Tablo ızgarası yetersiz (en az 2 satır bekleniyordu, {} satır bulundu)",
            lines.len()
        );
    }

    let has_date_or_keyword = lines.iter().any(|line| {
        let lower = line.to_lowercase();
        lower.contains("kahvalt")
            || lower.contains("akşam")
            || lower.contains("aksam")
            || lower.contains("öğle")
            || lower.contains("ogle")
            || lower.contains("menü")
            || lower.contains("menu")
            || lower.contains("tarih")
            || lower.contains("pazartesi")
            || lower.contains("salı")
            || lower.contains("çarşamba")
            || lower.contains("perşembe")
            || lower.contains("cuma")
            || lower.contains("cumartesi")
            || lower.contains("pazar")
            || lower.contains("fiyat")
            || lower.contains("ücret")
            || lower.contains("çorba")
            || lower.contains("corba")
            || lower.contains("paket")
            || line.contains(".202")
            || line.contains("-202")
            || line.contains("/202")
    });

    if !has_date_or_keyword {
        anyhow::bail!(
            "Tablo ızgarasında geçerli bir menü, tarih veya öğün anahtar kelimesi tespit edilemedi"
        );
    }

    Ok(())
}

/// Etiketli ızgara (tagged grid) içeriğinden markdown çitlerini ve gereksiz kalıntıları temizler.
pub fn clean_tagged_grid(raw: &str) -> &str {
    let trimmed = raw.trim();

    if let Some(start_pos) = trimmed.find("```") {
        let after_fence = &trimmed[start_pos + 3..];
        let content_start = if let Some(nl) = after_fence.find('\n') {
            let tag = after_fence[..nl].trim().to_lowercase();
            if tag.is_empty() || tag == "text" || tag == "csv" || tag == "tsv" || tag == "markdown"
            {
                nl + 1
            } else {
                0
            }
        } else {
            0
        };
        let inner = &after_fence[content_start..];
        if let Some(end_pos) = inner.find("```") {
            inner[..end_pos].trim()
        } else {
            inner.trim()
        }
    } else {
        trimmed
    }
}

/// Etiketli ızgara içeriğinin geçerli bloklar veya menü satırları içerip içermediğini doğrular.
pub fn validate_tagged_grid(content: &str) -> Result<()> {
    let trimmed = content.trim();
    if trimmed.len() < 10 {
        anyhow::bail!("Etiketli ızgara içeriği çok kısa veya boş");
    }

    let lines: Vec<&str> = trimmed
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !is_chatter_line(l))
        .collect();

    if lines.len() < 2 {
        anyhow::bail!(
            "Izgara satır sayısı yetersiz (en az 2 satır bekleniyordu, {} satır bulundu)",
            lines.len()
        );
    }

    let has_tagged_block = lines.iter().any(|line| {
        line.starts_with("[BELGE_TURU")
            || line.starts_with("[TABLO")
            || line.starts_with("[PAKETLER")
            || line.starts_with("[TAKVİM")
            || line.starts_with("[TAKVIM")
            || line.starts_with("[DIPNOTLAR")
            || line.starts_with("[DİPNOTLAR")
    });

    let has_date_or_keyword = lines.iter().any(|line| {
        let lower = line.to_lowercase();
        lower.contains("kahvalt")
            || lower.contains("akşam")
            || lower.contains("aksam")
            || lower.contains("öğle")
            || lower.contains("ogle")
            || lower.contains("menü")
            || lower.contains("menu")
            || lower.contains("tarih")
            || lower.contains("pazartesi")
            || lower.contains("salı")
            || lower.contains("sali")
            || lower.contains("çarşamba")
            || lower.contains("carsamba")
            || lower.contains("perşembe")
            || lower.contains("persembe")
            || lower.contains("cuma")
            || lower.contains("cumartesi")
            || lower.contains("pazar")
            || lower.contains("fiyat")
            || lower.contains("ücret")
            || lower.contains("çorba")
            || lower.contains("corba")
            || lower.contains("paket")
            || lower.contains("sandviç")
            || lower.contains("sandvic")
            || line.contains(".202")
            || line.contains("-202")
            || line.contains("/202")
    });

    if !has_tagged_block && !has_date_or_keyword {
        anyhow::bail!(
            "Izgara içeriğinde geçerli bir etiket bloğu veya menü anahtar kelimesi bulunamadı"
        );
    }

    Ok(())
}

/// Sağlayıcı zincirini (OpenRouter -> Gemini) çalıştırır ve modelden gelen ham metni döner.
pub async fn call_llm_chain_raw(
    client: &Client,
    req: &LlmRequest<'_>,
    gemini_api_key: Option<&str>,
) -> Result<String> {
    let providers = resolve_provider_order();
    let openrouter_key = openrouter_api_key();
    let openrouter_model = openrouter_model();
    let openrouter_effort = openrouter_reasoning_effort();
    let gemini_models = resolve_gemini_models();
    let thinking_level = resolve_thinking_level();

    const MAX_ATTEMPTS_PER_MODEL: usize = 2;
    let mut last_error = String::new();
    let mut attempted_any = false;

    let passes = chain_passes();
    let base_delay_ms = chain_retry_delay_ms();
    let mut plan: Vec<(usize, LlmProvider)> = Vec::with_capacity(passes * providers.len());
    for pass in 1..=passes {
        for provider in providers.iter().copied() {
            plan.push((pass, provider));
        }
    }

    let mut current_pass = 0usize;
    for (pass, provider) in plan {
        if pass != current_pass {
            current_pass = pass;
            if pass > 1 {
                let wait_ms = base_delay_ms * (pass as u64 - 1);
                tracing::warn!(
                    "LLM zinciri {}/{} kez denenecek ({} ms bekleniyor). Son hata: {}",
                    pass,
                    passes,
                    wait_ms,
                    last_error
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
            }
        }

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

                    match call_openrouter(client, key, &openrouter_model, &openrouter_effort, req)
                        .await
                    {
                        Ok(text) => return Ok(text),
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

                        match call_gemini(client, key, model_name, &thinking_level, req).await {
                            Ok(text) => return Ok(text),
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
                        if is_model_switchable_error(&last_error) {
                            let wait = model_switch_delay_ms();
                            tracing::warn!(
                                "  Yoğunluk/kota hatası; {} ms sonra sonraki modele geçilecek.",
                                wait
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
                        }
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

    anyhow::bail!(
        "Tüm LLM sağlayıcıları başarısız oldu ({} tur). Son hata: {}",
        passes,
        last_error
    );
}

/// Görsel veya PDF belgeden saf CSV ızgarasını çıkarır (Aşama 1).
pub async fn extract_table_grid_csv(
    client: &Client,
    file_bytes: &[u8],
    mime_type: &str,
    gemini_api_key: Option<&str>,
) -> Result<String> {
    let base64_data = BASE64.encode(file_bytes);
    let input_type = if mime_type == "application/pdf" {
        "document"
    } else {
        "image"
    };

    let req = LlmRequest {
        prompt: TABLE_GRID_EXTRACTION_PROMPT,
        mime_type,
        base64_data: &base64_data,
        input_type,
        schema: None,
    };

    let raw = call_llm_chain_raw(client, &req, gemini_api_key).await?;
    let cleaned = clean_csv_markdown(&raw);
    Ok(cleaned.to_string())
}

/// Doğrulanmış CSV metnini tipli çok biçimli JSON veri yüküne dönüştürür (Aşama 2).
pub async fn parse_csv_to_polymorphic(
    client: &Client,
    csv_content: &str,
    file_name_hint: &str,
    gemini_api_key: Option<&str>,
) -> Result<(crate::parser::models::ParsedDocumentPayload, Vec<String>)> {
    // Önce deterministik Rust CSV ayrıştırıcısını dene (LLM çağrısı, gecikme ve halüsinasyon yok)
    if let Ok(payload) =
        crate::parser::csv_grid::parse_csv_grid_to_payload(csv_content, file_name_hint)
    {
        tracing::info!("CSV tablosu saf Rust (csv_grid) ile deterministik olarak ayrıştırıldı.");
        return Ok((payload, Vec::new()));
    }

    let prompt = build_csv_to_menu_prompt(csv_content);
    let schema = daily_menu_table_response_schema();

    let req = LlmRequest {
        prompt: &prompt,
        mime_type: "text/plain",
        base64_data: "",
        input_type: "text",
        schema: Some(&schema),
    };

    let raw_json = call_llm_chain_raw(client, &req, gemini_api_key).await?;
    tracing::info!(
        "Aşama 2 LLM ham yanıtı ({} karakter): {}",
        raw_json.len(),
        raw_json.chars().take(200).collect::<String>()
    );
    parse_and_finalize_polymorphic(&raw_json, file_name_hint)
}

/// Görsel veya PDF sayfadan etiketli ızgarayı çıkarır (Aşama 1).
pub async fn extract_page_tagged_grid(
    client: &Client,
    page_bytes: &[u8],
    mime_type: &str,
    gemini_api_key: Option<&str>,
) -> Result<String> {
    let base64_data = BASE64.encode(page_bytes);
    let input_type = if mime_type == "application/pdf" {
        "document"
    } else {
        "image"
    };

    let req = LlmRequest {
        prompt: TAGGED_GRID_EXTRACTION_PROMPT,
        mime_type,
        base64_data: &base64_data,
        input_type,
        schema: None,
    };

    let raw = call_llm_chain_raw(client, &req, gemini_api_key).await?;
    let cleaned = clean_tagged_grid(&raw);
    Ok(cleaned.to_string())
}

/// Doğrulanmış etiketli ızgara metnini tipli çok biçimli JSON veri yüküne dönüştürür (Aşama 2).
pub async fn parse_tagged_grid_to_payload(
    client: &Client,
    grid_content: &str,
    file_name_hint: &str,
    gemini_api_key: Option<&str>,
) -> Result<(crate::parser::models::ParsedDocumentPayload, Vec<String>)> {
    // Önce deterministik Rust CSV ayrıştırıcısını dene (LLM çağrısı, gecikme ve halüsinasyon yok)
    if let Ok(payload) =
        crate::parser::csv_grid::parse_csv_grid_to_payload(grid_content, file_name_hint)
    {
        tracing::info!(
            "Etiketli ızgara saf Rust (csv_grid) ile deterministik olarak ayrıştırıldı."
        );
        return Ok((payload, Vec::new()));
    }

    let prompt = build_tagged_grid_to_payload_prompt(grid_content);
    let schema = unified_document_response_schema();

    let req = LlmRequest {
        prompt: &prompt,
        mime_type: "text/plain",
        base64_data: "",
        input_type: "text",
        schema: Some(&schema),
    };

    let raw_json = call_llm_chain_raw(client, &req, gemini_api_key).await?;
    tracing::info!(
        "Aşama 2 LLM ham yanıtı ({} karakter): {}",
        raw_json.len(),
        raw_json.chars().take(200).collect::<String>()
    );
    parse_and_finalize_polymorphic(&raw_json, file_name_hint)
}

fn merge_into_menu_database(
    target: &mut crate::parser::models::MenuDatabase,
    incoming: crate::parser::models::MenuDatabase,
) {
    for (date_str, incoming_day) in incoming {
        match target.get_mut(&date_str) {
            Some(existing_day) => {
                merge_daily_menu(&mut existing_day.normal, incoming_day.normal);
                merge_daily_menu(&mut existing_day.colyak, incoming_day.colyak);
            }
            None => {
                target.insert(date_str, incoming_day);
            }
        }
    }
}

fn merge_daily_menu(
    target: &mut crate::parser::models::DailyMenu,
    incoming: crate::parser::models::DailyMenu,
) {
    if !incoming.breakfast.is_empty() {
        if target.breakfast.is_empty() {
            target.breakfast = incoming.breakfast;
        } else {
            target.breakfast.extend(incoming.breakfast);
        }
    }
    if incoming.breakfast_kcal.is_some() {
        target.breakfast_kcal = incoming.breakfast_kcal;
    }

    if !incoming.lunch.is_empty() {
        if target.lunch.is_empty() {
            target.lunch = incoming.lunch;
        } else {
            target.lunch.extend(incoming.lunch);
        }
    }
    if incoming.lunch_kcal.is_some() {
        target.lunch_kcal = incoming.lunch_kcal;
    }

    if !incoming.dinner.is_empty() {
        if target.dinner.is_empty() {
            target.dinner = incoming.dinner;
        } else {
            target.dinner.extend(incoming.dinner);
        }
    }
    if incoming.dinner_kcal.is_some() {
        target.dinner_kcal = incoming.dinner_kcal;
    }
}

/// Çok sayfalı veya tek sayfalı belgelerden elde edilen sayfa yüklerini deterministik olarak birleştirir.
pub fn merge_page_payloads(
    pages_results: Vec<(
        crate::parser::models::ParsedDocumentPayload,
        crate::parser::core::ParseDiagnostics,
    )>,
) -> Result<(
    crate::parser::models::ParsedDocumentPayload,
    crate::parser::core::ParseDiagnostics,
)> {
    if pages_results.is_empty() {
        anyhow::bail!("Birleştirilecek sayfa sonucu yok.");
    }

    if pages_results.len() == 1 {
        let (payload, diag) = pages_results.into_iter().next().unwrap();
        return Ok((payload, diag));
    }

    let mut merged_menu: Option<crate::parser::models::MenuDatabase> = None;
    let mut merged_pricing: Option<crate::parser::models::OfficialPricingData> = None;
    let mut merged_takeaway: Option<crate::parser::models::TakeawayData> = None;

    let mut combined_mismatches = Vec::new();
    let mut combined_grid_csv = String::new();
    let mut any_orientation_uncertain = false;

    for (page_idx, (payload, diag)) in pages_results.into_iter().enumerate() {
        let page_num = page_idx + 1;
        combined_mismatches.extend(diag.date_raw_mismatches);
        if diag.orientation_uncertain {
            any_orientation_uncertain = true;
        }
        if let Some(grid) = diag.table_grid_csv {
            if !combined_grid_csv.is_empty() {
                combined_grid_csv.push_str("\n\n");
            }
            combined_grid_csv.push_str(&format!("--- SAYFA {} ---\n{}", page_num, grid));
        }

        match payload {
            crate::parser::models::ParsedDocumentPayload::DailyMenu(db) => {
                merge_into_menu_database(
                    merged_menu.get_or_insert_with(std::collections::HashMap::new),
                    db,
                );
            }
            crate::parser::models::ParsedDocumentPayload::OfficialPricing(pricing) => {
                if let Some(existing) = &mut merged_pricing {
                    existing.items.extend(pricing.items);
                    if existing.period_start.is_none() {
                        existing.period_start = pricing.period_start;
                    }
                    if existing.period_end.is_none() {
                        existing.period_end = pricing.period_end;
                    }
                    if existing.academic_year.is_none() {
                        existing.academic_year = pricing.academic_year;
                    }
                } else {
                    merged_pricing = Some(pricing);
                }
            }
            crate::parser::models::ParsedDocumentPayload::Takeaway(takeaway) => {
                if let Some(existing) = &mut merged_takeaway {
                    existing.packages.extend(takeaway.packages);
                    if existing.academic_year.is_none() {
                        existing.academic_year = takeaway.academic_year;
                    }
                } else {
                    merged_takeaway = Some(takeaway);
                }
            }
            crate::parser::models::ParsedDocumentPayload::Compound {
                menu,
                pricing,
                takeaway,
            } => {
                if let Some(m) = menu {
                    merge_into_menu_database(
                        merged_menu.get_or_insert_with(std::collections::HashMap::new),
                        m,
                    );
                }
                if let Some(p) = pricing {
                    if let Some(existing) = &mut merged_pricing {
                        existing.items.extend(p.items);
                    } else {
                        merged_pricing = Some(p);
                    }
                }
                if let Some(t) = takeaway {
                    if let Some(existing) = &mut merged_takeaway {
                        existing.packages.extend(t.packages);
                    } else {
                        merged_takeaway = Some(t);
                    }
                }
            }
        }
    }

    let final_diag = crate::parser::core::ParseDiagnostics {
        date_raw_mismatches: combined_mismatches,
        table_grid_csv: if combined_grid_csv.is_empty() {
            None
        } else {
            Some(combined_grid_csv)
        },
        orientation_uncertain: any_orientation_uncertain,
        ..Default::default()
    };

    let payload = match (merged_menu, merged_pricing, merged_takeaway) {
        (Some(menu), None, None) => crate::parser::models::ParsedDocumentPayload::DailyMenu(menu),
        (None, Some(pricing), None) => {
            crate::parser::models::ParsedDocumentPayload::OfficialPricing(pricing)
        }
        (None, None, Some(takeaway)) => {
            crate::parser::models::ParsedDocumentPayload::Takeaway(takeaway)
        }
        (menu, pricing, takeaway) => crate::parser::models::ParsedDocumentPayload::Compound {
            menu,
            pricing,
            takeaway,
        },
    };

    Ok((payload, final_diag))
}

/// İki aşamalı sayfa bazlı tablo çıkarım hattı:
/// 1. Aşama: Sayfa Görseli -> Etiketli Izgara / CSV (extract_page_tagged_grid)
/// 2. Aşama: Rust deterministik ızgara denetimi (validate_tagged_grid)
/// 3. Aşama: Doğrulanmış Izgara -> Tipli çok biçimli JSON (parse_tagged_grid_to_payload)
/// 4. Aşama: Sayfalar arası deterministik birleştirme (merge_page_payloads)
pub async fn parse_document_two_stage(
    client: &Client,
    file_bytes: &[u8],
    mime_type: &str,
    file_name_hint: &str,
    gemini_api_key: Option<&str>,
) -> Result<(
    crate::parser::models::ParsedDocumentPayload,
    crate::parser::core::ParseDiagnostics,
)> {
    tracing::info!("İki aşamalı sayfa bazlı tablo çıkarımı başlatılıyor...");

    // Belgeyi sayfa bazında ayrıştır ve dik (0° upright) konuma getir
    let pages = crate::parser::orientation::prepare_pages_for_extraction(file_bytes, mime_type);
    let total_pages = pages.len();
    tracing::info!(
        "Belge {} sayfa olarak ayrıştırıldı, sayfa sayfa etiketli ızgara çıkarımı yapılacak.",
        total_pages
    );

    let mut page_results = Vec::with_capacity(total_pages);

    for page in pages {
        tracing::info!(
            "Sayfa {}/{} işleniyor (Aşama 1: Etiketli Izgara Çıkarımı)...",
            page.page_number,
            total_pages
        );

        let grid_content =
            extract_page_tagged_grid(client, &page.bytes, &page.mime_type, gemini_api_key).await?;

        tracing::info!(
            "Sayfa {}/{} ızgarası başarıyla çıkarıldı ({} karakter), denetim yapılıyor...",
            page.page_number,
            total_pages,
            grid_content.len()
        );
        validate_tagged_grid(&grid_content)?;

        tracing::info!(
            "Sayfa {}/{} ızgarası doğrulandı, Aşama 2 (tipli çok biçimli JSON) başlatılıyor...",
            page.page_number,
            total_pages
        );
        let (payload, mismatches) =
            parse_tagged_grid_to_payload(client, &grid_content, file_name_hint, gemini_api_key)
                .await?;

        page_results.push((
            payload,
            crate::parser::core::ParseDiagnostics {
                date_raw_mismatches: mismatches,
                table_grid_csv: Some(grid_content),
                ..Default::default()
            },
        ));
    }

    tracing::info!(
        "Tüm sayfalar ({}) başarıyla ayrıştırıldı, yükler birleştiriliyor...",
        total_pages
    );
    merge_page_payloads(page_results)
}

pub async fn parse_document_with_llm_polymorphic(
    client: &Client,
    gemini_api_key: Option<&str>,
    file_path: &Path,
) -> Result<(
    crate::parser::models::ParsedDocumentPayload,
    crate::parser::core::ParseDiagnostics,
)> {
    tracing::info!(
        "Belge LLM (çok biçimli) ile ayrıştırılıyor: {:?}",
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
    let original_mime = detect_mime_type(file_path, &file_bytes);

    let corrected = crate::parser::orientation::correct_document(&file_bytes, original_mime);
    let orientation_uncertain = !corrected.attempted_correction;
    if corrected.corrected_pages > 0 {
        tracing::info!(
            "Yön düzeltmesi uygulandı: {} sayfa, {} düzeltildi ({:?}).",
            corrected.pages,
            corrected.corrected_pages,
            file_path
        );
    }

    let mime_type = corrected.mime_type.as_str();
    let file_name_hint = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document_llm");

    // Öncelikli olarak Google Developer Knowledge ilkelerine dayalı iki aşamalı
    // tablo çıkarım hattı denenir. Eğer belge tablo içermiyorsa veya ızgara
    // denetiminden geçemezse tek aşamalı genel ayrıştırıcıya güvenle düşülür.
    match parse_document_two_stage(
        client,
        &corrected.bytes,
        mime_type,
        file_name_hint,
        gemini_api_key,
    )
    .await
    {
        Ok((payload, mut diag)) => {
            diag.orientation_uncertain = orientation_uncertain;
            tracing::info!("İki aşamalı tablo çıkarımı başarıyla tamamlandı.");
            return Ok((payload, diag));
        }
        Err(e) => {
            tracing::warn!(
                "İki aşamalı tablo çıkarımı uygulanamadı veya başarısız ({}), tek aşamalı ayrıştırıcıya geçiliyor.",
                e
            );
        }
    }

    let base64_data = BASE64.encode(&corrected.bytes);
    let prompt = UNIFIED_EXTRACTION_PROMPT;
    let schema = unified_document_response_schema();

    let input_type = if mime_type == "application/pdf" {
        "document"
    } else {
        "image"
    };

    let llm_req = LlmRequest {
        prompt,
        mime_type,
        base64_data: &base64_data,
        input_type,
        schema: Some(&schema),
    };

    let text = call_llm_chain_raw(client, &llm_req, gemini_api_key).await?;
    let (payload, mismatches) = parse_and_finalize_polymorphic(&text, file_name_hint)?;
    Ok((
        payload,
        crate::parser::core::ParseDiagnostics {
            date_raw_mismatches: mismatches,
            orientation_uncertain,
            ..Default::default()
        },
    ))
}

pub async fn parse_document_with_llm(
    client: &Client,
    gemini_api_key: Option<&str>,
    file_path: &Path,
) -> Result<(MenuDatabase, crate::parser::core::ParseDiagnostics)> {
    let (payload, diag) =
        parse_document_with_llm_polymorphic(client, gemini_api_key, file_path).await?;
    match payload {
        crate::parser::models::ParsedDocumentPayload::DailyMenu(db) => Ok((db, diag)),
        crate::parser::models::ParsedDocumentPayload::Compound { menu: Some(db), .. } => {
            Ok((db, diag))
        }
        _ => {
            anyhow::bail!("Belge günlük tabldot menüsü değil (fiyat panosu veya al götür paketi).")
        }
    }
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

    /// 5xx/yoğunluk hataları da model/sağlayıcı devrini tetiklemeli.
    #[test]
    fn test_transient_5xx_is_switchable() {
        assert!(is_model_switchable_error(
            "Gemini API Error (502 Bad Gateway): upstream error"
        ));
        assert!(is_model_switchable_error(
            "API Error (504 Deadline Exceeded)"
        ));
        assert!(is_model_switchable_error("model is currently unavailable"));
        assert!(is_model_switchable_error("request timeout"));
        assert!(!is_model_switchable_error(
            "API Error (400 Bad Request): Unknown parameter 'thinking_budget'"
        ));
    }

    /// Zincir yeniden deneme ayarları sınırlı olmalı (env verilmese de).
    #[test]
    fn test_chain_retry_settings_bounds() {
        let passes = chain_passes();
        assert!((1..=5).contains(&passes), "tur sayısı 1..=5 olmalı");
        assert!(chain_retry_delay_ms() <= 30_000);
        assert!(model_switch_delay_ms() <= 30_000);
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
            "gemini-pro-latest".to_string(),
        ];
        let out = sanitize_gemini_models(input);
        assert_eq!(
            out,
            vec![
                "gemini-flash-latest".to_string(),
                "gemini-pro-latest".to_string()
            ]
        );

        assert!(is_forbidden_gemini_model("gemini-flash-lite-latest"));
        assert!(is_forbidden_gemini_model("GEMINI-FLASH-LITE-LATEST"));
        assert!(is_forbidden_gemini_model("gemini-3.5-flash-lite"));
        assert!(is_forbidden_gemini_model("gemini-2.5-flash"));
        assert!(!is_forbidden_gemini_model("gemini-flash-latest"));
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
    /// `#[ignore]` olduğu için CI'da ÇALIŞMAZ (ağ + API anahtarı gerektirir). Yerelde:
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

        let (db, _diag) = parse_document_with_llm(&client, gemini_key.as_deref(), &path)
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

    #[test]
    fn test_polymorphic_pricing_board_json_parsing() {
        let json = r#"{
            "document_type": "official_pricing",
            "city": "istanbul",
            "academic_year": "2026-2027",
            "pricing_board": {
                "items": [
                    {
                        "meal_type": "dinner",
                        "category_name": "1. GRUP YEMEKLER (ÇORBALAR)",
                        "portion_amount": "250 GR",
                        "price": 35.0
                    },
                    {
                        "meal_type": "dinner",
                        "category_name": "2. GRUP YEMEKLER (ANA YEMEKLER)",
                        "portion_amount": "200 GR",
                        "price": 95.0
                    }
                ]
            }
        }"#;

        let (payload, mismatches) =
            parse_and_finalize_polymorphic(json, "Istanbul_Tavan_Fiyat.jpg").unwrap();
        assert!(mismatches.is_empty());
        match payload {
            crate::parser::models::ParsedDocumentPayload::OfficialPricing(data) => {
                assert_eq!(data.city_slug.as_deref(), Some("istanbul"));
                assert_eq!(data.academic_year.as_deref(), Some("2026-2027"));
                assert_eq!(data.items.len(), 2);
                assert_eq!(data.items[0].category_name, "1. GRUP YEMEKLER (ÇORBALAR)");
                assert_eq!(data.items[0].portion_amount.as_deref(), Some("250 GR"));
                assert_eq!(data.items[0].price, sea_orm::prelude::Decimal::from(35));
                assert_eq!(data.items[1].price, sea_orm::prelude::Decimal::from(95));
            }
            _ => panic!("OfficialPricing bekleniyordu, başka tip döndü"),
        }
    }

    #[test]
    fn test_polymorphic_takeaway_json_parsing() {
        let json = r#"{
            "document_type": "takeaway_package",
            "city": "istanbul",
            "academic_year": "2026-2027",
            "takeaway": {
                "packages": [
                    {
                        "package_name": "Sandviç Menü Paketi",
                        "slots": [
                            {
                                "slot_index": 1,
                                "slot_title": "Ana Sandviç",
                                "is_required": true,
                                "items": [
                                    { "dish_name": "Kaşarlı Sandviç", "portion": "1 Adet" },
                                    { "dish_name": "Tavuklu Sandviç", "portion": "1 Adet" }
                                ]
                            },
                            {
                                "slot_index": 2,
                                "slot_title": "İçecek",
                                "is_required": true,
                                "items": [
                                    { "dish_name": "Ayran", "portion": "200 ml" },
                                    { "dish_name": "Meyve Suyu", "portion": "200 ml" }
                                ]
                            }
                        ]
                    }
                ]
            }
        }"#;

        let (payload, _mismatches) =
            parse_and_finalize_polymorphic(json, "Al_Gotur_Menu.jpg").unwrap();
        match payload {
            crate::parser::models::ParsedDocumentPayload::Takeaway(data) => {
                assert_eq!(data.city_slug.as_deref(), Some("istanbul"));
                assert_eq!(data.academic_year.as_deref(), Some("2026-2027"));
                assert_eq!(data.packages.len(), 1);
                let pkg = &data.packages[0];
                assert_eq!(pkg.package_name, "Sandviç Menü Paketi");
                assert_eq!(pkg.slots.len(), 2);
                assert_eq!(pkg.slots[0].items.len(), 2);
                assert_eq!(pkg.slots[0].items[0].dish_name, "Kaşarlı Sandviç");
                assert_eq!(pkg.slots[0].items[0].portion.as_deref(), Some("1 Adet"));
            }
            _ => panic!("Takeaway bekleniyordu, başka tip döndü"),
        }
    }

    #[test]
    fn test_polymorphic_daily_menu_json_parsing() {
        let json = r#"{
            "document_type": "daily_menu",
            "city": "istanbul",
            "meal_type": "dinner",
            "days": [
                {
                    "date": "2026-10-15",
                    "items": [
                        { "name": "Mercimek Çorbası", "calories": "180" },
                        { "name": "Orman Kebabı", "calories": "420" },
                        { "name": "Pirinç Pilavı", "calories": "250" },
                        { "name": "Ayran", "calories": "75" }
                    ]
                }
            ]
        }"#;

        let (payload, _mismatches) =
            parse_and_finalize_polymorphic(json, "Ekim_Menu.xlsx").unwrap();
        match payload {
            crate::parser::models::ParsedDocumentPayload::DailyMenu(db) => {
                assert_eq!(db.len(), 1);
                let day = db.get("2026-10-15").expect("günün menüsü olmalı");
                assert_eq!(day.normal.dinner.len(), 4);
                assert_eq!(
                    day.normal.dinner[0].alternatives[0].name,
                    "Mercimek Çorbası"
                );
                assert_eq!(
                    day.normal.dinner[0].alternatives[0].calories.as_deref(),
                    Some("180")
                );
            }
            _ => panic!("DailyMenu bekleniyordu, başka tip döndü"),
        }
    }

    #[test]
    fn test_clean_csv_markdown_strips_codeblocks() {
        let raw = "```csv\nTarih,Yemek\n01.10.2026,Mercimek Çorbası\n```";
        assert_eq!(
            clean_csv_markdown(raw),
            "Tarih,Yemek\n01.10.2026,Mercimek Çorbası"
        );

        let raw_no_block = "Tarih,Yemek\n01.10.2026,Mercimek Çorbası";
        assert_eq!(clean_csv_markdown(raw_no_block), raw_no_block);

        // Modelin kod bloğu öncesi ve sonrası konuşması
        let chatter_with_fence = "İşte talep ettiğiniz menü CSV tablosu:\n\n```csv\nTarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\n```\n\nBaşka bir sorunuz var mı?";
        assert_eq!(
            clean_csv_markdown(chatter_with_fence),
            "Tarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal"
        );

        // Kod bloğu olmadan sohbet satırları ve virgül içeren veda cümlesi
        let chatter_no_fence = "Tablo aşağıda sunulmuştur:\nTarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\n2.10.2026,Cuma,Menemen,150 g,106 kcal\n---DIPNOTLAR---\n*Zeytin çeşitleri mevcuttur.\nUmarım bu tablo işinize yarar, iyi çalışmalar dilerim.";
        assert_eq!(
            clean_csv_markdown(chatter_no_fence),
            "Tarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\n2.10.2026,Cuma,Menemen,150 g,106 kcal\n---DIPNOTLAR---\n*Zeytin çeşitleri mevcuttur."
        );

        // Kod bloğu İÇİNDE gevezelik ve selamlama
        let chatter_inside_fence = "```csv\n// Ekim 2026 Menü Tablosu\nTarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\nNot: Kahvaltı 07:30'da başlar.\nHerhangi bir sorunuz olursa lütfen iletin.\n```";
        assert_eq!(
            clean_csv_markdown(chatter_inside_fence),
            "Tarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\nNot: Kahvaltı 07:30'da başlar."
        );

        // Kapanmamış kod bloğu
        let unclosed_fence = "İşte CSV:\n```csv\nTarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\n2.10.2026,Cuma,Menemen,150 g,106 kcal";
        assert_eq!(
            clean_csv_markdown(unclosed_fence),
            "Tarih,Gün,Yemek / Ürün,Gramaj,Enerji\n1.10.2026,Perşembe,Haşlanmış Yumurta,1 adet L boy,77 kcal\n2.10.2026,Cuma,Menemen,150 g,106 kcal"
        );
    }

    #[test]
    fn test_validate_table_grid_accepts_valid_calendar() {
        let valid_csv = "Tarih,Öğün,Çorba,Ana Yemek\n01.10.2026,Akşam Yemeği,Mercimek Çorbası,Orman Kebabı\n02.10.2026,Akşam Yemeği,Ezogelin,Tavuk Sote";
        assert!(validate_table_grid(valid_csv).is_ok());

        let days_csv = "Pazartesi,Salı,Çarşamba\nEzogelin,Mercimek,Tarhana";
        assert!(validate_table_grid(days_csv).is_ok());
    }

    #[test]
    fn test_validate_table_grid_rejects_empty_or_non_table() {
        assert!(validate_table_grid("").is_err());
        assert!(validate_table_grid("tek satır").is_err());
        assert!(
            validate_table_grid("Lorem ipsum dolor sit amet\nconsectetur adipiscing elit").is_err()
        );
    }

    #[test]
    fn test_build_csv_to_menu_prompt_contains_csv() {
        let csv = "01.10.2026,Mercimek Çorbası";
        let prompt = build_csv_to_menu_prompt(csv);
        assert!(prompt.contains(csv));
        assert!(prompt.contains("ISO formatında"));
    }

    #[test]
    fn test_validate_tagged_grid_accepts_blocks_and_keywords() {
        let grid_with_tag =
            "[BELGE_TURU: AKŞAM]\n[TABLO]\nTarih,Yemek\n17.09.2026,Mercimek Çorbası";
        assert!(validate_tagged_grid(grid_with_tag).is_ok());

        let grid_takeaway = "[BELGE_TURU: AL_GÖTÜR]\n[PAKETLER]\nAl Götür Menü 1\n- Sandviç";
        assert!(validate_tagged_grid(grid_takeaway).is_ok());

        let grid_no_tag_but_valid =
            "Tarih,Öğün,Yemek,Gramaj\n01.10.2026,Akşam Yemeği,Etli Kuru Fasulye,250 g";
        assert!(validate_tagged_grid(grid_no_tag_but_valid).is_ok());
    }

    #[test]
    fn test_validate_tagged_grid_rejects_empty_or_chatter() {
        assert!(validate_tagged_grid("").is_err());
        assert!(validate_tagged_grid("kısa").is_err());
        assert!(
            validate_tagged_grid("Merhaba ben bir yapay zekayım.\nSize nasıl yardımcı olabilirim?")
                .is_err()
        );
    }

    #[test]
    fn test_clean_tagged_grid_strips_fences() {
        let text_with_fence = "```text\n[BELGE_TURU: AKŞAM]\n[TABLO]\n17.09.2026,Tavuk Sote\n```";
        assert_eq!(
            clean_tagged_grid(text_with_fence),
            "[BELGE_TURU: AKŞAM]\n[TABLO]\n17.09.2026,Tavuk Sote"
        );
    }

    #[test]
    fn test_merge_page_payloads_merges_breakfast_and_dinner_same_day() {
        use crate::parser::core::ParseDiagnostics;
        use crate::parser::models::{
            DailyMenu, DayData, MenuComponent, MenuItem, ParsedDocumentPayload,
        };
        use std::collections::HashMap;

        let mut menu1 = HashMap::new();
        menu1.insert(
            "2026-09-17".to_string(),
            DayData {
                normal: DailyMenu {
                    dinner: vec![MenuItem {
                        takeaway_id: None,
                        alternatives: vec![MenuComponent::from("Orman Kebabı")],
                    }],
                    dinner_kcal: Some("1178 kcal".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let mut menu2 = HashMap::new();
        menu2.insert(
            "2026-09-17".to_string(),
            DayData {
                normal: DailyMenu {
                    breakfast: vec![MenuItem {
                        takeaway_id: None,
                        alternatives: vec![MenuComponent::from("Haşlanmış Yumurta")],
                    }],
                    breakfast_kcal: Some("450 kcal".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let pages = vec![
            (
                ParsedDocumentPayload::DailyMenu(menu1),
                ParseDiagnostics::default(),
            ),
            (
                ParsedDocumentPayload::DailyMenu(menu2),
                ParseDiagnostics::default(),
            ),
        ];

        let (merged, _diag) = merge_page_payloads(pages).expect("birleştirme başarılı olmalı");
        match merged {
            ParsedDocumentPayload::DailyMenu(db) => {
                assert_eq!(db.len(), 1);
                let day = db.get("2026-09-17").expect("gün mevcut olmalı");
                assert_eq!(day.normal.dinner.len(), 1);
                assert_eq!(day.normal.dinner[0].alternatives[0].name, "Orman Kebabı");
                assert_eq!(day.normal.dinner_kcal.as_deref(), Some("1178 kcal"));

                assert_eq!(day.normal.breakfast.len(), 1);
                assert_eq!(
                    day.normal.breakfast[0].alternatives[0].name,
                    "Haşlanmış Yumurta"
                );
                assert_eq!(day.normal.breakfast_kcal.as_deref(), Some("450 kcal"));
            }
            _ => panic!("DailyMenu bekleniyordu"),
        }
    }

    #[test]
    fn test_merge_page_payloads_merges_daily_and_takeaway_into_compound() {
        use crate::parser::core::ParseDiagnostics;
        use crate::parser::models::{
            DailyMenu, DayData, MenuComponent, MenuItem, ParsedDocumentPayload, TakeawayData,
            TakeawayPackageData,
        };
        use std::collections::HashMap;

        let mut menu = HashMap::new();
        menu.insert(
            "2026-09-18".to_string(),
            DayData {
                normal: DailyMenu {
                    dinner: vec![MenuItem {
                        takeaway_id: None,
                        alternatives: vec![MenuComponent::from("Kuru Fasulye")],
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let takeaway = TakeawayData {
            city_slug: Some("bursa".to_string()),
            packages: vec![TakeawayPackageData {
                package_name: "Al Götür Menü 1".to_string(),
                slots: vec![],
            }],
            ..Default::default()
        };

        let pages = vec![
            (
                ParsedDocumentPayload::DailyMenu(menu),
                ParseDiagnostics::default(),
            ),
            (
                ParsedDocumentPayload::Takeaway(takeaway),
                ParseDiagnostics::default(),
            ),
        ];

        let (merged, _diag) = merge_page_payloads(pages).expect("birleştirme başarılı olmalı");
        match merged {
            ParsedDocumentPayload::Compound { menu, takeaway, .. } => {
                assert!(menu.is_some());
                assert_eq!(menu.unwrap().len(), 1);
                assert!(takeaway.is_some());
                assert_eq!(takeaway.unwrap().packages.len(), 1);
            }
            _ => panic!("Compound bekleniyordu"),
        }
    }
}

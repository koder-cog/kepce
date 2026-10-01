use crate::parser::models::MenuDatabase;
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
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
        "description": "Official ceiling price and grammage board data.",
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
        "description": "Al Götür packages and choice slots.",
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
                "description": "List of daily menus extracted if this is a daily menu document.",
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
        .unwrap_or_else(|| "high".to_string())
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
    if doc_type == "official_pricing" || val.get("pricing_board").is_some() {
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

    // 2. Al Götür Menü Paketi
    if doc_type == "takeaway_package" || val.get("takeaway").is_some() {
        let takeaway_val = val.get("takeaway").unwrap_or(&val);
        let pkgs_val = takeaway_val.get("packages").and_then(|p| p.as_array());
        if let Some(pkgs_arr) = pkgs_val
            && !pkgs_arr.is_empty()
        {
            let mut packages = Vec::new();
            for pkg in pkgs_arr {
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

            return Ok((
                crate::parser::models::ParsedDocumentPayload::Takeaway(
                    crate::parser::models::TakeawayData {
                        city_slug,
                        academic_year,
                        packages,
                    },
                ),
                Vec::new(),
            ));
        }
    }

    // 3. Günlük Tabldot Menü (varsayılan veya doc_type == "daily_menu")
    if let Ok((mut db, mismatches)) =
        crate::parser::json::parse_json_str_with_diagnostics(cleaned, file_name_hint)
        && !db.is_empty()
    {
        for day_data in db.values_mut() {
            crate::parser::validation::finalize_day_metadata(day_data);
        }
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

/// Sağlayıcıdan bağımsız istek verisi (istem metni + belge + şema).
struct LlmRequest<'a> {
    prompt: &'a str,
    mime_type: &'a str,
    base64_data: &'a str,
    /// Gemini Interactions API için `document` | `image`.
    input_type: &'a str,
    schema: &'a serde_json::Value,
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
            "json_schema": { "name": "document", "strict": true, "schema": req.schema }
        },
        // NOT: `provider.require_parameters: true` BİLİNÇLİ OLARAK KULLANILMAZ.
        // Canlıda doğrulandı: bu bayrak, akışı 502 "provider_unavailable" ile
        // bozan bir uç noktaya yönlendiriyor ve KESİK JSON döndürüyor
        // (menu_bursa.pdf -> 1 gün / 0 kayıt). Bayrak kaldırıldığında aynı dosya
        // 28-31 gün olarak eksiksiz ayrıştırılıyor.
        // Muhakeme token'ları bu bütçeden düşer; cömert tutulur.
        "max_tokens": max_tokens
    });

    if !effort.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("reasoning".to_string(), json!({ "effort": effort }));
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
            "schema": req.schema
        }
    });

    // Yüksek muhakeme. Alan adı canlıda doğrulandı:
    // generation_config.thinking_level = "high" -> HTTP 200 + thought token.
    if !thinking_level.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "generation_config".to_string(),
            json!({ "thinking_level": thinking_level }),
        );
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

Extract all details strictly into the corresponding fields of the JSON schema:
1. If official_pricing: Extract every single row into 'pricing_board.items' with meal_type ('breakfast' for Kahvalti, 'dinner' for Aksam/Yemek, 'lunch' for Ogle), category_name (item name in uppercase), portion_amount, and numeric price in TL. If dates or academic year are stated (e.g. 2026-2027), extract period_start and period_end.
2. If takeaway_package: Extract each package into 'takeaway.packages', including slots (index, title, is_required) and alternative item names and portions.
3. If daily_menu: Extract all days into 'days' with ISO dates (YYYY-MM-DD), date_raw, meal_type, food items, amounts, calories, and alternatives.
Output strictly conforming to the requested JSON schema.";

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
    if corrected.corrected_pages > 0 {
        tracing::info!(
            "Yön düzeltmesi uygulandı: {} sayfa, {} düzeltildi ({:?}).",
            corrected.pages,
            corrected.corrected_pages,
            file_path
        );
    }

    let base64_data = BASE64.encode(&corrected.bytes);
    let mime_type = corrected.mime_type.as_str();

    let prompt = UNIFIED_EXTRACTION_PROMPT;
    let schema = unified_document_response_schema();

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
        schema: &schema,
    };

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

                    match call_openrouter(
                        client,
                        key,
                        &openrouter_model,
                        &openrouter_effort,
                        &llm_req,
                    )
                    .await
                    {
                        Ok(text) => match parse_and_finalize_polymorphic(&text, file_name_hint) {
                            Ok((payload, mismatches)) => {
                                tracing::info!(
                                    "  Başarıyla çok biçimli ayrıştırıldı (sağlayıcı: openrouter, model: {}).",
                                    openrouter_model
                                );
                                return Ok((
                                    payload,
                                    crate::parser::core::ParseDiagnostics {
                                        date_raw_mismatches: mismatches,
                                        ..Default::default()
                                    },
                                ));
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
                            Ok(text) => {
                                match parse_and_finalize_polymorphic(&text, file_name_hint) {
                                    Ok((payload, mismatches)) => {
                                        tracing::info!(
                                            "  Başarıyla çok biçimli ayrıştırıldı (sağlayıcı: gemini, model: {}).",
                                            model_name
                                        );
                                        return Ok((
                                            payload,
                                            crate::parser::core::ParseDiagnostics {
                                                date_raw_mismatches: mismatches,
                                                ..Default::default()
                                            },
                                        ));
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "  Ayrıştırma hatası (deneme {}): {}",
                                            attempt,
                                            e
                                        );
                                        last_error = format!("{}", e);
                                    }
                                }
                            }
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

    Err(anyhow::anyhow!(
        "Tüm LLM denemeleri başarısız oldu. Son hata: {}",
        last_error
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
}

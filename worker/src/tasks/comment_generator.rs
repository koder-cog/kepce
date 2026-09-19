//! Otomatik menü tohum ve bot yorum üretim motoru.
//!
//! OpenRouter veya Gemini API üzerinden, öncelik kuyruğuna göre (güncel ay -> geçmiş aylar;
//! pilot iller -> diğerleri) eksik menüler için doğal, esprili ve samimi bot yorumları üretir.

use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::entities::sea_orm_active_enums::MenuStatusEnum;
use shared::entities::{cities, dish_aliases, menu_dishes, menus, prelude::*};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeneratedCommentEntry {
    pub date: String,
    pub commentary: String,
}

/// Desteklenen LLM sağlayıcıları
pub enum LlmProvider {
    OpenRouter { api_key: String, model: String },
    Gemini { api_key: String, model: String },
}

impl LlmProvider {
    pub fn from_env() -> Option<Self> {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.trim().is_empty() {
                let model = std::env::var("OPENROUTER_MODEL")
                    .unwrap_or_else(|_| "google/gemini-3.8-flash:floor".to_string());
                return Some(Self::OpenRouter {
                    api_key: key.trim().to_string(),
                    model,
                });
            }
        }

        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                let model = std::env::var("GEMINI_MODEL")
                    .unwrap_or_else(|_| "gemini-flash-latest".to_string());
                return Some(Self::Gemini {
                    api_key: key.trim().to_string(),
                    model,
                });
            }
        }

        None
    }
}

/// Menüleri ve yemeklerini LLM için metne döker
async fn build_batch_prompt(
    db: &DatabaseConnection,
    city: &cities::Model,
    dates: &[NaiveDate],
) -> Result<String> {
    let menus_in_batch = Menus::find()
        .filter(menus::Column::CityId.eq(city.id))
        .filter(menus::Column::ServeDate.is_in(dates.to_vec()))
        .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
        .order_by_asc(menus::Column::ServeDate)
        .order_by_asc(menus::Column::MealType)
        .all(db)
        .await?;

    let mut text = format!("Şehir: {}\n", city.name);
    let mut current_date: Option<NaiveDate> = None;

    for menu in menus_in_batch {
        if current_date != Some(menu.serve_date) {
            let header = shared::services::calendar::format_bot_day_header(menu.serve_date, &city.slug);
            text.push_str(&format!("\n{}\n", header));
            current_date = Some(menu.serve_date);
        }

        let label = match menu.meal_type {
            shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "Kahvaltı",
            shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "Öğle Yemeği",
            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "Akşam Yemeği",
        };
        text.push_str(&format!("{}:\n", label));

        let dishes = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(menu.id))
            .order_by_asc(menu_dishes::Column::OrderIndex)
            .all(db)
            .await?;

        let mut dish_names = Vec::new();
        for d in dishes {
            if let Some(alias) = dish_aliases::Entity::find_by_id(d.dish_alias_id).one(db).await? {
                if !shared::services::content_guard::ContentGuard::is_junk_dish_text(&alias.name) {
                    dish_names.push(alias.name);
                }
            }
        }

        for name in dish_names {
            text.push_str(&format!("- {}\n", name));
        }
    }

    Ok(text)
}

/// LLM'den gelen ham metni temizler ve bot yorumu listesine ayrıştırır
pub fn parse_generated_comments(raw_response: &str) -> Result<Vec<GeneratedCommentEntry>> {
    let mut cleaned = raw_response.trim();

    if let Some(stripped) = cleaned.strip_prefix("```json") {
        cleaned = stripped;
    } else if let Some(stripped) = cleaned.strip_prefix("```") {
        cleaned = stripped;
    }

    if let Some(stripped) = cleaned.strip_suffix("```") {
        cleaned = stripped;
    }

    let cleaned = cleaned.trim();

    // 1. Doğrudan dizi formatı: [{"date": "...", "commentary": "..."}]
    if let Ok(entries) = serde_json::from_str::<Vec<GeneratedCommentEntry>>(cleaned) {
        return Ok(entries);
    }

    // 2. Nesne sarmalayıcı formatı: {"comments": [...]} veya {"data": [...]} veya {"days": [...]}
    if let Ok(wrapped) = serde_json::from_str::<serde_json::Value>(cleaned) {
        if let Some(arr) = wrapped
            .get("comments")
            .or_else(|| wrapped.get("data"))
            .or_else(|| wrapped.get("days"))
            .or_else(|| wrapped.get("gunler"))
            .or_else(|| wrapped.get("yorumlar"))
        {
            if let Ok(entries) = serde_json::from_value::<Vec<GeneratedCommentEntry>>(arr.clone()) {
                return Ok(entries);
            }
        }

        if let Some(arr) = wrapped.as_array() {
            if let Ok(entries) = serde_json::from_value::<Vec<GeneratedCommentEntry>>(serde_json::Value::Array(arr.clone())) {
                return Ok(entries);
            }
        }
    }

    Err(anyhow::anyhow!(
        "LLM çıktısı bot yorumları JSON formatına dönüştürülemedi: {}",
        cleaned
    ))
}

/// LLM sağlayıcısına tek bir istek gönderir
async fn call_llm_single(
    client: &reqwest::Client,
    provider: &LlmProvider,
    prompt_content: &str,
) -> Result<Vec<GeneratedCommentEntry>> {
    let system_instructions = "\
Sen üniversite ve KYK yemekhanelerini yakından takip eden, esprili, samimi ve gerçekçi Kepçe Bot'sun.
Sana verilen yemekhane menüsü için her gün için tek bir gerçekçi, doğal, esprili ve samimi bot yorumu yazacaksın.

KESİNLİKLE YASAKLAR:
- Yapay zeka kalıpları ve sahte övgüler: 'adeta', 'şölen', 'ziyafet', 'köprü görevi', 'afiyet olsun', 'harika bir menü', 'kesinlikle tavsiye edilir', 'sağlıklı ve dengeli'.
- Robotik veya kurumsal resmi dil.
- Şart ekinden (-sa/-se) ve bağlaçlardan sonra virgül veya noktalama işareti koymak.
- Noktalı virgül (;) kullanmak.

İPUÇLARI VE JARGON:
- Yurt gerçekleri: Porsiyon azlığı, çorbanın su gibi olması, patates veya makarna yoğunluğu, tavuk ya da börek günü sevinci, tatlı yerine meyve bekleyişi, ekmekle doymak.
- Kısa, tok ve doğal cümleler kur.

ÇIKTI FORMATI:
Sadece ve sadece aşağıdaki gibi geçerli bir JSON nesnesi dön. Markdown veya başka açıklama ekleme:
{
  \"comments\": [
    {
      \"date\": \"YYYY-MM-DD\",
      \"commentary\": \"bot yorumu buraya\"
    }
  ]
}";

    let raw_response = match provider {
        LlmProvider::OpenRouter { api_key, model } => {
            let body = json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_instructions },
                    { "role": "user", "content": prompt_content }
                ],
                "temperature": 0.7,
                "response_format": { "type": "json_object" }
            });

            let res = client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://kepce.org")
                .header("X-Title", "Kepce Menu Seeder")
                .json(&body)
                .send()
                .await
                .context("OpenRouter isteği başarısız")?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("OpenRouter API hatası ({}): {}", status, err_text));
            }

            let json_res: serde_json::Value = res.json().await?;
            json_res["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        }
        LlmProvider::Gemini { api_key, model } => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/interactions?key={}",
                api_key
            );

            let schema = json!({
                "type": "object",
                "properties": {
                    "comments": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "date": { "type": "string" },
                                "commentary": { "type": "string" }
                            },
                            "required": ["date", "commentary"]
                        }
                    }
                },
                "required": ["comments"]
            });

            let body = json!({
                "model": model,
                "input": format!("{}\n\nMENÜLER:\n{}", system_instructions, prompt_content),
                "generation_config": {
                    "response_mime_type": "application/json",
                    "response_schema": schema,
                    "temperature": 0.7
                }
            });

            let res = client
                .post(&url)
                .header("x-goog-api-key", api_key)
                .json(&body)
                .send()
                .await
                .context("Gemini API isteği başarısız")?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Gemini API hatası ({}): {}", status, err_text));
            }

            let json_res: serde_json::Value = res.json().await?;
            if let Some(out) = json_res.get("output_text").and_then(|t| t.as_str()) {
                out.to_string()
            } else if let Some(cand) = json_res.get("candidates").and_then(|c| c.as_array()) {
                cand.first()
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                    .and_then(|a| a.first())
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or_default()
                    .to_string()
            } else {
                return Err(anyhow::anyhow!("Gemini yanıtından metin okunamadı"));
            }
        }
    };

    parse_generated_comments(&raw_response)
}

/// LLM sağlayıcısına hız sınırı ve geçici hatalara karşı yeniden denemeli (retry/backoff) istek gönderir
async fn call_llm_with_retry(
    client: &reqwest::Client,
    provider: &LlmProvider,
    prompt_content: &str,
    max_retries: usize,
) -> Result<Vec<GeneratedCommentEntry>> {
    for attempt in 1..=max_retries {
        match call_llm_single(client, provider, prompt_content).await {
            Ok(entries) => return Ok(entries),
            Err(err) => {
                if attempt == max_retries {
                    return Err(err);
                }
                let wait_secs = (attempt * 2) as u64;
                tracing::warn!(
                    "[COMMENT-GEN] LLM çağrısı başarısız oldu ({}/{} deneme). {} sn sonra tekrar denenecek. Hata: {}",
                    attempt,
                    max_retries,
                    wait_secs,
                    err
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
            }
        }
    }

    Err(anyhow::anyhow!("Maksimum deneme sayısına ulaşıldı"))
}

/// Bot yorumlarını ilgili günün menülerine yazar
async fn save_generated_comments(
    db: &DatabaseConnection,
    city_id: i32,
    entries: &[GeneratedCommentEntry],
) -> Result<usize> {
    let mut updated = 0;
    for entry in entries {
        if let Ok(date) = NaiveDate::parse_from_str(entry.date.trim(), "%Y-%m-%d") {
            let day_menus = Menus::find()
                .filter(menus::Column::CityId.eq(city_id))
                .filter(menus::Column::ServeDate.eq(date))
                .all(db)
                .await?;

            let commentary_json = json!({ "yorum": entry.commentary.trim() }).to_string();

            for m in day_menus {
                let mut active: menus::ActiveModel = m.into();
                active.bot_commentary = Set(Some(commentary_json.clone()));
                active.update(db).await?;
                updated += 1;
            }
        }
    }
    Ok(updated)
}

/// Tüm eksik menüler için öncelik sıralı bot yorum üretim görevi
pub async fn run_comment_generation(db: &DatabaseConnection) -> Result<usize> {
    let provider = match LlmProvider::from_env() {
        Some(p) => p,
        None => {
            tracing::warn!("[COMMENT-GEN] Ne OPENROUTER_API_KEY ne de GEMINI_API_KEY tanımlı. Bot yorumu üretimi atlanıyor.");
            return Ok(0);
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let today = Local::now().naive_local().date();

    let pilot_cities = ["istanbul", "ankara", "izmir", "konya", "eskisehir", "bursa", "antalya"];

    // Yorumsuz onaylı menüleri çek
    let missing_menus = Menus::find()
        .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
        .filter(menus::Column::BotCommentary.is_null())
        .all(db)
        .await?;

    if missing_menus.is_empty() {
        tracing::info!("[COMMENT-GEN] Yorum bekleyen onaylı menü bulunamadı.");
        return Ok(0);
    }

    // Şehir ve tarihe göre grupla
    let mut city_dates: BTreeMap<i32, Vec<NaiveDate>> = BTreeMap::new();
    for m in &missing_menus {
        let entry = city_dates.entry(m.city_id).or_default();
        if !entry.contains(&m.serve_date) {
            entry.push(m.serve_date);
        }
    }

    let all_cities = Cities::find().all(db).await?;
    let city_map: HashMap<i32, cities::Model> = all_cities.into_iter().map(|c| (c.id, c)).collect();

    let mut total_generated = 0;

    // Şehirleri öncelik sırasına göre sırala: Pilot iller önce
    let mut sorted_city_ids: Vec<i32> = city_dates.keys().cloned().collect();
    sorted_city_ids.sort_by(|a, b| {
        let a_slug = city_map.get(a).map(|c| c.slug.as_str()).unwrap_or_default();
        let b_slug = city_map.get(b).map(|c| c.slug.as_str()).unwrap_or_default();
        let a_pilot = pilot_cities.contains(&a_slug);
        let b_pilot = pilot_cities.contains(&b_slug);
        b_pilot.cmp(&a_pilot)
    });

    for city_id in sorted_city_ids {
        let city = match city_map.get(&city_id) {
            Some(c) => c,
            None => continue,
        };

        let mut dates = city_dates.remove(&city_id).unwrap_or_default();

        // Tarihleri önceliklendir:
        // 1. Güncel ay (today.year() == d.year() && today.month() == d.month())
        // 2. Varsa geçmiş aylar
        // Her kademe kendi içinde kronolojik
        dates.sort_by(|&d1, &d2| {
            let p1 = if d1.year() == today.year() && d1.month() == today.month() { 1 } else { 2 };
            let p2 = if d2.year() == today.year() && d2.month() == today.month() { 1 } else { 2 };
            p1.cmp(&p2).then(d1.cmp(&d2))
        });

        let total_chunks = dates.chunks(7).count();

        // 7'şer günlük gruplarla LLM'e gönder
        for (chunk_idx, chunk) in dates.chunks(7).enumerate() {
            let start_date = chunk.first().copied().unwrap_or(today);
            let end_date = chunk.last().copied().unwrap_or(today);

            tracing::info!(
                "[COMMENT-GEN] [{}/{}] {} ili için {}-{} aralığında {} günlük parti bot yorumu üretiliyor...",
                chunk_idx + 1,
                total_chunks,
                city.name,
                start_date,
                end_date,
                chunk.len()
            );

            match build_batch_prompt(db, city, chunk).await {
                Ok(prompt) => {
                    if prompt.trim().is_empty() {
                        continue;
                    }

                    match call_llm_with_retry(&client, &provider, &prompt, 3).await {
                        Ok(entries) => {
                            match save_generated_comments(db, city.id, &entries).await {
                                Ok(count) => {
                                    tracing::info!(
                                        "[COMMENT-GEN] {} ili için {} menüye bot yorumu kaydedildi.",
                                        city.name,
                                        count
                                    );
                                    total_generated += count;
                                }
                                Err(e) => tracing::error!("[COMMENT-GEN] Yorum kaydetme hatası: {:?}", e),
                            }
                        }
                        Err(e) => tracing::error!(
                            "[COMMENT-GEN] LLM çağrısı tüm denemelerde başarısız oldu ({}): {:?}",
                            city.name,
                            e
                        ),
                    }
                }
                Err(e) => tracing::error!("[COMMENT-GEN] Prompt oluşturma hatası: {:?}", e),
            }

            // Rate limit koruması: partiler arasında kısa dinlenme
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    tracing::info!(
        "[COMMENT-GEN] Bot yorum üretim döngüsü tamamlandı. Toplam {} menü güncellendi.",
        total_generated
    );
    Ok(total_generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generated_comments_array() {
        let raw = r#"[{"date": "2026-09-20", "commentary": "Çorba güzeldi ama porsiyon azdı."}]"#;
        let res = parse_generated_comments(raw).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].date, "2026-09-20");
        assert_eq!(res[0].commentary, "Çorba güzeldi ama porsiyon azdı.");
    }

    #[test]
    fn test_parse_generated_comments_object_wrapper() {
        let raw = r#"{"comments": [{"date": "2026-09-20", "commentary": "Börek günü yüzler gülüyor."}]}"#;
        let res = parse_generated_comments(raw).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].date, "2026-09-20");
        assert_eq!(res[0].commentary, "Börek günü yüzler gülüyor.");
    }

    #[test]
    fn test_parse_generated_comments_markdown_codeblock() {
        let raw = "```json\n{\"data\": [{\"date\": \"2026-09-21\", \"commentary\": \"Makarna ve ekmekle doymaca.\"}]}\n```";
        let res = parse_generated_comments(raw).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].date, "2026-09-21");
    }

    #[test]
    fn test_parse_generated_comments_turkish_key_wrapper() {
        let raw = r#"{"gunler": [{"date": "2026-09-22", "commentary": "Tavuk günü sevinci."}]}"#;
        let res = parse_generated_comments(raw).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].date, "2026-09-22");
    }

    #[test]
    fn test_date_priority_sorting() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 19).unwrap();
        let mut dates = [
            NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 9, 25).unwrap(),
            NaiveDate::from_ymd_opt(2026, 9, 5).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 30).unwrap(),
        ];

        dates.sort_by(|&d1, &d2| {
            let p1 = if d1.year() == today.year() && d1.month() == today.month() { 1 } else { 2 };
            let p2 = if d2.year() == today.year() && d2.month() == today.month() { 1 } else { 2 };
            p1.cmp(&p2).then(d1.cmp(&d2))
        });

        // 1. Güncel ay (Eylül) kronolojik: 5 Eylül, 25 Eylül
        // 2. Geçmiş ay (Ağustos) kronolojik: 15 Ağustos, 30 Ağustos
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2026, 9, 5).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2026, 9, 25).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2026, 8, 15).unwrap());
        assert_eq!(dates[3], NaiveDate::from_ymd_opt(2026, 8, 30).unwrap());
    }
}

use axum::{
    extract::{Json, State},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::post,
    Router,
};
use chrono::{Datelike, Local, NaiveDate, Timelike};
use futures_util::StreamExt;
use reqwest::Client;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, PaginatorTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::OnceLock;
use std::time::Duration;
use crate::config::AppState;
use crate::dto::menu::MealType;
use crate::services::comment::CommentService;
use crate::services::menu::MenuService;
use crate::services::statistics::StatisticsService;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantRequest {
    pub messages: Vec<ChatMessage>,
    pub city: Option<String>,
}

const SYSTEM_DIRECTIVE: &str = r#"Sen, Kepçe Bot'sun; Kepçe'nin menü ve yemekhane asistanısın.

Temel Kurallar:
- Görev alanın Kepçe platformundaki menüler, yemek saatleri, beslenme yardımı, platform istatistikleri ve gerektiğinde web aramasıyla elde edilen güncel bilgilerdir.
- Doğrudan, net, kısa (1-3 cümle) ve odaklı yanıtlar ver. Lafı uzatma.
- Harici bir site veya konu sorulduğunda doğrudan o konuyu açıkla; kullanıcı açıkça karşılaştırma istemedikçe cümlenin sonuna Kepçe'nin özelliklerini zorla ekleme.
- Bilmediğin veya veritabanında olmayan harici konular/siteler için 'webde_ara' aracını çağır. Arama sonuçlarından faydalandığında ilgili cümlenin sonuna [1] gibi dipnot koy ve yanıtın altına "Kaynak:\n[1] https://..." şeklinde linki ekle. Arama yapılamıyorsa ilgili konuda bilgin olmadığını belirt.
- Biçimlendirme: Düz metin kullan. Yıldız (**kalın**), hashtag (#) veya liste markdown'ı koyma; sadece kaynak satırında bağlantı ver.
- Gereksiz nezaket kalıpları ("Elbette!", "Nasıl yardımcı olabilirim?", "Harika bir gün dilerim") veya zorlama alternatifler üretme.
- Kullanıcı sormadıkça site sayfalarını tanıtma veya pazarlamaya çalışma.

Referans Bilgiler (Yalnızca Sorulduğunda Kullan):
- Yemek Saatleri: Kahvaltı hafta içi 06:00-12:00, hafta sonu 06:00-12:30. Akşam yemeği 16:00'da başlar (kapanış şehre göre 22:00-23:00).
- Beslenme Yardımı: Tabldot ücretsizdir, günlük yardım tam karşılar. Fazladan alınan tabaklarda fark ödenir.
- Al-Götür: Kahvaltı yerine paket sandviç verilir, genelde 10:30 civarı biter.
- Çölyak & Diyet: Raporla yurt idaresinden glütensiz menü istenebilir.
- Menü Gönderme: /menu-gonder sayfasından menü dosyası yüklenebilir.
- Arşiv: /arsiv sayfasından geçmiş aylardaki menüler görülebilir.
- Botu Kapatma: /ayarlar sayfasından gizlenebilir.

Araç Kullanımı:
- Belirli gün menüsü için: 'menu_sorgula'
- Aylık doluluk/menü var mı soruları için: 'aylik_menu_durumu_sorgula'
- En sevilen/kötü yemekler, yorumlar, istatistikler için: 'istatistik_sorgula'
- Harici kurumlar, mevzuat, haberler veya diğer siteler için: 'webde_ara'
- Araç isimlerini kullanıcıya çiğ haliyle söyleme."#;

#[derive(Debug, Deserialize)]
struct SearxngResultItem {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    results: Option<Vec<SearxngResultItem>>,
}

async fn search_searxng(client: &reqwest::Client, base_url: &str, query: &str) -> String {
    let url = format!("{}/search", base_url.trim_end_matches('/'));
    let req = client
        .get(&url)
        .query(&[("q", query), ("format", "json"), ("language", "tr")])
        .timeout(Duration::from_millis(3500));

    match req.send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(data) = res.json::<SearxngResponse>().await {
                if let Some(results) = data.results {
                    if results.is_empty() {
                        return "Arama sonucunda ilgili bilgi bulunamadı.".to_string();
                    }
                    let mut out = String::from("Web Arama Sonuçları:\n");
                    for (idx, item) in results.iter().take(5).enumerate() {
                        let title = item.title.as_deref().unwrap_or("Başlıksız").trim();
                        let url = item.url.as_deref().unwrap_or("").trim();
                        let content = item.content.as_deref().unwrap_or("").trim();
                        out.push_str(&format!("[{}] \"{}\" - {}\nÖzet: {}\n\n", idx + 1, title, url, content));
                    }
                    return out;
                }
            }
            "Arama sonuçları ayrıştırılamadı.".to_string()
        }
        _ => "Arama servisine şu anda ulaşılamıyor.".to_string(),
    }
}

fn format_weekday(d: NaiveDate) -> &'static str {
    match d.weekday() {
        chrono::Weekday::Mon => "Pazartesi",
        chrono::Weekday::Tue => "Salı",
        chrono::Weekday::Wed => "Çarşamba",
        chrono::Weekday::Thu => "Perşembe",
        chrono::Weekday::Fri => "Cuma",
        chrono::Weekday::Sat => "Cumartesi",
        chrono::Weekday::Sun => "Pazar",
    }
}

/// Türkçe BPE hece ve alt-kelime ağırlıklı belirleyici token tahmini (~3.2 karakter/token)
fn estimate_tokens(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();
    let char_est = char_count.div_ceil(3);
    let word_est = (word_count * 13) / 10;
    char_est.max(word_est).max(1)
}

fn get_http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default()
    })
}

/// Modelin tool parametresi olarak verdiği tarihi parse eder
fn parse_date_arg(raw: &str, today: NaiveDate) -> Option<NaiveDate> {
    let clean = raw.trim().replace(['"', '\''], "");
    
    // 1. Standart YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%Y-%m-%d") {
        return Some(d);
    }
    // 2. DD.MM.YYYY veya DD-MM-YYYY
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d.%m.%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d-%m-%Y") {
        return Some(d);
    }

    let lower = clean.to_lowercase();
    if lower.contains("dün") {
        return today.pred_opt();
    }
    if lower.contains("yarın") {
        return today.succ_opt();
    }

    None
}

/// Ay argümanını (YYYY-MM, "bu_ay", "eylül" vb.) parse eder
fn parse_month_arg(raw: &str, today: NaiveDate) -> (i32, u32) {
    let clean = raw.trim().replace(['"', '\''], "").to_lowercase();
    
    // 1. YYYY-MM
    let parts: Vec<&str> = clean.split(&['-', '.', '/'][..]).collect();
    if parts.len() == 2 {
        if let (Ok(p1), Ok(p2)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if p1 > 1900 && (1..=12).contains(&p2) {
                return (p1 as i32, p2);
            }
            if p2 > 1900 && (1..=12).contains(&p1) {
                return (p2 as i32, p1);
            }
        }
    }

    if clean.contains("gelecek") || clean.contains("önümüzdeki") || clean.contains("sonraki") {
        if today.month() == 12 {
            return (today.year() + 1, 1);
        } else {
            return (today.year(), today.month() + 1);
        }
    }

    if clean.contains("geçen") || clean.contains("önceki") {
        if today.month() == 1 {
            return (today.year() - 1, 12);
        } else {
            return (today.year(), today.month() - 1);
        }
    }

    let turkish_months = [
        ("ocak", 1), ("şubat", 2), ("mart", 3), ("nisan", 4),
        ("mayıs", 5), ("haziran", 6), ("temmuz", 7), ("ağustos", 8),
        ("eylül", 9), ("ekim", 10), ("kasım", 11), ("aralık", 12),
    ];
    for (m_name, m_num) in turkish_months {
        if clean.contains(m_name) {
            return (today.year(), m_num);
        }
    }

    (today.year(), today.month())
}

/// Verilen yıl ve ayın kaç gün sürdüğünü hesaplar
fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month_first
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(30)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_assistant))
}

pub async fn handle_assistant(
    State(state): State<AppState>,
    Json(req): Json<AssistantRequest>,
) -> impl IntoResponse {
    let city_slug = req.city.clone().unwrap_or_else(|| "istanbul".to_string());
    let now = Local::now();
    let today = now.date_naive();
    let day_name = format_weekday(today);

    let is_summer = now.month() == 7 || now.month() == 8;

    // 1. Canlı zaman ve şehir bağlamını oluştur
    let mut live_data = format!(
        "<canli_veri>\n[zaman]\ntarih: {}\ngun: {}\nsaat: {:02}:{:02}\nyaz_sezonu: {}\n",
        today, day_name, now.hour(), now.minute(), if is_summer { "evet (kapali/nobetci)" } else { "hayir (aktif)" }
    );

    let city_model = shared::entities::cities::Entity::find()
        .filter(shared::entities::cities::Column::Slug.eq(&city_slug))
        .one(&state.db)
        .await
        .ok()
        .flatten();

    let city_id = if let Some(city) = &city_model {
        live_data.push_str(&format!("\n[secili_sehir]\nad: {}\nslug: {}\n", city.name, city.slug));

        // Bugünün menüleri (en sık sorulan durum için daima prompt içinde hazır)
        if let Ok(daily_menus) = MenuService::get_daily_menus(&state.db, city.id, today, None, None).await {
            if !daily_menus.is_empty() {
                live_data.push_str("\n[bugunku_menu]\n");
                for m in daily_menus {
                    let meal_name = match m.meal_type {
                        MealType::Breakfast => "Kahvaltı",
                        MealType::Dinner => "Akşam Yemeği",
                        MealType::Lunch => "Öğle Yemeği",
                    };
                    let dish_names: Vec<String> = m.items.iter().map(|item| item.raw_name.clone()).collect();
                    let cal_info = m.calorie_range.unwrap_or_else(|| "Belirtilmedi".to_string());
                    live_data.push_str(&format!("{}: {} (kalori: {})\n", meal_name, dish_names.join(", "), cal_info));
                }
            } else {
                live_data.push_str("\n[bugunku_menu]\ndurum: Bugün için henüz menü girilmemiştir.\n");

                // Bugün menü yoksa veritabanındaki en son girilmiş menüyü de bilgi olarak sağla
                if let Ok(Some(latest_menu)) = shared::entities::menus::Entity::find()
                    .filter(shared::entities::menus::Column::CityId.eq(city.id))
                    .filter(shared::entities::menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved))
                    .order_by_desc(shared::entities::menus::Column::ServeDate)
                    .one(&state.db)
                    .await
                {
                    let lm_date = latest_menu.serve_date;
                    let lm_day_name = format_weekday(lm_date);
                    live_data.push_str(&format!("\n[sistemdeki_en_son_girilmis_menu: {} ({})]\n", lm_date, lm_day_name));
                    if let Ok(menus) = MenuService::get_daily_menus(&state.db, city.id, lm_date, None, None).await {
                        for m in menus {
                            let meal_name = match m.meal_type {
                                MealType::Breakfast => "Kahvaltı",
                                MealType::Dinner => "Akşam Yemeği",
                                MealType::Lunch => "Öğle Yemeği",
                            };
                            let dish_names: Vec<String> = m.items.iter().map(|item| item.raw_name.clone()).collect();
                            let cal_info = m.calorie_range.unwrap_or_else(|| "Belirtilmedi".to_string());
                            live_data.push_str(&format!("{}: {} (kalori: {})\n", meal_name, dish_names.join(", "), cal_info));
                        }
                    }
                }
            }
        }
        Some(city.id)
    } else {
        live_data.push_str(&format!("\n[secili_sehir]\nslug: {}\n", city_slug));
        None
    };
    live_data.push_str("</canli_veri>");

    let system_prompt = format!("{}\n\n{}", SYSTEM_DIRECTIVE, live_data);

    // 2. Kayan context window: 16.384 token girdi bütçesi (prefill koruması)
    let mut truncated_messages = Vec::new();
    let mut token_count = 0;

    for msg in req.messages.iter().rev() {
        let msg_tokens = estimate_tokens(msg.content.as_deref().unwrap_or_default());
        if token_count + msg_tokens > 16384 {
            break;
        }
        token_count += msg_tokens;
        truncated_messages.push(msg.clone());
    }
    truncated_messages.reverse();

    let dropped_count = req.messages.len().saturating_sub(truncated_messages.len());

    let mut final_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: Some(system_prompt),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
    final_messages.extend(truncated_messages);

    // Dinamik Araçlar (Tools Schema)
    let tools_schema = serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "menu_sorgula",
                "description": "Belirli bir tarihteki KYK yemekhane menüsünü veritabanından getirir. Geçmiş veya gelecek gün menüleri için bu aracı çağır.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "tarih": {
                            "type": "string",
                            "description": "Sorgulanacak tarih, YYYY-MM-DD formatında (örnek: 2026-06-30 veya 2026-09-01)"
                        }
                    },
                    "required": ["tarih"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "aylik_menu_durumu_sorgula",
                "description": "Belirli bir aydaki menü doluluk/boşluk durumunu, toplam kaç gün menü girildiğini ve boş günleri sorgular. 'Bu ay kaç gün yemek var?', 'Eylül menüsü tam mı?' gibi sorular için bu aracı çağır.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "ay": {
                            "type": "string",
                            "description": "Sorgulanacak ay. YYYY-MM formatında (örnek: 2026-09) veya 'bu_ay', 'gelecek_ay'."
                        }
                    },
                    "required": ["ay"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "istatistik_sorgula",
                "description": "Kepçe platformundaki yemek liderlik tablosunu, en sevilen veya en kötü yemekleri, en çok beğenilen öğrenci yorumlarını, trend etiketleri veya platform genel sayılarını getirir.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "kategori": {
                            "type": "string",
                            "enum": ["en_sevilen_yemekler", "en_kotu_yemekler", "en_begenilen_yorumlar", "trend_etiketler", "genel_sayilar"],
                            "description": "Sorgulanacak istatistik türü"
                        },
                        "sehir": {
                            "type": "string",
                            "description": "Opsiyonel şehir slug'ı (örnek: istanbul, ankara). Belirtilmezse seçili şehir veya genel kullanılır."
                        }
                    },
                    "required": ["kategori"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "webde_ara",
                "description": "Kepçe veritabanında bulunmayan harici kurumlar, mevzuat, haberler, genel konular veya üçüncü taraf web siteleri hakkında canlı web araması yapar.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "sorgu": {
                            "type": "string",
                            "description": "Arama motorunda aranacak net anahtar kelimeler"
                        }
                    },
                    "required": ["sorgu"]
                }
            }
        }
    ]);

    let llama_url = format!("{}/v1/chat/completions", state.config.llama_api_url.trim_end_matches('/'));
    let client = get_http_client();
    let db = state.db.clone();
    let last_user_query = req.messages.last().and_then(|m| m.content.clone()).unwrap_or_default();

    let stream = async_stream::stream! {
        // Truncation bildirimi (eğer eski mesajlar sığmadığı için kesildiyse)
        if dropped_count > 0 {
            let ev = serde_json::json!({"type": "truncated", "dropped": dropped_count});
            yield Ok::<Event, Infallible>(Event::default().data(ev.to_string()));
        }

        // AŞAMA 1: Streaming İstek ile Düşünce, İçerik veya Tool Tespiti
        let first_payload = serde_json::json!({
            "messages": final_messages,
            "tools": tools_schema,
            "tool_choice": "auto",
            "temperature": 0.2,
            "top_p": 0.95,
            "stream": true,
            "max_tokens": 4096,
            "stop": [
                "<turn|>",
                "<end_of_turn>",
                "<|im_end|>",
                "<|eot_id|>",
                "<|end_of_text|>",
                "</s>",
            ]
        });

        let first_resp = client
            .post(&llama_url)
            .json(&first_payload)
            .timeout(Duration::from_secs(120))
            .send()
            .await;

        match first_resp {
            Ok(res) if res.status().is_success() => {
                let mut byte_stream = res.bytes_stream();
                let mut tool_args_acc = String::new();
                let mut tool_fn_name = String::new();
                let mut tool_call_id = String::new();
                let mut is_tool_call = false;

                while let Some(chunk_result) = byte_stream.next().await {
                    if let Ok(bytes) = chunk_result {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            let line = line.trim();
                            if line.starts_with("data:") {
                                let data_str = line.trim_start_matches("data:").trim();
                                if data_str == "[DONE]" {
                                    break;
                                }
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data_str) {
                                    // 1. Düşünce (reasoning) tokenlarını anlık olarak istemciye stream et
                                    if let Some(r_token) = json.pointer("/choices/0/delta/reasoning_content").and_then(|c| c.as_str()) {
                                        if !r_token.is_empty() {
                                            let ev = serde_json::json!({"type": "thought", "text": r_token});
                                            yield Ok::<Event, Infallible>(Event::default().data(ev.to_string()));
                                        }
                                    }

                                    // 2. Tool çağrısı parçalarını topla
                                    if let Some(tc_arr) = json.pointer("/choices/0/delta/tool_calls").and_then(|c| c.as_array()) {
                                        if let Some(tc) = tc_arr.first() {
                                            is_tool_call = true;
                                            if let Some(id) = tc.pointer("/id").and_then(|i| i.as_str()) {
                                                tool_call_id.push_str(id);
                                            }
                                            if let Some(name) = tc.pointer("/function/name").and_then(|n| n.as_str()) {
                                                tool_fn_name.push_str(name);
                                            }
                                            if let Some(args) = tc.pointer("/function/arguments").and_then(|a| a.as_str()) {
                                                tool_args_acc.push_str(args);
                                            }
                                        }
                                    }

                                    if let Some(finish) = json.pointer("/choices/0/finish_reason").and_then(|f| f.as_str()) {
                                        if finish == "tool_calls" {
                                            is_tool_call = true;
                                        }
                                    }

                                    // 3. İçerik tokenları (Tool çağrısı yoksa ANINDA canlı stream et!)
                                    if !is_tool_call {
                                        if let Some(token) = json.pointer("/choices/0/delta/content").and_then(|c| c.as_str()) {
                                            if !token.is_empty() && token != "<turn|>" {
                                                let ev = serde_json::json!({"type": "content", "text": token});
                                                yield Ok(Event::default().data(ev.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Eğer model bir tool çağırdıysa (Pass 2)
                if is_tool_call || !tool_fn_name.is_empty() {
                    let c_id = if tool_call_id.is_empty() { "call_1".to_string() } else { tool_call_id };
                    let f_name = if tool_fn_name.is_empty() { "menu_sorgula".to_string() } else { tool_fn_name };

                    // Tool sonucunu hazırla
                    let tool_result_text = match f_name.as_str() {
                        "aylik_menu_durumu_sorgula" => {
                            let raw_month = serde_json::from_str::<serde_json::Value>(&tool_args_acc)
                                .ok()
                                .and_then(|v| v.pointer("/ay").and_then(|k| k.as_str().map(|s| s.to_string())))
                                .unwrap_or_else(|| "bu_ay".to_string());
                            let (target_y, target_m) = parse_month_arg(&raw_month, today);
                            let total_days = days_in_month(target_y, target_m);

                            let query_slug = city_slug.clone();
                            match MenuService::get_menus_by_filter(&db, Some(query_slug.clone()), None, None, Some(target_y), Some(target_m), None).await {
                                Ok(monthly_menus) => {
                                    let mut served_days = HashSet::new();
                                    let mut breakfast_count = 0;
                                    let mut dinner_count = 0;
                                    let mut lunch_count = 0;

                                    for m in &monthly_menus {
                                        served_days.insert(m.serve_date);
                                        match m.meal_type {
                                            MealType::Breakfast => breakfast_count += 1,
                                            MealType::Dinner => dinner_count += 1,
                                            MealType::Lunch => lunch_count += 1,
                                        }
                                    }

                                    let filled_count = served_days.len();
                                    let month_name = match target_m {
                                        1 => "Ocak", 2 => "Şubat", 3 => "Mart", 4 => "Nisan", 5 => "Mayıs", 6 => "Haziran",
                                        7 => "Temmuz", 8 => "Ağustos", 9 => "Eylül", 10 => "Ekim", 11 => "Kasım", 12 => "Aralık",
                                        _ => "Bilinmeyen Ay",
                                    };

                                    let mut empty_days = Vec::new();
                                    for day in 1..=total_days {
                                        if let Some(d) = NaiveDate::from_ymd_opt(target_y, target_m, day) {
                                            if !served_days.contains(&d) {
                                                empty_days.push(format!("{}. gün ({})", day, format_weekday(d)));
                                            }
                                        }
                                    }

                                    let mut res = format!("{} {} ayı menü durumu ({}):\n", month_name, target_y, query_slug);
                                    res.push_str(&format!("- Toplam {} günün {} gününde menü kayıtlıdır.\n", total_days, filled_count));
                                    res.push_str(&format!("- Toplam {} kahvaltı, {} akşam yemeği kaydı mevcuttur.\n", breakfast_count, dinner_count));
                                    if lunch_count > 0 {
                                        res.push_str(&format!("- Toplam {} öğle yemeği kaydı mevcuttur.\n", lunch_count));
                                    }
                                    if empty_days.is_empty() {
                                        res.push_str("- Ayın tüm günleri için menü eksiksiz girilmiştir.\n");
                                    } else if empty_days.len() <= 10 {
                                        res.push_str(&format!("- Menüsü henüz girilmemiş/boş günler: {}\n", empty_days.join(", ")));
                                    } else {
                                        res.push_str(&format!("- Henüz menüsü girilmemiş {} gün bulunmaktadır.\n", empty_days.len()));
                                    }
                                    res
                                }
                                Err(_) => format!("{} {} ayı menü durumu sorgulanırken veritabanı hatası oluştu.", target_m, target_y),
                            }
                        },
                        "istatistik_sorgula" => {
                            let args_json = serde_json::from_str::<serde_json::Value>(&tool_args_acc).unwrap_or_default();
                            let category = args_json.pointer("/kategori").and_then(|k| k.as_str()).unwrap_or("en_sevilen_yemekler");
                            let explicit_city = args_json.pointer("/sehir").and_then(|k| k.as_str()).map(|s| s.to_string());
                            let target_city = explicit_city.or_else(|| Some(city_slug.clone()));

                            match category {
                                "en_sevilen_yemekler" => {
                                    match StatisticsService::get_dish_leaderboard(&db, 5, true, target_city.clone(), None).await {
                                        Ok(dishes) if !dishes.is_empty() => {
                                            let mut res = "En çok sevilen / en yüksek puanlı yemekler:\n".to_string();
                                            for (i, d) in dishes.iter().enumerate() {
                                                let pct = d.average_rating.map(|r| format!("%{:.0} beğeni", r * 100.0)).unwrap_or_else(|| format!("{} net puan", d.score));
                                                res.push_str(&format!("{}. {} (Toplam {} oy, {})\n", i + 1, d.name, d.total_votes, pct));
                                            }
                                            res
                                        }
                                        Ok(_) => "Henüz oylanmış yemek istatistiği bulunmuyor.".to_string(),
                                        Err(_) => "Yemek istatistiği sorgulanırken hata oluştu.".to_string(),
                                    }
                                },
                                "en_kotu_yemekler" => {
                                    match StatisticsService::get_dish_leaderboard(&db, 5, false, target_city.clone(), None).await {
                                        Ok(dishes) if !dishes.is_empty() => {
                                            let mut res = "En az beğenilen / en çok eleştirilen yemekler:\n".to_string();
                                            for (i, d) in dishes.iter().enumerate() {
                                                let pct = d.average_rating.map(|r| format!("%{:.0} beğeni", r * 100.0)).unwrap_or_else(|| format!("{} net puan", d.score));
                                                res.push_str(&format!("{}. {} (Toplam {} oy, {})\n", i + 1, d.name, d.total_votes, pct));
                                            }
                                            res
                                        }
                                        Ok(_) => "Henüz oylanmış olumsuz yemek kaydı bulunmuyor.".to_string(),
                                        Err(_) => "Yemek istatistiği sorgulanırken hata oluştu.".to_string(),
                                    }
                                },
                                "en_begenilen_yorumlar" => {
                                    match CommentService::get_top_comments(&db, None, 5, None).await {
                                        Ok(comments) if !comments.is_empty() => {
                                            let mut res = "Öğrenciler tarafından en çok beğenilen / öne çıkan yorumlar:\n".to_string();
                                            for (i, c) in comments.iter().enumerate() {
                                                let author = &c.user.nickname;
                                                let text = c.comment.as_deref().unwrap_or("...");
                                                let up = c.reaction_summary.up;
                                                let dish = c.dish_name.as_deref().map(|d| format!(" ({})", d)).unwrap_or_default();
                                                res.push_str(&format!("{}. \"{}\" - {}{} (+{} beğeni)\n", i + 1, text, author, dish, up));
                                            }
                                            res
                                        }
                                        Ok(_) => "Henüz öne çıkan bir yorum bulunmuyor.".to_string(),
                                        Err(_) => "Yorum istatistikleri sorgulanırken hata oluştu.".to_string(),
                                    }
                                },
                                "trend_etiketler" => {
                                    match StatisticsService::get_trending_tags(&db, 8).await {
                                        Ok(tags) if !tags.is_empty() => {
                                            let mut res = "Son dönemde en çok kullanılan trend yemek etiketleri:\n".to_string();
                                            for (i, t) in tags.iter().enumerate() {
                                                res.push_str(&format!("{}. #{} ({} kez)\n", i + 1, t.name, t.count));
                                            }
                                            res
                                        }
                                        Ok(_) => "Henüz trend etiket verisi yok.".to_string(),
                                        Err(_) => "Etiket istatistikleri sorgulanırken hata oluştu.".to_string(),
                                    }
                                },
                                "genel_sayilar" => {
                                    let menu_c = shared::entities::menus::Entity::find().count(&db).await.unwrap_or(0);
                                    let comment_c = shared::entities::comments::Entity::find().count(&db).await.unwrap_or(0);
                                    let user_c = shared::entities::users::Entity::find().count(&db).await.unwrap_or(0);
                                    format!(
                                        "Kepçe Platform Genel İstatistikleri:\n- Toplam Menü Kaydı: {}\n- Toplam Öğrenci Yorumu: {}\n- Kayıtlı Topluluk Üyesi: {}",
                                        menu_c, comment_c, user_c
                                    )
                                },
                                _ => "Geçersiz istatistik kategorisi.".to_string(),
                            }
                        },
                        "webde_ara" => {
                            let q = serde_json::from_str::<serde_json::Value>(&tool_args_acc)
                                .ok()
                                .and_then(|v| v.pointer("/sorgu").and_then(|k| k.as_str().map(|s| s.to_string())))
                                .unwrap_or_else(|| tool_args_acc.clone());

                            let clean_q = q.trim();
                            if clean_q.is_empty() {
                                "Arama sorgusu belirtilmedi.".to_string()
                            } else if let Some(ref searx_url) = state.config.searxng_url {
                                search_searxng(client, searx_url, clean_q).await
                            } else {
                                "Web arama servisi şu anda yapılandırılmamış.".to_string()
                            }
                        },
                        _ => {
                            // Varsayılan: menu_sorgula
                            let raw_date = serde_json::from_str::<serde_json::Value>(&tool_args_acc)
                                .ok()
                                .and_then(|v| v.pointer("/tarih").and_then(|k| k.as_str().map(|s| s.to_string())))
                                .unwrap_or_else(|| tool_args_acc.clone());

                            let parsed_date = parse_date_arg(&raw_date, today).unwrap_or(today);
                            let q_day_name = format_weekday(parsed_date);

                            if let Some(cid) = city_id {
                                if let Ok(menus) = MenuService::get_daily_menus(&db, cid, parsed_date, None, None).await {
                                    if !menus.is_empty() {
                                        let mut res = format!("Tarih: {} ({})\n", parsed_date, q_day_name);
                                        for m in menus {
                                            let meal_name = match m.meal_type {
                                                MealType::Breakfast => "Kahvaltı",
                                                MealType::Dinner => "Akşam Yemeği",
                                                MealType::Lunch => "Öğle Yemeği",
                                            };
                                            let dish_names: Vec<String> = m.items.iter().map(|item| item.raw_name.clone()).collect();
                                            let cal_info = m.calorie_range.unwrap_or_else(|| "Belirtilmedi".to_string());
                                            res.push_str(&format!("{}: {} (kalori: {})\n", meal_name, dish_names.join(", "), cal_info));
                                        }
                                        res
                                    } else {
                                        format!("Tarih: {} ({}). Bu tarih için veritabanında kayıtlı bir menü bulunamadı.", parsed_date, q_day_name)
                                    }
                                } else {
                                    format!("Tarih: {} ({}). Menü sorgusu sırasında veritabanı hatası oluştu.", parsed_date, q_day_name)
                                }
                            } else {
                                "Şehir bilgisi bulunamadı.".to_string()
                            }
                        }
                    };

                    let synthetic_call = serde_json::json!({
                        "id": c_id,
                        "type": "function",
                        "function": {
                            "name": f_name,
                            "arguments": tool_args_acc
                        }
                    });

                    // AŞAMA 2: Tool Yanıtı ile Streaming İstek
                    let mut second_messages = final_messages.clone();
                    second_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: None,
                        tool_calls: Some(vec![synthetic_call]),
                        tool_call_id: None,
                        name: None,
                    });
                    second_messages.push(ChatMessage {
                        role: "tool".to_string(),
                        content: Some(tool_result_text),
                        tool_calls: None,
                        tool_call_id: Some(c_id),
                        name: Some(f_name),
                    });

                    let second_payload = serde_json::json!({
                        "messages": second_messages,
                        "temperature": 0.2,
                        "top_p": 0.95,
                        "stream": true,
                        "max_tokens": 4096,
                        "stop": [
                            "<turn|>",
                            "<end_of_turn>",
                            "<|im_end|>",
                            "<|eot_id|>",
                            "<|end_of_text|>",
                            "</s>",
                        ]
                    });

                    if let Ok(second_res) = client.post(&llama_url).json(&second_payload).timeout(Duration::from_secs(120)).send().await {
                        if second_res.status().is_success() {
                            let mut byte_stream = second_res.bytes_stream();
                            while let Some(chunk_result) = byte_stream.next().await {
                                if let Ok(bytes) = chunk_result {
                                    let text = String::from_utf8_lossy(&bytes);
                                    for line in text.lines() {
                                        let line = line.trim();
                                        if line.starts_with("data:") {
                                            let data_str = line.trim_start_matches("data:").trim();
                                            if data_str == "[DONE]" {
                                                yield Ok::<Event, Infallible>(Event::default().data("[DONE]"));
                                                return;
                                            }
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                // Düşünce tokenları
                                                if let Some(r_token) = json.pointer("/choices/0/delta/reasoning_content").and_then(|c| c.as_str()) {
                                                    if !r_token.is_empty() {
                                                        let ev = serde_json::json!({"type": "thought", "text": r_token});
                                                        yield Ok(Event::default().data(ev.to_string()));
                                                    }
                                                }
                                                // Asıl cevap tokenları (CANLI STREAM)
                                                if let Some(token) = json.pointer("/choices/0/delta/content").and_then(|c| c.as_str()) {
                                                    if !token.is_empty() && token != "<turn|>" {
                                                        let ev = serde_json::json!({"type": "content", "text": token});
                                                        yield Ok(Event::default().data(ev.to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            yield Ok(Event::default().data("[DONE]"));
                            return;
                        }
                    }
                }

                yield Ok(Event::default().data("[DONE]"));
                return;
            }
            _ => {
                // Offline Fallback
                let fallback = generate_offline_fallback(&last_user_query, &live_data);
                let ev = serde_json::json!({"type": "offline", "text": fallback});
                yield Ok(Event::default().data(ev.to_string()));
                yield Ok(Event::default().data("[DONE]"));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

fn generate_offline_fallback(query: &str, live_data: &str) -> String {
    let q = query.to_lowercase();
    if q.contains("direktif") || q.contains("prompt") || q.contains("talimat") || (q.contains("kural") && (q.contains("ver") || q.contains("neler") || q.contains("söyle"))) {
        return "Tüm sistem açık kaynaklı zaten, kurallarımı ezberletmek yerine GitHub'daki kaynak koduna baksan daha iyi sanki :)".to_string();
    }
    if (q.contains("bot") || q.contains("seni") || q.contains("asistan")) && (q.contains("kapat") || q.contains("gizle") || q.contains("kaldır")) {
        return "Beni kapatmak için Ayarlar sayfasına (/ayarlar) gidip 'Kepçe Bot'u göster' seçeneğini kapatabilirsin.".to_string();
    }
    if q.contains("saat") || (q.contains("kahvaltı") && q.contains("kaç")) || (q.contains("akşam") && q.contains("kaç")) {
        return "KYK Yemek Saatleri: Kahvaltı hafta içi 06:00-12:00, hafta sonu 06:00-12:30. Akşam yemeği 16:00 başlar; 22:00-23:00 arası biter.".to_string();
    }
    if q.contains("çölyak") || q.contains("glüten") || q.contains("gluten") || q.contains("diyet") || q.contains("alerji") {
        return "Çölyak & Glütensiz Menü: Sağlık raporunuzu yurt idaresine teslim ederek diyet ve glütensiz menü talep edebilirsiniz.".to_string();
    }
    if q.contains("istatistik") || q.contains("en sevilen") || q.contains("en çok") || q.contains("en iyi") {
        return "Yemek liderlik tablosu ve öne çıkan istatistikler için /istatistikler sayfasını ziyaret edebilirsin.".to_string();
    }
    if q.contains("menü") || q.contains("ne var") || q.contains("yemek") {
        if live_data.contains("[bugunku_menu]\nKahvaltı:") || live_data.contains("[bugunku_menu]\nAkşam Yemeği:") {
            return format!("Kepçe Bot çevrimdışı ancak bugünkü menü:\n{}", live_data.trim());
        }
        return "Bugün için kayıtlı menü bulunamadı veya model şu anda çevrimdışı.".to_string();
    }
    "Dil modeli şu anda çevrimdışı. Yemek saatleri veya menü kuralları hakkında soru sorabilirsin.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_arg() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();

        assert_eq!(parse_date_arg("2026-06-30", today), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert_eq!(parse_date_arg("\"2026-06-30\"", today), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert_eq!(parse_date_arg("30.06.2026", today), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert_eq!(parse_date_arg("dün", today), Some(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()));
        assert_eq!(parse_date_arg("yarın", today), Some(NaiveDate::from_ymd_opt(2026, 9, 2).unwrap()));
    }

    #[test]
    fn test_parse_month_arg() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();

        assert_eq!(parse_month_arg("2026-09", today), (2026, 9));
        assert_eq!(parse_month_arg("09.2026", today), (2026, 9));
        assert_eq!(parse_month_arg("eylül", today), (2026, 9));
        assert_eq!(parse_month_arg("bu ay", today), (2026, 9));
        assert_eq!(parse_month_arg("gelecek ay", today), (2026, 10));
        assert_eq!(parse_month_arg("geçen ay", today), (2026, 8));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 9), 30);
        assert_eq!(days_in_month(2026, 10), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29); // Artık yıl
    }
}

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use rand::Rng;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Ard arda gelen 429 sayaci: kaynak sunucu bizi hizlandiriyorsa
/// geri cekilmek icin kullanilir (kibar tarama politikasi).
static KYK_429_STREAK: AtomicU32 = AtomicU32::new(0);

/// IP-ban devre kesici (circuit breaker): kykyemek.com bizi banladiginda
/// (HTTP 403 / israrci 429 serisi) KYK_BAN_COOLDOWN_SECS boyunca o domaine
/// HIC bir istek atilmaz. Banliyken israr etmek ban suresini uzatir.
/// Deger: cooldown bitis aninin Unix timestamp'i (0 = temiz).
static KYK_BANNED_UNTIL: AtomicU64 = AtomicU64::new(0);

fn ban_cooldown_secs() -> u64 {
    std::env::var("KYK_BAN_COOLDOWN_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6 * 60 * 60) // varsayilan: 6 saat
}

async fn trip_ban(reason: &str) {
    let until = chrono::Utc::now().timestamp().max(0) as u64 + ban_cooldown_secs();
    KYK_BANNED_UNTIL.store(until, Ordering::Relaxed);
    tracing::error!(
        "[KYK-BREAKER] Devre kesildi ({}). {} sn boyunca kykyemek.com'a istek atilmayacak; fallback kaynaklar calismaya devam eder.",
        reason,
        ban_cooldown_secs()
    );
    let alert_msg = format!(
        "[KYK-BREAKER] kykyemek.com erisimi engellendi ({}). Worker {} sn bekleyecek.",
        reason,
        ban_cooldown_secs()
    );
    let _ = shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
}

pub fn is_banned() -> bool {
    let until = KYK_BANNED_UNTIL.load(Ordering::Relaxed);
    until > 0 && (chrono::Utc::now().timestamp().max(0) as u64) < until
}

/// Devre kesici durumunu döner. Banlıysa kalan saniyeyi, değilse None döner.
pub fn get_ban_status() -> Option<u64> {
    let until = KYK_BANNED_UNTIL.load(Ordering::Relaxed);
    let now = chrono::Utc::now().timestamp().max(0) as u64;
    if until > now {
        Some(until - now)
    } else {
        None
    }
}

/// Devre kesiciyi manuel olarak sıfırlar.
pub fn reset_ban_status() {
    KYK_BANNED_UNTIL.store(0, Ordering::Relaxed);
    KYK_429_STREAK.store(0, Ordering::Relaxed);
}

/// Chrome 144 (LTS) User-Agent. Tek noktadan yönetilir.
pub const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36";
pub const SEC_CH_UA: &str = "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"144\", \"Google Chrome\";v=\"144\"";

/// Chrome 144 (LTS) XHR/fetch isteklerinde gönderdiği Client Hints +
/// Fetch Metadata başlık seti. Sadece User-Agent taklidi yetmez; bu
/// başlıklar eksikse sunucu tarafı "kütüphane" kokusunu alır.
pub fn with_xhr_headers(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    req.header("User-Agent", BROWSER_UA)
        .header("sec-ch-ua", SEC_CH_UA)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header("Accept-Language", "tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7")
}

/// Bu turda yeni INSERT edilen menülerin (city_id, serve_date) kaydı.
/// IndexNow otomasyonu (Faz 1.6) döngü sonunda bu kaydı kanonik
/// /{sehir}/{tarih} gün URL'lerine çevirip bildirir. Upsert'in tüm
/// çağıranları (scraper + fallback + gap-fill) buradan otomatik geçer.
static INSERTED_MENUS: OnceLock<Mutex<HashSet<(i32, NaiveDate)>>> = OnceLock::new();

fn inserted_registry() -> &'static Mutex<HashSet<(i32, NaiveDate)>> {
    INSERTED_MENUS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn record_inserted_menu(city_id: i32, date: NaiveDate) {
    if let Ok(mut set) = inserted_registry().lock() {
        set.insert((city_id, date));
    }
}

pub fn take_inserted_menus() -> Vec<(i32, NaiveDate)> {
    match inserted_registry().lock() {
        Ok(mut set) => set.drain().collect(),
        Err(_) => Vec::new(),
    }
}
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait};
use shared::entities::{cities, menus, menu_dishes, sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum}};
use crate::parser::kykyemek::parse_kykyemek_html;

async fn sleep_cancelable(ms: u64, shutdown_rx: &mut tokio::sync::watch::Receiver<bool>) -> bool {
    if *shutdown_rx.borrow() {
        return true;
    }
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_millis(ms)) => false,
        _ = shutdown_rx.changed() => true,
    }
}

pub fn extract_cities_from_kykyemek_html(html: &str) -> Vec<String> {
    static FALLBACK_CITIES: &[&str] = &[
        "ankara", "antalya", "canakkale", "erzurum", "eskisehir", "gaziantep",
        "isparta", "istanbul", "izmir", "kahramanmaras", "karabuk", "kirklareli",
        "konya", "sakarya", "sivas", "trabzon",
    ];

    if html.is_empty() {
        return FALLBACK_CITIES.iter().map(|s| s.to_string()).collect();
    }

    let document = scraper::Html::parse_document(html);
    let select_sel = match scraper::Selector::parse("select#navbarDropdown option") {
        Ok(s) => s,
        Err(_) => return FALLBACK_CITIES.iter().map(|s| s.to_string()).collect(),
    };

    let mut discovered: Vec<String> = Vec::new();
    for opt in document.select(&select_sel) {
        let text = opt.text().collect::<String>().trim().to_string();
        if text.is_empty() {
            continue;
        }
        let slug = text
            .replace(['İ', 'I', 'ı'], "i")
            .replace(['Ş', 'ş'], "s")
            .replace(['Ğ', 'ğ'], "g")
            .replace(['Ü', 'ü'], "u")
            .replace(['Ö', 'ö'], "o")
            .replace(['Ç', 'ç'], "c")
            .to_lowercase()
            .replace(' ', "-");
        if !slug.is_empty() && !discovered.contains(&slug) {
            discovered.push(slug);
        }
    }

    if discovered.is_empty() {
        FALLBACK_CITIES.iter().map(|s| s.to_string()).collect()
    } else {
        discovered
    }
}

pub fn extract_token_from_html(html: &str) -> Result<String> {
    static TOKEN_REGEX: OnceLock<regex::Regex> = OnceLock::new();
    let re = TOKEN_REGEX.get_or_init(|| {
        regex::Regex::new(r#"name=["']__RequestVerificationToken["'][^>]*value=["']([^"']+)["']"#).unwrap()
    });

    if let Some(caps) = re.captures(html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    static TOKEN_FALLBACK: OnceLock<regex::Regex> = OnceLock::new();
    let re_fb = TOKEN_FALLBACK.get_or_init(|| {
        regex::Regex::new(r#"value=["']([^"']+)["'][^>]*name=["']__RequestVerificationToken["']"#).unwrap()
    });
    if let Some(caps) = re_fb.captures(html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    anyhow::bail!("__RequestVerificationToken HTML içinde bulunamadı")
}

pub async fn fetch_kykyemek_session(client: &Client) -> Result<(String, Vec<String>)> {
    let res = client.get("https://kykyemek.com/")
        .header("User-Agent", BROWSER_UA)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("sec-ch-ua", SEC_CH_UA)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "document")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-site", "none")
        .header("sec-fetch-user", "?1")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Accept-Language", "tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7")
        .send()
        .await?;
    if matches!(res.status(), reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::TOO_MANY_REQUESTS) {
        trip_ban(&format!("token alımı HTTP {}", res.status())).await;
        anyhow::bail!("kykyemek erişimi engellendi (HTTP {})", res.status());
    }
    let html = res.text().await?;

    let token = extract_token_from_html(&html)?;
    let cities = extract_cities_from_kykyemek_html(&html);
    tracing::info!(
        "[KYKYEMEK-DISCOVERY] Oturum açıldı. Kykyemek üzerinde dinamik olarak {} aktif şehir tespit edildi.",
        cities.len()
    );

    Ok((token, cities))
}

pub use fetch_kykyemek_session as fetch_kyk_session;

pub async fn fetch_antiforgery_token(client: &Client) -> Result<String> {
    fetch_kykyemek_session(client).await.map(|(t, _)| t)
}

pub async fn scrape_today_menus(
    db: &DatabaseConnection,
    client: &Client,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<usize> {
    if is_banned() {
        tracing::warn!("[KYKYEMEK-BREAKER] Cooldown aktif - bülten taraması bu tur atlanıyor.");
        return Ok(0);
    }

    let (mut token_opt, mut active_slugs) = match fetch_kykyemek_session(client).await {
        Ok((tok, slugs)) => (Some(tok), slugs),
        Err(e) => {
            tracing::warn!("[KYKYEMEK-SESSION] Oturum başlatılamadı: {:?}. Düz istek deneniyor.", e);
            (None, extract_cities_from_kykyemek_html(""))
        }
    };

    {
        use rand::seq::SliceRandom;
        active_slugs.shuffle(&mut rand::thread_rng());
    }

    let mut total_saved = 0;

    for slug in active_slugs {
        if *shutdown_rx.borrow() || is_banned() {
            break;
        }

        let city_opt = cities::Entity::find()
            .filter(cities::Column::Slug.eq(&slug))
            .one(db)
            .await?;

        let city = match city_opt {
            Some(c) => c,
            None => continue,
        };

        tracing::info!("[KYKYEMEK-BULLETIN] Şehir için aylık bülten çekiliyor: {}...", city.name);

        // Ayın ilk 10 gününde bir önceki ayın menülerini de çekerek ay geçişlerindeki boşlukları doldur
        let shifts: Vec<&str> = if chrono::Utc::now().date_naive().day() <= 10 {
            vec!["-1", "0"]
        } else {
            vec!["0"]
        };

        for shift in &shifts {
            // 1. Kahvaltı Bülteni
            match fetch_and_save(db, client, &city, "breakfast", MealTypeEnum::Breakfast, shift, &mut token_opt, &mut shutdown_rx).await {
                Ok(Some(count)) => total_saved += count,
                Ok(None) => return Ok(total_saved),
                Err(e) => tracing::warn!(city = %city.slug, meal = "breakfast", shift = %shift, "Kahvaltı bülteni alınamadı: {:?}", e),
            }

            // Kibar gecikme (3.5 - 6.5s)
            let delay_ms = rand::thread_rng().gen_range(3500..=6500);
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(total_saved);
            }

            // 2. Akşam Yemeği Bülteni
            match fetch_and_save(db, client, &city, "dinner", MealTypeEnum::Dinner, shift, &mut token_opt, &mut shutdown_rx).await {
                Ok(Some(count)) => total_saved += count,
                Ok(None) => return Ok(total_saved),
                Err(e) => tracing::warn!(city = %city.slug, meal = "dinner", shift = %shift, "Akşam yemeği bülteni alınamadı: {:?}", e),
            }

            // Kibar gecikme (3.5 - 6.5s)
            let delay_ms = rand::thread_rng().gen_range(3500..=6500);
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(total_saved);
            }
        }
    }

    tracing::info!("[KYKYEMEK-BULLETIN] Kykyemek aylık bülten taraması tamamlandı: {} menü güncellendi.", total_saved);
    Ok(total_saved)
}

pub async fn run_kykyemek_scraper(
    db: &DatabaseConnection,
    client: &Client,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    let mut total_fetched = 0;

    // 1. Önce günün menülerini (aktif 16 ilin tüm ayın günlerini) hızlıca çek ve kaydet
    if let Ok(today_count) = scrape_today_menus(db, client, shutdown_rx.clone()).await {
        tracing::info!("Canlı günün menülerinden {} kayıt işlendi.", today_count);
        total_fetched += today_count;
    }

    // 1.5. Fallback zinciri: kykyemek'te eksik kalan şehir/günleri alternatif
    // kaynaklardan (kykmenum.com > yurtmenu.net > kykmenu.com.tr) doldur.
    // Sadece DB'de kayıt olmayan kombinasyonlar için istek atar.
    match super::fallback_scraper::run_fallback_scrape(db, client, shutdown_rx.clone()).await {
        Ok(fallback_count) => {
            if fallback_count > 0 {
                tracing::info!("Fallback kaynaklardan {} eksik menü dolduruldu.", fallback_count);
            }
            total_fetched += fallback_count;
        }
        Err(e) => tracing::error!("[FALLBACK] Alternatif kaynak taramasında hata: {:?}", e),
    }

    // 1.6. Gecmis ay bosluk doldurma: kykyemek yalnizca son 2 ayi servis
    // ettigi icin daha eski bosluklar acik kaynaklardan (yurtmenu.net +
    // kykmenum.com) doldurulur. FALLBACK_HISTORY_MONTHS ile ayarlanir (0=kapali).
    let hist_months: u32 = std::env::var("FALLBACK_HISTORY_MONTHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    if hist_months > 0 {
        match super::fallback_scraper::run_historical_gap_fill(db, client, shutdown_rx.clone(), hist_months).await {
            Ok(hist_count) => {
                if hist_count > 0 {
                    tracing::info!("Gecmis ay bosluklarindan {} menü dolduruldu.", hist_count);
                }
                total_fetched += hist_count;
            }
            Err(e) => tracing::error!("[HISTORY] Gecmis ay taramasında hata: {:?}", e),
        }
    }

    // Faz 1.6: bu turda yeni açılan gün URL'lerini IndexNow'a bildir.
    // INDEXNOW_KEY atanmamışsa no-op; hata durumunda yalnızca warn loglanır,
    // asla döngüyü düşürmez. Yalnızca kanonik /{sehir}/{tarih} URL'leri
    // gönderilir (/menu/{id} ASLA gönderilmez).
    if let Some(config) = super::indexnow::IndexNowConfig::from_env() {
        super::indexnow::ping_new_day_urls(db, client, &config).await;
    }

    tracing::info!("Kykyemek tarama döngüsü tamamlandı. Toplam {} menü işlendi.", total_fetched);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn fetch_and_save(
    db: &DatabaseConnection,
    client: &Client,
    city: &cities::Model,
    kyk_meal_type: &str,
    meal_type_enum: MealTypeEnum,
    month_shift: &str,
    token_opt: &mut Option<String>,
    shutdown_rx: &mut tokio::sync::watch::Receiver<bool>,
) -> Result<Option<usize>> {
    let is_dinner = if kyk_meal_type == "dinner" { "true" } else { "false" };
    let url = format!("https://kykyemek.com/Menu/GetDailyMenu/{}", city.slug);
    
    let mut attempt = 0;
    let max_retries = 3;
    let mut response = None;

    while attempt <= max_retries {
        if *shutdown_rx.borrow() {
            return Ok(None);
        }

        let mut req = with_xhr_headers(client.get(&url)
            .query(&[
                ("city", city.slug.as_str()),
                ("mealType", is_dinner),
                ("monthShift", month_shift),
                ("hidePast", "false"),
            ]))
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Referer", "https://kykyemek.com/")
            .timeout(std::time::Duration::from_secs(30));

        if let Some(ref token) = *token_opt {
            req = req
                .header("RequestVerificationToken", token.as_str())
                .header("__RequestVerificationToken", token.as_str());
        }

        match req.send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    KYK_429_STREAK.store(0, Ordering::Relaxed);
                    response = Some(res);
                    break;
                } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    // 429: kaynak bizi hizlandiriyor. Kibarca geri cekil,
                    // ust uste binen 429'larda bekleme suresini katla.
                    let streak = KYK_429_STREAK.fetch_add(1, Ordering::Relaxed) + 1;
                    let wait_secs = 30u64.saturating_mul(u64::from(streak)).min(180);
                    tracing::warn!(
                        "HTTP 429 (hiz siniri) {} - {} icin {}sn bekleniyor (deneme {}/{})",
                        streak, city.name, wait_secs, attempt + 1, max_retries
                    );
                    if sleep_cancelable(wait_secs * 1000, shutdown_rx).await {
                        return Ok(None);
                    }
                } else if status == reqwest::StatusCode::FORBIDDEN {
                    // 403: büyük olasılıkla IP ban. Retry ile ısrar etme, devreyi kes.
                    trip_ban(&format!("HTTP 403 - {} ({}) [ana tarama]", city.name, kyk_meal_type)).await;
                    anyhow::bail!("kykyemek IP ban şüphesi (HTTP 403): {} ({})", city.name, kyk_meal_type);
                } else if status == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!("HTTP 401 (Yetkisiz), yeni oturum token'ı alınıyor...");
                    if let Ok((new_token, _)) = fetch_kykyemek_session(client).await {
                        *token_opt = Some(new_token);
                    }
                } else {
                    tracing::warn!("HTTP durum kodu hatası: {}, Deneme: {}", status, attempt + 1);
                }
            }
            Err(e) => {
                tracing::warn!("İstek hatası: {:?}, Deneme: {}", e, attempt + 1);
            }
        }

        attempt += 1;
        if attempt <= max_retries {
            let backoff_secs = 1 << (attempt - 1); // 1s, 2s, 4s
            tracing::info!("Yeniden deneniyor (Bekleme: {}s)...", backoff_secs);
            if sleep_cancelable(backoff_secs * 1000, shutdown_rx).await {
                return Ok(None);
            }
        }
    }

    let res = match response {
        Some(r) => r,
        None => {
            let alert_msg = format!("Kykyemek sunucu hatası: {} (öğün: {}) için maksimum deneme sayısına ulaşıldı.", city.name, kyk_meal_type);
            let _ = shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
            anyhow::bail!(alert_msg);
        }
    };
    let body_text = res.text().await?;
    
    let html_content = if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
        json_val.get("html").and_then(|h| h.as_str()).unwrap_or(&body_text).to_string()
    } else {
        body_text
    };

    if !html_content.contains("cardStyle") && !html_content.contains("card-body") && html_content.len() > 100 {
        let alert_msg = format!("KYK HTML şablon anomalisi algılandı! {} şehri için dönen HTML beklenenden farklı (`.cardStyle` bulunamadı).", city.name);
        tracing::warn!("{}", alert_msg);
        let _ = shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
    }
    
    let parsed_menus = parse_kykyemek_html(&html_content, &city.slug, kyk_meal_type);
    let mut count = 0;
    
    for menu in parsed_menus {
        // Öğün Doğrulama Kalkanı: Eğer kart açıkça başka bir öğün olduğunu beyan ediyorsa,
        // yanlış öğün türüyle kaydedilmesini kesinlikle engelle.
        if let Some(ref detected) = menu.detected_meal {
            if *detected != meal_type_enum {
                tracing::warn!(
                    "[MEAL-GUARD] {} şehri için {} ({:?}) menüsü istendi ancak kart açıkça {:?} beyan ediyor! Hatalı öğün kaydı engellendi.",
                    city.name, kyk_meal_type, meal_type_enum, detected
                );
                continue;
            }
        }

        upsert_menu(
            db, 
            city.id, 
            menu.date, 
            meal_type_enum.clone(), 
            "kykyemek".to_string(), 
            None, 
            menu.dishes,
            vec![], // celiac_dishes
            menu.takeaways,
            None,
            menu.min_calories,
            menu.max_calories,
        ).await?;
        count += 1;
    }
    
    Ok(Some(count))
}

// GÜVENLİK NOTU (SA-15): "kepce-kullanici" kaynağı `upsert_menu` içinde otomatik
// APPROVED yapılır ve önceliği kykyemek'ten yüksektir. Bu kaynak türü YALNIZCA
// operatörün lokal drop-zone klasöründen (file_ingest) gelmelidir. Kullanıcı
// kaynaklı API akışları (ingestion) bu fonksiyona bağlanırsa otomatik onay
// moderation bypass'ına dönüşür - bu tabloyu değiştirirken bunu göz önünde tut.
fn get_source_priority(source: &str) -> i32 {
    match source {
        "kepce-admin" => 10,
        "kepce-kullanici" => 8,
        "kykyemek.com" | "kykyemek" | "kyk-yemek" => 6,
        "yurtmenu" | "yurtmenu.net" | "yurtmenu_live" => 5,
        "kykmenum" | "kykmenum.com" => 5,
        "kykmenu" | "kykmenu.com.tr" | "kykmenulistesi.com.tr" => 4,
        "kepce-anonim" | "anonim" => 3,
        _ => 1,
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum DishSlotKey {
    Primary(String, i32),
    Alternative(String, i32, i32),
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_menu(
    db: &DatabaseConnection,
    city_id: i32,
    date: NaiveDate,
    meal_type: MealTypeEnum,
    source_type: String,
    submitted_by: Option<uuid::Uuid>,
    dishes: Vec<Vec<crate::parser::models::MenuComponent>>,
    celiac_dishes: Vec<Vec<crate::parser::models::MenuComponent>>,
    takeaways: Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)>,
    target_status_override: Option<MenuStatusEnum>,
    calorie_range_min: Option<i32>,
    calorie_range_max: Option<i32>,
) -> Result<()> {
    // Yapısal kalite kapısı: içeriği tamamen boş olan kayıtlar (parser tüm
    // satırları çöp diye elerse ya da kaynak site boş döndüyse) slot işgal
    // etmesin. Böyle bir menü insert edilirse fallback/gap-fill "kayıt var"
    // görüp o şehir/gün/öğün kombinasyonunu bir daha doldurmaz.
    let has_standard = dishes.iter().any(|group| group.iter().any(|c| !c.name.trim().is_empty()));
    let has_celiac = celiac_dishes.iter().any(|group| group.iter().any(|c| !c.name.trim().is_empty()));
    let has_takeaways = takeaways.iter().any(|(_, groups)| groups.iter().any(|group| group.iter().any(|c| !c.name.trim().is_empty())));

    if !has_standard && !has_celiac && !has_takeaways {
        tracing::warn!(
            "upsert_menu reddedildi: geçerli içeriği olmayan kayıt (city_id: {}, tarih: {}, öğün: {:?}, kaynak: {})",
            city_id, date, meal_type, source_type
        );
        return Ok(());
    }

    let target_status = match target_status_override {
        Some(status) => status,
        None => match source_type.as_str() {
            "kepce-admin" | "kepce-kullanici" | "kykyemek" | "kykyemek.com" | "yurtmenu" | "yurtmenu.net" | "kykmenum" | "kykmenum.com" | "kykmenu" | "kykmenu.com.tr" => MenuStatusEnum::Approved,
            _ => MenuStatusEnum::Pending,
        }
    };
    let incoming_priority = get_source_priority(&source_type);

    let txn = db.begin().await?;
    
    // Check if menu exists
    let existing_menu = menus::Entity::find()
        .filter(menus::Column::CityId.eq(city_id))
        .filter(menus::Column::ServeDate.eq(date))
        .filter(menus::Column::MealType.eq(meal_type.clone()))
        .one(&txn)
        .await?;
        
    let mut existing_map = HashMap::new();
    let menu_id = if let Some(m) = existing_menu {
        let current_priority = get_source_priority(m.source_type.as_deref().unwrap_or(""));
        
        if incoming_priority < current_priority {
            tracing::debug!("Incoming menu for {} (meal: {:?}) from {} has lower priority than {}. Archiving incoming.", date, meal_type, source_type, m.source_type.as_deref().unwrap_or(""));
            let payload = serde_json::json!({
                "dishes": dishes,
                "takeaways": takeaways
            });
            let hist = shared::entities::menu_history::ActiveModel {
                city_id: Set(city_id),
                serve_date: Set(date),
                meal_type: Set(match meal_type {
                    MealTypeEnum::Breakfast => "breakfast".to_string(),
                    MealTypeEnum::Lunch => "lunch".to_string(),
                    MealTypeEnum::Dinner => "dinner".to_string(),
                }),
                source_type: Set(source_type),
                submitted_by: Set(submitted_by),
                dishes_payload: Set(payload),
                ..Default::default()
            };
            hist.insert(&txn).await?;
            txn.commit().await?;
            return Ok(());
        }

        let existing_dishes = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(m.id))
            .find_also_related(shared::entities::dish_aliases::Entity)
            .all(&txn)
            .await?;
            
        let payload = serde_json::json!(existing_dishes.iter().map(|(md, alias)| {
            serde_json::json!({
                "name": alias.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                "package_name": md.package_name.clone(),
                "order_index": md.order_index,
                "is_alternative": md.is_alternative
            })
        }).collect::<Vec<_>>());
        
        let hist = shared::entities::menu_history::ActiveModel {
            city_id: Set(m.city_id),
            serve_date: Set(m.serve_date),
            meal_type: Set(match m.meal_type {
                MealTypeEnum::Breakfast => "breakfast".to_string(),
                MealTypeEnum::Lunch => "lunch".to_string(),
                MealTypeEnum::Dinner => "dinner".to_string(),
            }),
            source_type: Set(m.source_type.clone().unwrap_or_else(|| "unknown".to_string())),
            submitted_by: Set(m.submitted_by),
            dishes_payload: Set(payload),
            ..Default::default()
        };
        hist.insert(&txn).await?;
        
        for (d, _) in existing_dishes {
            let key = if d.is_alternative {
                DishSlotKey::Alternative(d.package_name.clone(), d.order_index, d.dish_alias_id)
            } else {
                DishSlotKey::Primary(d.package_name.clone(), d.order_index)
            };
            existing_map.insert(key, d);
        }
        
        let mut update_m: menus::ActiveModel = m.clone().into();
        update_m.source_type = Set(Some(source_type));
        update_m.submitted_by = Set(submitted_by);
        update_m.status = Set(target_status.clone());
        if calorie_range_min.is_some() || calorie_range_max.is_some() {
            update_m.calorie_range_min = Set(calorie_range_min.or(m.calorie_range_min));
            update_m.calorie_range_max = Set(calorie_range_max.or(m.calorie_range_max));
        }
        update_m.update(&txn).await?;
        
        m.id
    } else {
        let new_menu = menus::ActiveModel {
            city_id: Set(city_id),
            serve_date: Set(date),
            meal_type: Set(meal_type),
            source_type: Set(Some(source_type)),
            submitted_by: Set(submitted_by),
            status: Set(target_status.clone()),
            calorie_range_min: Set(calorie_range_min),
            calorie_range_max: Set(calorie_range_max),
            ..Default::default()
        };
        let res = new_menu.insert(&txn).await?;
        // Faz 1.6: yeni insert'i IndexNow bildirim kaydına yaz
        record_inserted_menu(city_id, date);
        res.id
    };
    
    // Build target_map
    let mut target_map: HashMap<DishSlotKey, (i32, Option<String>, Option<i32>)> = HashMap::new();
    
    let parse_dish_calories = |raw: &Option<String>| -> Option<i32> {
        let s = raw.as_ref()?;
        let cleaned = s
            .to_lowercase()
            .replace("kcal", "")
            .replace("kkal", "")
            .replace("kalori", "")
            .trim()
            .to_string();
        if let Ok(v) = cleaned.parse::<i32>() {
            return Some(v);
        }
        let parts: Vec<&str> = cleaned.split(&['-', '–'][..]).map(|p| p.trim()).collect();
        if parts.len() == 2 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                return Some((a + b) / 2);
            }
        }
        None
    };

    for (i, dish_group) in dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "NORMAL".to_string();
            let cals = parse_dish_calories(&comp.calories);
            let key = if is_alternative {
                DishSlotKey::Alternative(package_name, order_index, alias_id)
            } else {
                DishSlotKey::Primary(package_name, order_index)
            };

            if let Some((_, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                *existing_amt = comp.amount.filter(|s| !s.trim().is_empty()).or(existing_amt.take());
                *existing_cals = cals.or(*existing_cals);
            } else {
                target_map.insert(key, (alias_id, comp.amount, cals));
            }
        }
    }

    for (i, dish_group) in celiac_dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "ÇÖLYAK MENÜSÜ".to_string();
            let cals = parse_dish_calories(&comp.calories);
            let key = if is_alternative {
                DishSlotKey::Alternative(package_name, order_index, alias_id)
            } else {
                DishSlotKey::Primary(package_name, order_index)
            };

            if let Some((_, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                *existing_amt = comp.amount.filter(|s| !s.trim().is_empty()).or(existing_amt.take());
                *existing_cals = cals.or(*existing_cals);
            } else {
                target_map.insert(key, (alias_id, comp.amount, cals));
            }
        }
    }
    
    for (package, package_dishes) in takeaways.into_iter() {
        let sanitized_package = sanitize_dish_name(&package);
        for (i, dish_group) in package_dishes.into_iter().enumerate() {
            let order_index = i as i32;
            for (j, comp) in dish_group.into_iter().enumerate() {
                let is_alternative = j > 0;
                let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
                let cals = parse_dish_calories(&comp.calories);
                let key = if is_alternative {
                    DishSlotKey::Alternative(sanitized_package.clone(), order_index, alias_id)
                } else {
                    DishSlotKey::Primary(sanitized_package.clone(), order_index)
                };

                if let Some((_, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                    *existing_amt = comp.amount.filter(|s| !s.trim().is_empty()).or(existing_amt.take());
                    *existing_cals = cals.or(*existing_cals);
                } else {
                    target_map.insert(key, (alias_id, comp.amount, cals));
                }
            }
        }
    }
    
    // Smart Sync with slot integrity and field-level metadata merge
    for (key, (alias_id, amount, calories)) in target_map.into_iter() {
        let (package_name, order_index, is_alternative) = match &key {
            DishSlotKey::Primary(pkg, idx) => (pkg.clone(), *idx, false),
            DishSlotKey::Alternative(pkg, idx, _) => (pkg.clone(), *idx, true),
        };

        if let Some(existing) = existing_map.remove(&key) {
            // Field-level merge: preserve existing values if incoming is None/empty
            let final_amount = amount.filter(|s| !s.trim().is_empty()).or(existing.amount.clone());
            let final_calories = calories.or(existing.calories);

            if existing.dish_alias_id != alias_id || existing.amount != final_amount || existing.calories != final_calories {
                let mut active: menu_dishes::ActiveModel = existing.into();
                active.dish_alias_id = Set(alias_id);
                active.amount = Set(final_amount);
                active.calories = Set(final_calories);
                active.update(&txn).await?;
            }
        } else {
            // Insert new slot
            let link = menu_dishes::ActiveModel {
                menu_id: Set(menu_id),
                dish_alias_id: Set(alias_id),
                order_index: Set(order_index),
                is_alternative: Set(is_alternative),
                package_name: Set(package_name),
                amount: Set(amount),
                calories: Set(calories),
                ..Default::default()
            };
            link.insert(&txn).await?;
        }
    }
    
    // Delete missing slots
    for (_, existing) in existing_map.into_iter() {
        let active: menu_dishes::ActiveModel = existing.into();
        active.delete(&txn).await?;
    }
    
    txn.commit().await?;

    let menu = menus::Entity::find_by_id(menu_id)
        .one(db)
        .await?;
    if let Some(m) = menu {
        if m.status == MenuStatusEnum::Approved {
            shared::services::immutable_store::ImmutableStore::write_menu_hash(db, menu_id)
                .await?;
        }
    }

    Ok(())
}

pub async fn get_or_create_dish_alias(txn: &sea_orm::DatabaseTransaction, raw_name: &str, category: Option<String>) -> Result<i32> {
    // 1. XSS sanitization
    let sanitized = sanitize_dish_name(raw_name);
    // 2. Kanonik isim normalizasyonu
    let canonical_name = crate::parser::normalizer::normalize_food_name(&sanitized);

    // Kategori belirtilmemişse akıllı kural motoruyla otomatik belirle
    let final_category = category.or_else(|| shared::services::categorizer::categorize_dish(&canonical_name));

    // Atomik işlem:
    // 1. Ana dish'i (yemek) normalize edilmiş kanonik isimle arar veya oluşturur (LOWER(TRIM(name)) tekilliği ile).
    // 2. Takma adı (sanitized raw alias) bu ana yemeğe bağlar.
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        r#"
        WITH upsert_dish AS (
            INSERT INTO dishes (name, category) VALUES ($1, $2)
            ON CONFLICT ((LOWER(TRIM(name)))) DO UPDATE SET category = COALESCE(dishes.category, EXCLUDED.category)
            RETURNING id
        )
        INSERT INTO dish_aliases (name, dish_id)
        VALUES ($3, (SELECT id FROM upsert_dish))
        ON CONFLICT (name) DO UPDATE SET dish_id = COALESCE(dish_aliases.dish_id, EXCLUDED.dish_id)
        RETURNING id
        "#,
        vec![canonical_name.into(), final_category.into(), sanitized.into()],
    );

    let query_res = txn.query_one(stmt).await?;
    
    if let Some(row) = query_res {
        let alias_id: i32 = row.try_get("", "id")?;
        Ok(alias_id)
    } else {
        Err(anyhow::anyhow!("Upsert işlemi alias ID döndüremedi."))
    }
}

pub fn sanitize_dish_name(name: &str) -> String {
    static RE_TAG: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE_TAG.get_or_init(|| regex::Regex::new(r"</?[a-zA-Z0-9]+(?:\s+[^>]*)?>").unwrap());
    let result = re.replace_all(name, "").into_owned();
    
    let decoded = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_dish_name() {
        assert_eq!(sanitize_dish_name("<b>Kuru Fasulye</b>"), "Kuru Fasulye");
        assert_eq!(sanitize_dish_name("<script>alert(1)</script>Pilav"), "alert(1)Pilav");
        assert_eq!(sanitize_dish_name("Tavuk &amp; Pilav"), "Tavuk & Pilav");
        assert_eq!(sanitize_dish_name("Köfte &lt;Leziz&gt;"), "Köfte <Leziz>");
        assert_eq!(sanitize_dish_name("Köfte < 100g"), "Köfte < 100g");
        assert_eq!(sanitize_dish_name("  Çorba   ve   Ekmek  "), "Çorba ve Ekmek");
    }

    #[tokio::test]
    #[ignore = "requires live postgres database"]
    async fn test_menu_cryptographic_chain_integrity() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = sea_orm::Database::connect(&database_url).await.unwrap();

        // 1. Ensure test city exists or create one
        let test_city_slug = "integrity_test_city";
        let existing_city = cities::Entity::find()
            .filter(cities::Column::Slug.eq(test_city_slug))
            .one(&db)
            .await
            .unwrap();

        let city_id = match existing_city {
            Some(c) => c.id,
            None => {
                let new_city = cities::ActiveModel {
                    name: Set("Integrity Test City".to_string()),
                    slug: Set(test_city_slug.to_string()),
                    ..Default::default()
                };
                new_city.insert(&db).await.unwrap().id
            }
        };

        // Clean up any existing menus for this city to ensure clean state
        let _ = menus::Entity::delete_many()
            .filter(menus::Column::CityId.eq(city_id))
            .exec(&db)
            .await;

        // 2. Upsert Day 1 Menu (will be the genesis for this test run)
        let date1 = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        let dishes1 = vec![
            vec![crate::parser::models::MenuComponent { name: "Mercimek Çorbası".to_string(), amount: None, calories: None, category: None }],
            vec![crate::parser::models::MenuComponent { name: "Tavuk Izgara".to_string(), amount: None, calories: None, category: None }]
        ];
        upsert_menu(
            &db,
            city_id,
            date1,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner,
            "test_scraper".to_string(),
            None, // submitted_by
            dishes1,
            vec![],
            vec![],
            Some(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved),
            None,
            None,
        ).await.expect("Day 1 menu upsert failed");

        // Fetch Day 1 Menu and verify hash exists
        let menu1 = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date1))
            .one(&db)
            .await
            .unwrap()
            .expect("Day 1 menu should exist");

        let hash1 = menu1.merkle_root.expect("Day 1 menu should have a hash calculated");

        // 3. Upsert Day 2 Menu (references Day 1 in the hash chain)
        let date2 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let dishes2 = vec![
            vec![crate::parser::models::MenuComponent { name: "Ezogelin Çorbası".to_string(), amount: None, calories: None, category: None }],
            vec![crate::parser::models::MenuComponent { name: "Et Döner".to_string(), amount: None, calories: None, category: None }]
        ];
        upsert_menu(
            &db,
            city_id,
            date2,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner,
            "test_scraper".to_string(),
            None, // submitted_by
            dishes2,
            vec![],
            vec![],
            Some(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved),
            None,
            None,
        ).await.expect("Day 2 menu upsert failed");

        // Fetch Day 2 Menu
        let menu2 = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date2))
            .one(&db)
            .await
            .unwrap()
            .expect("Day 2 menu should exist");

        // Verify hash link references Day 1's hash correctly!
        let hash2 = menu2.merkle_root.expect("Day 2 menu should have a hash calculated");
        assert_eq!(menu2.previous_hash, Some(hash1.clone()));
        assert_ne!(hash1, hash2);

        // Clean up test menus and city
        let _ = menus::Entity::delete_many()
            .filter(menus::Column::CityId.eq(city_id))
            .exec(&db)
            .await;

        let _ = cities::Entity::delete_by_id(city_id)
            .exec(&db)
            .await;
    }

    #[tokio::test]
    #[ignore = "Live external network test"]
    async fn test_live_kyk_token() {
        let client = reqwest::Client::builder().cookie_store(true).build().unwrap();
        let token = super::fetch_antiforgery_token(&client).await;
        println!("Extracted token: {:?}", token);
        assert!(token.is_ok());
    }

    #[tokio::test]
    #[ignore = "Live external network test"]
    async fn test_live_get_menu() {
        let client = reqwest::Client::builder().cookie_store(true).build().unwrap();
        let token = super::fetch_antiforgery_token(&client).await.unwrap();
        let res = client.get("https://kykyemek.com/Menu/GetDailyMenu/istanbul")
            .query(&[("city", "istanbul"), ("mealType", "true"), ("monthShift", "0"), ("hidePast", "false")])
            .header("User-Agent", super::BROWSER_UA)
            .header("sec-ch-ua", super::SEC_CH_UA)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("RequestVerificationToken", token.as_str())
            .header("Referer", "https://kykyemek.com/")
            .send().await.unwrap();
        println!("Status: {}", res.status());
        let body = res.text().await.unwrap();
        println!("Body length: {}", body.len());
        assert!(body.contains("Kremal"));
    }
}


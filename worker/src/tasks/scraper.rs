use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use rand::Rng;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
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
    persist_ban_state(until);
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

/// Kalıcı ban durumu dosyası.
///
/// Devre kesici durumu yalnızca süreç belleğinde tutulduğunda her yeniden başlatmada
/// sıfırlanır ve worker banlı olduğu hâlde yeniden istek atar. Bu da ban süresini
/// uzatır. Durum bu dosyada saklanır ve yeniden başlatmalara dayanır.
fn ban_state_path() -> String {
    std::env::var("KYK_BAN_STATE_FILE").unwrap_or_else(|_| "/app/cache/kyk_ban_until".to_string())
}

/// Kalıcı ban durumunu süreç başına bir kez yükler.
fn ensure_ban_state_loaded() {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        let Ok(raw) = std::fs::read_to_string(ban_state_path()) else {
            return;
        };
        let Ok(until) = raw.trim().parse::<u64>() else {
            return;
        };
        let now = chrono::Utc::now().timestamp().max(0) as u64;
        if until > now {
            KYK_BANNED_UNTIL.store(until, Ordering::Relaxed);
            tracing::warn!(
                "[KYK-BREAKER] Kalıcı durumdan ban yüklendi. Kalan bekleme: {} sn.",
                until - now
            );
        }
    });
}

/// Ban bitiş zamanını diske yazar (en iyi çaba).
fn persist_ban_state(until: u64) {
    let path = ban_state_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, until.to_string());
}

pub fn is_banned() -> bool {
    ensure_ban_state_loaded();
    let until = KYK_BANNED_UNTIL.load(Ordering::Relaxed);
    until > 0 && (chrono::Utc::now().timestamp().max(0) as u64) < until
}

/// Devre kesici durumunu döner. Banlıysa kalan saniyeyi, değilse None döner.
pub fn get_ban_status() -> Option<u64> {
    ensure_ban_state_loaded();
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
    persist_ban_state(0);
}

/// Chrome 144 (LTS) User-Agent. Tek noktadan yönetilir.
pub const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36";
pub const SEC_CH_UA: &str =
    "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"144\", \"Google Chrome\";v=\"144\"";

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

/// Bu tarama turunda yapılan dinamik Al Götür (fastmenu) istek sayısı.
/// Tur başına üst sınırla toplu istek (burst) engellenir.
static FASTMENU_FETCHED_THIS_CYCLE: AtomicUsize = AtomicUsize::new(0);

/// Tur başına en fazla dinamik Al Götür isteği. `KYK_FASTMENU_MAX_PER_CYCLE` ile ayarlanır.
fn fastmenu_max_per_cycle() -> usize {
    std::env::var("KYK_FASTMENU_MAX_PER_CYCLE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|v| *v > 0)
        .unwrap_or(20)
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
use crate::parser::kykyemek::parse_kykyemek_html;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};
use shared::entities::{
    cities, menu_dishes, menus,
    sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum},
};

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
        "ankara",
        "antalya",
        "canakkale",
        "erzurum",
        "eskisehir",
        "gaziantep",
        "isparta",
        "istanbul",
        "izmir",
        "kahramanmaras",
        "karabuk",
        "kirklareli",
        "konya",
        "sakarya",
        "sivas",
        "trabzon",
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
        regex::Regex::new(r#"name=["']__RequestVerificationToken["'][^>]*value=["']([^"']+)["']"#)
            .unwrap()
    });

    if let Some(caps) = re.captures(html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    static TOKEN_FALLBACK: OnceLock<regex::Regex> = OnceLock::new();
    let re_fb = TOKEN_FALLBACK.get_or_init(|| {
        regex::Regex::new(r#"value=["']([^"']+)["'][^>]*name=["']__RequestVerificationToken["']"#)
            .unwrap()
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
    if matches!(
        res.status(),
        reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::TOO_MANY_REQUESTS
    ) {
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
            tracing::warn!(
                "[KYKYEMEK-SESSION] Oturum başlatılamadı: {:?}. Düz istek deneniyor.",
                e
            );
            (None, extract_cities_from_kykyemek_html(""))
        }
    };

    {
        use rand::seq::SliceRandom;
        active_slugs.shuffle(&mut rand::thread_rng());
    }

    // Bu turda yapılan dinamik Al Götür isteklerini sıfırla (tur başına üst sınır).
    FASTMENU_FETCHED_THIS_CYCLE.store(0, Ordering::Relaxed);

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

        tracing::info!(
            "[KYKYEMEK-BULLETIN] Şehir için aylık bülten çekiliyor: {}...",
            city.name
        );

        // Ayın ilk 10 gününde bir önceki ayın menülerini de çekerek ay geçişlerindeki boşlukları doldur
        let shifts: Vec<&str> = if chrono::Utc::now().date_naive().day() <= 10 {
            vec!["-1", "0"]
        } else {
            vec!["0"]
        };

        for shift in &shifts {
            // 1. Kahvaltı Bülteni
            match fetch_and_save(
                db,
                client,
                &city,
                "breakfast",
                MealTypeEnum::Breakfast,
                shift,
                &mut token_opt,
                &mut shutdown_rx,
            )
            .await
            {
                Ok(Some(count)) => total_saved += count,
                Ok(None) => return Ok(total_saved),
                Err(e) => {
                    tracing::warn!(city = %city.slug, meal = "breakfast", shift = %shift, "Kahvaltı bülteni alınamadı: {:?}", e)
                }
            }

            // Kibar gecikme (3.5 - 6.5s)
            let delay_ms = rand::thread_rng().gen_range(3500..=6500);
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(total_saved);
            }

            // 2. Akşam Yemeği Bülteni
            match fetch_and_save(
                db,
                client,
                &city,
                "dinner",
                MealTypeEnum::Dinner,
                shift,
                &mut token_opt,
                &mut shutdown_rx,
            )
            .await
            {
                Ok(Some(count)) => total_saved += count,
                Ok(None) => return Ok(total_saved),
                Err(e) => {
                    tracing::warn!(city = %city.slug, meal = "dinner", shift = %shift, "Akşam yemeği bülteni alınamadı: {:?}", e)
                }
            }

            // Kibar gecikme (3.5 - 6.5s)
            let delay_ms = rand::thread_rng().gen_range(3500..=6500);
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(total_saved);
            }
        }
    }

    tracing::info!(
        "[KYKYEMEK-BULLETIN] Kykyemek aylık bülten taraması tamamlandı: {} menü güncellendi.",
        total_saved
    );
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
                tracing::info!(
                    "Fallback kaynaklardan {} eksik menü dolduruldu.",
                    fallback_count
                );
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
        match super::fallback_scraper::run_historical_gap_fill(
            db,
            client,
            shutdown_rx.clone(),
            hist_months,
        )
        .await
        {
            Ok(hist_count) => {
                if hist_count > 0 {
                    tracing::info!("Gecmis ay bosluklarindan {} menü dolduruldu.", hist_count);
                }
                total_fetched += hist_count;
            }
            Err(e) => tracing::error!("[HISTORY] Gecmis ay taramasında hata: {:?}", e),
        }
    }

    // Bu turda yeni eklenen menülerin şehirlerini arama motoru dizinlemesi için IndexNow'a bildir.
    // INDEXNOW_KEY atanmamışsa işlem yapmaz; hata durumunda yalnızca warn loglanır.
    // Yalnızca kanonik /{sehir} hub URL'leri gönderilir (yönlendirme URL'leri veya
    // ikincil /menu/{id} gönderilmez).
    if let Some(config) = super::indexnow::IndexNowConfig::from_env() {
        super::indexnow::ping_new_day_urls(db, client, &config).await;
    }

    tracing::info!(
        "Kykyemek tarama döngüsü tamamlandı. Toplam {} menü işlendi.",
        total_fetched
    );
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
    let is_dinner = if kyk_meal_type == "dinner" {
        "true"
    } else {
        "false"
    };
    let url = format!("https://kykyemek.com/Menu/GetDailyMenu/{}", city.slug);

    let mut attempt = 0;
    let max_retries = 3;
    let mut response = None;

    while attempt <= max_retries {
        if *shutdown_rx.borrow() {
            return Ok(None);
        }

        let mut req = with_xhr_headers(client.get(&url).query(&[
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
                        streak,
                        city.name,
                        wait_secs,
                        attempt + 1,
                        max_retries
                    );
                    if sleep_cancelable(wait_secs * 1000, shutdown_rx).await {
                        return Ok(None);
                    }
                } else if status == reqwest::StatusCode::FORBIDDEN {
                    // 403: büyük olasılıkla IP ban. Retry ile ısrar etme, devreyi kes.
                    trip_ban(&format!(
                        "HTTP 403 - {} ({}) [ana tarama]",
                        city.name, kyk_meal_type
                    ))
                    .await;
                    anyhow::bail!(
                        "kykyemek IP ban şüphesi (HTTP 403): {} ({})",
                        city.name,
                        kyk_meal_type
                    );
                } else if status == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!("HTTP 401 (Yetkisiz), yeni oturum token'ı alınıyor...");
                    if let Ok((new_token, _)) = fetch_kykyemek_session(client).await {
                        *token_opt = Some(new_token);
                    }
                } else {
                    tracing::warn!(
                        "HTTP durum kodu hatası: {}, Deneme: {}",
                        status,
                        attempt + 1
                    );
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
            let alert_msg = format!(
                "Kykyemek sunucu hatası: {} (öğün: {}) için maksimum deneme sayısına ulaşıldı.",
                city.name, kyk_meal_type
            );
            let _ =
                shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
            anyhow::bail!(alert_msg);
        }
    };
    let body_text = res.text().await?;

    let html_content = if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
        json_val
            .get("html")
            .and_then(|h| h.as_str())
            .unwrap_or(&body_text)
            .to_string()
    } else {
        body_text
    };

    if !html_content.contains("cardStyle")
        && !html_content.contains("card-body")
        && !html_content.contains("Menü bulunamadı")
        && !html_content.contains("bulunamadı")
        && html_content.len() > 100
    {
        let sample: String = html_content
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ")
            .replace('\r', "");
        let alert_msg = format!(
            "KYK HTML şablon anomalisi algılandı! Şehir: {}, Öğün: {}, Shift: {}. Dönen içerik `.cardStyle` içermiyor. Kesit: `{}`",
            city.name, kyk_meal_type, month_shift, sample.trim()
        );
        tracing::warn!("{}", alert_msg);
        let _ = shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
    }

    // Dinamik Al Götür (Takeaway) Ön-Yükleme ve Önbellekleme:
    // Kartlardaki tüm data-fastmenus UUID'lerini topla; henüz önbellekte olmayanları
    // /Menu/GetFastMenuFoods üzerinden tek seferlik çekip parse_fast_menu_foods_html ile önbelleğe yaz.
    // std::sync::RwLock kilitleri get_cached_fastmenu / insert_cached_fastmenu içinde nanosaniyelik açılıp
    // kapandığından, await çağrısı sırasında elde hiçbir kilit tutulmaz (Send trait & thread starvation koruması).
    // Kibar tarama: her dinamik Al Götür isteği arasında gecikme uygulanır ve tur başına
    // istek sayısı sınırlanır. Aksi halde ilk turda (önbellek soğukken) yüzlerce istek
    // saniyeler içinde gidip karşı tarafta toplu istek (DDoS) korumasını tetikleyebilir.
    let fastmenu_items = crate::parser::kykyemek::extract_fastmenu_items(&html_content);
    let max_per_cycle = fastmenu_max_per_cycle();
    for (fast_id, fast_name) in fastmenu_items {
        if *shutdown_rx.borrow() || is_banned() {
            break;
        }
        if crate::parser::takeaway::get_cached_fastmenu(&fast_id).is_some() {
            continue;
        }
        if FASTMENU_FETCHED_THIS_CYCLE.load(Ordering::Relaxed) >= max_per_cycle {
            tracing::info!(
                "[TAKEAWAY] Tur başına Al Götür istek sınırına ulaşıldı ({}); kalan paketler sonraki turlara bırakıldı.",
                max_per_cycle
            );
            break;
        }
        FASTMENU_FETCHED_THIS_CYCLE.fetch_add(1, Ordering::Relaxed);

        let fast_url = "https://kykyemek.com/Menu/GetFastMenuFoods";
        let req = with_xhr_headers(client.get(fast_url).query(&[("id", fast_id.as_str())]))
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://kykyemek.com/")
            .timeout(std::time::Duration::from_secs(10));

        match req.send().await {
            Ok(res) if res.status().is_success() => {
                if let Ok(foods_html) = res.text().await {
                    let slots = crate::parser::takeaway::parse_fast_menu_foods_html(&foods_html);
                    if !slots.is_empty() {
                        tracing::info!(
                            "[TAKEAWAY] Dinamik Al Götür menüsü başarıyla çekildi: {} (id: {}, {} slot)",
                            fast_name, fast_id, slots.len()
                        );
                        crate::parser::takeaway::insert_cached_fastmenu(fast_id, slots);
                    }
                }
            }
            Ok(res) => {
                tracing::warn!(
                    "[TAKEAWAY] Dinamik Al Götür çekilemedi (HTTP {}): {} (id: {}). Statik fallback kullanılacak.",
                    res.status(), fast_name, fast_id
                );
            }
            Err(err) => {
                tracing::warn!(
                    "[TAKEAWAY] Dinamik Al Götür isteği başarısız oldu: {} (id: {}): {}. Statik fallback kullanılacak.",
                    fast_name, fast_id, err
                );
            }
        }

        // Kibar gecikme (1.2 - 2.6 sn).
        let delay_ms = rand::thread_rng().gen_range(1200..=2600);
        if sleep_cancelable(delay_ms, shutdown_rx).await {
            break;
        }
    }

    let parsed_menus = parse_kykyemek_html(&html_content, &city.slug, kyk_meal_type);
    let mut count = 0;

    let incoming_days: Vec<(NaiveDate, Vec<String>)> = parsed_menus
        .iter()
        .map(|m| {
            let dishes = m
                .dishes
                .iter()
                .flat_map(|group| group.iter().map(|c| c.name.clone()))
                .collect();
            (m.date, dishes)
        })
        .collect();

    let stale_match = crate::parser::stale_detector::StaleSequenceDetector::check_stale_for_city(
        db,
        city.id,
        meal_type_enum.clone(),
        &incoming_days,
    )
    .await
    .unwrap_or(None);

    let (batch_source_type, batch_status_override) = if let Some(ref stale) = stale_match {
        let alert_msg = format!(
            "Bayat veri serisi algılandı! Şehir: {}, Öğün: {:?}, Gün aralığı: {}-{} ({} ardışık gün geçen ayla birebir aynı). Gelen bülten karantinaya alınıyor.",
            city.name, meal_type_enum, stale.start_date, stale.end_date, stale.matching_days_count
        );
        tracing::warn!("{}", alert_msg);
        let _ = shared::services::alerting::AlertingService::send_webhook_alert(&alert_msg).await;
        ("kykyemek-stale".to_string(), Some(MenuStatusEnum::Pending))
    } else {
        ("kykyemek".to_string(), None)
    };

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
            batch_source_type.clone(),
            None,
            menu.dishes,
            vec![], // celiac_dishes
            menu.takeaways,
            batch_status_override.clone(),
            menu.min_calories,
            menu.max_calories,
        )
        .await?;
        count += 1;
    }

    Ok(Some(count))
}

// GÜVENLİK NOTU (SA-15): "kepce-kullanici" kaynağı `upsert_menu` içinde otomatik
// APPROVED yapılır ve önceliği kykyemek'ten yüksektir. Bu kaynak türü YALNIZCA
// operatörün lokal drop-zone klasöründen (file_ingest) gelmelidir. Kullanıcı
// kaynaklı API akışları (ingestion) bu fonksiyona bağlanırsa otomatik onay
// moderation bypass'ına dönüşür - bu tabloyu değiştirirken bunu göz önünde tut.
pub(crate) fn is_quarantined_source(source: &str) -> bool {
    shared::services::source_registry::SourceRegistry::is_quarantined(source)
}

pub(crate) fn get_source_priority(source: &str) -> i32 {
    shared::services::source_registry::SourceRegistry::resolve(source).priority
}

pub(crate) fn check_dish_consensus(
    existing_dishes: &[(
        menu_dishes::Model,
        Option<shared::entities::dish_aliases::Model>,
    )],
    incoming_dishes: &[Vec<crate::parser::models::MenuComponent>],
) -> bool {
    let existing_names: std::collections::HashSet<String> = existing_dishes
        .iter()
        .filter(|(md, _)| !md.is_alternative)
        .filter_map(|(_, alias)| {
            alias
                .as_ref()
                .map(|a| crate::parser::normalizer::normalize_food_name(&a.name))
        })
        .filter(|n| !n.is_empty())
        .collect();

    let incoming_names: std::collections::HashSet<String> = incoming_dishes
        .iter()
        .filter_map(|group| {
            group
                .first()
                .map(|c| crate::parser::normalizer::normalize_food_name(&c.name))
        })
        .filter(|n| !n.is_empty())
        .collect();

    if existing_names.is_empty() || incoming_names.is_empty() {
        return false;
    }

    let common_count = existing_names.intersection(&incoming_names).count();
    common_count >= 2
        || (common_count >= 1
            && (common_count * 2 >= existing_names.len()
                || common_count * 2 >= incoming_names.len()))
}

fn parse_dish_calories(raw: &Option<String>) -> Option<i32> {
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
) -> Result<bool> {
    // Yapısal kalite ve çöp metin kapısı:
    // 1. Her bir bileşeni ContentGuard ile kontrol et; çöp navigasyon ve bürokrat adlarını filtrele.
    let dishes: Vec<Vec<crate::parser::models::MenuComponent>> = dishes
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .filter(|c| {
                    let trimmed = c.name.trim();
                    !trimmed.is_empty()
                        && !shared::services::content_guard::ContentGuard::is_junk_dish_text(
                            trimmed,
                        )
                })
                .collect::<Vec<_>>()
        })
        .filter(|group| !group.is_empty())
        .collect();

    let celiac_dishes: Vec<Vec<crate::parser::models::MenuComponent>> = celiac_dishes
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .filter(|c| {
                    let trimmed = c.name.trim();
                    !trimmed.is_empty()
                        && !shared::services::content_guard::ContentGuard::is_junk_dish_text(
                            trimmed,
                        )
                })
                .collect::<Vec<_>>()
        })
        .filter(|group| !group.is_empty())
        .collect();

    let takeaways: Vec<(String, Vec<Vec<crate::parser::models::MenuComponent>>)> = takeaways
        .into_iter()
        .map(|(pkg, groups)| {
            let filtered_groups: Vec<Vec<crate::parser::models::MenuComponent>> = groups
                .into_iter()
                .map(|group| {
                    group
                        .into_iter()
                        .filter(|c| {
                            let trimmed = c.name.trim();
                            !trimmed.is_empty() && !shared::services::content_guard::ContentGuard::is_junk_dish_text(trimmed)
                        })
                        .collect::<Vec<_>>()
                })
                .filter(|group| !group.is_empty())
                .collect();
            (pkg, filtered_groups)
        })
        .filter(|(_, groups)| !groups.is_empty())
        .collect();

    let valid_primary_count = dishes.len();
    let has_celiac = !celiac_dishes.is_empty();
    let has_takeaways = !takeaways.is_empty();

    // Bir KYK menüsünün geçerli sayılabilmesi için en az 2 geçerli yemek içermesi gerekir.
    // Navigasyon kalıntısı tekil satırlar veya tüm satırları çöp olan menüler veritabanına alınmaz.
    if valid_primary_count < 2 && !has_celiac && !has_takeaways {
        tracing::warn!(
            "upsert_menu reddedildi: yetersiz veya çöp yemek listesi (geçerli kap: {}, city_id: {}, tarih: {}, öğün: {:?}, kaynak: {})",
            valid_primary_count, city_id, date, meal_type, source_type
        );
        return Ok(false);
    }

    let target_status = match target_status_override {
        Some(status) => status,
        None => {
            let meta = shared::services::source_registry::SourceRegistry::resolve(&source_type);
            if meta.is_auto_approvable {
                MenuStatusEnum::Approved
            } else {
                MenuStatusEnum::Pending
            }
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
    let mut existing_dishes_list = Vec::new();

    if let Some(ref m) = existing_menu {
        // Eğer mevcut menü bu kaynak için daha önce reddedildiyse (Rejected),
        // aynı kaynaktan gelen verilerle menüyü tekrar diriltme veya güncelleme.
        if m.status == MenuStatusEnum::Rejected && m.source_type.as_deref() == Some(&source_type) {
            tracing::debug!(
                "upsert_menu atlandı: menü bu kaynak ({}) için daha önce reddedilmiş (city_id: {}, tarih: {}, öğün: {:?})",
                source_type, city_id, date, meal_type
            );
            txn.rollback().await?;
            return Ok(false);
        }

        let existing_dishes = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(m.id))
            .find_also_related(shared::entities::dish_aliases::Entity)
            .all(&txn)
            .await?;

        for (d, _) in &existing_dishes {
            let key = if d.is_alternative {
                DishSlotKey::Alternative(d.package_name.clone(), d.order_index, d.dish_alias_id)
            } else {
                DishSlotKey::Primary(d.package_name.clone(), d.order_index)
            };
            existing_map.insert(key, d.clone());
        }
        existing_dishes_list = existing_dishes;

        let current_priority = get_source_priority(m.source_type.as_deref().unwrap_or(""));
        let existing_source = m.source_type.as_deref().unwrap_or("");
        let incoming_is_quarantined = is_quarantined_source(&source_type);
        let existing_is_quarantined = is_quarantined_source(existing_source);

        // Dinamik Kalite Skoru (CQS) Hesaplaması
        let existing_dish_inputs: Vec<shared::services::quality_score::DishInput> =
            existing_dishes_list
                .iter()
                .map(|(d, alias)| {
                    let name = alias.as_ref().map(|a| a.name.clone()).unwrap_or_default();
                    let weight = d.amount.as_ref().and_then(|a| {
                        let num: String = a.chars().filter(|c| c.is_ascii_digit()).collect();
                        num.parse::<i32>().ok()
                    });
                    shared::services::quality_score::DishInput {
                        name,
                        weight_g: weight,
                        calories: d.calories,
                        is_alternative: d.is_alternative,
                    }
                })
                .collect();

        let existing_meal_str = match m.meal_type {
            MealTypeEnum::Breakfast => "breakfast",
            MealTypeEnum::Lunch => "lunch",
            MealTypeEnum::Dinner => "dinner",
        };

        let existing_quality_input = shared::services::quality_score::MenuQualityInput {
            meal_type: existing_meal_str.to_string(),
            primary_dishes: existing_dish_inputs,
            has_celiac: existing_dishes_list
                .iter()
                .any(|(d, _)| d.package_name == "celiac" || d.package_name == "glutensiz"),
            has_takeaways: existing_dishes_list.iter().any(|(d, _)| {
                d.package_name != "standard"
                    && d.package_name != "celiac"
                    && d.package_name != "glutensiz"
            }),
            calorie_min: m.calorie_range_min,
            calorie_max: m.calorie_range_max,
            anomaly_score: None,
            dictionary_match_ratio: None,
        };
        let existing_quality = shared::services::quality_score::QualityScoreService::calculate(
            &existing_quality_input,
        );

        let incoming_meal_str = match meal_type {
            MealTypeEnum::Breakfast => "breakfast",
            MealTypeEnum::Lunch => "lunch",
            MealTypeEnum::Dinner => "dinner",
        };

        let mut incoming_dish_inputs = Vec::new();
        for group in &dishes {
            for (idx, comp) in group.iter().enumerate() {
                if comp.name.trim().is_empty() {
                    continue;
                }
                let cal = parse_dish_calories(&comp.calories);
                let weight = comp.amount.as_ref().and_then(|a| {
                    let num: String = a.chars().filter(|c| c.is_ascii_digit()).collect();
                    num.parse::<i32>().ok()
                });
                incoming_dish_inputs.push(shared::services::quality_score::DishInput {
                    name: comp.name.clone(),
                    weight_g: weight,
                    calories: cal,
                    is_alternative: idx > 0,
                });
            }
        }

        let all_text = dishes
            .iter()
            .flat_map(|g| g.iter().map(|c| c.name.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        let dict_ratio = crate::parser::dictionary::calculate_match_ratio(&all_text);

        let incoming_quality_input = shared::services::quality_score::MenuQualityInput {
            meal_type: incoming_meal_str.to_string(),
            primary_dishes: incoming_dish_inputs,
            has_celiac,
            has_takeaways,
            calorie_min: calorie_range_min,
            calorie_max: calorie_range_max,
            anomaly_score: None,
            dictionary_match_ratio: Some(dict_ratio),
        };
        let incoming_quality = shared::services::quality_score::QualityScoreService::calculate(
            &incoming_quality_input,
        );

        let existing_meta =
            shared::services::source_registry::SourceRegistry::resolve(existing_source);
        let incoming_meta =
            shared::services::source_registry::SourceRegistry::resolve(&source_type);

        let quality_delta = incoming_quality.total - existing_quality.total;
        let priority_delta = incoming_priority - current_priority;

        let should_archive_incoming = if incoming_is_quarantined && !existing_is_quarantined {
            // 1. Karantina Koruması: Karantinadaki kaynak güvenilir kaynağı asla ezemez
            true
        } else if existing_meta.tier == shared::services::source_registry::TrustTier::GroundTruth
            && incoming_meta.tier < shared::services::source_registry::TrustTier::GroundTruth
        {
            // 2. Saha Gerçeği Koruması: Mevcut menü saha teyitliyse (admin veya teyitli kullanıcı/pano),
            // merkezi web kazıyıcıları bu menüyü ezemez.
            tracing::debug!(
                "Saha gerçeği koruması: Mevcut menü ({}) saha teyitli (GroundTruth), gelen ({}) menüsü ezemez. Arşivleniyor.",
                existing_source, source_type
            );
            true
        } else if source_type == existing_source {
            // 3. Aynı Kaynak Güncellemesi (Self-Poisoning / Regresyon Koruması):
            // Eğer gelen verinin kalitesi mevcut kayıttan belirgin şekilde düşükse
            // (örneğin scraper 4 kaplık menüyü 1 kaba düşürdü, Cloudflare engeli, eksik parse):
            if quality_delta < -10 {
                tracing::warn!(
                    "Aynı kaynak regresyon koruması: kaynak {} için mevcut skor {} iken gelen skor {}. Gelen arşivleniyor.",
                    source_type, existing_quality.total, incoming_quality.total
                );
                true
            } else {
                // Kalite eşit, daha iyi veya tolere edilebilir değişim: Meşru revizyon
                false
            }
        } else {
            // 4. Farklı Kaynaklar Arası Karar (Delta & Dinamik Marjinal Üstünlük):
            if quality_delta >= 15 && !incoming_is_quarantined {
                // Gelen menü belirgin şekilde daha zengin/kaliteli (+15 delta)
                tracing::info!(
                    "Kalite üstünlüğü ile menü güncellemesi: gelen ({}) skoru {}, mevcut ({}) skoru {}. (Delta: +{})",
                    source_type, incoming_quality.total, existing_source, existing_quality.total, quality_delta
                );
                false
            } else if existing_quality.total < 45
                && incoming_quality.total >= 50
                && !incoming_is_quarantined
            {
                // Düşük kaliteli kayıt kurtarma
                tracing::info!(
                    "Düşük kaliteli kayıt kurtarma: mevcut ({}) skoru {} < 45 iken gelen ({}) skoru {}.",
                    existing_source, existing_quality.total, source_type, incoming_quality.total
                );
                false
            } else if quality_delta <= -15 && current_priority >= 4 {
                // Yüksek öncelikli ama bariz düşük kaliteli/eksik gelen veri engellenir
                tracing::warn!(
                    "Yüksek öncelikli ama düşük kaliteli kaynak reddedildi: gelen ({}) skoru {}, mevcut ({}) skoru {}.",
                    source_type, incoming_quality.total, existing_source, existing_quality.total
                );
                true
            } else {
                // Normal öncelik ve kalite karşılaştırması
                priority_delta < 0 || (priority_delta == 0 && quality_delta < 0)
            }
        };

        if should_archive_incoming {
            tracing::debug!(
                "Gelen menü ({}, tarih: {}, öğün: {:?}) mevcut menüye ({}) göre arşivleniyor (Mevcut Skor: {}, Gelen Skor: {}, Öncelik Farkı: {}).",
                source_type, date, meal_type, existing_source, existing_quality.total, incoming_quality.total, priority_delta
            );
            let payload = serde_json::json!({
                "dishes": dishes,
                "takeaways": takeaways
            });
            let meal_str = match meal_type {
                MealTypeEnum::Breakfast => "breakfast",
                MealTypeEnum::Lunch => "lunch",
                MealTypeEnum::Dinner => "dinner",
            };
            let existing_hist = shared::entities::menu_history::Entity::find()
                .filter(shared::entities::menu_history::Column::CityId.eq(city_id))
                .filter(shared::entities::menu_history::Column::ServeDate.eq(date))
                .filter(shared::entities::menu_history::Column::MealType.eq(meal_str))
                .filter(shared::entities::menu_history::Column::SourceType.eq(&source_type))
                .one(&txn)
                .await?;

            let mut already_in_hist = false;
            if let Some(eh) = existing_hist {
                if eh.dishes_payload == payload {
                    already_in_hist = true;
                }
            }

            if !already_in_hist {
                let hist = shared::entities::menu_history::ActiveModel {
                    city_id: Set(city_id),
                    serve_date: Set(date),
                    meal_type: Set(meal_str.to_string()),
                    source_type: Set(source_type.clone()),
                    submitted_by: Set(submitted_by),
                    dishes_payload: Set(payload),
                    ..Default::default()
                };
                hist.insert(&txn).await?;
            }

            // Konsensüs zenginleştirmesi: Eğer mevcut menü Onaylı ise ve gelen karantina kaynağı
            // kalori bilgisi getiriyorsa, çapraz aile konsensüsü ve yemek eşleşmesi sağlandığı takdirde kalori aralığı güncellenir.
            if m.status == MenuStatusEnum::Approved
                && (calorie_range_min.is_some() || calorie_range_max.is_some())
                && shared::services::source_registry::SourceRegistry::is_cross_family_consensus(
                    existing_source,
                    &source_type,
                )
                && check_dish_consensus(&existing_dishes_list, &dishes)
            {
                let mut update_m: menus::ActiveModel = m.clone().into();
                let mut changed = false;
                if m.calorie_range_min.is_none() && calorie_range_min.is_some() {
                    update_m.calorie_range_min = Set(calorie_range_min);
                    changed = true;
                }
                if m.calorie_range_max.is_none() && calorie_range_max.is_some() {
                    update_m.calorie_range_max = Set(calorie_range_max);
                    changed = true;
                }
                if changed {
                    update_m.update(&txn).await?;
                }
            }

            txn.commit().await?;
            return Ok(!already_in_hist);
        }
    }

    // Build target_map
    // Değer: (alias_id, dish_id, amount, calories)
    let mut target_map: HashMap<DishSlotKey, (i32, i32, Option<String>, Option<i32>)> =
        HashMap::new();
    let mut seen_package_dishes: std::collections::HashSet<(String, i32, i32)> =
        std::collections::HashSet::new();
    // Global (package_name, dish_alias_id) tekilliği: aynı yemek farklı slotlarda
    // tekrar ederse unique constraint (menu_id, dish_alias_id, package_name) patlar.
    let mut seen_package_aliases: std::collections::HashSet<(String, i32)> =
        std::collections::HashSet::new();

    for (i, dish_group) in dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let (alias_id, dish_id) =
                get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "NORMAL".to_string();
            let cals = parse_dish_calories(&comp.calories);

            // Aynı pakette ve aynı yuvada aynı dish_id tekrar ediyorsa atla
            if !seen_package_dishes.insert((package_name.clone(), order_index, dish_id)) {
                continue;
            }

            let key = if is_alternative {
                DishSlotKey::Alternative(package_name.clone(), order_index, alias_id)
            } else {
                DishSlotKey::Primary(package_name.clone(), order_index)
            };

            if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                *existing_amt = comp
                    .amount
                    .filter(|s| !s.trim().is_empty())
                    .or(existing_amt.take());
                *existing_cals = cals.or(*existing_cals);
            } else if seen_package_aliases.insert((package_name, alias_id)) {
                target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
            }
        }
    }

    for (i, dish_group) in celiac_dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let (alias_id, dish_id) =
                get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "ÇÖLYAK MENÜSÜ".to_string();
            let cals = parse_dish_calories(&comp.calories);

            if !seen_package_dishes.insert((package_name.clone(), order_index, dish_id)) {
                continue;
            }

            let key = if is_alternative {
                DishSlotKey::Alternative(package_name.clone(), order_index, alias_id)
            } else {
                DishSlotKey::Primary(package_name.clone(), order_index)
            };

            if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                *existing_amt = comp
                    .amount
                    .filter(|s| !s.trim().is_empty())
                    .or(existing_amt.take());
                *existing_cals = cals.or(*existing_cals);
            } else if seen_package_aliases.insert((package_name, alias_id)) {
                target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
            }
        }
    }

    for (package, package_dishes) in takeaways.into_iter() {
        let sanitized_package = sanitize_dish_name(&package);
        for (i, dish_group) in package_dishes.into_iter().enumerate() {
            let order_index = i as i32;
            for (j, comp) in dish_group.into_iter().enumerate() {
                let is_alternative = j > 0;
                let (alias_id, dish_id) =
                    get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
                let cals = parse_dish_calories(&comp.calories);

                if !seen_package_dishes.insert((sanitized_package.clone(), order_index, dish_id)) {
                    continue;
                }

                let key = if is_alternative {
                    DishSlotKey::Alternative(sanitized_package.clone(), order_index, alias_id)
                } else {
                    DishSlotKey::Primary(sanitized_package.clone(), order_index)
                };

                if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                    *existing_amt = comp
                        .amount
                        .filter(|s| !s.trim().is_empty())
                        .or(existing_amt.take());
                    *existing_cals = cals.or(*existing_cals);
                } else if seen_package_aliases.insert((sanitized_package.clone(), alias_id)) {
                    target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
                }
            }
        }
    }

    let menu_id = if let Some(ref m) = existing_menu {
        let same_len = existing_map.len() == target_map.len();
        let same_dishes = same_len
            && target_map
                .iter()
                .all(|(key, (alias_id, _dish_id, amt, cals))| {
                    if let Some(existing) = existing_map.get(key) {
                        existing.dish_alias_id == *alias_id
                            && (amt.is_none() || amt.as_deref() == existing.amount.as_deref())
                            && (cals.is_none() || *cals == existing.calories)
                    } else {
                        false
                    }
                });
        let same_calories =
            m.calorie_range_min == calorie_range_min && m.calorie_range_max == calorie_range_max;
        let same_source = m.source_type.as_deref() == Some(&source_type);

        if same_dishes && same_calories && same_source {
            tracing::trace!(
                "upsert_menu no-op: menü ve yemekler zaten güncel (city_id: {}, tarih: {}, öğün: {:?}, kaynak: {})",
                city_id, date, meal_type, source_type
            );
            txn.rollback().await?;
            return Ok(false);
        }

        // Eğer mevcut menüden farklı bir içerik geldiyse (revize edildiyse veya yeni kaynak geldiyse),
        // mevcut halini menu_history tablosuna arşivle
        let payload = serde_json::json!(existing_dishes_list
            .iter()
            .map(|(md, alias)| {
                serde_json::json!({
                    "name": alias.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                    "package_name": md.package_name.clone(),
                    "order_index": md.order_index,
                    "is_alternative": md.is_alternative
                })
            })
            .collect::<Vec<_>>());

        let hist = shared::entities::menu_history::ActiveModel {
            city_id: Set(m.city_id),
            serve_date: Set(m.serve_date),
            meal_type: Set(match m.meal_type {
                MealTypeEnum::Breakfast => "breakfast".to_string(),
                MealTypeEnum::Lunch => "lunch".to_string(),
                MealTypeEnum::Dinner => "dinner".to_string(),
            }),
            source_type: Set(m
                .source_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string())),
            submitted_by: Set(m.submitted_by),
            dishes_payload: Set(payload),
            ..Default::default()
        };
        hist.insert(&txn).await?;

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
        // Yeni eklenen menü gününü IndexNow bildirim kuyruğuna kaydet
        record_inserted_menu(city_id, date);
        res.id
    };

    // Akıllı Slot Uzlaşması (In-Place Diff & Preservation):
    let mut matched_existing_ids = std::collections::HashSet::new();

    // Mevcut satırların (package_name, dish_alias_id) ve (package_name, order_index, dish_id)
    // sahipliği: güncelleme/insert sırasında unique constraint ve slot trigger'ını
    // önceden denetlemek için kullanılır.
    let mut existing_alias_owner: std::collections::HashMap<(String, i32), i32> =
        std::collections::HashMap::new();
    let mut existing_dish_in_slot: std::collections::HashMap<(String, i32, i32), i32> =
        std::collections::HashMap::new();
    for (d, alias) in &existing_dishes_list {
        existing_alias_owner.insert((d.package_name.clone(), d.dish_alias_id), d.id);
        if let Some(a) = alias {
            if let Some(did) = a.dish_id {
                existing_dish_in_slot.insert((d.package_name.clone(), d.order_index, did), d.id);
            }
        }
    }

    for (key, (alias_id, dish_id, amount, calories)) in target_map.into_iter() {
        let (package_name, order_index, is_alternative) = match &key {
            DishSlotKey::Primary(pkg, idx) => (pkg.clone(), *idx, false),
            DishSlotKey::Alternative(pkg, idx, _) => (pkg.clone(), *idx, true),
        };

        if let Some(existing) = existing_map.get(&key) {
            // Yuva veritabanında zaten var: yeni satır eklemek yerine var olan satırı
            // yerinde güncelle ki Postgres unique constraint patlamasın.
            matched_existing_ids.insert(existing.id);

            // Güncelleme hedefi (package, alias) başka bir satır tarafından
            // sahiplenilmişse unique constraint'i önlemek için güncellemeyi atla.
            if let Some(owner_id) = existing_alias_owner.get(&(package_name.clone(), alias_id)) {
                if *owner_id != existing.id {
                    tracing::debug!(
                        "upsert_menu: alias {} paket '{}' içinde başka satırda mevcut, güncelleme atlandı (menu_id: {})",
                        alias_id, package_name, menu_id
                    );
                    continue;
                }
            }

            // Slot trigger koruması: hedef dish_id aynı yuvada başka bir satırda
            // zaten varsa güncelleme trigger'ı patlatır, atla.
            if let Some(owner_id) =
                existing_dish_in_slot.get(&(package_name.clone(), order_index, dish_id))
            {
                if *owner_id != existing.id {
                    tracing::debug!(
                        "upsert_menu: dish_id {} slot {} içinde başka satırda mevcut, güncelleme atlandı (menu_id: {})",
                        dish_id, order_index, menu_id
                    );
                    continue;
                }
            }

            let final_amount = amount
                .filter(|s| !s.trim().is_empty())
                .or(existing.amount.clone());
            let final_calories = calories.or(existing.calories);

            let mut active: menu_dishes::ActiveModel = existing.clone().into();
            active.dish_alias_id = Set(alias_id);
            active.amount = Set(final_amount);
            active.calories = Set(final_calories);
            active.update(&txn).await?;
            continue;
        }

        // Slot trigger koruması: aynı yuvada aynı dish_id zaten varsa insert atla.
        if let Some(owner_id) =
            existing_dish_in_slot.get(&(package_name.clone(), order_index, dish_id))
        {
            tracing::debug!(
                "upsert_menu: dish_id {} slot {} içinde zaten mevcut, insert atlandı (menu_id: {}, sahip: {})",
                dish_id, order_index, menu_id, owner_id
            );
            continue;
        }

        // Veritabanında daha önce hiç olmayan yepyeni bir slot ise insert et.
        // ON CONFLICT: yarış durumunda veya kaçırılan bir tekrarda unique
        // constraint patlamasını önlemek için upsert davranışı uygula.
        let stmt = sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            r#"
            INSERT INTO menu_dishes (menu_id, dish_alias_id, order_index, is_alternative, package_name, amount, calories)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (menu_id, dish_alias_id, package_name)
            DO UPDATE SET order_index = EXCLUDED.order_index,
                          is_alternative = EXCLUDED.is_alternative,
                          amount = COALESCE(EXCLUDED.amount, menu_dishes.amount),
                          calories = COALESCE(EXCLUDED.calories, menu_dishes.calories)
            "#,
            vec![
                menu_id.into(),
                alias_id.into(),
                order_index.into(),
                is_alternative.into(),
                package_name.into(),
                amount.into(),
                calories.into(),
            ],
        );
        txn.execute(stmt).await?;
    }

    // Yeni menüde artık yer almayan eski slotları temizle
    let obsolete_ids: Vec<i32> = existing_dishes_list
        .iter()
        .map(|(d, _)| d.id)
        .filter(|id| !matched_existing_ids.contains(id))
        .collect();

    if !obsolete_ids.is_empty() {
        menu_dishes::Entity::delete_many()
            .filter(menu_dishes::Column::Id.is_in(obsolete_ids))
            .exec(&txn)
            .await?;
    }

    txn.commit().await?;

    let menu = menus::Entity::find_by_id(menu_id).one(db).await?;
    if let Some(m) = menu {
        if m.status == MenuStatusEnum::Approved {
            shared::services::immutable_store::ImmutableStore::write_menu_hash(db, menu_id).await?;
        }
    }

    Ok(true)
}

pub async fn get_or_create_dish_alias(
    txn: &sea_orm::DatabaseTransaction,
    raw_name: &str,
    category: Option<String>,
) -> Result<(i32, i32)> {
    let sanitized = sanitize_dish_name(raw_name);
    let canonical_name = crate::parser::normalizer::normalize_food_name(&sanitized);
    let final_category =
        category.or_else(|| shared::services::categorizer::categorize_dish(&canonical_name));

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
        RETURNING id, dish_id
        "#,
        vec![
            canonical_name.into(),
            final_category.into(),
            sanitized.into(),
        ],
    );

    let query_res = txn.query_one(stmt).await?;

    if let Some(row) = query_res {
        let alias_id: i32 = row.try_get("", "id")?;
        let dish_id: i32 = row.try_get("", "dish_id")?;
        Ok((alias_id, dish_id))
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

    static RE_PLUS: OnceLock<regex::Regex> = OnceLock::new();
    let re_plus = RE_PLUS.get_or_init(|| regex::Regex::new(r"\s*\+\s*").unwrap());
    let spaced = re_plus.replace_all(&decoded, " + ").into_owned();

    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_dish_name() {
        assert_eq!(sanitize_dish_name("<b>Kuru Fasulye</b>"), "Kuru Fasulye");
        assert_eq!(
            sanitize_dish_name("<script>alert(1)</script>Pilav"),
            "alert(1)Pilav"
        );
        assert_eq!(sanitize_dish_name("Tavuk &amp; Pilav"), "Tavuk & Pilav");
        assert_eq!(sanitize_dish_name("Köfte &lt;Leziz&gt;"), "Köfte <Leziz>");
        assert_eq!(sanitize_dish_name("Köfte < 100g"), "Köfte < 100g");
        assert_eq!(
            sanitize_dish_name("  Çorba   ve   Ekmek  "),
            "Çorba ve Ekmek"
        );
        assert_eq!(sanitize_dish_name("Bal+tereyağ"), "Bal + tereyağ");
        assert_eq!(
            sanitize_dish_name("Tavuk Sote +Pilav"),
            "Tavuk Sote + Pilav"
        );
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
            vec![crate::parser::models::MenuComponent {
                name: "Mercimek Çorbası".to_string(),
                amount: None,
                calories: None,
                category: None,
            }],
            vec![crate::parser::models::MenuComponent {
                name: "Tavuk Izgara".to_string(),
                amount: None,
                calories: None,
                category: None,
            }],
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
        )
        .await
        .expect("Day 1 menu upsert failed");

        // Fetch Day 1 Menu and verify hash exists
        let menu1 = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date1))
            .one(&db)
            .await
            .unwrap()
            .expect("Day 1 menu should exist");

        let hash1 = menu1
            .merkle_root
            .expect("Day 1 menu should have a hash calculated");

        // 3. Upsert Day 2 Menu (references Day 1 in the hash chain)
        let date2 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let dishes2 = vec![
            vec![crate::parser::models::MenuComponent {
                name: "Ezogelin Çorbası".to_string(),
                amount: None,
                calories: None,
                category: None,
            }],
            vec![crate::parser::models::MenuComponent {
                name: "Et Döner".to_string(),
                amount: None,
                calories: None,
                category: None,
            }],
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
        )
        .await
        .expect("Day 2 menu upsert failed");

        // Fetch Day 2 Menu
        let menu2 = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date2))
            .one(&db)
            .await
            .unwrap()
            .expect("Day 2 menu should exist");

        // Verify hash link references Day 1's hash correctly!
        let hash2 = menu2
            .merkle_root
            .expect("Day 2 menu should have a hash calculated");
        assert_eq!(menu2.previous_hash, Some(hash1.clone()));
        assert_ne!(hash1, hash2);

        // Clean up test menus and city
        let _ = menus::Entity::delete_many()
            .filter(menus::Column::CityId.eq(city_id))
            .exec(&db)
            .await;

        let _ = cities::Entity::delete_by_id(city_id).exec(&db).await;
    }

    #[tokio::test]
    #[ignore = "Live external network test"]
    async fn test_live_kyk_token() {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let token = super::fetch_antiforgery_token(&client).await;
        println!("Extracted token: {:?}", token);
        assert!(token.is_ok());
    }

    #[tokio::test]
    #[ignore = "Live external network test"]
    async fn test_live_get_menu() {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let token = super::fetch_antiforgery_token(&client).await.unwrap();
        let res = client
            .get("https://kykyemek.com/Menu/GetDailyMenu/istanbul")
            .query(&[
                ("city", "istanbul"),
                ("mealType", "true"),
                ("monthShift", "0"),
                ("hidePast", "false"),
            ])
            .header("User-Agent", super::BROWSER_UA)
            .header("sec-ch-ua", super::SEC_CH_UA)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("RequestVerificationToken", token.as_str())
            .header("Referer", "https://kykyemek.com/")
            .send()
            .await
            .unwrap();
        println!("Status: {}", res.status());
        let body = res.text().await.unwrap();
        println!("Body length: {}", body.len());
        assert!(body.contains("Kremal"));
    }

    #[test]
    fn test_quarantined_source_priority_and_identification() {
        assert!(super::is_quarantined_source("yurtmenu"));
        assert!(super::is_quarantined_source("yurtmenu.net"));
        assert!(super::is_quarantined_source("yurtmenu.com"));
        assert!(super::is_quarantined_source("yurtmenu_live"));
        assert!(super::is_quarantined_source("kykyemek-stale"));
        assert!(super::is_quarantined_source("source-quarantine"));
        assert!(!super::is_quarantined_source("kykyemek.com"));
        assert!(!super::is_quarantined_source("kykmenum.com"));
        assert!(!super::is_quarantined_source("kepce-admin"));

        // Temiz toplayıcılar (kykyemek, kykmenum), karantinalı yurtmenu (2) üzerinde yer alır
        let kykyemek_prio = super::get_source_priority("kykyemek.com");
        let kykmenum_prio = super::get_source_priority("kykmenum.com");
        let yurtmenu_prio = super::get_source_priority("yurtmenu.net");
        assert!(kykyemek_prio >= kykmenum_prio);
        assert!(kykmenum_prio > yurtmenu_prio);
    }

    #[test]
    fn test_parse_dish_calories() {
        assert_eq!(
            super::parse_dish_calories(&Some("350 kcal".to_string())),
            Some(350)
        );
        assert_eq!(
            super::parse_dish_calories(&Some("200 - 300 kkal".to_string())),
            Some(250)
        );
        assert_eq!(
            super::parse_dish_calories(&Some("geçersiz".to_string())),
            None
        );
        assert_eq!(super::parse_dish_calories(&None), None);
    }

    #[test]
    fn test_check_dish_consensus() {
        use shared::entities::{dish_aliases, menu_dishes};

        let make_alias = |id: i32, name: &str| dish_aliases::Model {
            id,
            name: name.to_string(),
            dish_id: Some(id),
            created_at: None,
        };

        let make_menu_dish = |dish_alias_id: i32| menu_dishes::Model {
            id: 1,
            menu_id: 1,
            dish_alias_id,
            order_index: 0,
            package_name: "NORMAL".to_string(),
            amount: None,
            calories: None,
            is_alternative: false,
        };

        let existing = vec![
            (make_menu_dish(1), Some(make_alias(1, "Mercimek Çorbası"))),
            (make_menu_dish(2), Some(make_alias(2, "Orman Kebabı"))),
            (make_menu_dish(3), Some(make_alias(3, "Pirinç Pilavı"))),
            (make_menu_dish(4), Some(make_alias(4, "Ayran"))),
        ];

        // 1. En az 2 yemek eşleşiyorsa konsensüs vardır
        let incoming_match = vec![
            vec![crate::parser::models::MenuComponent::from(
                "Mercimek Çorbası",
            )],
            vec![crate::parser::models::MenuComponent::from("Orman Kebabı")],
            vec![crate::parser::models::MenuComponent::from("Bulgur Pilavı")],
        ];
        assert!(super::check_dish_consensus(&existing, &incoming_match));

        // 2. Tamamen alakasız menüde konsensüs yoktur
        let incoming_conflict = vec![
            vec![crate::parser::models::MenuComponent::from(
                "Tarhana Çorbası",
            )],
            vec![crate::parser::models::MenuComponent::from("Tavuk Sote")],
            vec![crate::parser::models::MenuComponent::from("Makarna")],
        ];
        assert!(!super::check_dish_consensus(&existing, &incoming_conflict));
    }

    #[test]
    fn test_decision_matrix_hierarchy_and_regression() {
        use shared::services::source_registry::{SourceRegistry, TrustTier};

        let admin_meta = SourceRegistry::resolve("kepce-admin");
        let user_meta = SourceRegistry::resolve("kepce-kullanici");
        let kykyemek_meta = SourceRegistry::resolve("kykyemek");
        let kykmenum_meta = SourceRegistry::resolve("kykmenum");
        let yurtmenu_meta = SourceRegistry::resolve("yurtmenu");

        // Saha ve yönetici her zaman toplayıcıların üstündedir
        assert_eq!(admin_meta.tier, TrustTier::GroundTruth);
        assert_eq!(user_meta.tier, TrustTier::GroundTruth);
        assert!(user_meta.priority > kykyemek_meta.priority);
        assert!(kykyemek_meta.priority >= kykmenum_meta.priority);
        assert!(kykmenum_meta.priority > yurtmenu_meta.priority);

        // Karantina kaynağı güvenilir kaynağın altında kalmalıdır
        assert_eq!(yurtmenu_meta.tier, TrustTier::Quarantined);
        assert_ne!(kykyemek_meta.tier, TrustTier::Quarantined);
    }

    #[tokio::test]
    #[ignore = "requires live postgres database"]
    async fn test_upsert_menu_deduplicates_dish_across_slots() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = sea_orm::Database::connect(&database_url).await.unwrap();

        // Test şehri hazırla
        let test_city_slug = "dedup_test_city";
        let city_id = match cities::Entity::find()
            .filter(cities::Column::Slug.eq(test_city_slug))
            .one(&db)
            .await
            .unwrap()
        {
            Some(c) => c.id,
            None => {
                cities::ActiveModel {
                    name: Set("Dedup Test City".to_string()),
                    slug: Set(test_city_slug.to_string()),
                    ..Default::default()
                }
                .insert(&db)
                .await
                .unwrap()
                .id
            }
        };

        let date = NaiveDate::from_ymd_opt(2099, 1, 1).unwrap();
        let _ = menus::Entity::delete_many()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date))
            .exec(&db)
            .await;

        // Aynı yemek iki farklı slotta: unique constraint
        // (menu_id, dish_alias_id, package_name) ihlal edilmemeli.
        let dishes = vec![
            vec![crate::parser::models::MenuComponent::from(
                "Mercimek Çorbası",
            )],
            vec![crate::parser::models::MenuComponent::from(
                "Mercimek Çorbası",
            )],
            vec![crate::parser::models::MenuComponent::from("Pirinç Pilavı")],
        ];

        let res = super::upsert_menu(
            &db,
            city_id,
            date,
            MealTypeEnum::Dinner,
            "kepce-admin".to_string(),
            None,
            dishes,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(res.is_ok(), "upsert_menu hata verdi: {:?}", res.err());

        // Aynı yemek yalnızca bir kez kaydedilmiş olmalı
        let menu = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date))
            .one(&db)
            .await
            .unwrap()
            .expect("menü oluşturulmalıydı");
        let count = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(menu.id))
            .all(&db)
            .await
            .unwrap()
            .len();
        assert_eq!(count, 2, "aynı yemek iki slotta tekrar etmemeli");

        // Temizlik
        let _ = menus::Entity::delete_many()
            .filter(menus::Column::CityId.eq(city_id))
            .exec(&db)
            .await;
    }
}

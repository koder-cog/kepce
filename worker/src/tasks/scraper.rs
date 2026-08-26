use anyhow::Result;
use chrono::{Datelike, NaiveDate};
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

fn is_banned() -> bool {
    let until = KYK_BANNED_UNTIL.load(Ordering::Relaxed);
    until > 0 && (chrono::Utc::now().timestamp().max(0) as u64) < until
}

/// Chrome 126 (Windows) User-Agent. Tek noktadan yönetilir.
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// Chrome 126'nın XHR/fetch isteklerinde gönderdiği Client Hints +
/// Fetch Metadata başlık seti. Sadece User-Agent taklidi yetmez; bu
/// başlıklar eksikse sunucu tarafı "kütüphane" kokusunu alır.
fn with_xhr_headers(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    req.header("User-Agent", BROWSER_UA)
        .header("sec-ch-ua", "\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Google Chrome\";v=\"126\"")
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
use crate::parser::kykyemek::parse_kyk_html;

async fn sleep_cancelable(ms: u64, shutdown_rx: &mut tokio::sync::watch::Receiver<bool>) -> bool {
    if *shutdown_rx.borrow() {
        return true;
    }
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_millis(ms)) => false,
        _ = shutdown_rx.changed() => true,
    }
}

pub async fn scrape_today_menus(
    db: &DatabaseConnection,
    client: &Client,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<usize> {
    if is_banned() {
        tracing::warn!("[KYK-BREAKER] Cooldown aktif - [TODAY] taraması bu tur atlanıyor.");
        return Ok(0);
    }

    // Alfabetik/deterministik gezinme sırası bot klişesidir; her turda karıştır.
    let mut active_cities = [
        "istanbul", "ankara", "izmir", "antalya", "canakkale", "erzurum",
        "eskisehir", "gaziantep", "isparta", "kahramanmaras", "karabuk",
        "kirklareli", "konya", "sakarya", "sivas", "trabzon"
    ];
    {
        use rand::seq::SliceRandom;
        active_cities.shuffle(&mut rand::thread_rng());
    }

    let mut token_opt = match fetch_antiforgery_token(client).await {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!("Kykyemek token alınamadı: {:?}", e);
            None
        }
    };

    let now = chrono::Local::now().naive_local().date();
    let current_day = now.day() as i32;
    let days_in_month = match now.month() {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if now.year() % 4 == 0 && (now.year() % 100 != 0 || now.year() % 400 == 0) { 29 } else { 28 },
        _ => 30,
    };

    let mut total_saved = 0;

    for slug in active_cities {
        if *shutdown_rx.borrow() {
            return Ok(total_saved);
        }

        let city_opt = cities::Entity::find()
            .filter(cities::Column::Slug.eq(slug))
            .one(db)
            .await?;

        let city = match city_opt {
            Some(c) => c,
            None => continue,
        };

        let start_of_month = match NaiveDate::from_ymd_opt(now.year(), now.month(), 1) {
            Some(d) => d,
            None => continue,
        };
        let end_of_month = match NaiveDate::from_ymd_opt(now.year(), now.month(), days_in_month as u32) {
            Some(d) => d,
            None => continue,
        };

        // DB-SKIP: Bu şehrin bu ayına ait onaylı menüleri çek.
        // Zaten veritabanında onaylı olan günler için dış sunucuya (kykyemek) tekrar istek ATMA.
        let existing_approved_dates: HashSet<NaiveDate> = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city.id))
            .filter(menus::Column::ServeDate.gte(start_of_month))
            .filter(menus::Column::ServeDate.lte(end_of_month))
            .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.serve_date)
            .collect();

        let skipped_count = (1..=days_in_month)
            .filter_map(|d| NaiveDate::from_ymd_opt(now.year(), now.month(), d as u32))
            .filter(|d| existing_approved_dates.contains(d))
            .count();

        tracing::info!(
            "[TODAY] Şehir için günün menüleri taranıyor: {} (toplam: {} gün, DB'de onaylı atlanan: {} gün)",
            city.name, days_in_month, skipped_count
        );

        for day in 1..=days_in_month {
            if *shutdown_rx.borrow() {
                return Ok(total_saved);
            }

            let target_date = match NaiveDate::from_ymd_opt(now.year(), now.month(), day as u32) {
                Some(d) => d,
                None => continue,
            };

            // DB-SKIP: Eğer bu günün menüsü veritabanında zaten onaylı olarak mevcutsa dış kaynağa istek ATMA.
            if existing_approved_dates.contains(&target_date) {
                continue;
            }

            let day_shift = day - current_day;
            let url = "https://kykyemek.com/Menu/GetDailyMenu";

            let mut req = with_xhr_headers(client.get(url)
                .query(&[
                    ("city", city.slug.as_str()),
                    ("mealType", "false"),
                    ("isToday", "true"),
                    ("dayShift", &day_shift.to_string()),
                ]))
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Accept", "application/json, text/javascript, */*; q=0.01")
                .header("Referer", "https://kykyemek.com/Menu/TodayMenu")
                .timeout(std::time::Duration::from_secs(30));

            if let Some(ref token) = token_opt {
                req = req
                    .header("RequestVerificationToken", token.as_str())
                    .header("__RequestVerificationToken", token.as_str());
            }

            let res = match req.send().await {
                Ok(r) if r.status().is_success() => {
                    KYK_429_STREAK.store(0, Ordering::Relaxed);
                    r
                }
                Ok(r) if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    // Kaynak sunucu hiz siniri uyguluyor: geri cekil, sakin ısrar etme.
                    // Israrci 429 serisi ban'a evrilir -> esik asilinca devreyi kes.
                    let streak = KYK_429_STREAK.fetch_add(1, Ordering::Relaxed) + 1;
                    if streak >= 5 {
                        trip_ban(&format!("art arda {} kez HTTP 429 ([TODAY] taraması)", streak)).await;
                        return Ok(total_saved);
                    }
                    let wait_secs = 30u64.saturating_mul(u64::from(streak)).min(300);
                    tracing::warn!("HTTP 429 (hiz siniri) - {} kesinti/streak, {}sn bekleniyor...", streak, wait_secs);
                    tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
                    continue;
                }
                Ok(r) if r.status() == reqwest::StatusCode::FORBIDDEN => {
                    // 403: büyük olasılıkla IP ban. Israr etmeden devreyi kes.
                    trip_ban(&format!("HTTP 403 ([TODAY] taraması, şehir: {})", city.slug)).await;
                    return Ok(total_saved);
                }
                Ok(r) if r.status() == reqwest::StatusCode::UNAUTHORIZED => {
                    if let Ok(new_token) = fetch_antiforgery_token(client).await {
                        token_opt = Some(new_token);
                    }
                    continue;
                }
                _ => continue,
            };

            let body_text = match res.text().await {
                Ok(b) => b,
                Err(_) => continue,
            };

            let html_content = if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
                json_val.get("html").and_then(|h| h.as_str()).unwrap_or(&body_text).to_string()
            } else {
                body_text
            };

            if html_content.contains("Menü bulunamadı") || !html_content.contains("cardStyle") {
                continue;
            }

            let parsed_menus = parse_kyk_html(&html_content, &city.slug, "dinner");
            for menu in parsed_menus {
                upsert_menu(
                    db,
                    city.id,
                    menu.date,
                    MealTypeEnum::Dinner,
                    "kykyemek".to_string(),
                    None,
                    menu.dishes,
                    vec![],
                    menu.takeaways,
                    None,
                    None,
                    None,
                ).await?;
                total_saved += 1;
            }

            // Kibar tarama: eski sabit 150ms'lik patlama (~500 istek/dk) IP ban
            // yedirdi. Artık istekler arası 1.5-3sn rastgele gecikme uygulanır.
            let delay_ms = {
                use rand::Rng;
                rand::thread_rng().gen_range(1500..=3000)
            };
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    }

    tracing::info!("[TODAY] Günün menüsü taraması tamamlandı: {} menü kaydedildi.", total_saved);
    Ok(total_saved)
}

async fn fetch_antiforgery_token(client: &Client) -> Result<String> {
    // Ana sayfa ziyareti = oturum ısıtma (warm-up). Gerçek bir tarayıcı
    // gezinmesi gibi tam başlık setiyle gider.
    let res = client.get("https://kykyemek.com/")
        .header("User-Agent", BROWSER_UA)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("sec-ch-ua", "\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Google Chrome\";v=\"126\"")
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

    static TOKEN_REGEX: OnceLock<regex::Regex> = OnceLock::new();
    let re = TOKEN_REGEX.get_or_init(|| {
        regex::Regex::new(r#"name=["']__RequestVerificationToken["'][^>]*value=["']([^"']+)["']"#).unwrap()
    });

    if let Some(caps) = re.captures(&html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    static TOKEN_FALLBACK: OnceLock<regex::Regex> = OnceLock::new();
    let re_fb = TOKEN_FALLBACK.get_or_init(|| {
        regex::Regex::new(r#"value=["']([^"']+)["'][^>]*name=["']__RequestVerificationToken["']"#).unwrap()
    });
    if let Some(caps) = re_fb.captures(&html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    anyhow::bail!("__RequestVerificationToken HTML içinde bulunamadı")
}

pub async fn run_kykyemek_scraper(
    db: &DatabaseConnection,
    client: &Client,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
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

    // Şehir sırasını her turda karıştır: A'dan Z'ye sabit tarama deseni
    // sunucu tarafı istatistikte çizgi gibi görünür.
    let mut all_cities = cities::Entity::find().all(db).await?;
    {
        use rand::seq::SliceRandom;
        all_cities.shuffle(&mut rand::thread_rng());
    }

    let mut token_opt = if is_banned() {
        tracing::warn!("[KYK-BREAKER] Cooldown aktif - ana tarama bu tur atlanıyor, token alınmayacak.");
        None
    } else {
        match fetch_antiforgery_token(client).await {
            Ok(t) => {
                tracing::info!("Kykyemek oturum token'ı başarıyla alındı.");
                Some(t)
            }
            Err(e) => {
                tracing::warn!("Kykyemek token alınamadı: {:?}. Düz istek deneniyor.", e);
                None
            }
        }
    };

    for city in all_cities {
        if *shutdown_rx.borrow() {
            tracing::info!("Kapatma sinyali algılandı. Tarayıcı durduruluyor.");
            return Ok(());
        }
        if is_banned() {
            tracing::warn!("[KYK-BREAKER] Tur içinde devre kesildi, kalan şehirler atlanıyor.");
            break;
        }
        tracing::info!("[WEB] Şehir taranıyor: {}...", city.name);

        for month_shift in ["-2", "-1", "0"] {
            if *shutdown_rx.borrow() || is_banned() {
                break;
            }

            // Fetch breakfast
            match fetch_and_save(db, client, &city, "breakfast", MealTypeEnum::Breakfast, month_shift, &mut token_opt, &mut shutdown_rx).await {
                Ok(Some(count)) => total_fetched += count,
                Ok(None) => return Ok(()), // Aborted via shutdown signal
                Err(e) => tracing::error!(city = %city.slug, meal = "breakfast", month = month_shift, "Menü çekme hatası: {:?}", e),
            }

            // Sleep between requests (random 4000 to 8000 ms)
            // 81 il x 3 ay x 2 ogun = ~486 istek; kaynak sunucuya kibar ol.
            // (IP ban olayı sonrası temkin: eski 2.5-5sn aralığı artırıldı.)
            let delay_ms = {
                use rand::Rng;
                rand::thread_rng().gen_range(4000..=8000)
            };
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(());
            }

            // Fetch dinner
            match fetch_and_save(db, client, &city, "dinner", MealTypeEnum::Dinner, month_shift, &mut token_opt, &mut shutdown_rx).await {
                Ok(Some(count)) => total_fetched += count,
                Ok(None) => return Ok(()), // Aborted via shutdown signal
                Err(e) => tracing::error!(city = %city.slug, meal = "dinner", month = month_shift, "Menü çekme hatası: {:?}", e),
            }

            // Sleep between requests (random 4000 to 8000 ms)
            let delay_ms = {
                use rand::Rng;
                rand::thread_rng().gen_range(4000..=8000)
            };
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(());
            }
        }
    }

    // Faz 1.6: bu turda yeni açılan gün URL'lerini IndexNow'a bildir.
    // INDEXNOW_KEY atanmamışsa no-op; hata durumunda yalnızca warn loglanır,
    // asla döngüyü düşürmez. Yalnızca kanonik /{sehir}/{tarih} URL'leri
    // gönderilir (/menu/{id} ASLA gönderilmez).
    if let Some(config) = super::indexnow::IndexNowConfig::from_env() {
        super::indexnow::ping_new_day_urls(db, client, &config).await;
    }

    if total_fetched == 0 {
        let alert_msg = "Kykyemek taraması tamamlandı ancak 81 il genelinde HİÇBİR menü çekilemedi (Global Ingestion Blackout)!";
        tracing::error!("{}", alert_msg);
        let _ = shared::services::alerting::AlertingService::send_webhook_alert(alert_msg).await;
    } else {
        tracing::info!("Kykyemek taraması tamamlandı. {} menü kontrol edildi.", total_fetched);
    }
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
                    if let Ok(new_token) = fetch_antiforgery_token(client).await {
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
    
    let parsed_menus = parse_kyk_html(&html_content, &city.slug, kyk_meal_type);
    let mut count = 0;
    
    for menu in parsed_menus {
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
            None,
            None,
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
    if dishes.is_empty() && celiac_dishes.is_empty() && takeaways.is_empty() {
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
            existing_map.insert((d.dish_alias_id, d.package_name.clone()), d);
        }
        
        let mut update_m: menus::ActiveModel = m.clone().into();
        update_m.source_type = Set(Some(source_type));
        update_m.submitted_by = Set(submitted_by);
        update_m.status = Set(target_status.clone());
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
    
    // Build target_map: Key: (dish_alias_id, package_name), Value: (order_index, is_alternative, amount, calories)
    let mut target_map = HashMap::new();
    
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
            target_map.insert((alias_id, package_name), (order_index, is_alternative, comp.amount, cals));
        }
    }

    for (i, dish_group) in celiac_dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "ÇÖLYAK MENÜSÜ".to_string();
            let cals = parse_dish_calories(&comp.calories);
            target_map.insert((alias_id, package_name), (order_index, is_alternative, comp.amount, cals));
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
                target_map.insert((alias_id, sanitized_package.clone()), (order_index, is_alternative, comp.amount, cals));
            }
        }
    }
    
    // Smart Sync
    for (key, target_vals) in target_map.into_iter() {
        let (order_index, is_alternative, amount, calories) = target_vals;
        if let Some(existing) = existing_map.remove(&key) {
            // Update if changed
            if existing.order_index != order_index || existing.is_alternative != is_alternative || existing.amount != amount || existing.calories != calories {
                let mut active: menu_dishes::ActiveModel = existing.into();
                active.order_index = Set(order_index);
                active.is_alternative = Set(is_alternative);
                active.amount = Set(amount);
                active.calories = Set(calories);
                active.update(&txn).await?;
            }
        } else {
            // Insert new
            let link = menu_dishes::ActiveModel {
                menu_id: Set(menu_id),
                dish_alias_id: Set(key.0),
                order_index: Set(order_index),
                is_alternative: Set(is_alternative),
                package_name: Set(key.1),
                amount: Set(amount),
                calories: Set(calories),
                ..Default::default()
            };
            link.insert(&txn).await?;
        }
    }
    
    // Delete missing
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
    // XSS sanitization
    let sanitized = sanitize_dish_name(raw_name);

    // Kategori belirtilmemişse akıllı kural motoruyla otomatik belirle
    let final_category = category.or_else(|| shared::services::categorizer::categorize_dish(&sanitized));

    // Atomik işlem (Race Condition önleyici):
    // 1. Ana dish (yemek) oluştur veya varsa IDsini döndür. Eğer mevcut kaydın kategorisi yoksa (NULL), tespit edilen kategoriyi yaz (COALESCE).
    // 2. Takma adı (alias) ana yemeğe bağlayarak oluştur veya güncelleyip idsini döndür.
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        r#"
        WITH upsert_dish AS (
            INSERT INTO dishes (name, category) VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name, category = COALESCE(dishes.category, EXCLUDED.category)
            RETURNING id
        )
        INSERT INTO dish_aliases (name, dish_id)
        VALUES ($1, (SELECT id FROM upsert_dish))
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name, dish_id = COALESCE(dish_aliases.dish_id, EXCLUDED.dish_id)
        RETURNING id
        "#,
        vec![sanitized.into(), final_category.into()],
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
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
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


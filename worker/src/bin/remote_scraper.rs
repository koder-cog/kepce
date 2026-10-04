use chrono::Datelike;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

use worker::parser::kykyemek::{extract_fastmenu_items, parse_kykyemek_html};
use worker::parser::takeaway::{
    get_cached_fastmenu, insert_cached_fastmenu, parse_fast_menu_foods_html,
};
use worker::tasks::scraper::{
    ActiveKykSession, KykYemekClientPool, fetch_kykyemek_session, throttle_kykyemek,
    with_xhr_headers,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DishItemDto {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    calories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TakeawayDto {
    package_name: String,
    dishes: Vec<Vec<DishItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MenuDto {
    city_slug: String,
    serve_date: String,
    meal_type: String,
    dishes: Vec<Vec<DishItemDto>>,
    #[serde(default)]
    celiac_dishes: Vec<Vec<DishItemDto>>,
    #[serde(default)]
    takeaways: Vec<TakeawayDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    calorie_range_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    calorie_range_max: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KykyemekIngestRequest {
    menus: Vec<MenuDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IngestResponseDto {
    total_received: usize,
    total_inserted: usize,
    total_updated: usize,
    total_skipped: usize,
    #[serde(default)]
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = env::args().collect();
    let mut dry_run = false;
    let mut target_city: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--city" => {
                if i + 1 < args.len() {
                    target_city = Some(args[i + 1].to_lowercase());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Kullanım: remote_scraper [SEÇENEKLER]");
                println!();
                println!("Seçenekler:");
                println!("  --dry-run       API'ye göndermeden sadece çeker ve parse eder");
                println!("  --city <slug>   Yalnızca belirtilen şehir için çalışır (örn: ankara)");
                println!("  --help, -h      Bu yardım mesajını gösterir");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    let ingest_url = env::var("INTERNAL_INGEST_URL")
        .unwrap_or_else(|_| "https://kepce.org/api/v1/internal/ingest/kykyemek".to_string());
    // GitHub Actions secret'ları kopyala-yapıştır sırasında sonunda görünmez bir
    // satır sonu (\n / \r) veya boşluk taşıyabilir. Bu değer doğrudan HTTP
    // header'ına yazıldığında reqwest `InvalidHeaderValue` ile patlar; bu yüzden
    // değeri burada temizleyip erken doğrulıyoruz.
    let ingest_secret = env::var("INTERNAL_INGEST_SECRET")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Cloudflare Access service token (opsiyonel). Tünel + Access kurulumunda
    // ingest isteği bot challenge'ını bu token ile aşar. Boşsa header eklenmez.
    let cf_access_client_id = env::var("CF_ACCESS_CLIENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let cf_access_client_secret = env::var("CF_ACCESS_CLIENT_SECRET")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if !dry_run {
        let secret = ingest_secret.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "INTERNAL_INGEST_SECRET ortam değişkeni tanımlanmamış. Canlı aktarım için gereklidir (veya --dry-run kullanın)."
            )
        })?;
        reqwest::header::HeaderValue::try_from(secret).map_err(|e| {
            anyhow::anyhow!(
                "INTERNAL_INGEST_SECRET geçersiz bir HTTP header değeri içeriyor (görünmez karakter / satır sonu olabilir): {e}"
            )
        })?;
        if let (Some(id), Some(cf_secret)) = (&cf_access_client_id, &cf_access_client_secret) {
            reqwest::header::HeaderValue::try_from(id.as_str()).map_err(|e| {
                anyhow::anyhow!("CF_ACCESS_CLIENT_ID geçersiz bir HTTP header değeri içeriyor: {e}")
            })?;
            reqwest::header::HeaderValue::try_from(cf_secret.as_str()).map_err(|e| {
                anyhow::anyhow!(
                    "CF_ACCESS_CLIENT_SECRET geçersiz bir HTTP header değeri içeriyor: {e}"
                )
            })?;
        }
    }

    tracing::info!("Remote Scraper başlatılıyor (dry_run: {})...", dry_run);

    let default_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let pool = KykYemekClientPool::from_env(default_client);
    let mut session_cache: HashMap<Option<usize>, ActiveKykSession> = HashMap::new();

    // 1. Keşif: İlk geçerli oturumu al ve illeri öğren
    let (mut session, discovered_cities) =
        obtain_ready_session_and_cities(&pool, &mut session_cache)
            .await
            .ok_or_else(|| anyhow::anyhow!("Kullanılabilir proxy oturumu bulunamadı"))?;

    let active_slugs = if let Some(ref slug) = target_city {
        vec![slug.clone()]
    } else {
        discovered_cities
    };

    tracing::info!(
        "İlk oturum hazır (çıkış: {}). Toplam {} il için tarama yapılacak.",
        session.endpoint_label,
        active_slugs.len()
    );

    tracing::info!(
        "Toplam {} il için tarama yapılacak: {:?}",
        active_slugs.len(),
        active_slugs
    );

    let now_date = chrono::Utc::now().date_naive();
    let shifts: Vec<&str> = if now_date.day() <= 2 {
        vec!["-1", "0"]
    } else {
        vec!["0"]
    };

    let ingest_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    let secret_val = ingest_secret.unwrap_or_default();

    let mut grand_total_received = 0;
    let mut grand_total_inserted = 0;
    let mut grand_total_updated = 0;
    let mut grand_total_skipped = 0;
    let mut all_errors = Vec::new();

    let mut consecutive_failures = 0;
    const MAX_CONSECUTIVE_FAILURES: usize = 3;

    for slug in &active_slugs {
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            tracing::error!(
                "Hedef sunucu ardışık {} istekte hata verdi. Kota israfını önlemek için tarama erken sonlandırılıyor.",
                consecutive_failures
            );
            break;
        }

        // Her şehir başında torbadan sıradaki oturumu çek (yükü 10 proxy'ye homojen dağıt)
        if let Some(new_s) = obtain_ready_session(&pool, &mut session_cache).await {
            session = new_s;
        }

        tracing::info!(
            "Şehir taranıyor: {} (çıkış: {})...",
            slug,
            session.endpoint_label
        );

        let mut city_menus: Vec<MenuDto> = Vec::new();

        for shift in &shifts {
            // 1. Kahvaltı
            let ok_b = scrape_and_collect(
                &pool,
                &mut session,
                &mut session_cache,
                slug,
                "breakfast",
                shift,
                &mut city_menus,
            )
            .await;
            if ok_b {
                consecutive_failures = 0;
            } else {
                consecutive_failures += 1;
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::error!(
                        "Hedef sunucu ardışık {} istekte hata verdi. Kota israfını önlemek için tarama erken sonlandırılıyor.",
                        consecutive_failures
                    );
                    break;
                }
            }

            let delay = rand::thread_rng().gen_range(1500..=3000);
            tokio::time::sleep(Duration::from_millis(delay)).await;

            // 2. Akşam Yemeği
            let ok_d = scrape_and_collect(
                &pool,
                &mut session,
                &mut session_cache,
                slug,
                "dinner",
                shift,
                &mut city_menus,
            )
            .await;
            if ok_d {
                consecutive_failures = 0;
            } else {
                consecutive_failures += 1;
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::error!(
                        "Hedef sunucu ardışık {} istekte hata verdi. Kota israfını önlemek için tarama erken sonlandırılıyor.",
                        consecutive_failures
                    );
                    break;
                }
            }

            let delay = rand::thread_rng().gen_range(1500..=3000);
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            break;
        }

        if city_menus.is_empty() {
            tracing::warn!("{} için menü bulunamadı.", slug);
            continue;
        }

        if dry_run {
            println!(
                "=== DRY RUN: {} === ({} gün/öğün menüsü)",
                slug,
                city_menus.len()
            );
            if let Some(first) = city_menus.first() {
                println!("{}", serde_json::to_string_pretty(first)?);
            }
            continue;
        }

        // Şehir bülteni bittiği anda Ingest API'sine aktar (Pipelined Ingest)
        let payload = KykyemekIngestRequest {
            menus: city_menus,
            source_type: Some("kykyemek".to_string()),
        };

        // Cloudflare, datacenter IP'lerinden gelen çıplak (tarayıcı başlığı olmayan)
        // istekleri "managed challenge" ile karşılayabiliyor. İstek zaten gizli
        // token ile korunuyor; yine de CDN'in bot sinyalini düşürmek için gerçekçi
        // tarayıcı başlıkları gönderiyoruz.
        let mut ingest_req = ingest_client
            .post(&ingest_url)
            .header("X-Internal-Token", &secret_val)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Accept-Language", "tr-TR,tr;q=0.9,en;q=0.8")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
            );
        // Cloudflare Access service token varsa ekle (tünel + Access kurulumu).
        if let (Some(id), Some(cf_secret)) = (&cf_access_client_id, &cf_access_client_secret) {
            ingest_req = ingest_req
                .header("CF-Access-Client-Id", id)
                .header("CF-Access-Client-Secret", cf_secret);
        }
        match ingest_req.json(&payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    if let Ok(dto) = serde_json::from_str::<IngestResponseDto>(&text) {
                        tracing::info!(
                            "[{}] Aktarıldı -> Alınan: {}, Eklenen: {}, Güncellenen: {}, Atlanan: {}",
                            slug,
                            dto.total_received,
                            dto.total_inserted,
                            dto.total_updated,
                            dto.total_skipped
                        );
                        grand_total_received += dto.total_received;
                        grand_total_inserted += dto.total_inserted;
                        grand_total_updated += dto.total_updated;
                        grand_total_skipped += dto.total_skipped;
                        all_errors.extend(dto.errors);
                    }
                } else {
                    let err_text = resp.text().await.unwrap_or_default();
                    tracing::error!(
                        "[{}] Ingest API HTTP {} hatası verdi: {}",
                        slug,
                        status,
                        err_text
                    );
                }
            }
            Err(e) => {
                tracing::error!("[{}] Ingest API bağlantı hatası: {:?}", slug, e);
            }
        }
    }

    if !dry_run {
        tracing::info!(
            "Tüm illerin aktarımı tamamlandı! Toplam Alınan: {}, Eklenen: {}, Güncellenen: {}, Atlanan: {}, Hatalar: {}",
            grand_total_received,
            grand_total_inserted,
            grand_total_updated,
            grand_total_skipped,
            all_errors.len()
        );
        if !all_errors.is_empty() {
            for err in &all_errors {
                tracing::warn!("API Uyarısı: {}", err);
            }
        }

        let total_processed = grand_total_inserted + grand_total_updated + grand_total_skipped;
        if !active_slugs.is_empty() && total_processed == 0 {
            anyhow::bail!(
                "Tarama başarısız: {} il hedeflendi ancak hiçbir menü aktarılamadı (Alınan: {}, Hata sayısı: {}).",
                active_slugs.len(),
                grand_total_received,
                all_errors.len()
            );
        }
    }

    Ok(())
}

async fn obtain_ready_session_and_cities(
    pool: &KykYemekClientPool,
    session_cache: &mut HashMap<Option<usize>, ActiveKykSession>,
) -> Option<(ActiveKykSession, Vec<String>)> {
    for _ in 0..15 {
        if let Some(s) = pool.acquire_session() {
            if let Some(cached) = session_cache.get(&s.proxy_idx)
                && cached.token.is_some()
            {
                return Some((cached.clone(), Vec::new()));
            }

            match fetch_kykyemek_session(&s.client).await {
                Ok((tok, cities)) => {
                    let mut ready = s.clone();
                    ready.token = Some(tok);
                    session_cache.insert(s.proxy_idx, ready.clone());
                    return Some((ready, cities));
                }
                Err(e) => {
                    tracing::warn!(
                        "Oturum hazırlama hatası ({:?}): {}. Proxy devre dışı bırakılıyor.",
                        e,
                        s.endpoint_label
                    );
                    pool.trip_session_ban(&s, "Oturum hazırlama hatası").await;
                }
            }
        } else {
            break;
        }
    }
    None
}

async fn obtain_ready_session(
    pool: &KykYemekClientPool,
    session_cache: &mut HashMap<Option<usize>, ActiveKykSession>,
) -> Option<ActiveKykSession> {
    obtain_ready_session_and_cities(pool, session_cache)
        .await
        .map(|(session, _)| session)
}

async fn scrape_and_collect(
    pool: &KykYemekClientPool,
    session: &mut ActiveKykSession,
    session_cache: &mut HashMap<Option<usize>, ActiveKykSession>,
    slug: &str,
    meal_type: &str,
    shift: &str,
    collected_menus: &mut Vec<MenuDto>,
) -> bool {
    let is_dinner = if meal_type == "dinner" {
        "true"
    } else {
        "false"
    };
    let url = format!("https://kykyemek.com/Menu/GetDailyMenu/{}", slug);

    let mut attempt = 0;
    let max_retries = 2;

    while attempt < max_retries {
        let mut req = with_xhr_headers(session.client.get(&url).query(&[
            ("city", slug),
            ("mealType", is_dinner),
            ("monthShift", shift),
            ("hidePast", "false"),
        ]))
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header("Referer", "https://kykyemek.com/")
        .timeout(Duration::from_secs(12));

        if let Some(ref tok) = session.token {
            req = req
                .header("RequestVerificationToken", tok.as_str())
                .header("__RequestVerificationToken", tok.as_str());
        }

        throttle_kykyemek().await;
        match req.send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    let body_text = match res.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::warn!(
                                "Gövde okuma hatası ({} - {}): {:?}",
                                slug,
                                meal_type,
                                e
                            );
                            return false;
                        }
                    };

                    let html_content = if let Ok(json_val) =
                        serde_json::from_str::<serde_json::Value>(&body_text)
                    {
                        json_val
                            .get("html")
                            .and_then(|h| h.as_str())
                            .unwrap_or(&body_text)
                            .to_string()
                    } else {
                        body_text
                    };

                    // Al Götür ögelerini tespit et ve çek
                    let fastmenu_items = extract_fastmenu_items(&html_content);
                    for (fast_id, fast_name) in fastmenu_items {
                        if get_cached_fastmenu(&fast_id).is_some() {
                            continue;
                        }

                        let fast_url = "https://kykyemek.com/Menu/GetFastMenuFoods";
                        let mut fast_req = with_xhr_headers(
                            session
                                .client
                                .get(fast_url)
                                .query(&[("id", fast_id.as_str())]),
                        )
                        .header("X-Requested-With", "XMLHttpRequest")
                        .header("Referer", "https://kykyemek.com/")
                        .header("Origin", "https://kykyemek.com")
                        .header("Accept", "text/html, */*; q=0.01")
                        .timeout(Duration::from_secs(10));

                        if let Some(ref tok) = session.token {
                            fast_req = fast_req
                                .header("RequestVerificationToken", tok.as_str())
                                .header("__RequestVerificationToken", tok.as_str());
                        }

                        throttle_kykyemek().await;
                        if let Ok(fast_res) = fast_req.send().await
                            && fast_res.status().is_success()
                            && let Ok(foods_html) = fast_res.text().await
                        {
                            let slots = parse_fast_menu_foods_html(&foods_html);
                            if !slots.is_empty() {
                                insert_cached_fastmenu(fast_id, Some(fast_name), slots);
                            }
                        }

                        let delay = rand::thread_rng().gen_range(1500..=3000);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }

                    // HTML'i parse et
                    let parsed = parse_kykyemek_html(&html_content, slug, meal_type);
                    for m in parsed {
                        let dishes: Vec<Vec<DishItemDto>> = m
                            .dishes
                            .into_iter()
                            .map(|grp| {
                                grp.into_iter()
                                    .map(|c| DishItemDto {
                                        name: c.name,
                                        calories: c.calories,
                                        amount: c.amount,
                                        category: c.category,
                                    })
                                    .collect()
                            })
                            .collect();

                        let takeaways: Vec<TakeawayDto> = m
                            .takeaways
                            .into_iter()
                            .map(|(pkg, grps)| TakeawayDto {
                                package_name: pkg,
                                dishes: grps
                                    .into_iter()
                                    .map(|grp| {
                                        grp.into_iter()
                                            .map(|c| DishItemDto {
                                                name: c.name,
                                                calories: c.calories,
                                                amount: c.amount,
                                                category: c.category,
                                            })
                                            .collect()
                                    })
                                    .collect(),
                            })
                            .collect();

                        collected_menus.push(MenuDto {
                            city_slug: slug.to_string(),
                            serve_date: m.date.format("%Y-%m-%d").to_string(),
                            meal_type: meal_type.to_string(),
                            dishes,
                            celiac_dishes: vec![],
                            takeaways,
                            calorie_range_min: m.min_calories,
                            calorie_range_max: m.max_calories,
                        });
                    }

                    return true;
                } else if status == reqwest::StatusCode::FORBIDDEN
                    || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                {
                    tracing::warn!(
                        "Sunucu veya engel hatası (HTTP {}): {} [{}]. Failover deneniyor...",
                        status,
                        slug,
                        session.endpoint_label
                    );
                    session_cache.remove(&session.proxy_idx);
                    pool.trip_session_ban(session, &format!("HTTP {}", status))
                        .await;
                    if let Some(new_s) = obtain_ready_session(pool, session_cache).await {
                        tracing::info!(
                            "Oturum devredildi: {} -> {}",
                            session.endpoint_label,
                            new_s.endpoint_label
                        );
                        *session = new_s;
                    } else {
                        tracing::error!("Havuzdaki tüm proxy oturumları tükendi.");
                        return false;
                    }
                } else if status.is_server_error() {
                    // 5xx hatası hedef sunucu çöküşüdür; proxy ban'i sayılmaz.
                    // Proxy değiştirilmez, böylece ek ana sayfa indirme maliyetine girilmez.
                    tracing::warn!(
                        "Hedef sunucu hatası (HTTP {}): {} [{}] (deneme {}/{})",
                        status,
                        slug,
                        session.endpoint_label,
                        attempt + 1,
                        max_retries
                    );
                } else {
                    tracing::warn!(
                        "HTTP {} ({} - {}), deneme {}/{}",
                        status,
                        slug,
                        meal_type,
                        attempt + 1,
                        max_retries
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Ağ/proxy bağlantı hatası ({:?}): {} [{}] (deneme {}/{})",
                    e,
                    slug,
                    session.endpoint_label,
                    attempt + 1,
                    max_retries
                );
            }
        }

        attempt += 1;
        if attempt < max_retries {
            let backoff = 1 << attempt;
            tokio::time::sleep(Duration::from_secs(backoff)).await;
        }
    }

    false
}

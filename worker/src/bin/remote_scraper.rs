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
    fetch_kykyemek_session, throttle_kykyemek, with_xhr_headers, ActiveKykSession,
    KykYemekClientPool,
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
    let ingest_secret = env::var("INTERNAL_INGEST_SECRET").ok();

    if !dry_run && ingest_secret.is_none() {
        anyhow::bail!(
            "INTERNAL_INGEST_SECRET ortam değişkeni tanımlanmamış. Canlı aktarım için gereklidir (veya --dry-run kullanın)."
        );
    }

    tracing::info!("Remote Scraper başlatılıyor (dry_run: {})...", dry_run);

    let default_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let pool = KykYemekClientPool::from_env(default_client);
    let mut session_cache: HashMap<Option<usize>, ActiveKykSession> = HashMap::new();

    // 1. Keşif: İlk geçerli oturumu al ve illeri öğren
    let mut session = obtain_ready_session(&pool, &mut session_cache)
        .await
        .ok_or_else(|| anyhow::anyhow!("Kullanılabilir proxy oturumu bulunamadı"))?;

    tracing::info!(
        "İlk oturum hazır (çıkış: {}). Doğrulama token'ı ve aktif iller alınıyor...",
        session.endpoint_label
    );

    let (_, discovered_cities) = fetch_kykyemek_session(&session.client).await?;

    let active_slugs = if let Some(ref slug) = target_city {
        vec![slug.clone()]
    } else {
        discovered_cities
    };

    tracing::info!(
        "Toplam {} il için tarama yapılacak: {:?}",
        active_slugs.len(),
        active_slugs
    );

    let now_date = chrono::Utc::now().date_naive();
    let shifts: Vec<&str> = if now_date.day() <= 10 {
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

    for slug in active_slugs {
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
            scrape_and_collect(
                &pool,
                &mut session,
                &mut session_cache,
                &slug,
                "breakfast",
                shift,
                &mut city_menus,
            )
            .await;

            let delay = rand::thread_rng().gen_range(1500..=3000);
            tokio::time::sleep(Duration::from_millis(delay)).await;

            // 2. Akşam Yemeği
            scrape_and_collect(
                &pool,
                &mut session,
                &mut session_cache,
                &slug,
                "dinner",
                shift,
                &mut city_menus,
            )
            .await;

            let delay = rand::thread_rng().gen_range(1500..=3000);
            tokio::time::sleep(Duration::from_millis(delay)).await;
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

        match ingest_client
            .post(&ingest_url)
            .header("X-Internal-Token", &secret_val)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
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
                    tracing::error!("[{}] Ingest API HTTP {} hatası verdi: {}", slug, status, err_text);
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
            for err in all_errors {
                tracing::warn!("API Uyarısı: {}", err);
            }
        }
    }

    Ok(())
}

async fn obtain_ready_session(
    pool: &KykYemekClientPool,
    session_cache: &mut HashMap<Option<usize>, ActiveKykSession>,
) -> Option<ActiveKykSession> {
    for _ in 0..15 {
        if let Some(s) = pool.acquire_session() {
            if let Some(cached) = session_cache.get(&s.proxy_idx) {
                if cached.token.is_some() {
                    return Some(cached.clone());
                }
            }

            match fetch_kykyemek_session(&s.client).await {
                Ok((tok, _)) => {
                    let mut ready = s.clone();
                    ready.token = Some(tok);
                    session_cache.insert(s.proxy_idx, ready.clone());
                    return Some(ready);
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

async fn scrape_and_collect(
    pool: &KykYemekClientPool,
    session: &mut ActiveKykSession,
    session_cache: &mut HashMap<Option<usize>, ActiveKykSession>,
    slug: &str,
    meal_type: &str,
    shift: &str,
    collected_menus: &mut Vec<MenuDto>,
) {
    let is_dinner = if meal_type == "dinner" {
        "true"
    } else {
        "false"
    };
    let url = format!("https://kykyemek.com/Menu/GetDailyMenu/{}", slug);

    let mut attempt = 0;
    let max_retries = 3;

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
        .timeout(Duration::from_secs(30));

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
                            return;
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

                    // Al Götür öğelerini tespit et ve çek
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
                        .timeout(Duration::from_secs(15));

                        if let Some(ref tok) = session.token {
                            fast_req = fast_req
                                .header("RequestVerificationToken", tok.as_str())
                                .header("__RequestVerificationToken", tok.as_str());
                        }

                        throttle_kykyemek().await;
                        if let Ok(fast_res) = fast_req.send().await {
                            if fast_res.status().is_success() {
                                if let Ok(foods_html) = fast_res.text().await {
                                    let slots = parse_fast_menu_foods_html(&foods_html);
                                    if !slots.is_empty() {
                                        insert_cached_fastmenu(fast_id, Some(fast_name), slots);
                                    }
                                }
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

                    return;
                } else if status == reqwest::StatusCode::FORBIDDEN
                    || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
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
                        return;
                    }
                } else {
                    tracing::warn!(
                        "HTTP {} ({} - {}), deneme {}",
                        status,
                        slug,
                        meal_type,
                        attempt + 1
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Ağ/proxy bağlantı hatası ({:?}): {} [{}]. Failover deneniyor...",
                    e,
                    slug,
                    session.endpoint_label
                );
                session_cache.remove(&session.proxy_idx);
                pool.trip_session_ban(session, &format!("Bağlantı hatası: {:?}", e))
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
                    return;
                }
            }
        }

        attempt += 1;
        let backoff = 1 << attempt;
        tokio::time::sleep(Duration::from_secs(backoff)).await;
    }
}

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::OnceLock;
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
    let active_cities = [
        "istanbul", "ankara", "izmir", "antalya", "canakkale", "erzurum", 
        "eskisehir", "gaziantep", "isparta", "kahramanmaras", "karabuk", 
        "kirklareli", "konya", "sakarya", "sivas", "trabzon"
    ];

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

        tracing::info!("[TODAY] Şehir için günün menüleri taranıyor: {} (1..={})", city.name, days_in_month);

        for day in 1..=days_in_month {
            if *shutdown_rx.borrow() {
                return Ok(total_saved);
            }

            let day_shift = day - current_day;
            let url = "https://kykyemek.com/Menu/GetDailyMenu";

            let mut req = client.get(url)
                .query(&[
                    ("city", city.slug.as_str()),
                    ("mealType", "false"),
                    ("isToday", "true"),
                    ("dayShift", &day_shift.to_string()),
                ])
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Accept", "application/json, text/javascript, */*; q=0.01")
                .header("Accept-Language", "tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7")
                .header("Referer", "https://kykyemek.com/Menu/TodayMenu")
                .timeout(std::time::Duration::from_secs(30));

            if let Some(ref token) = token_opt {
                req = req
                    .header("RequestVerificationToken", token.as_str())
                    .header("__RequestVerificationToken", token.as_str());
            }

            let res = match req.send().await {
                Ok(r) if r.status().is_success() => r,
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

            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
    }

    tracing::info!("[TODAY] Günün menüsü taraması tamamlandı: {} menü kaydedildi.", total_saved);
    Ok(total_saved)
}

async fn fetch_antiforgery_token(client: &Client) -> Result<String> {
    let res = client.get("https://kykyemek.com/")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .send()
        .await?;
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
    let all_cities = cities::Entity::find().all(db).await?;
    let mut total_fetched = 0;

    // 1. Önce günün menülerini (aktif 16 ilin tüm Ağustos günlerini) hızlıca çek ve kaydet
    if let Ok(today_count) = scrape_today_menus(db, client, shutdown_rx.clone()).await {
        tracing::info!("Canlı günün menülerinden {} kayıt işlendi.", today_count);
        total_fetched += today_count;
    }

    let mut token_opt = match fetch_antiforgery_token(client).await {
        Ok(t) => {
            tracing::info!("Kykyemek oturum token'ı başarıyla alındı.");
            Some(t)
        }
        Err(e) => {
            tracing::warn!("Kykyemek token alınamadı: {:?}. Düz istek deneniyor.", e);
            None
        }
    };

    for city in all_cities {
        if *shutdown_rx.borrow() {
            tracing::info!("Kapatma sinyali algılandı. Tarayıcı durduruluyor.");
            return Ok(());
        }
        tracing::info!("[WEB] Şehir taranıyor: {}...", city.name);

        for month_shift in ["-2", "-1", "0"] {
            if *shutdown_rx.borrow() {
                return Ok(());
            }

            // Fetch breakfast
            match fetch_and_save(db, client, &city, "breakfast", MealTypeEnum::Breakfast, month_shift, &mut token_opt, &mut shutdown_rx).await {
                Ok(Some(count)) => total_fetched += count,
                Ok(None) => return Ok(()), // Aborted via shutdown signal
                Err(e) => tracing::error!(city = %city.slug, meal = "breakfast", month = month_shift, "Menü çekme hatası: {:?}", e),
            }

            // Sleep between requests (random 1500 to 3000 ms)
            let delay_ms = {
                use rand::Rng;
                rand::thread_rng().gen_range(1500..=3000)
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

            // Sleep between requests (random 1500 to 3000 ms)
            let delay_ms = {
                use rand::Rng;
                rand::thread_rng().gen_range(1500..=3000)
            };
            if sleep_cancelable(delay_ms, &mut shutdown_rx).await {
                return Ok(());
            }
        }
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

        let mut req = client.get(&url)
            .query(&[
                ("city", city.slug.as_str()),
                ("mealType", is_dinner),
                ("monthShift", month_shift),
                ("hidePast", "false"),
            ])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Language", "tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7")
            .header("Referer", "https://kykyemek.com/")
            .timeout(std::time::Duration::from_secs(30));

        if let Some(ref token) = *token_opt {
            req = req
                .header("RequestVerificationToken", token.as_str())
                .header("__RequestVerificationToken", token.as_str());
        }

        match req.send().await {
            Ok(res) => {
                if res.status().is_success() {
                    response = Some(res);
                    break;
                } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!("HTTP 401 (Yetkisiz), yeni oturum token'ı alınıyor...");
                    if let Ok(new_token) = fetch_antiforgery_token(client).await {
                        *token_opt = Some(new_token);
                    }
                } else {
                    tracing::warn!("HTTP durum kodu hatası: {}, Deneme: {}", res.status(), attempt + 1);
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
    let target_status = match target_status_override {
        Some(status) => status,
        None => match source_type.as_str() {
            "kepce-admin" | "kepce-kullanici" | "kykyemek" | "kykyemek.com" | "yurtmenu" | "yurtmenu.net" | "kykmenu" | "kykmenu.com.tr" => MenuStatusEnum::Approved,
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
        res.id
    };
    
    // Build target_map: Key: (dish_alias_id, package_name), Value: (order_index, is_alternative, amount, calories)
    let mut target_map = HashMap::new();
    
    for (i, dish_group) in dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "NORMAL".to_string();
            let cals = comp.calories.and_then(|c| c.parse::<i32>().ok());
            target_map.insert((alias_id, package_name), (order_index, is_alternative, comp.amount, cals));
        }
    }

    for (i, dish_group) in celiac_dishes.into_iter().enumerate() {
        let order_index = i as i32;
        for (j, comp) in dish_group.into_iter().enumerate() {
            let is_alternative = j > 0;
            let alias_id = get_or_create_dish_alias(&txn, &comp.name, comp.category.clone()).await?;
            let package_name = "ÇÖLYAK MENÜSÜ".to_string();
            let cals = comp.calories.and_then(|c| c.parse::<i32>().ok());
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
                let cals = comp.calories.and_then(|c| c.parse::<i32>().ok());
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

async fn get_or_create_dish_alias(txn: &sea_orm::DatabaseTransaction, raw_name: &str, category: Option<String>) -> Result<i32> {
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


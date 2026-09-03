// Kepçe Worker - Task: Zamanlanmış Öğün Bildirim Görevcisi (Meal Notifier)
// ======================================================================

use anyhow::Result;
use base64::Engine;
use chrono::{Local, NaiveDate};
use rand::Rng;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, ModelTrait,
};
use serde::{Deserialize, Serialize};
use shared::entities::{
    menus, menu_dishes, dish_aliases, push_subscriptions,
    sea_orm_active_enums::MealTypeEnum, prelude::*,
};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;
use web_push_native::{
    jwt_simple::algorithms::ES256KeyPair, p256::PublicKey, Auth, WebPushBuilder,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

fn get_vapid_subject() -> String {
    env::var("VAPID_SUBJECT").unwrap_or_else(|_| "mailto:iletisim@kepce.org".to_string())
}

fn get_key_pair() -> &'static ES256KeyPair {
    static KEYPAIR: OnceLock<ES256KeyPair> = OnceLock::new();
    KEYPAIR.get_or_init(|| {
        if let Ok(pem) = env::var("VAPID_PRIVATE_KEY") {
            let normalized = pem.replace("\\n", "\n");
            if let Ok(kp) = ES256KeyPair::from_pem(&normalized) {
                return kp;
            }
        }
        ES256KeyPair::generate()
    })
}

pub async fn send_to_subscription(
    db: &DatabaseConnection,
    sub: &push_subscriptions::Model,
    payload: &PushPayload,
) -> Result<bool> {
    let endpoint_uri = http::Uri::from_str(&sub.endpoint)?;

    let p256dh_raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&sub.p256dh)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&sub.p256dh))?;

    let auth_raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&sub.auth)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&sub.auth))?;

    let ua_public = PublicKey::from_sec1_bytes(&p256dh_raw)
        .map_err(|e| anyhow::anyhow!("Geçersiz p256dh public key: {:?}", e))?;

    let ua_auth = Auth::clone_from_slice(&auth_raw);
    let key_pair = get_key_pair();
    let subject = get_vapid_subject();

    let builder = WebPushBuilder::new(endpoint_uri, ua_public, ua_auth)
        .with_vapid(key_pair, &subject);

    let payload_bytes = serde_json::to_vec(payload)?;
    let request: http::Request<Vec<u8>> = builder.build(payload_bytes)?;

    let (parts, body) = request.into_parts();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let mut req_builder = client.request(parts.method, parts.uri.to_string());
    for (k, v) in parts.headers.iter() {
        req_builder = req_builder.header(k, v);
    }
    let req = req_builder.body(body).build()?;

    match client.execute(req).await {
        Ok(res) if res.status().is_success() => {
            tracing::debug!("Push bildirimi iletildi: {}", sub.endpoint);
            Ok(true)
        }
        Ok(res) if res.status().as_u16() == 410 || res.status().as_u16() == 404 => {
            tracing::info!("Abonelik geçersiz (HTTP {}), siliniyor: {}", res.status(), sub.endpoint);
            let _ = sub.clone().delete(db).await;
            Ok(false)
        }
        Ok(res) => {
            tracing::warn!("Push gateway beklenmeyen durum döndürdü (HTTP {}): {}", res.status(), sub.endpoint);
            Ok(false)
        }
        Err(e) => {
            tracing::warn!("Push iletim hatası ({}): {:?}", sub.endpoint, e);
            Ok(false)
        }
    }
}

pub fn build_meal_notification(
    city_name: &str,
    meal_type: &str,
    dishes: &[String],
) -> PushPayload {
    let mut rng = rand::thread_rng();
    let dishes_str = dishes.join(", ");
    let dishes_lower: Vec<String> = dishes.iter().map(|d| d.to_lowercase()).collect();

    let icon = Some("/icons/icon-192.png".to_string());
    let badge = Some("/icons/badge-72.png".to_string());
    let today_str = Local::now().format("%Y-%m-%d").to_string();
    let tag = Some(format!("meal-{}-{}", meal_type, today_str));
    let url = Some(format!("/{}", city_name.to_lowercase()
        .replace('ı', "i")
        .replace('ğ', "g")
        .replace('ü', "u")
        .replace('ş', "s")
        .replace('ö', "o")
        .replace('ç', "c")));

    if meal_type == "breakfast" {
        let has_sicak = dishes_lower.iter().any(|d| d.contains("pişi") || d.contains("börek") || d.contains("kızartma") || d.contains("menemen") || d.contains("omlet") || d.contains("pizza"));
        let has_yumurta = dishes_lower.iter().any(|d| d.contains("haşlanmış yumurta") || d.contains("haslanmis yumurta") || d.contains("yumurta"));
        let has_hamuris = dishes_lower.iter().any(|d| d.contains("simit") || d.contains("poğaça") || d.contains("pogaca") || d.contains("açma"));

        let main_dish = dishes.first().map(|s| s.as_str()).unwrap_or("Kahvaltı");

        if has_sicak {
            let variants = [
                (format!("{} - Bugünün kahvaltısı", city_name), format!("Kahvaltıda {} var. Soğumadan koşun gelin.", main_dish)),
                (format!("{} - Sıcak kahvaltı menüsü", city_name), format!("Aşağıda {} kokusu var. Olay yerinde olmak için menüye bak.", main_dish)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else if has_yumurta {
            let variants = [
                (format!("{} - Bugünün kahvaltısı", city_name), "Yine haşlanmış yumurta nöbeti. Ekmek arası yapmaya aşağı in.".to_string()),
                (format!("{} - Sabah kahvaltı listesi", city_name), "Yumurta ve peynir ikilisi hazır. Aşağı inmeden önce listeye göz at.".to_string()),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else if has_hamuris {
            let variants = [
                (format!("{} - Bugünün kahvaltısı", city_name), format!("Kahvaltıda {} var. Çayın yanına kapmak için menüye göz at.", main_dish)),
                (format!("{} - Hamurişi kahvaltı günü", city_name), format!("Tepside {} bekliyor. Menü detaylarına dokun.", main_dish)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else {
            let variants = [
                (format!("{} - Bugünün kahvaltısı", city_name), "Yataktan çıkmaya değer mi? Menüye bakıp öyle karar ver.".to_string()),
                (format!("{} - Sabah yoklaması", city_name), "Yemekhane sırası başlamadan önce bugünün kahvaltısına göz at.".to_string()),
                (format!("{} - Günün ilk öğünü", city_name), format!("Kahvaltı tabldotu: {}. Detaylar için dokun.", dishes_str)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        }
    } else {
        let has_tavuk_et = dishes_lower.iter().any(|d| d.contains("tavuk") || d.contains("köfte") || d.contains("kofte") || d.contains("fajita") || d.contains("şinitzel") || d.contains("kavurma") || d.contains("tas kebabı"));
        let has_sebze = dishes_lower.iter().any(|d| d.contains("türlü") || d.contains("turlu") || d.contains("bezelye") || d.contains("ıspanak") || d.contains("ispanak") || d.contains("kabak") || d.contains("karnabahar") || d.contains("pırasa") || d.contains("pirasa"));
        let has_bakliyat = dishes_lower.iter().any(|d| d.contains("nohut") || d.contains("kuru fasulye") || d.contains("barbunya") || d.contains("mercimek"));

        let main_dish = dishes.iter()
            .find(|d| {
                let l = d.to_lowercase();
                !l.contains("çorba") && !l.contains("corba") && !l.contains("su") && !l.contains("ekmek") && !l.contains("salata") && !l.contains("cacık") && !l.contains("ayran")
            })
            .map(|s| s.as_str())
            .unwrap_or_else(|| dishes.first().map(|s| s.as_str()).unwrap_or("Akşam Menüsü"));

        if has_tavuk_et {
            let variants = [
                (format!("{} - Bugünün akşam yemeği", city_name), format!("Ana yemekte {} var, tam bir protein bombası. Olay yerinde ol.", main_dish)),
                (format!("{} - Akşam menüsü açıklandı", city_name), format!("Tabldotta {} var. Sıra blokların arasına taşmadan yerini al.", main_dish)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else if has_sebze {
            let variants = [
                (format!("{} - Bugünün akşam yemeği", city_name), format!("Ana yemekte {} var. Uzak durup dışarıdan mı söylesek?", main_dish)),
                (format!("{} - Akşam menüsü yayında", city_name), format!("Bugün sebze günü: {} çıkmış. Beklentini ayarlamak için listeye bak.", main_dish)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else if has_bakliyat {
            let variants = [
                (format!("{} - Bugünün akşam yemeği", city_name), format!("Klasik menü devrede: {} ve pilav. Akşam menüsüne göz at.", main_dish)),
                (format!("{} - Günün akşam tabldotu", city_name), format!("Pilavın yanına {} eşlik ediyor. Menü detaylarına dokun.", main_dish)),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        } else {
            let variants = [
                (format!("{} - Bugünün akşam yemeği", city_name), "Dünkü yemeğin bugünkü evrimi ne oldu? Yemeğe inmeden önce menüye dokun.".to_string()),
                (format!("{} - Akşam yemekhanesi hazır", city_name), format!("Akşam tabldotu: {}. Sıraya girmeden menüyü gör.", dishes_str)),
                (format!("{} - Mutfaktan son durum", city_name), "Akşam tabldotunda ne olduğunu öğrenmek için dokun.".to_string()),
            ];
            let chosen = &variants[rng.gen_range(0..variants.len())];
            PushPayload { title: chosen.0.clone(), body: chosen.1.clone(), icon, badge, tag, url }
        }
    }
}

/// Her dakika çalışarak o dakikaya denk gelen aktif öğün bildirimlerini tetikler.
pub async fn check_and_dispatch_meal_notifications(db: &DatabaseConnection) -> Result<usize> {
    let now = Local::now();
    let current_time_str = now.format("%H:%M").to_string();
    let today_date = now.date_naive();

    tracing::debug!("[MEAL-NOTIFIER] Saat kontrolü yapılıyor: {}", current_time_str);

    // 1. Kahvaltı zamanı gelen abonelikler
    let breakfast_subs = PushSubscriptions::find()
        .filter(push_subscriptions::Column::NotifBreakfastEnabled.eq(true))
        .filter(push_subscriptions::Column::NotifBreakfastTime.eq(&current_time_str))
        .all(db)
        .await?;

    // 2. Akşam yemeği zamanı gelen abonelikler
    let dinner_subs = PushSubscriptions::find()
        .filter(push_subscriptions::Column::NotifDinnerEnabled.eq(true))
        .filter(push_subscriptions::Column::NotifDinnerTime.eq(&current_time_str))
        .all(db)
        .await?;

    if breakfast_subs.is_empty() && dinner_subs.is_empty() {
        return Ok(0);
    }

    let mut total_sent = 0;

    // Şehirler önbelleği
    let all_cities = Cities::find().all(db).await?;
    let city_map: HashMap<i32, String> = all_cities.into_iter().map(|c| (c.id, c.name)).collect();

    // Kahvaltı Gönderimi
    for sub in breakfast_subs {
        let city_id = match sub.city_id {
            Some(id) => id,
            None => continue,
        };
        let city_name = city_map.get(&city_id).cloned().unwrap_or_else(|| "KYK Yurdu".to_string());

        let dishes = get_menu_dishes_for_date(db, city_id, today_date, MealTypeEnum::Breakfast).await?;
        if dishes.is_empty() {
            // Boş menü koruması: Menü yoksa sessiz kal
            continue;
        }

        let payload = build_meal_notification(&city_name, "breakfast", &dishes);
        if send_to_subscription(db, &sub, &payload).await.unwrap_or(false) {
            total_sent += 1;
        }
    }

    // Akşam Yemeği Gönderimi
    for sub in dinner_subs {
        let city_id = match sub.city_id {
            Some(id) => id,
            None => continue,
        };
        let city_name = city_map.get(&city_id).cloned().unwrap_or_else(|| "KYK Yurdu".to_string());

        let dishes = get_menu_dishes_for_date(db, city_id, today_date, MealTypeEnum::Dinner).await?;
        if dishes.is_empty() {
            // Boş menü koruması: Menü yoksa sessiz kal
            continue;
        }

        let payload = build_meal_notification(&city_name, "dinner", &dishes);
        if send_to_subscription(db, &sub, &payload).await.unwrap_or(false) {
            total_sent += 1;
        }
    }

    if total_sent > 0 {
        tracing::info!("[MEAL-NOTIFIER] Saat {} için {} adet öğün push bildirimi başarıyla iletildi.", current_time_str, total_sent);
    }

    Ok(total_sent)
}

async fn get_menu_dishes_for_date(
    db: &DatabaseConnection,
    city_id: i32,
    date: NaiveDate,
    meal_type: MealTypeEnum,
) -> Result<Vec<String>> {
    let menu = Menus::find()
        .filter(menus::Column::CityId.eq(city_id))
        .filter(menus::Column::ServeDate.eq(date))
        .filter(menus::Column::MealType.eq(meal_type))
        .one(db)
        .await?;

    let menu = match menu {
        Some(m) => m,
        None => return Ok(vec![]),
    };

    let dish_rows = menu_dishes::Entity::find()
        .filter(menu_dishes::Column::MenuId.eq(menu.id))
        .find_also_related(dish_aliases::Entity)
        .all(db)
        .await?;

    let mut result = Vec::new();
    for (_md, alias_opt) in dish_rows {
        if let Some(alias) = alias_opt {
            let d_name = alias.name;
            let lower = d_name.to_lowercase();
            if !lower.contains("500 ml su") && !lower.contains("su") && !lower.contains("ekmek") {
                result.push(d_name);
            }
        }
    }

    Ok(result)
}

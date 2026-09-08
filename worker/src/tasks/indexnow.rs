//! IndexNow arama motoru dizinleme otomasyonu.
//!
//! Scraper döngüsü sonunda yeni eklenen menülerin ait olduğu şehirler için kanonik
//! `/{sehir}` hub URL'leri tek POST ile IndexNow'a bildirilir.
//!
//! `/{sehir}` sayfaları kanoniktir; `/{sehir}/{tarih}` istekleri 301 ile query paramına
//! (`/{sehir}?gun={tarih}`) yönlendiği, `/menu/{id}` ise ikincil kalıp canonical sinyalini
//! şehir sayfasına devrettiği için IndexNow'a yalnızca doğrudan `/{sehir}` bildirilir.
//!
//!
//! Env:
//! - `INDEXNOW_KEY` (zorunlu; boş/atanmamışsa özellik kapalı - geriye uyumlu)
//! - `INDEXNOW_HOST` (varsayılan `kepce.org`)
//! - `INDEXNOW_ENDPOINT` (varsayılan `https://api.indexnow.org/indexnow`)

use anyhow::Result;
use reqwest::Client;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashSet;

use super::scraper::take_inserted_menus;
use shared::entities::cities;

/// IndexNow API tek istekte en fazla 10.000 URL kabul eder; liste bu
/// sınıra göre bölünerek gönderilir.
const MAX_URLS_PER_REQUEST: usize = 10_000;

#[derive(Debug, Clone)]
pub struct IndexNowConfig {
    pub key: String,
    pub host: String,
    pub endpoint: String,
}

impl IndexNowConfig {
    /// `INDEXNOW_KEY` atanmamış/boşsa `None` döner → özellik kapalı başlar.
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("INDEXNOW_KEY").ok()?;
        let key = key.trim().to_string();
        if key.is_empty() {
            return None;
        }
        let host = std::env::var("INDEXNOW_HOST")
            .ok()
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| "kepce.org".to_string());
        let endpoint = std::env::var("INDEXNOW_ENDPOINT")
            .ok()
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "https://api.indexnow.org/indexnow".to_string());
        Some(Self { key, host, endpoint })
    }
}

#[derive(Serialize)]
struct IndexNowPayload<'a> {
    host: &'a str,
    key: &'a str,
    #[serde(rename = "keyLocation")]
    key_location: String,
    #[serde(rename = "urlList")]
    url_list: Vec<String>,
}

/// Döngü sonunda çağrılır: yeni eklenen menülerin şehirlerini kanonik URL'lere
/// (`/{sehir}`) çevirip IndexNow'a bildirir. Hata durumunda yalnızca `tracing::warn!`
/// basılır, scraper döngüsü kesintiye uğramaz. Kayıt listesi boşsa işlem yapmaz.
pub async fn ping_new_day_urls(db: &DatabaseConnection, client: &Client, config: &IndexNowConfig) {
    if let Err(e) = ping_inner(db, client, config).await {
        tracing::warn!("IndexNow ping başarısız (döngü etkilenmez): {:?}", e);
    }
}

async fn ping_inner(db: &DatabaseConnection, client: &Client, config: &IndexNowConfig) -> Result<()> {
    let inserted = take_inserted_menus();
    if inserted.is_empty() {
        return Ok(());
    }

    // Eklenen menülerin tekil şehir ID'leri toplanır; IndexNow'a doğrudan kanonik /{sehir} bildirilir.
    let city_ids: HashSet<i32> = inserted.into_iter().map(|(id, _)| id).collect();

    let city_rows = cities::Entity::find()
        .filter(cities::Column::Id.is_in(city_ids))
        .all(db)
        .await?;

    let mut urls: Vec<String> = city_rows
        .into_iter()
        .map(|c| format!("https://{}/{}", config.host, c.slug))
        .collect();
    urls.sort();

    if urls.is_empty() {
        return Ok(());
    }

    let key_location = format!("https://{}/{}.txt", config.host, config.key);

    for chunk in urls.chunks(MAX_URLS_PER_REQUEST) {
        let payload = IndexNowPayload {
            host: &config.host,
            key: &config.key,
            key_location: key_location.clone(),
            url_list: chunk.to_vec(),
        };
        let res = client.post(&config.endpoint).json(&payload).send().await?;
        let status = res.status();
        if status.is_success() {
            tracing::info!("IndexNow ping: {} ({} URL)", status.as_u16(), chunk.len());
        } else {
            let body = res.text().await.unwrap_or_default();
            tracing::warn!("IndexNow ping HTTP {}: {}", status.as_u16(), body);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Not: env değişkeni process-global olduğu için iki senaryo bilinçli
    /// olarak TEK testte sıralı çalıştırılır (paralel testlerde yarış olmasın).
    #[test]
    fn test_from_env_lifecycle() {
        // 1) INDEXNOW_KEY atanmamış → özellik kapalı (geriye uyumlu)
        unsafe { std::env::remove_var("INDEXNOW_KEY") };
        assert!(IndexNowConfig::from_env().is_none());

        // 2) KEY set edilince varsayılan host/endpoint ile açılır
        unsafe { std::env::set_var("INDEXNOW_KEY", "test-key") };
        unsafe { std::env::remove_var("INDEXNOW_HOST") };
        unsafe { std::env::remove_var("INDEXNOW_ENDPOINT") };
        let cfg = IndexNowConfig::from_env().expect("config beklenir");
        assert_eq!(cfg.host, "kepce.org");
        assert_eq!(cfg.endpoint, "https://api.indexnow.org/indexnow");

        // 3) Boş KEY de kapalı sayılır
        unsafe { std::env::set_var("INDEXNOW_KEY", "   ") };
        assert!(IndexNowConfig::from_env().is_none());

        unsafe { std::env::remove_var("INDEXNOW_KEY") };
    }
}

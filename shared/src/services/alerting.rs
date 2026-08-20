// Kepçe Shared - Service: Alerting (Webhook Uyarı Servisi)
// ========================================================
//
// Scraper ve sistem hatalarında Discord veya Telegram webhook'larına
// uyarı bildirimi gönderir.
//
// Niyetli olarak `ALERT_WEBHOOK_URL` veya `DISCORD_WEBHOOK_URL` ortam
// değişkenini okur. Eğer webhook tanımlı değilse uyarısı verip loglar.

use reqwest::Client;
use serde_json::json;

pub struct AlertingService;

impl AlertingService {
    /// Webhook uyarısı gönderir (Discord/Telegram uyumlu JSON payload)
    pub async fn send_webhook_alert(message: &str) -> anyhow::Result<()> {
        let webhook_url = match std::env::var("ALERT_WEBHOOK_URL").or_else(|_| std::env::var("DISCORD_WEBHOOK_URL")) {
            Ok(url) if !url.trim().is_empty() => url,
            _ => {
                tracing::warn!("Webhook uyarısı gönderilemedi: ALERT_WEBHOOK_URL veya DISCORD_WEBHOOK_URL ayarlanmamış.");
                return Ok(());
            }
        };

        let client = Client::new();
        let payload = json!({
            "content": format!("[ALARM] **[KEPÇE ALARM]** {}", message)
        });

        tracing::info!("Webhook uyarısı gönderiliyor: {}", message);

        let res = client.post(&webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            tracing::info!("Webhook uyarısı başarıyla iletildi.");
        } else {
            tracing::error!("Webhook isteği başarısız oldu. HTTP durum kodu: {}", res.status());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_webhook_alert_missing_env() {
        // Environtment değişkeni olmasa dahi paniklemeden Ok(()) dönmeli
        std::env::remove_var("ALERT_WEBHOOK_URL");
        std::env::remove_var("DISCORD_WEBHOOK_URL");
        let res = AlertingService::send_webhook_alert("Test uyarısı").await;
        assert!(res.is_ok());
    }
}

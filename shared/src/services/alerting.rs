// Kepçe Shared - Service: Alerting (Webhook & Telegram Uyarı Servisi)
// ===================================================================
//
// Scraper ve sistem hatalarında Discord webhook'larına veya Telegram botuna
// uyarı bildirimi gönderir.
//
// Desteklenen Ortam Değişkenleri:
// - `TELEGRAM_BOT_TOKEN`: BotFather'dan alınan bot token'ı
// - `TELEGRAM_ADMIN_CHAT_ID` veya `TELEGRAM_CHAT_ID`: Bildirimin düşeceği chat/kullanıcı ID'si
// - `ALERT_WEBHOOK_URL` veya `DISCORD_WEBHOOK_URL`: Discord/Slack uyumlu webhook URL'si

use reqwest::Client;
use serde_json::json;

pub struct AlertingService;

impl AlertingService {
    /// Birleşik alarm gönderir: Hem Telegram hem Discord yapılandırılmışsa ikisine de iletir.
    pub async fn send_alert(message: &str) -> anyhow::Result<()> {
        let _ = Self::send_webhook_alert(message).await;
        let _ = Self::send_telegram_alert(message).await;
        Ok(())
    }

    /// Telegram Bot API üzerinden doğrudan Markdown formatında alarm mesajı gönderir.
    pub async fn send_telegram_alert(message: &str) -> anyhow::Result<()> {
        let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(token) if !token.trim().is_empty() => token,
            _ => return Ok(()),
        };

        let chat_id = match std::env::var("TELEGRAM_ADMIN_CHAT_ID").or_else(|_| std::env::var("TELEGRAM_CHAT_ID")) {
            Ok(id) if !id.trim().is_empty() => id,
            _ => {
                tracing::warn!("Telegram alarmı gönderilemedi: TELEGRAM_ADMIN_CHAT_ID ayarlanmamış.");
                return Ok(());
            }
        };

        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token.trim());
        let client = Client::new();
        let payload = json!({
            "chat_id": chat_id.trim(),
            "text": format!("🚨 *[KEPÇE ALARM]*\n\n{}", message),
            "parse_mode": "Markdown"
        });

        tracing::info!("Telegram alarmı gönderiliyor: {}", message);

        let res = client.post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            tracing::info!("Telegram alarmı başarıyla iletildi.");
        } else {
            let err_body = res.text().await.unwrap_or_default();
            tracing::error!("Telegram alarm isteği başarısız oldu: {}", err_body);
        }

        Ok(())
    }

    /// Webhook uyarısı gönderir (Discord uyumlu JSON payload).
    /// Geriye dönük uyumluluk için, Telegram yapılandırılmışsa Telegram'a da iletir.
    pub async fn send_webhook_alert(message: &str) -> anyhow::Result<()> {
        // Eğer Telegram ayarlıysa Telegram'a da kopyala
        if std::env::var("TELEGRAM_BOT_TOKEN").is_ok() {
            let _ = Self::send_telegram_alert(message).await;
        }

        let webhook_url = match std::env::var("ALERT_WEBHOOK_URL").or_else(|_| std::env::var("DISCORD_WEBHOOK_URL")) {
            Ok(url) if !url.trim().is_empty() => url,
            _ => {
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
    async fn test_send_alert_missing_env() {
        std::env::remove_var("ALERT_WEBHOOK_URL");
        std::env::remove_var("DISCORD_WEBHOOK_URL");
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        std::env::remove_var("TELEGRAM_ADMIN_CHAT_ID");
        let res = AlertingService::send_alert("Test uyarısı").await;
        assert!(res.is_ok());
    }
}

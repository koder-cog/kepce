//! Webhook ve Telegram alarm bildirim servisi.
//!
//! Scraper ve sistem seviyesindeki kritik hataları Discord/Slack uyumlu webhook'lara
//! veya Telegram bot yöneticisine iletir.
//!
//! Desteklenen ortam değişkenleri:
//! - `TELEGRAM_BOT_TOKEN`: BotFather bot token değeri
//! - `TELEGRAM_ADMIN_CHAT_ID` / `TELEGRAM_CHAT_ID`: Bildirimin iletileceği hedef sohbet ID'si
//! - `ALERT_WEBHOOK_URL` / `DISCORD_WEBHOOK_URL`: Discord veya Slack uyumlu webhook adresi

use reqwest::Client;
use serde_json::json;

pub struct AlertingService;

impl AlertingService {
    /// En az bir operatör uyarı kanalı (Telegram admin veya webhook) tanımlı mı?
    ///
    /// Karantina akışı bu kontrole dayanır: kanal yoksa karantinaya düşen ögeler
    /// kimseye ulaşmaz ve sessiz kaybın yeni adı olur (plan 5.4).
    pub fn alert_channel_configured() -> bool {
        let telegram_ready = std::env::var("TELEGRAM_BOT_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .is_some()
            && std::env::var("TELEGRAM_ADMIN_CHAT_ID")
                .or_else(|_| std::env::var("TELEGRAM_CHAT_ID"))
                .ok()
                .filter(|s| !s.trim().is_empty())
                .is_some();
        let webhook_ready = std::env::var("ALERT_WEBHOOK_URL")
            .or_else(|_| std::env::var("DISCORD_WEBHOOK_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .is_some();
        telegram_ready || webhook_ready
    }

    /// Test ortamında mıyız kontrol eder.
    /// `cargo test`, test koşucusu ikilileri veya `KEPCE_TEST` aktif olduğunda
    /// dış servislere (Telegram / Discord) bildirim gitmesini engeller.
    pub fn is_test_env() -> bool {
        if cfg!(test) {
            return true;
        }
        if std::env::var("KEPCE_TEST")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false)
        {
            return true;
        }
        if let Ok(exe) = std::env::current_exe() {
            let path_str = exe.to_string_lossy();
            if path_str.contains("/deps/") {
                return true;
            }
        }
        false
    }

    /// Birleşik alarm gönderir: Hem Telegram hem Discord yapılandırılmışsa ikisine de iletir.
    pub async fn send_alert(message: &str) -> anyhow::Result<()> {
        let _ = Self::send_webhook_alert(message).await;
        let _ = Self::send_telegram_alert(message).await;
        Ok(())
    }

    /// Telegram Bot API üzerinden alarm mesajı gönderir.
    ///
    /// `parse_mode` kullanılmaz: biçimlendirme hatası, mesajın Telegram tarafından
    /// tümden reddedilip operatöre hiç ulaşmamasına yol açabiliyor.
    pub async fn send_telegram_alert(message: &str) -> anyhow::Result<()> {
        if Self::is_test_env() {
            tracing::debug!("Test ortamı algılandı; Telegram alarmı gönderilmedi.");
            return Ok(());
        }

        let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(token) if !token.trim().is_empty() => token,
            _ => return Ok(()),
        };

        let chat_id = match std::env::var("TELEGRAM_ADMIN_CHAT_ID")
            .or_else(|_| std::env::var("TELEGRAM_CHAT_ID"))
        {
            Ok(id) if !id.trim().is_empty() => id,
            _ => {
                tracing::warn!(
                    "Telegram alarmı gönderilemedi: TELEGRAM_ADMIN_CHAT_ID ayarlanmamış."
                );
                return Ok(());
            }
        };

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            bot_token.trim()
        );
        let client = Client::new();
        let payload = json!({
            "chat_id": chat_id.trim(),
            "text": format!("[KEPÇE ALARMI]\n\n{}", message)
        });

        tracing::info!("Telegram alarmı gönderiliyor: {}", message);

        let res = client
            .post(&url)
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

    /// Telegram Bot API üzerinden dosya (belge/fotoğraf) ile birlikte alarm mesajı gönderir.
    pub async fn send_telegram_document(
        caption: &str,
        file_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let empty_kb = serde_json::json!({});
        Self::send_telegram_document_with_buttons(caption, file_path, empty_kb).await
    }

    /// Telegram Bot API üzerinden doküman ve inline butonlarla birlikte mesaj gönderir.
    pub async fn send_telegram_document_with_buttons(
        caption: &str,
        file_path: &std::path::Path,
        inline_keyboard: serde_json::Value,
    ) -> anyhow::Result<()> {
        if Self::is_test_env() {
            tracing::debug!("Test ortamı algılandı; Telegram dokümanı gönderilmedi.");
            return Ok(());
        }

        let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(token) if !token.trim().is_empty() => token,
            _ => return Ok(()),
        };

        let chat_id = match std::env::var("TELEGRAM_ADMIN_CHAT_ID")
            .or_else(|_| std::env::var("TELEGRAM_CHAT_ID"))
        {
            Ok(id) if !id.trim().is_empty() => id,
            _ => return Ok(()),
        };

        if !file_path.exists() {
            return Self::send_telegram_message_with_buttons(caption, inline_keyboard).await;
        }

        let file_bytes = std::fs::read(file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_string();

        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name);
        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.trim().to_string())
            .text("caption", caption.to_string())
            .part("document", part);

        if inline_keyboard.get("inline_keyboard").is_some() {
            let kb_str = serde_json::to_string(&inline_keyboard).unwrap_or_default();
            if !kb_str.is_empty() {
                form = form.text("reply_markup", kb_str);
            }
        }

        let url = format!(
            "https://api.telegram.org/bot{}/sendDocument",
            bot_token.trim()
        );
        let client = Client::new();
        let res = client.post(&url).multipart(form).send().await?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            tracing::warn!(
                "sendDocumentWithButtons başarısız ({}), metin uyarısına geçiliyor",
                err
            );
            return Self::send_telegram_message_with_buttons(caption, inline_keyboard).await;
        }

        Ok(())
    }

    /// Telegram Bot API üzerinden inline butonlu metin mesajı gönderir.
    pub async fn send_telegram_message_with_buttons(
        text: &str,
        inline_keyboard: serde_json::Value,
    ) -> anyhow::Result<()> {
        if Self::is_test_env() {
            tracing::debug!("Test ortamı algılandı; Telegram butonlu mesaj gönderilmedi.");
            return Ok(());
        }

        let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(token) if !token.trim().is_empty() => token,
            _ => return Ok(()),
        };

        let chat_id = match std::env::var("TELEGRAM_ADMIN_CHAT_ID")
            .or_else(|_| std::env::var("TELEGRAM_CHAT_ID"))
        {
            Ok(id) if !id.trim().is_empty() => id,
            _ => return Ok(()),
        };

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            bot_token.trim()
        );
        let client = Client::new();
        let mut payload = serde_json::Map::new();
        payload.insert(
            "chat_id".to_string(),
            serde_json::Value::String(chat_id.trim().to_string()),
        );
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );
        if inline_keyboard.get("inline_keyboard").is_some() {
            payload.insert("reply_markup".to_string(), inline_keyboard);
        }

        let res = client.post(&url).json(&payload).send().await?;
        if !res.status().is_success() {
            let err_body = res.text().await.unwrap_or_default();
            tracing::error!("Telegram butonlu mesaj isteği başarısız oldu: {}", err_body);
        }

        Ok(())
    }

    /// Webhook uyarısı gönderir (Discord uyumlu JSON payload).
    /// Geriye dönük uyumluluk için, Telegram yapılandırılmışsa Telegram'a da iletir.
    pub async fn send_webhook_alert(message: &str) -> anyhow::Result<()> {
        if Self::is_test_env() {
            tracing::debug!("Test ortamı algılandı; Webhook alarmı gönderilmedi.");
            return Ok(());
        }

        if std::env::var("TELEGRAM_BOT_TOKEN").is_ok() {
            let _ = Self::send_telegram_alert(message).await;
        }

        let webhook_url = match std::env::var("ALERT_WEBHOOK_URL")
            .or_else(|_| std::env::var("DISCORD_WEBHOOK_URL"))
        {
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

        let res = client
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            tracing::info!("Webhook uyarısı başarıyla iletildi.");
        } else {
            tracing::error!(
                "Webhook isteği başarısız oldu. HTTP durum kodu: {}",
                res.status()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_alert_missing_env() {
        // Rust 2024: set_var/remove_var artık unsafe. Test tek başına çalıştığı ve
        // bu değişkenleri okuyan başka bir thread olmadığı için güvenli.
        unsafe {
            std::env::remove_var("ALERT_WEBHOOK_URL");
            std::env::remove_var("DISCORD_WEBHOOK_URL");
            std::env::remove_var("TELEGRAM_BOT_TOKEN");
            std::env::remove_var("TELEGRAM_ADMIN_CHAT_ID");
        }
        let res = AlertingService::send_alert("Test uyarısı").await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_is_test_env_in_tests() {
        assert!(AlertingService::is_test_env());
    }
}

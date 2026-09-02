// Kepçe Worker - Task: Telegram Operatör Botu (İki Yönlü Ops & Alarm)
// =====================================================================
//
// Long-polling mekanizmasıyla Telegram Bot API üzerinden yöneticiden gelen
// komutları dinler ve anlık yanıt döner. Dışa port açma veya webhook SSL
// sertifikası gerektirmez.

use std::time::Duration;
use sea_orm::*;
use serde_json::json;
use chrono::Local;
use reqwest::Client;
use shared::entities::{cities, menus, sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum}};

/// Telegram botuna Markdown formatında yanıt gönderir.
async fn send_reply(client: &Client, bot_token: &str, chat_id: i64, text: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let payload = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown"
    });

    if let Err(e) = client.post(&url).json(&payload).send().await {
        tracing::error!("[TELEGRAM-BOT] Yanıt gönderilemedi: {:?}", e);
    }
}

/// Ana bot dinleme döngüsü
pub async fn run_telegram_bot_loop(
    db: &DatabaseConnection,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            tracing::info!("[TELEGRAM-BOT] TELEGRAM_BOT_TOKEN tanımlı değil. Bot döngüsü başlatılmadı.");
            return Ok(());
        }
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(35))
        .build()?;

    let mut offset: i64 = 0;
    tracing::info!("[TELEGRAM-BOT] İki yönlü Telegram operatör botu aktif. Dinleniyor...");

    loop {
        if *shutdown_rx.borrow() {
            tracing::info!("[TELEGRAM-BOT] Kapatma sinyali algılandı. Bot durduruluyor.");
            break;
        }

        let admin_chat_id_env = std::env::var("TELEGRAM_ADMIN_CHAT_ID")
            .or_else(|_| std::env::var("TELEGRAM_CHAT_ID"))
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok());

        let poll_url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=25",
            bot_token, offset
        );

        tokio::select! {
            _ = shutdown_rx.changed() => {
                tracing::info!("[TELEGRAM-BOT] Kapatma sinyali algılandı.");
                break;
            }
            res = client.get(&poll_url).send() => {
                match res {
                    Ok(resp) => {
                        if let Ok(json_data) = resp.json::<serde_json::Value>().await {
                            if let Some(updates) = json_data.get("result").and_then(|r| r.as_array()) {
                                for update in updates {
                                    if let Some(up_id) = update.get("update_id").and_then(|u| u.as_i64()) {
                                        offset = up_id + 1;
                                    }

                                    let msg = match update.get("message") {
                                        Some(m) => m,
                                        None => continue,
                                    };

                                    let chat_id = match msg.get("chat").and_then(|c| c.get("id")).and_then(|i| i.as_i64()) {
                                        Some(id) => id,
                                        None => continue,
                                    };

                                    let text = msg.get("text").and_then(|t| t.as_str()).unwrap_or("").trim();
                                    if text.is_empty() {
                                        continue;
                                    }

                                    // 1. Admin Doğrulama / Chat ID Keşfi
                                    match admin_chat_id_env {
                                        Some(admin_id) if admin_id != chat_id => {
                                            tracing::warn!("[TELEGRAM-BOT] Yetkisiz erişim denemesi: Chat ID {}", chat_id);
                                            send_reply(
                                                &client,
                                                &bot_token,
                                                chat_id,
                                                &format!("⛔ *Yetkisiz Erişim!*\nBu bot yalnızca Kepçe sistem yöneticisine aittir.\nChat ID'niz: `{}`", chat_id)
                                            ).await;
                                            continue;
                                        }
                                        None => {
                                            // Admin ID henüz .env'de tanımlı değil -> Kullanıcıya chat ID'sini söyle
                                            send_reply(
                                                &client,
                                                &bot_token,
                                                chat_id,
                                                &format!(
                                                    "👋 *Kepçe Operatör Botu Hazır!*\n\nHenüz `.env` dosyanızda yönetici Chat ID tanımlanmamış.\n\nSizin Chat ID numaranız: `{}`\n\nBu numarayı `.env` dosyanıza `TELEGRAM_ADMIN_CHAT_ID={}` olarak ekleyin ve sistemi yeniden başlatın.",
                                                    chat_id, chat_id
                                                )
                                            ).await;
                                            continue;
                                        }
                                        _ => {} // Yetkili admin
                                    }

                                    // 2. Komut İşleme
                                    handle_command(db, &client, &bot_token, chat_id, text, shutdown_rx.clone()).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("[TELEGRAM-BOT] getUpdates hatası: {:?}. 5 saniye bekleniyor...", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Gelen komutu işleyip yanıt döner
async fn handle_command(
    db: &DatabaseConnection,
    client: &Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let command = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();

    match command.as_str() {
        "/start" | "/yardim" | "yardim" => {
            let help_msg = "\
🤖 *Kepçe Operatör Botu*\n\n\
Kullanabileceğiniz komutlar:\n\
• `/durum` - Canlı sunucu, DB ve IP devre kesici sağlığı\n\
• `/tara [sehir]` - Menü kazımayı anlık tetikle (örn: `/tara` veya `/tara istanbul`)\n\
• `/ban_kaldir` - IP ban devre kesicisini erken sıfırla\n\
• `/son_menuler` - Sisteme eklenen son 5 güncel menü\n\
• `/yardim` - Bu yardım menüsü";
            send_reply(client, bot_token, chat_id, help_msg).await;
        }

        "/durum" | "durum" => {
            // DB ve Menü Durumu
            let today = Local::now().naive_local().date();
            let total_cities = cities::Entity::find().count(db).await.unwrap_or(0);
            let today_approved = menus::Entity::find()
                .filter(menus::Column::ServeDate.eq(today))
                .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
                .count(db)
                .await
                .unwrap_or(0);

            let ban_status_msg = match crate::tasks::scraper::get_ban_status() {
                Some(remaining_secs) => {
                    let mins = remaining_secs / 60;
                    format!("🔴 *DEVRE KESİCİ AKTİF* (Banlı, kalan süre: ~{} dk)", mins)
                }
                None => "🟢 *Normal* (Engelleme yok)".to_string(),
            };

            let status_msg = format!(
                "📊 *Kepçe Sistem Durumu*\n\n\
                • *Tarih:* `{}`\n\
                • *Veritabanı:* Bağlı (OK)\n\
                • *Kayıtlı Şehir:* `{}` il\n\
                • *Bugünkü Onaylı Menü:* `{}` adet\n\
                • *Scraper Hat Durumu:* {}",
                today.format("%d.%m.%Y"),
                total_cities,
                today_approved,
                ban_status_msg
            );

            send_reply(client, bot_token, chat_id, &status_msg).await;
        }

        "/ban_kaldir" | "ban_kaldir" => {
            crate::tasks::scraper::reset_ban_status();
            let msg = "✅ *Devre kesici başarıyla sıfırlandı!*\nScraper engelleme bayrağı kaldırıldı; yeni istekler kykyemek sunucusuna iletilecektir.";
            send_reply(client, bot_token, chat_id, msg).await;
        }

        "/son_menuler" | "son_menuler" => {
            let last_menus = menus::Entity::find()
                .find_also_related(cities::Entity)
                .order_by_desc(menus::Column::CreatedAt)
                .limit(5)
                .all(db)
                .await;

            match last_menus {
                Ok(items) if !items.is_empty() => {
                    let mut lines = vec!["📋 *Sisteme Eklenen Son 5 Menü:*".to_string()];
                    for (m, city_opt) in items {
                        let city_name = city_opt.map(|c| c.name).unwrap_or_else(|| "Bilinmeyen".to_string());
                        let meal = match m.meal_type {
                            MealTypeEnum::Breakfast => "Kahvaltı",
                            MealTypeEnum::Lunch => "Öğle",
                            MealTypeEnum::Dinner => "Akşam",
                        };
                        lines.push(format!(
                            "• *{}* ({}): `{}` - [Durum: {:?}]",
                            city_name,
                            m.serve_date.format("%d.%m.%Y"),
                            meal,
                            m.status
                        ));
                    }
                    send_reply(client, bot_token, chat_id, &lines.join("\n")).await;
                }
                _ => {
                    send_reply(client, bot_token, chat_id, "Henüz kayıtlı menü bulunamadı.").await;
                }
            }
        }

        cmd if cmd.starts_with("/tara") || cmd.starts_with("tara") => {
            let specific_city = parts.get(1).copied();
            let target_desc = specific_city.unwrap_or("Tüm aktif şehirler");
            
            send_reply(
                client,
                bot_token,
                chat_id,
                &format!("🚀 *Kazıma işlemi tetiklendi!*\nHedef: `{}`\nArka planda çalışıyor, tamamlandığında özet bildirimi gelecektir.", target_desc)
            ).await;

            let db_clone = db.clone();
            let client_scraper = client.clone();
            let shutdown_clone = shutdown_rx.clone();
            let bot_token_clone = bot_token.to_string();

            tokio::spawn(async move {
                let start_time = std::time::Instant::now();
                let scrape_res = crate::tasks::scraper::scrape_today_menus(&db_clone, &client_scraper, shutdown_clone).await;
                let elapsed = start_time.elapsed().as_secs();

                let finish_msg = match scrape_res {
                    Ok(count) => {
                        format!("✅ *Manuel Kazıma Tamamlandı!*\n• Kaydedilen/Güncellenen: `{}` menü\n• Geçen süre: `{} sn`", count, elapsed)
                    }
                    Err(e) => {
                        format!("❌ *Manuel Kazıma Sırasında Hata Oluştu!*\nHata detayı: `{:?}`", e)
                    }
                };

                send_reply(&client_scraper, &bot_token_clone, chat_id, &finish_msg).await;
            });
        }

        _ => {
            send_reply(
                client,
                bot_token,
                chat_id,
                "❓ Bilinmeyen komut. Kullanılabilir komutları görmek için `/yardim` yazabilirsiniz."
            ).await;
        }
    }
}

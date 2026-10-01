//! Telegram operatör botu dinleyicisi.
//!
//! Long-polling yöntemiyle Telegram Bot API üzerinden gelen yönetici komutlarını
//! işler ve anlık sistem durumu/işlem yanıtlarını döner. Dışa açık port veya
//! webhook sertifikası gerektirmez.

use chrono::Local;
use reqwest::Client;
use sea_orm::*;
use serde_json::json;
use shared::entities::{
    cities, menus,
    sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum},
};
use std::time::Duration;

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
            tracing::info!(
                "[TELEGRAM-BOT] TELEGRAM_BOT_TOKEN tanımlı değil. Bot döngüsü başlatılmadı."
            );
            return Ok(());
        }
    };

    let client = Client::builder().timeout(Duration::from_secs(35)).build()?;

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
                        if let Ok(json_data) = resp.json::<serde_json::Value>().await
                            && let Some(updates) = json_data.get("result").and_then(|r| r.as_array()) {
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
        "/" | "/start" | "/help" | "help" | "/yardim" | "yardim" => {
            let help_msg = "\
🤖 *Kepçe Operatör Botu*\n\n\
Kullanabileceğiniz komutlar:\n\
• `/durum` - Canlı sunucu, DB ve IP devre kesici sağlığı\n\
• `/tara [sehir]` - Menü kazımayı anlık tetikle (örn: `/tara` veya `/tara istanbul`)\n\
• `/karantina` - Karantina kuyruğunu listele (karar bekleyen dosyalar)\n\
• `/karantina detay <id>` - Karantina öğesinin teşhis ayrıntıları\n\
• `/dosya <id>` - Karantinadaki dosyanın orijinalini sohbete gönderir\n\
• `/onayla <id>` - Karantinadaki dosyayı kapsam içi tarihlerle işle\n\
• `/reddet <id>` - Karantinadaki dosyayı hatali/ altına taşı\n\
• `/ata <id> <sehir>` - Şehirsiz karantina öğesine şehir ata ve işleme al\n\
• `/yorumlar_uret` - Eksik menüler için otomatik LLM öğrenci yorumu üret\n\
• `/ban_kaldir` - IP ban devre kesicisini erken sıfırla\n\
• `/son_menuler` - Sisteme eklenen son 5 güncel menü\n\
• `/yardim` - Bu yardım menüsü";
            send_reply(client, bot_token, chat_id, help_msg).await;
        }

        "/dosya" | "dosya" => {
            let Some(id) = parts.get(1).copied() else {
                send_reply(client, bot_token, chat_id, "Kullanım: `/dosya <id>`").await;
                return;
            };
            let base = crate::tasks::quarantine::menu_base_dir();
            match crate::tasks::quarantine::find_item(&base, id).await {
                Some(item) => {
                    if item.file_path.exists() {
                        let caption = format!(
                            "📁 Karantina Dosyası: {} ({})",
                            item.meta.file, item.meta.id
                        );
                        if let Err(e) =
                            shared::services::alerting::AlertingService::send_telegram_document(
                                &caption,
                                &item.file_path,
                            )
                            .await
                        {
                            send_reply(
                                client,
                                bot_token,
                                chat_id,
                                &format!("❌ Dosya gönderilemedi: {:?}", e),
                            )
                            .await;
                        }
                    } else {
                        send_reply(client, bot_token, chat_id, "❌ Dosya diskte bulunamadı.").await;
                    }
                }
                None => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("❓ `'{}'` kimlikli karantina öğesi bulunamadı.", id),
                    )
                    .await;
                }
            }
        }

        "/karantina" | "karantina" => {
            let base = crate::tasks::quarantine::menu_base_dir();
            if parts.get(1).map(|s| s.to_lowercase()) == Some("detay".to_string()) {
                match parts.get(2) {
                    Some(id) => match crate::tasks::quarantine::find_item(&base, id).await {
                        Some(item) => {
                            let msg = crate::tasks::quarantine::format_item_detail(&item).await;
                            send_reply(client, bot_token, chat_id, &msg).await;
                        }
                        None => {
                            send_reply(
                                client,
                                bot_token,
                                chat_id,
                                &format!("❓ `'{}'` kimlikli karantina öğesi bulunamadı.", id),
                            )
                            .await;
                        }
                    },
                    None => {
                        send_reply(
                            client,
                            bot_token,
                            chat_id,
                            "Kullanım: `/karantina detay <id>`",
                        )
                        .await;
                    }
                }
            } else {
                let msg = crate::tasks::quarantine::format_queue_listing(&base).await;
                send_reply(client, bot_token, chat_id, &msg).await;
            }
        }

        "/onayla" | "onayla" => {
            let Some(id) = parts.get(1).copied() else {
                send_reply(client, bot_token, chat_id, "Kullanım: `/onayla <id>`").await;
                return;
            };
            send_reply(
                client,
                bot_token,
                chat_id,
                &format!("⏳ `{}` onaylanıyor, dosya yeniden ayrıştırılıyor...", id),
            )
            .await;

            let db_clone = db.clone();
            let client_clone = client.clone();
            let bot_token_clone = bot_token.to_string();
            let id_owned = id.to_string();
            let gemini_key = std::env::var("GEMINI_API_KEY").ok();
            tokio::spawn(async move {
                let res = crate::tasks::file_ingest::approve_quarantine_item(
                    &db_clone,
                    &client_clone,
                    gemini_key.as_deref(),
                    &id_owned,
                )
                .await;
                let msg = match res {
                    Ok(m) => m,
                    Err(e) => {
                        let raw = format!("{:?}", e);
                        // Geçici sağlayıcı hatası (503/429) ile kalıcı hatayı ayır:
                        // operatöre ne yapacağını söylemeyen çıplak hata metni bırakma.
                        let hint = if crate::tasks::file_ingest::is_transient_error(
                            &raw.to_lowercase(),
                        ) {
                            "⏳ Sağlayıcı geçici olarak yanıt vermedi (503/429). Öğe karantinada KALDI; birkaç dakika sonra `/onayla` komutunu tekrar deneyin."
                        } else {
                            "ℹ️ Öğe karantinada kaldı (`/karantina detay` ile inceleyin). Sorun kalıcıysa `/reddet` ile hatali/ altına alın."
                        };
                        format!(
                            "❌ *Onaylama başarısız* `{}`\n{}\n\n`{}`",
                            id_owned, hint, raw
                        )
                    }
                };
                send_reply(&client_clone, &bot_token_clone, chat_id, &msg).await;
            });
        }

        "/reddet" | "reddet" => {
            let Some(id) = parts.get(1).copied() else {
                send_reply(client, bot_token, chat_id, "Kullanım: `/reddet <id>`").await;
                return;
            };
            match crate::tasks::file_ingest::reject_quarantine_item(id).await {
                Ok(m) => send_reply(client, bot_token, chat_id, &m).await,
                Err(e) => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("❌ *Reddetme başarısız* `{}`\n`{:?}`", id, e),
                    )
                    .await
                }
            }
        }

        "/ata" | "ata" => {
            let (Some(id), Some(slug)) = (parts.get(1).copied(), parts.get(2).copied()) else {
                send_reply(client, bot_token, chat_id, "Kullanım: `/ata <id> <sehir>`").await;
                return;
            };
            match crate::tasks::file_ingest::assign_quarantine_item(db, id, slug).await {
                Ok(m) => {
                    send_reply(client, bot_token, chat_id, &m).await;
                    // Taşınan dosya hemen işleme alınsın (plan 2.3).
                    let db_clone = db.clone();
                    let client_clone = client.clone();
                    let bot_token_clone = bot_token.to_string();
                    let gemini_key = std::env::var("GEMINI_API_KEY").ok();
                    tokio::spawn(async move {
                        let res = crate::tasks::file_ingest::process_local_files(
                            &db_clone,
                            &client_clone,
                            gemini_key.as_deref(),
                        )
                        .await;
                        if let Err(e) = res {
                            send_reply(
                                &client_clone,
                                &bot_token_clone,
                                chat_id,
                                &format!("❌ /ata sonrası işleme hatası: `{:?}`", e),
                            )
                            .await;
                        }
                    });
                }
                Err(e) => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("❌ *Şehir atanamadı* `{}`\n`{:?}`", id, e),
                    )
                    .await
                }
            }
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
                        let city_name = city_opt
                            .map(|c| c.name)
                            .unwrap_or_else(|| "Bilinmeyen".to_string());
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
                let scrape_res = crate::tasks::scraper::scrape_today_menus(
                    &db_clone,
                    &client_scraper,
                    shutdown_clone,
                )
                .await;
                let elapsed = start_time.elapsed().as_secs();

                let finish_msg = match scrape_res {
                    Ok(count) => {
                        format!(
                            "✅ *Manuel Kazıma Tamamlandı!*\n• Kaydedilen/Güncellenen: `{}` menü\n• Geçen süre: `{} sn`",
                            count, elapsed
                        )
                    }
                    Err(e) => {
                        format!(
                            "❌ *Manuel Kazıma Sırasında Hata Oluştu!*\nHata detayı: `{:?}`",
                            e
                        )
                    }
                };

                send_reply(&client_scraper, &bot_token_clone, chat_id, &finish_msg).await;
            });
        }

        "/yorumlar_uret" | "yorumlar_uret" => {
            send_reply(
                client,
                bot_token,
                chat_id,
                "🤖 *Otomatik LLM yorum üretimi başlatıldı!*\nEksik menüler öncelik sırasına göre işleniyor. Tamamlandığında bildirim gelecektir."
            ).await;

            let db_clone = db.clone();
            let client_clone = client.clone();
            let bot_token_clone = bot_token.to_string();

            tokio::spawn(async move {
                let start_time = std::time::Instant::now();
                let gen_res =
                    crate::tasks::comment_generator::run_comment_generation(&db_clone).await;
                let elapsed = start_time.elapsed().as_secs();

                let finish_msg = match gen_res {
                    Ok(count) => {
                        format!(
                            "✅ *Yorum Üretimi Tamamlandı!*\n• Güncellenen menü: `{}` adet\n• Geçen süre: `{} sn`",
                            count, elapsed
                        )
                    }
                    Err(e) => {
                        format!(
                            "❌ *Yorum Üretimi Sırasında Hata Oluştu!*\nHata detayı: `{:?}`",
                            e
                        )
                    }
                };

                send_reply(&client_clone, &bot_token_clone, chat_id, &finish_msg).await;
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

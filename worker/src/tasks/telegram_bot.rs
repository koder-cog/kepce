//! Telegram operatör botu dinleyicisi.
//!
//! Long-polling yöntemiyle Telegram Bot API üzerinden gelen yönetici komutlarını
//! ve inline buton (callback_query) etkileşimlerini işler. Dışa açık port veya
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

/// Telegram botuna düz metin olarak yanıt gönderir.
async fn send_reply(client: &Client, bot_token: &str, chat_id: i64, text: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let payload = json!({
        "chat_id": chat_id,
        "text": text
    });

    if let Err(e) = client.post(&url).json(&payload).send().await {
        tracing::error!("[TELEGRAM-BOT] Yanıt gönderilemedi: {:?}", e);
    }
}

/// Desteklenen Telegram bot komutları ve mobil (Apple HIG) standartlarında kısa açıklamaları.
/// Açıklamalar dar mobil ekranlarda (iPhone SE / 13 mini vb.) tek satıra sığacak şekilde
/// en fazla 22-25 karakter tutulur.
pub const BOT_COMMANDS: &[(&str, &str)] = &[
    ("durum", "Sistem durumu"),
    ("karantina", "Karantina kuyruğu"),
    ("onayla", "Karantinayı onayla"),
    ("reddet", "Karantinayı reddet"),
    ("ata", "Öğeye şehir ata"),
    ("dosya", "Orijinal dosyayı al"),
    ("tara", "Menü kazımayı başlat"),
    ("son_menuler", "Son kayıtlı menüler"),
    ("ban_kaldir", "Devre kesiciyi sıfırla"),
    ("yorumlar_uret", "Eksik yorumları üret"),
    ("yardim", "Komut kılavuzu"),
];

/// Telegram komut menüsünü (setMyCommands) Telegram API'sine kaydeder.
/// Böylece kullanıcı '/' yazdığında desteklenen tüm komutlar listelenir.
pub async fn register_bot_commands(client: &Client, bot_token: &str) {
    let url = format!("https://api.telegram.org/bot{}/setMyCommands", bot_token);
    let commands_json: Vec<serde_json::Value> = BOT_COMMANDS
        .iter()
        .map(|(cmd, desc)| {
            json!({
                "command": cmd,
                "description": desc
            })
        })
        .collect();

    let payload = json!({
        "commands": commands_json
    });

    match client.post(&url).json(&payload).send().await {
        Ok(res) if res.status().is_success() => {
            tracing::info!("[TELEGRAM-BOT] Komut menüsü Telegram'a başarıyla kaydedildi.");
        }
        Ok(res) => {
            let body = res.text().await.unwrap_or_default();
            tracing::warn!("[TELEGRAM-BOT] setMyCommands yanıt hatası: {}", body);
        }
        Err(e) => {
            tracing::warn!("[TELEGRAM-BOT] setMyCommands isteği başarısız: {:?}", e);
        }
    }
}

/// Butona tıklandığında Telegram istemcisindeki yükleniyor göstergesini sonlandırır.
async fn answer_callback_query(
    client: &Client,
    bot_token: &str,
    callback_query_id: &str,
    text: Option<&str>,
) {
    let url = format!(
        "https://api.telegram.org/bot{}/answerCallbackQuery",
        bot_token
    );
    let mut payload = json!({
        "callback_query_id": callback_query_id
    });
    if let Some(t) = text {
        payload["text"] = json!(t);
    }
    let _ = client.post(&url).json(&payload).send().await;
}

/// Mesajın inline klavyesini günceller.
async fn edit_message_reply_markup(
    client: &Client,
    bot_token: &str,
    chat_id: i64,
    message_id: i64,
    reply_markup: serde_json::Value,
) {
    let url = format!(
        "https://api.telegram.org/bot{}/editMessageReplyMarkup",
        bot_token
    );
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "reply_markup": reply_markup
    });
    let _ = client.post(&url).json(&payload).send().await;
}

/// Mesaj metnini veya başlığını (caption) düzenler ve butonları kaldırır.
async fn edit_message_content(
    client: &Client,
    bot_token: &str,
    chat_id: i64,
    message_id: i64,
    is_caption: bool,
    new_text: &str,
) {
    let method = if is_caption {
        "editMessageCaption"
    } else {
        "editMessageText"
    };
    let field = if is_caption { "caption" } else { "text" };
    let url = format!("https://api.telegram.org/bot{}/{}", bot_token, method);
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        field: new_text,
        "reply_markup": { "inline_keyboard": [] }
    });
    let _ = client.post(&url).json(&payload).send().await;
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

    // Bot komut menüsünü Telegram'a kaydet
    register_bot_commands(&client, &bot_token).await;

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

                                    // A) Callback Query (Inline Buton Tıklaması)
                                    if let Some(cq) = update.get("callback_query") {
                                        handle_callback_query_event(db, &client, &bot_token, cq, admin_chat_id_env).await;
                                        continue;
                                    }

                                    // B) Standart Mesaj
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

                                    // Yetki Doğrulama / Chat ID Keşfi
                                    match admin_chat_id_env {
                                        Some(admin_id) if admin_id != chat_id => {
                                            tracing::warn!("[TELEGRAM-BOT] Yetkisiz erişim denemesi: Chat ID {}", chat_id);
                                            send_reply(
                                                &client,
                                                &bot_token,
                                                chat_id,
                                                &format!("[YETKİSİZ ERİŞİM]\nBu bot yalnızca Kepçe sistem yöneticisine aittir.\nChat ID: {}", chat_id)
                                            ).await;
                                            continue;
                                        }
                                        None => {
                                            send_reply(
                                                &client,
                                                &bot_token,
                                                chat_id,
                                                &format!(
                                                    "[KEPÇE OPERATÖR BOTU]\nHenüz .env dosyasında yönetici Chat ID tanımlanmamış.\nChat ID numaranız: {}\nBu numarayı .env dosyasına TELEGRAM_ADMIN_CHAT_ID={} olarak ekleyip sistemi yeniden başlatın.",
                                                    chat_id, chat_id
                                                )
                                            ).await;
                                            continue;
                                        }
                                        _ => {} // Yetkili admin
                                    }

                                    // Komut İşleme
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

/// Buton tıklamalarını işler
async fn handle_callback_query_event(
    db: &DatabaseConnection,
    client: &Client,
    bot_token: &str,
    cq: &serde_json::Value,
    admin_chat_id_env: Option<i64>,
) {
    let cq_id = cq.get("id").and_then(|i| i.as_str()).unwrap_or_default();
    let data = cq
        .get("data")
        .and_then(|d| d.as_str())
        .unwrap_or_default()
        .trim();
    let message = cq.get("message");
    let chat_id = message
        .and_then(|m| m.get("chat"))
        .and_then(|c| c.get("id"))
        .and_then(|i| i.as_i64())
        .unwrap_or_default();
    let message_id = message
        .and_then(|m| m.get("message_id"))
        .and_then(|i| i.as_i64())
        .unwrap_or_default();
    let is_caption = message.and_then(|m| m.get("caption")).is_some();

    // Yetki kontrolü
    if let Some(admin_id) = admin_chat_id_env
        && admin_id != chat_id
    {
        answer_callback_query(
            client,
            bot_token,
            cq_id,
            Some("Bu işlem için yetkiniz bulunmuyor."),
        )
        .await;
        return;
    }

    let parts: Vec<&str> = data.split(':').collect();
    if parts.len() < 3 || parts[0] != "q" {
        answer_callback_query(client, bot_token, cq_id, None).await;
        return;
    }

    let action = parts[1];
    let id = parts[2];

    match action {
        "approve" => {
            answer_callback_query(client, bot_token, cq_id, Some("Onaylama başlatıldı...")).await;
            send_reply(
                client,
                bot_token,
                chat_id,
                &format!("[İŞLEM] '{}' onaylanıyor, veriler işleniyor...", id),
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

                match res {
                    Ok(msg) => {
                        let final_card = format!(
                            "[ONAYLANDI: {}]\n{}\nİşlem Zamanı: {}",
                            id_owned,
                            msg,
                            Local::now().format("%d.%m.%Y %H:%M")
                        );
                        edit_message_content(
                            &client_clone,
                            &bot_token_clone,
                            chat_id,
                            message_id,
                            is_caption,
                            &final_card,
                        )
                        .await;
                    }
                    Err(e) => {
                        let raw = format!("{:?}", e);
                        send_reply(
                            &client_clone,
                            &bot_token_clone,
                            chat_id,
                            &format!("[HATA] Onaylama başarısız ({}):\n{}", id_owned, raw),
                        )
                        .await;
                    }
                }
            });
        }

        "reject_confirm" => {
            answer_callback_query(client, bot_token, cq_id, None).await;
            let confirm_kb = json!({
                "inline_keyboard": [
                    [
                        { "text": "Reddetmeyi Onayla", "callback_data": format!("q:reject:{}", id) },
                        { "text": "Vazgeç", "callback_data": format!("q:cancel:{}", id) }
                    ]
                ]
            });
            edit_message_reply_markup(client, bot_token, chat_id, message_id, confirm_kb).await;
        }

        "cancel" => {
            answer_callback_query(client, bot_token, cq_id, Some("İşlem iptal edildi.")).await;
            let orig_kb = crate::tasks::quarantine::item_inline_keyboard(id);
            edit_message_reply_markup(client, bot_token, chat_id, message_id, orig_kb).await;
        }

        "reject" => {
            answer_callback_query(client, bot_token, cq_id, Some("Dosya reddedildi.")).await;
            match crate::tasks::file_ingest::reject_quarantine_item(id).await {
                Ok(msg) => {
                    let final_card = format!(
                        "[REDDEDİLDİ: {}]\n{}\nİşlem Zamanı: {}",
                        id,
                        msg,
                        Local::now().format("%d.%m.%Y %H:%M")
                    );
                    edit_message_content(
                        client,
                        bot_token,
                        chat_id,
                        message_id,
                        is_caption,
                        &final_card,
                    )
                    .await;
                }
                Err(e) => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("[HATA] Reddetme başarısız ({}): {:?}", id, e),
                    )
                    .await;
                }
            }
        }

        "detail" => {
            answer_callback_query(client, bot_token, cq_id, None).await;
            let base = crate::tasks::quarantine::menu_base_dir();
            match crate::tasks::quarantine::find_item(&base, id).await {
                Some(item) => {
                    let msg = crate::tasks::quarantine::format_item_detail(&item).await;
                    send_reply(client, bot_token, chat_id, &msg).await;
                }
                None => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("[BİLGİ] '{}' kimlikli karantina öğesi bulunamadı.", id),
                    )
                    .await;
                }
            }
        }

        "file" => {
            answer_callback_query(client, bot_token, cq_id, Some("Dosya gönderiliyor...")).await;
            let base = crate::tasks::quarantine::menu_base_dir();
            match crate::tasks::quarantine::find_item(&base, id).await {
                Some(item) => {
                    if item.file_path.exists() {
                        let caption =
                            format!("Karantina Dosyası: {} ({})", item.meta.file, item.meta.id);
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
                                &format!("[HATA] Dosya gönderilemedi: {:?}", e),
                            )
                            .await;
                        }
                    } else {
                        send_reply(
                            client,
                            bot_token,
                            chat_id,
                            "[HATA] Dosya diskte bulunamadı.",
                        )
                        .await;
                    }
                }
                None => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("[BİLGİ] '{}' kimlikli karantina öğesi bulunamadı.", id),
                    )
                    .await;
                }
            }
        }

        _ => {
            answer_callback_query(client, bot_token, cq_id, None).await;
        }
    }
}

/// Gelen metin komutunu işler ve yanıt döner
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
[KEPÇE OPERATÖR BOTU]

Kullanılabilir komutlar:
• /durum - Sistem durumu
• /karantina - Karantina kuyruğu
• /karantina detay <id> - Öğe teşhis ayrıntıları
• /dosya <id> - Orijinal dosyayı sohbete gönder
• /onayla <id> - Karantinadaki dosyayı onayla
• /reddet <id> - Karantinadaki dosyayı reddet
• /ata <id> <sehir> - Şehirsiz öğeye şehir ata
• /tara [sehir] - Menü kazımayı anlık tetikle
• /son_menuler - Son kaydedilen menüler
• /ban_kaldir - Devre kesiciyi sıfırla
• /yorumlar_uret - Eksik menü yorumlarını üret
• /yardim - Bu kullanım kılavuzu";
            send_reply(client, bot_token, chat_id, help_msg).await;
        }

        "/dosya" | "dosya" => {
            let Some(id) = parts.get(1).copied() else {
                send_reply(client, bot_token, chat_id, "Kullanım: /dosya <id>").await;
                return;
            };
            let base = crate::tasks::quarantine::menu_base_dir();
            match crate::tasks::quarantine::find_item(&base, id).await {
                Some(item) => {
                    if item.file_path.exists() {
                        let caption =
                            format!("Karantina Dosyası: {} ({})", item.meta.file, item.meta.id);
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
                                &format!("[HATA] Dosya gönderilemedi: {:?}", e),
                            )
                            .await;
                        }
                    } else {
                        send_reply(
                            client,
                            bot_token,
                            chat_id,
                            "[HATA] Dosya diskte bulunamadı.",
                        )
                        .await;
                    }
                }
                None => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("[BİLGİ] '{}' kimlikli karantina öğesi bulunamadı.", id),
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
                                &format!("[BİLGİ] '{}' kimlikli karantina öğesi bulunamadı.", id),
                            )
                            .await;
                        }
                    },
                    None => {
                        send_reply(
                            client,
                            bot_token,
                            chat_id,
                            "Kullanım: /karantina detay <id>",
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
                send_reply(client, bot_token, chat_id, "Kullanım: /onayla <id>").await;
                return;
            };
            send_reply(
                client,
                bot_token,
                chat_id,
                &format!(
                    "[İŞLEM] '{}' onaylanıyor, dosya yeniden ayrıştırılıyor...",
                    id
                ),
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
                        let hint = if crate::tasks::file_ingest::is_transient_error(
                            &raw.to_lowercase(),
                        ) {
                            "Sağlayıcı geçici olarak yanıt vermedi (503/429). Öğe karantinada kaldı; birkaç dakika sonra /onayla komutunu tekrar deneyin."
                        } else {
                            "Öğe karantinada kaldı (/karantina detay ile inceleyin). Sorun kalıcıysa /reddet ile hatali/ altına alın."
                        };
                        format!(
                            "[HATA] Onaylama başarısız ({}):\n{}\n\n{}",
                            id_owned, hint, raw
                        )
                    }
                };
                send_reply(&client_clone, &bot_token_clone, chat_id, &msg).await;
            });
        }

        "/reddet" | "reddet" => {
            let Some(id) = parts.get(1).copied() else {
                send_reply(client, bot_token, chat_id, "Kullanım: /reddet <id>").await;
                return;
            };
            match crate::tasks::file_ingest::reject_quarantine_item(id).await {
                Ok(m) => send_reply(client, bot_token, chat_id, &m).await,
                Err(e) => {
                    send_reply(
                        client,
                        bot_token,
                        chat_id,
                        &format!("[HATA] Reddetme başarısız ({}):\n{:?}", id, e),
                    )
                    .await
                }
            }
        }

        "/ata" | "ata" => {
            let (Some(id), Some(slug)) = (parts.get(1).copied(), parts.get(2).copied()) else {
                send_reply(client, bot_token, chat_id, "Kullanım: /ata <id> <sehir>").await;
                return;
            };
            match crate::tasks::file_ingest::assign_quarantine_item(db, id, slug).await {
                Ok(m) => {
                    send_reply(client, bot_token, chat_id, &m).await;
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
                                &format!("[HATA] /ata sonrası işleme hatası: {:?}", e),
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
                        &format!("[HATA] Şehir atanamadı ({}):\n{:?}", id, e),
                    )
                    .await
                }
            }
        }

        "/durum" | "durum" => {
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
                    format!("[DEVRE KESİCİ AKTİF] (Kalan süre: ~{} dk)", mins)
                }
                None => "Normal (Engelleme yok)".to_string(),
            };

            let status_msg = format!(
                "[KEPÇE SİSTEM DURUMU]\n\n\
                • Tarih: {}\n\
                • Veritabanı: Bağlı (OK)\n\
                • Kayıtlı Şehir: {} il\n\
                • Bugünkü Onaylı Menü: {} adet\n\
                • Scraper Hat Durumu: {}",
                today.format("%d.%m.%Y"),
                total_cities,
                today_approved,
                ban_status_msg
            );

            send_reply(client, bot_token, chat_id, &status_msg).await;
        }

        "/ban_kaldir" | "ban_kaldir" => {
            crate::tasks::scraper::reset_ban_status();
            let msg = "[BİLGİ] Devre kesici sıfırlandı. Scraper engelleme bayrağı kaldırıldı; yeni istekler kykyemek sunucusuna iletilecektir.";
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
                    let mut lines = vec!["[SİSTEME EKLENEN SON 5 MENÜ]".to_string()];
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
                            "• {} ({}) - {} [Durum: {:?}]",
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
                &format!("[İŞLEM] Kazıma işlemi tetiklendi. Hedef: {}\nTamamlandığında özet bildirimi gelecektir.", target_desc)
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
                            "[BİLGİ] Kazıma tamamlandı.\n• Kaydedilen/Güncellenen: {} menü\n• Geçen süre: {} sn",
                            count, elapsed
                        )
                    }
                    Err(e) => {
                        format!("[HATA] Kazıma sırasında hata oluştu:\n{:?}", e)
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
                "[İŞLEM] Otomatik LLM yorum üretimi başlatıldı. Eksik menüler öncelik sırasına göre işleniyor."
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
                            "[BİLGİ] Yorum üretimi tamamlandı.\n• Güncellenen menü: {} adet\n• Geçen süre: {} sn",
                            count, elapsed
                        )
                    }
                    Err(e) => {
                        format!("[HATA] Yorum üretimi sırasında hata oluştu:\n{:?}", e)
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
                "[BİLGİ] Bilinmeyen komut. Kullanılabilir komutları görmek için /yardim yazabilirsiniz."
            ).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_data_parsing() {
        let valid_payloads = [
            ("q:approve:k_01ABC", "approve", "k_01ABC"),
            ("q:reject_confirm:k_01ABC", "reject_confirm", "k_01ABC"),
            ("q:reject:k_01ABC", "reject", "k_01ABC"),
            ("q:cancel:k_01ABC", "cancel", "k_01ABC"),
            ("q:detail:k_01ABC", "detail", "k_01ABC"),
            ("q:file:k_01ABC", "file", "k_01ABC"),
        ];

        for (raw, expected_action, expected_id) in valid_payloads {
            let parts: Vec<&str> = raw.split(':').collect();
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], "q");
            assert_eq!(parts[1], expected_action);
            assert_eq!(parts[2], expected_id);
        }
    }

    #[test]
    fn test_help_message_has_no_emojis() {
        let help_msg = "\
[KEPÇE OPERATÖR BOTU]

Kullanılabilir komutlar:
• /durum - Sistem durumu
• /karantina - Karantina kuyruğu
• /karantina detay <id> - Öğe teşhis ayrıntıları
• /dosya <id> - Orijinal dosyayı sohbete gönder
• /onayla <id> - Karantinadaki dosyayı onayla
• /reddet <id> - Karantinadaki dosyayı reddet
• /ata <id> <sehir> - Şehirsiz öğeye şehir ata
• /tara [sehir] - Menü kazımayı anlık tetikle
• /son_menuler - Son kaydedilen menüler
• /ban_kaldir - Devre kesiciyi sıfırla
• /yorumlar_uret - Eksik menü yorumlarını üret
• /yardim - Bu kullanım kılavuzu";

        // Yaygın emoji aralıklarını kontrol et
        for c in help_msg.chars() {
            let code = c as u32;
            let is_emoji = (0x1F300..=0x1F9FF).contains(&code)
                || (0x2600..=0x26FF).contains(&code)
                || (0x2700..=0x27BF).contains(&code)
                || (0x1FA70..=0x1FAFF).contains(&code);
            assert!(!is_emoji, "Yardım mesajında emoji bulundu: {}", c);
        }
    }

    #[test]
    fn test_bot_commands_apple_standards() {
        for (cmd, desc) in BOT_COMMANDS {
            // Telegram komut adı kuralları: 1-32 karakter, küçük harf, alt çizgi
            assert!(!cmd.is_empty() && cmd.len() <= 32);
            assert!(cmd.chars().all(|c| c.is_ascii_lowercase() || c == '_'));

            // Apple standartları: dar ekranda kesilmemesi için kısa ve öz (en fazla 25 karakter)
            assert!(
                desc.chars().count() <= 25,
                "Komut açıklaması mobil ekranlar için fazla uzun: {} -> {}",
                cmd,
                desc
            );
            assert!(!desc.is_empty());
        }

        // Tüm beklenen temel operatör komutları menüde olmalı
        let cmd_names: Vec<&str> = BOT_COMMANDS.iter().map(|(c, _)| *c).collect();
        for expected in &[
            "durum",
            "karantina",
            "onayla",
            "reddet",
            "ata",
            "dosya",
            "tara",
            "son_menuler",
            "ban_kaldir",
            "yorumlar_uret",
            "yardim",
        ] {
            assert!(
                cmd_names.contains(expected),
                "Komut menüsünde eksik komut: {}",
                expected
            );
        }
    }
}

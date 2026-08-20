// Kepçe API — Service: Bildirim Yönetimi ve Olay Dağıtımı
// ========================================================
//
// Uygulama içi bildirimlerin kullanıcı tercihlerine göre filtrelenerek
// oluşturulması ve iletilmesini yönetir.

use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;
use shared::entities::{prelude::*, notifications};

pub struct NotificationService;

impl NotificationService {
    /// Kullanıcı bildirim tercihlerini kontrol ederek veritabanına bildirim kaydeder.
    /// Eğer kullanıcının ilgili bildirim tercihi kapalıysa bildirim oluşturulmaz ve `Ok(false)` döner.
    pub async fn send_notification(
        db: &DatabaseConnection,
        user_id: Uuid,
        notif_type: &str,
        title: &str,
        message: &str,
        action_label: Option<&str>,
        action_href: Option<&str>,
    ) -> Result<bool, anyhow::Error> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await?;

        let user = match user {
            Some(u) => u,
            None => return Ok(false), // Kullanıcı bulunamadıysa atla
        };

        // Kullanıcı tercihine göre filtreleme
        let should_send = match notif_type {
            "reply" | "comment" => user.notif_replies,
            "interaction" | "like" | "favorite" => user.notif_interactions,
            "system" | "moderation" | "announcement" | "achievement" => user.notif_system,
            _ => true,
        };

        if !should_send {
            tracing::debug!("Kullanıcı ({}) bildirim tercihini kapattığı için '{}' bildirimi atlandı.", user.username, notif_type);
            return Ok(false);
        }

        let notif = notifications::ActiveModel {
            user_id: Set(user_id),
            r#type: Set(notif_type.to_string()),
            title: Set(title.to_string()),
            message: Set(message.to_string()),
            action_label: Set(action_label.map(|s| s.to_string())),
            action_href: Set(action_href.map(|s| s.to_string())),
            is_read: Set(Some(false)),
            created_at: Set(Some(Utc::now().into())),
            ..Default::default()
        };

        notif.insert(db).await?;
        tracing::info!("Kullanıcıya ({}) yeni bildirim gönderildi: [{}] {}", user.username, notif_type, title);

        // Kullanıcının kayıtlı cihazlarına Web Push bildirimi fırlat
        let push_payload = crate::services::push::PushPayload {
            title: title.to_string(),
            body: message.to_string(),
            icon: Some("/icons/icon-192.png".to_string()),
            badge: Some("/icons/badge-72.png".to_string()),
            tag: Some(format!("notif-{}", notif_type)),
            url: action_href.map(|s| s.to_string()),
        };
        let _ = crate::services::push::PushService::send_to_user(db, user_id, &push_payload).await;

        Ok(true)
    }
}

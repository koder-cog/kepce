// Kepçe API — Routes: Web Push Abonelik ve VAPID Endpoint'leri
// ============================================================

use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use serde_json::{json, Value};
use shared::entities::{push_subscriptions, prelude::*};
use crate::error::AppError;
use crate::services::push::{PushService, PushPayload};

#[derive(Debug, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
    pub city_id: Option<i32>,
    pub notif_breakfast_enabled: Option<bool>,
    pub notif_breakfast_time: Option<String>,
    pub notif_dinner_enabled: Option<bool>,
    pub notif_dinner_time: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct TestPushRequest {
    pub endpoint: String,
}

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/vapid-public-key", get(get_vapid_key))
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe", post(unsubscribe))
        .route("/test", post(send_test_push))
}

/// GET /api/v1/public/push/vapid-public-key
async fn get_vapid_key() -> Json<Value> {
    Json(json!({
        "public_key": PushService::get_vapid_public_key()
    }))
}

/// POST /api/v1/public/push/subscribe
async fn subscribe(
    State(db): State<DatabaseConnection>,
    opt_user: Option<crate::extractors::auth::AuthenticatedUser>,
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<Value>, AppError> {
    if req.endpoint.trim().is_empty() || req.keys.p256dh.trim().is_empty() || req.keys.auth.trim().is_empty() {
        return Err(AppError::BadRequest("Geçersiz push abonelik bilgileri.".to_string()));
    }

    let user_id = opt_user.map(|u| u.id);

    let existing = PushSubscriptions::find()
        .filter(push_subscriptions::Column::Endpoint.eq(&req.endpoint))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(format!("DB Hatası: {:?}", e)))?;

    let now = Utc::now().into();

    if let Some(sub) = existing {
        let mut active: push_subscriptions::ActiveModel = sub.into();
        if let Some(uid) = user_id {
            active.user_id = Set(Some(uid));
        }
        if let Some(cid) = req.city_id {
            active.city_id = Set(Some(cid));
        }
        active.p256dh = Set(req.keys.p256dh);
        active.auth = Set(req.keys.auth);
        if let Some(b) = req.notif_breakfast_enabled {
            active.notif_breakfast_enabled = Set(b);
        }
        if let Some(ref t) = req.notif_breakfast_time {
            active.notif_breakfast_time = Set(t.clone());
        }
        if let Some(d) = req.notif_dinner_enabled {
            active.notif_dinner_enabled = Set(d);
        }
        if let Some(ref t) = req.notif_dinner_time {
            active.notif_dinner_time = Set(t.clone());
        }
        if let Some(ua) = req.user_agent {
            active.user_agent = Set(Some(ua));
        }
        active.updated_at = Set(now);
        active.update(&db).await.map_err(|e| AppError::Internal(format!("Abonelik güncellenemedi: {:?}", e)))?;
    } else {
        let new_sub = push_subscriptions::ActiveModel {
            user_id: Set(user_id),
            city_id: Set(req.city_id),
            endpoint: Set(req.endpoint),
            p256dh: Set(req.keys.p256dh),
            auth: Set(req.keys.auth),
            notif_breakfast_enabled: Set(req.notif_breakfast_enabled.unwrap_or(true)),
            notif_breakfast_time: Set(req.notif_breakfast_time.unwrap_or_else(|| "07:30".to_string())),
            notif_dinner_enabled: Set(req.notif_dinner_enabled.unwrap_or(true)),
            notif_dinner_time: Set(req.notif_dinner_time.unwrap_or_else(|| "17:00".to_string())),
            user_agent: Set(req.user_agent),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        new_sub.insert(&db).await.map_err(|e| AppError::Internal(format!("Abonelik kaydedilemedi: {:?}", e)))?;
    }

    Ok(Json(json!({
        "success": true,
        "message": "Bildirim aboneliği başarıyla kaydedildi."
    })))
}

/// POST /api/v1/public/push/unsubscribe
async fn unsubscribe(
    State(db): State<DatabaseConnection>,
    Json(req): Json<UnsubscribeRequest>,
) -> Result<Json<Value>, AppError> {
    let res = push_subscriptions::Entity::delete_many()
        .filter(push_subscriptions::Column::Endpoint.eq(&req.endpoint))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(format!("DB Hatası: {:?}", e)))?;

    Ok(Json(json!({
        "success": true,
        "deleted_count": res.rows_affected
    })))
}

/// POST /api/v1/public/push/test
async fn send_test_push(
    State(db): State<DatabaseConnection>,
    Json(req): Json<TestPushRequest>,
) -> Result<Json<Value>, AppError> {
    let sub = PushSubscriptions::find()
        .filter(push_subscriptions::Column::Endpoint.eq(&req.endpoint))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(format!("DB Hatası: {:?}", e)))?;

    let sub = match sub {
        Some(s) => s,
        None => return Err(AppError::NotFound("Bu cihaza ait bildirim aboneliği bulunamadı.".to_string())),
    };

    let payload = PushPayload {
        title: "Kepçe bildirimleri devrede".to_string(),
        body: "Harika! Öğün bildirimlerin başarıyla bağlandı. Menü açıklandığında ilk senin haberin olacak.".to_string(),
        icon: Some("/icons/icon-192.png".to_string()),
        badge: Some("/icons/badge-72.png".to_string()),
        tag: Some("kepce-test-notification".to_string()),
        url: Some("/".to_string()),
    };

    let sent = PushService::send_to_subscription(&db, &sub, &payload)
        .await
        .map_err(|e| AppError::Internal(format!("Push iletim hatası: {:?}", e)))?;

    if sent {
        Ok(Json(json!({
            "success": true,
            "message": "Test bildirimi cihazınıza başarıyla iletildi."
        })))
    } else {
        Err(AppError::Internal("Bildirim iletilemedi, abonelik geçersiz veya süresi dolmuş olabilir.".to_string()))
    }
}

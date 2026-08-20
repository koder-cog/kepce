// Kepçe API - Routes: Dinamik OG Image Endpoint'leri
// =======================================================

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sea_orm::*;
use shared::entities::{
    cities, comments, menu_dishes, menus, users, vote_reactions,
    sea_orm_active_enums::MealTypeEnum,
};
use chrono::{Datelike, Local, NaiveDate};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::Path as StdPath;
use crate::config::AppState;
use crate::services::og_image::{render_og_card, render_og_profile};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/menu/:id", get(get_menu_og))
        .route("/city/:city_slug", get(get_city_og))
        .route("/thread/:thread_id", get(get_thread_og))
        .route("/user/:username", get(get_user_og))
        .route("/page/:page_slug", get(get_page_og))
}

fn format_turkish_date(date: NaiveDate) -> String {
    let months = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
        "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
    ];
    let month_name = months.get(date.month0() as usize).unwrap_or(&"");
    format!("{} {} {}", date.day(), month_name, date.year())
}

fn format_meal_type(meal: &MealTypeEnum) -> &'static str {
    match meal {
        MealTypeEnum::Breakfast => "Kahvaltı",
        MealTypeEnum::Lunch => "Öğle Yemeği",
        MealTypeEnum::Dinner => "Akşam Yemeği",
    }
}

/// 1. Tekil Menü OG Kartı (/menu/:id)
async fn get_menu_og(
    State(state): State<AppState>,
    Path(menu_id): Path<i32>,
) -> Result<Response, StatusCode> {
    let menu = menus::Entity::find_by_id(menu_id)
        .find_also_related(cities::Entity)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let city_name = menu.1.map(|c| c.name).unwrap_or_else(|| "Menü".to_string());
    let menu_data = menu.0;
    
    let date_str = format_turkish_date(menu_data.serve_date);
    let meal_str = format_meal_type(&menu_data.meal_type);
    let sub1 = format!("{} · {}", date_str, meal_str);

    // Yemek çeşit sayısı
    let dish_count = menu_dishes::Entity::find()
        .filter(menu_dishes::Column::MenuId.eq(menu_id))
        .count(&state.db)
        .await
        .unwrap_or(0);

    let sub2 = match (menu_data.calorie_range_min, menu_data.calorie_range_max) {
        (Some(min), Some(max)) => format!("{} Çeşit Yemek · {}-{} kkal", dish_count, min, max),
        (Some(min), None) => format!("{} Çeşit Yemek · {} kkal", dish_count, min),
        (None, Some(max)) => format!("{} Çeşit Yemek · {} kkal", dish_count, max),
        (None, None) => format!("{} Çeşit Yemek", dish_count),
    };

    let png_bytes = render_og_card(
        Some(&city_name),
        &sub1,
        Some(&sub2),
        Some("Öğün"),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=86400, s-maxage=604800"),
        ],
        png_bytes,
    ).into_response())
}

/// 2. Günlük Şehir Menüsü OG Kartı (/:city_slug)
async fn get_city_og(
    State(state): State<AppState>,
    Path(city_slug): Path<String>,
) -> Result<Response, StatusCode> {
    let city = cities::Entity::find()
        .filter(cities::Column::Slug.eq(&city_slug))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let today = Local::now().date_naive();
    let date_str = format_turkish_date(today);

    // Şehrin bugünkü menülerini çek
    let day_menus = menus::Entity::find()
        .filter(menus::Column::CityId.eq(city.id))
        .filter(menus::Column::ServeDate.eq(today))
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut total_dishes = 0;
    let mut total_min = 0;
    let mut total_max = 0;
    let mut has_cal = false;

    for m in &day_menus {
        let count = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(m.id))
            .count(&state.db)
            .await
            .unwrap_or(0);
        total_dishes += count;

        if let (Some(min), Some(max)) = (m.calorie_range_min, m.calorie_range_max) {
            total_min += min;
            total_max += max;
            has_cal = true;
        }
    }

    let sub2 = if total_dishes > 0 && has_cal {
        format!("{} Çeşit Yemek · {}-{} kkal", total_dishes, total_min, total_max)
    } else if total_dishes > 0 {
        format!("{} Çeşit Yemek", total_dishes)
    } else {
        "Günlük Yemekhane Listesi".to_string()
    };

    let png_bytes = render_og_card(
        Some(&city.name),
        &date_str,
        Some(&sub2),
        Some("Günlük Menü"),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=86400, s-maxage=604800"),
        ],
        png_bytes,
    ).into_response())
}

/// 3. Tartışma Akışı OG Kartı (/menu/:id/:thread_id)
async fn get_thread_og(
    State(state): State<AppState>,
    Path(thread_id): Path<uuid::Uuid>,
) -> Result<Response, StatusCode> {
    let comment = comments::Entity::find_by_id(thread_id)
        .find_also_related(users::Entity)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let author_name = comment.1.map(|u| u.username).unwrap_or_else(|| "anonim".to_string());
    let comment_data = comment.0;

    // Menü bilgisi
    let (city_name, date_str) = if let Some(m) = menus::Entity::find_by_id(comment_data.menu_id)
        .find_also_related(cities::Entity)
        .one(&state.db)
        .await
        .unwrap_or(None)
    {
        let c_name = m.1.map(|c| c.name).unwrap_or_else(|| "Menü".to_string());
        let d_str = format_turkish_date(m.0.serve_date);
        (c_name, d_str)
    } else {
        ("Kepçe".to_string(), "Tartışma".to_string())
    };

    let title = format!("{} · {}", city_name, date_str);
    let sub1 = format!("@{} tarafından başlatıldı", author_name);

    // Yanıt sayısı
    let reply_count = comments::Entity::find()
        .filter(comments::Column::ParentId.eq(thread_id))
        .count(&state.db)
        .await
        .unwrap_or(0);

    // Oy sayısı
    let vote_count = vote_reactions::Entity::find()
        .filter(vote_reactions::Column::CommentId.eq(thread_id))
        .count(&state.db)
        .await
        .unwrap_or(0);

    let sub2 = format!("{} Yorum · {} Oy", reply_count + 1, vote_count);

    let png_bytes = render_og_card(
        Some(&title),
        &sub1,
        Some(&sub2),
        Some("Tartışma"),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=3600, s-maxage=86400"),
        ],
        png_bytes,
    ).into_response())
}

/// 4. Kullanıcı Profili OG Kartı (/biri/:username)
async fn get_user_og(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Response, StatusCode> {
    let clean_user = username.trim_start_matches('@');
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(clean_user))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let months = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
        "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
    ];
    let joined_str = if let Some(created) = user.created_at {
        let month_name = months.get(created.month0() as usize).unwrap_or(&"");
        format!("{} {}", month_name, created.year())
    } else {
        "2026".to_string()
    };

    let karma_sub = format!("{} Karma · {}", user.karma_score, joined_str);

    // Avatar resmi varsa yükle
    let mut avatar_b64 = None;
    if let Some(ref av_path) = user.avatar_url {
        let clean_path = av_path.trim_start_matches('/');
        let candidates = [
            format!("static/{}", clean_path),
            format!("webapp/static/{}", clean_path),
            clean_path.to_string(),
        ];
        for candidate in candidates {
            if StdPath::new(&candidate).exists() {
                if let Ok(bytes) = std::fs::read(&candidate) {
                    avatar_b64 = Some(BASE64.encode(&bytes));
                    break;
                }
            }
        }
    }

    let png_bytes = render_og_profile(
        &format!("@{}", user.username),
        &karma_sub,
        avatar_b64.as_deref(),
        "Kullanıcı Profili",
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=3600, s-maxage=86400"),
        ],
        png_bytes,
    ).into_response())
}

/// 5. Statik ve Genel Sayfalar OG Kartı (/page/:page_slug)
async fn get_page_og(
    State(state): State<AppState>,
    Path(page_slug): Path<String>,
) -> Result<Response, StatusCode> {
    let (title, sub1, badge, cache_control) = match page_slug.as_str() {
        "hakkinda" => (
            Some("Hakkında"),
            "Proje hakkında genel bilgiler",
            Some("Bilgi"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "sss" => (
            Some("Sıkça Sorulan Sorular"),
            "Site Hakkında Merak Edilenler",
            Some("Yardım"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "menu-gonder" => (
            Some("Menü Gönder"),
            "Yurdundaki Menüyü Sisteme Yükle",
            Some("Katkı"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "iletisim" => (
            Some("İletişim ve Künye"),
            "Öneri, Hata Bildirimi ve Yasal",
            Some("Yasal"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "rss" => (
            Some("RSS"),
            "Dedelere Otomasyon Desteği",
            Some("Veri Akışı"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "kullanim-kosullari" => (
            Some("Kullanım Koşulları"),
            "Topluluk Kuralları ve Hizmet Şartları",
            Some("Yasal"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "gizlilik-politikasi" => (
            Some("Gizlilik Politikası"),
            "KVKK Aydınlatma Metni ve İlgili Haklar",
            Some("Yasal"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "istatistikler" => (
            Some("İstatistikler"),
            "En Beğenilenler, Nefret Tablosu ve Jargon",
            Some("Topluluk Verileri"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "arsiv" => (
            Some("Menü Arşivi"),
            "Geçmiş Yemekhane Menüleri",
            Some("Geçmiş Kayıtlar"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "gelistirici" => (
            Some("Geliştirici Portalı"),
            "Proje Yönetimi ve API Anahtarları",
            Some("Geliştirici"),
            "public, max-age=86400, s-maxage=604800",
        ),
        "durum" => {
            // Canlı veritabanı / servis kontrolü
            let is_healthy = state.db.ping().await.is_ok();
            let sub = if is_healthy {
                "Tüm Sistemler Çalışıyor"
            } else {
                "Kısmi Yavaşlama Mevcut"
            };
            (
                Some("Sistem Durumu"),
                sub,
                Some("Servis"),
                "public, max-age=60",
            )
        }
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let png_bytes = render_og_card(
        title,
        sub1,
        None,
        badge,
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, cache_control),
        ],
        png_bytes,
    ).into_response())
}

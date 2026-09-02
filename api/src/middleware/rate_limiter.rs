// Kepçe API - Middleware: Rate Limiter
// =====================================
//
// IP, Cihaz (X-Client-ID) ve Kullanıcı bazlı çalışan in-memory rate limiter.
// Uygulama sunucusunu DoS ve brute-force saldırılarına karşı korur.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Extensions, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::config::AppState;
use crate::extractors::auth::Claims;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum RateLimitCategory {
    Login,
    Register,
    ForgotPassword,
    Passwordless,
    Ingestion,
    Vote,
    Comment,
    Assistant,
    SpikeArrest,
    General,
}

impl RateLimitCategory {
    pub fn rules(&self) -> (usize, Duration) {
        match self {
            Self::Login => (5, Duration::from_secs(60)),
            Self::Register => (3, Duration::from_secs(60)),
            Self::ForgotPassword => (3, Duration::from_secs(3600)),
            Self::Passwordless => (3, Duration::from_secs(300)), // 5 dakikada en fazla 3 istek
            Self::Ingestion => (10, Duration::from_secs(3600)),
            Self::Vote => (10, Duration::from_secs(60)),         // 1 dakikada maks 10 oy
            Self::Comment => (5, Duration::from_secs(60)),        // 1 dakikada maks 5 yorum
            Self::Assistant => (15, Duration::from_secs(60)),    // 1 dakikada maks 15 asistan isteği (15 RPM)
            Self::SpikeArrest => (10, Duration::from_secs(1)),   // DoS koruması: Saniyede maks 10 istek/IP
            Self::General => (240, Duration::from_secs(60)),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum RateLimitKey {
    User(Uuid),
    IpClient(IpAddr, String),
    Ip(IpAddr),
}

pub struct RateLimiter {
    requests: Mutex<HashMap<(RateLimitKey, RateLimitCategory), (f64, Instant)>>,
    check_count: AtomicUsize,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            check_count: AtomicUsize::new(0),
        }
    }

    pub fn check(&self, key: RateLimitKey, category: RateLimitCategory) -> Result<(), Duration> {
        let now = Instant::now();
        let (max_requests, duration) = category.rules();

        let mut map = self.requests.lock().unwrap();

        // 5000 kontrolde bir eski verileri temizle (bellek sızıntısı önleme)
        let count = self.check_count.fetch_add(1, Ordering::Relaxed);
        if count.is_multiple_of(5000) {
            map.retain(|(_, cat), (_, last_update)| {
                let (_, d) = cat.rules();
                now.duration_since(*last_update) < d
            });
        }

        let rate = max_requests as f64 / duration.as_secs_f64(); // saniyede yenilenen token miktarı
        
        // Eğer ilk kez geliyorsa, kova doludur
        let entry = map.entry((key, category)).or_insert_with(|| (max_requests as f64, now));
        
        let (tokens, last_update) = entry;
        
        // Geçen süreye göre yeni tokenları ekle
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        *tokens = (*tokens + elapsed * rate).min(max_requests as f64);
        *last_update = now;

        if *tokens < 1.0 {
            // İsteğe yetecek kadar token (1.0) yoksa bekleme süresini hesapla
            let wait_secs = (1.0 - *tokens) / rate;
            return Err(Duration::from_secs_f64(wait_secs.max(0.1))); // en az 100ms göster
        }

        *tokens -= 1.0;
        Ok(())
    }
}

/// Doğrudan bağlanan taraf (peer) güvenilir bir reverse proxy mi?
/// Sadece loopback ve RFC1918 özel ağ adresleri (Docker/nginx iç ağı) güvenilir
/// sayılır. İnternetten doğrudan gelen bir bağlantıda X-Real-IP / X-Forwarded-For
/// header'ları spoof edilebilir olduğundan dikkate alınmaz (SA-8).
pub fn is_trusted_proxy(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

pub fn get_client_ip(headers: &HeaderMap, extensions: &Extensions) -> Option<IpAddr> {
    let peer_ip = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip());

    // Forwarded header'lara YALNIZCA doğrudan bağlanan taraf güvenilir bir
    // proxy ise güven (nginx container'ı = özel IP). Aksi halde header spoofing'i
    // ile rate limit bypass mümkün olur.
    let trust_forwarded = peer_ip.map(|ip| is_trusted_proxy(&ip)).unwrap_or(false);

    if trust_forwarded {
        // 0. CF-Connecting-IP (Cloudflare gerçek istemci IP'si)
        if let Some(cf_ip) = headers.get("cf-connecting-ip") {
            if let Ok(cf_ip_str) = cf_ip.to_str() {
                if let Ok(ip) = cf_ip_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }

        // 1. X-Real-IP (Reverse proxy direkt ezer)
        if let Some(xri) = headers.get("x-real-ip") {
            if let Ok(xri_str) = xri.to_str() {
                if let Ok(ip) = xri_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }

        // 2. X-Forwarded-For (Proxy arkasında ise)
        if let Some(xff) = headers.get("x-forwarded-for") {
            if let Ok(xff_str) = xff.to_str() {
                // Nginx append yapar, bu yüzden en sondaki IP en güvenilir olanıdır.
                // İlk IP (next()) alınırsa spoofing yapılabilir.
                if let Some(last_ip) = xff_str.split(',').next_back() {
                    if let Ok(ip) = last_ip.trim().parse::<IpAddr>() {
                        return Some(ip);
                    }
                }
            }
        }
    }

    // 3. Soket Bağlantı Bilgisi Fallback
    if let Some(ip) = peer_ip {
        return Some(ip);
    }

    None
}

#[allow(clippy::result_large_err)]
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let method = req.method().clone();
    let path = req.uri().path();

    // Limit kategorisini belirle
    let category = if path.contains("/vote") {
        RateLimitCategory::Vote
    } else if path.starts_with("/api/v1/comments") && (method == axum::http::Method::POST || method == axum::http::Method::PUT) {
        RateLimitCategory::Comment
    } else if path.starts_with("/api/v1/auth/login") {
        RateLimitCategory::Login
    } else if path.starts_with("/api/v1/auth/register") {
        RateLimitCategory::Register
    } else if path.starts_with("/api/v1/auth/forgot-password") || path.starts_with("/api/v1/auth/reset-password") {
        RateLimitCategory::ForgotPassword
    } else if path.starts_with("/api/v1/auth/passwordless-login") {
        RateLimitCategory::Login
    } else if path.starts_with("/api/v1/auth/passwordless") {
        RateLimitCategory::Passwordless
    } else if path.starts_with("/api/v1/ingestion") {
        RateLimitCategory::Ingestion
    } else if path.starts_with("/api/v1/assistant") {
        RateLimitCategory::Assistant
    } else if path.starts_with("/api/v1/") {
        RateLimitCategory::General
    } else {
        // Genel API veya auth dışı (örn: static) dosyaları bypass et
        return Ok(next.run(req).await);
    };

    let headers = req.headers();

    // Ingestion endpointi X-API-Key ile kullanılıyorsa, 
    // kendi özel veritabanı rate-limit mantığını kullandığı için burayı bypass et.
    if category == RateLimitCategory::Ingestion && headers.contains_key("x-api-key") {
        return Ok(next.run(req).await);
    }

    let extensions = req.extensions();

    // Limit anahtarını belirle
    let mut key = None;

    // Öncelik 1: Giriş Yapmış Kullanıcı (JWT User ID)
    let mut token_opt = headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .map(|pair| pair.trim())
                .find(|pair| pair.starts_with("kepce_token="))
                .map(|pair| &pair["kepce_token=".len()..])
        });

    if token_opt.is_none() {
        token_opt = headers.get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .filter(|s| s.starts_with("Bearer "))
            .map(|s| &s[7..]);
    }

    if let Some(token) = token_opt {
        let mut validation = Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-web"]);
        if let Ok(token_data) = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &validation,
        ) {
            key = Some(RateLimitKey::User(token_data.claims.sub));
        }
    }

    let ip = get_client_ip(headers, extensions).unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

    // Öncelik 2: IP + Tarayıcı Cihaz Kimliği (X-Client-ID)
    if key.is_none() {
        if let Some(client_id_header) = headers.get("x-client-id")
            .and_then(|h| h.to_str().ok())
        {
            // Uzunluk üst sınırı: sınırsız uzunluktaki client-id'ler HashMap key'i
            // olarak bellek şişirmesin diye 64 karakterde kesilir.
            let client_id: String = client_id_header.chars().take(64).collect();
            key = Some(RateLimitKey::IpClient(ip, client_id));
        }
    }

    // Öncelik 3: Sadece IP Fallback
    let client_key = key.unwrap_or(RateLimitKey::Ip(ip));

    let reject_req = |wait_dur: Duration| -> Response {
        let retry_after = wait_dur.as_secs().max(1);
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header(axum::http::header::RETRY_AFTER, retry_after)
            .body(Body::from(format!("Too Many Requests. Lütfen {} saniye sonra tekrar deneyin.", retry_after)))
            .unwrap()
    };

    // 0. DoS Koruması: IP Bazlı SpikeArrest (Saniyede maks 10 istek/IP)
    if let Err(wait_dur) = state.rate_limiter.check(RateLimitKey::Ip(ip), RateLimitCategory::SpikeArrest) {
        return Err(reject_req(wait_dur));
    }

    // 1. Dar Limit Kontrolü (User, IpClient veya Ip)
    if let Err(wait_dur) = state.rate_limiter.check(client_key.clone(), category) {
        return Err(reject_req(wait_dur));
    }

    Ok(next.run(req).await)
}

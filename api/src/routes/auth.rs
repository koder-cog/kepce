// Kepçe API — Routes: Kimlik Doğrulama Endpoint'leri
// ===================================================
//
// İnce zarf. AuthService kullanır.
use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{State, Multipart, Query},
    Json,
    response::Redirect,
    http::{HeaderMap, header::SET_COOKIE},
};
use crate::config::Config;
use crate::extractors::auth::AuthenticatedUser;
use crate::services::auth::{AuthService, AuthError};
use crate::services::user::UserService;
use crate::dto::user::{RegisterRequestDto, LoginRequestDto, PasswordlessRequestDto, PasswordlessLoginDto, AuthResponseDto, UserProfileDto, UserRole};
use crate::error::AppError;
use crate::extractors::validated::ValidatedJson;
use sea_orm::{QueryFilter, ColumnTrait, EntityTrait};
use shared::entities::{prelude::Users, users};
use uuid::Uuid;
use rand::Rng;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/passwordless", post(request_passwordless))
        .route("/passwordless-login", post(passwordless_login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(get_me).put(update_me).delete(delete_me))
        .route("/me/favorites", get(get_favorites))
        .route("/me/favorites/toggle", post(toggle_favorite))
        .route("/me/pinned/toggle", post(toggle_pinned))
        .route("/me/sessions", get(get_sessions))
        .route("/me/sessions/:id", delete(revoke_session))
        .route("/me/notifications", get(get_notifications))
        .route("/me/notifications/mark-read", post(mark_notification_read))
        .route("/me/notifications/mark-all-read", post(mark_all_notifications_read))
        .route("/avatar", post(upload_avatar).delete(delete_avatar))
        .route("/verify", get(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/google/login", get(google_login))
        .route("/google/callback", get(google_callback))
        // Projects
        .route("/projects", get(get_projects).post(create_project))
        .route("/projects/:id", put(update_project).delete(delete_project))
        .route("/projects/usage", get(get_api_usage))
        // API Keys
        .route("/apikeys", get(get_api_keys).post(create_api_key))
        .route("/apikeys/:id", delete(revoke_api_key))
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::UserAlreadyExists => AppError::BadRequest("Bu e-posta veya kullanıcı adı sisteme zaten kayıtlı.".to_string()),
            AuthError::InvalidCredentials => AppError::Unauthorized("Giriş bilgileri hatalı. Lütfen bilgilerinizi kontrol edin.".to_string()),
            AuthError::AccountDisabled => AppError::Forbidden("Hesabınız askıya alınmış veya yasaklanmış. Destek ile iletişime geçin.".to_string()),
            AuthError::DatabaseError(e) => {
                tracing::error!("Database error in AuthService: {}", e);
                AppError::Internal("Veritabanına ulaşılamıyor. Lütfen daha sonra tekrar deneyin.".to_string())
            }
            AuthError::HashError(e) => {
                tracing::error!("Bcrypt error in AuthService: {}", e);
                AppError::Internal("Güvenlik modülü yanıt vermiyor. Lütfen daha sonra tekrar deneyin.".to_string())
            }
            AuthError::TokenError(msg) => {
                tracing::error!("JWT error in AuthService: {}", msg);
                AppError::Internal("Oturum anahtarı oluşturulamadı. Lütfen daha sonra tekrar deneyin.".to_string())
            }
        }
    }
}

async fn register(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequestDto>,
) -> Result<(HeaderMap, Json<AuthResponseDto>), AppError> {
    let ip_address = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts).map(|ip| ip.to_string())
    };
    let user_agent = req_headers.get(axum::http::header::USER_AGENT).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    
    let email_clone = payload.email.clone();
    let (access_token, refresh_token, user) = AuthService::register(&db, &config.jwt_secret, payload, ip_address, user_agent).await?;

    // E-posta doğrulama linki gönder (Eğer EmailService yapılandırılmışsa)
    if let Ok(verify_token) = AuthService::generate_verification_token(user.id, &config.jwt_secret) {
        let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
        
        // E-posta gönderimini bloklamaması için arka planda çalıştır
        tokio::spawn(async move {
            if let Err(e) = email_service.send_verification_email(&email_clone, &verify_token).await {
                tracing::error!("Failed to send verification email to {}: {:?}", email_clone, e);
            } else {
                tracing::info!("Verification email sent successfully to {}", email_clone);
            }
        });
    }

    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=900",
            access_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}",
            refresh_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_logged_in=true; Path=/; SameSite=Strict; {}",
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );

    Ok((headers, Json(AuthResponseDto { user })))
}

async fn login(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
    ValidatedJson(payload): ValidatedJson<LoginRequestDto>,
) -> Result<(HeaderMap, Json<AuthResponseDto>), AppError> {
    let remember = payload.remember.unwrap_or(false);
    
    let ip_address = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts).map(|ip| ip.to_string())
    };
    let user_agent = req_headers.get(axum::http::header::USER_AGENT).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    let (access_token, refresh_token, user) = AuthService::login(&db, &config.jwt_secret, payload, ip_address, user_agent).await?;

    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=900",
            access_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );

    let (refresh_cookie, logged_in_cookie) = if remember {
        (
            format!(
                "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=31536000",
                refresh_token,
                if secure { "Secure" } else { "" }
            ),
            format!(
                "kepce_logged_in=true; Path=/; SameSite=Strict; {}; Max-Age=31536000",
                if secure { "Secure" } else { "" }
            ),
        )
    } else {
        (
            format!(
                "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}",
                refresh_token,
                if secure { "Secure" } else { "" }
            ),
            format!(
                "kepce_logged_in=true; Path=/; SameSite=Strict; {}",
                if secure { "Secure" } else { "" }
            ),
        )
    };
    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());
    headers.append(SET_COOKIE, logged_in_cookie.parse().unwrap());

    Ok((headers, Json(AuthResponseDto { user })))
}

async fn refresh(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    let refresh_token = req_headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .map(|pair| pair.trim())
                .find(|pair| pair.starts_with("kepce_refresh_token="))
                .map(|pair| &pair["kepce_refresh_token=".len()..])
        })
        .ok_or_else(|| AppError::Unauthorized("Yenileme jetonu eksik.".to_string()))?;

    let mut validation = jsonwebtoken::Validation::default();
    validation.set_issuer(&["kepce"]);
    validation.set_audience(&["kepce-refresh"]);

    let token_data = jsonwebtoken::decode::<crate::extractors::auth::RefreshClaims>(
        refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    ).map_err(|e| {
        tracing::warn!("Refresh token validation failed: {:?}", e);
        AppError::Unauthorized("Geçersiz veya süresi dolmuş yenileme jetonu.".to_string())
    })?;

    let user_id = token_data.claims.sub;
    let remember = token_data.claims.rem;
    let jti = token_data.claims.jti;

    // 1. Veritabanında oturumu kontrol et
    let _session = shared::entities::prelude::UserSessions::find_by_id(jti)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Oturumunuz sonlandırılmış. Lütfen tekrar giriş yapın.".to_string()))?;

    // 2. Eski oturumu sil (Token Rotation)
    let _ = shared::entities::prelude::UserSessions::delete_by_id(jti)
        .exec(&db)
        .await;

    let user = Users::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Kullanıcı bulunamadı.".to_string()))?;

    // SA-3: Banlanan/askıya alınan kullanıcı refresh token ile yeni access token üretemez
    if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
        return Err(AppError::Forbidden("Hesabınız askıya alınmış veya yasaklanmış.".to_string()));
    }

    let role = match user.role {
        shared::entities::sea_orm_active_enums::UserRoleEnum::Admin => UserRole::Admin,
        shared::entities::sea_orm_active_enums::UserRoleEnum::SystemBot => UserRole::SystemBot,
        shared::entities::sea_orm_active_enums::UserRoleEnum::User => UserRole::User,
    };

    let ip_address = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts).map(|ip| ip.to_string())
    };
    let user_agent = req_headers.get(axum::http::header::USER_AGENT).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    let access_token = AuthService::generate_token(user.id, &user.username, &role, &config.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Failed to generate access token: {:?}", e)))?;
    let new_refresh_token = AuthService::generate_refresh_token(&db, user.id, &config.jwt_secret, remember, ip_address, user_agent).await
        .map_err(|e| AppError::Internal(format!("Failed to generate refresh token: {:?}", e)))?;

    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=900",
            access_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );

    let (refresh_cookie, logged_in_cookie) = if remember {
        (
            format!(
                "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=31536000",
                new_refresh_token,
                if secure { "Secure" } else { "" }
            ),
            format!(
                "kepce_logged_in=true; Path=/; SameSite=Strict; {}; Max-Age=31536000",
                if secure { "Secure" } else { "" }
            ),
        )
    } else {
        (
            format!(
                "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}",
                new_refresh_token,
                if secure { "Secure" } else { "" }
            ),
            format!(
                "kepce_logged_in=true; Path=/; SameSite=Strict; {}",
                if secure { "Secure" } else { "" }
            ),
        )
    };
    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());
    headers.append(SET_COOKIE, logged_in_cookie.parse().unwrap());

    Ok((headers, Json(serde_json::json!({ "status": "success" }))))
}

async fn logout(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    // SA-4: Logout artık sadece ilgili cihazdaki (tarayıcıdaki) oturumu siler.
    // Diğer cihazlardaki oturumlar (farklı jti'ler) açık kalmaya devam eder.
    if let Some(refresh_token) = req_headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .map(|pair| pair.trim())
                .find(|pair| pair.starts_with("kepce_refresh_token="))
                .map(|pair| pair["kepce_refresh_token=".len()..].to_string())
        })
    {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-refresh"]);
        if let Ok(td) = jsonwebtoken::decode::<crate::extractors::auth::RefreshClaims>(
            &refresh_token,
            &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        ) {
            let _ = shared::entities::prelude::UserSessions::delete_by_id(td.claims.jti)
                .exec(&db)
                .await;
        }
    }

    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token=; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=0",
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_refresh_token=; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=0",
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_logged_in=; Path=/; SameSite=Strict; {}; Max-Age=0",
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    Ok((headers, Json(serde_json::json!({ "status": "success" }))))
}

async fn get_me(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<UserProfileDto>, AppError> {
    let profile = UserService::get_user_profile_by_id(&db, user.id)
        .await
        .map_err(|e| {
            tracing::error!("UserService Error: {:?}", e);
            AppError::Internal("Kullanıcı profili alınamadı".to_string())
        })?;

    Ok(Json(profile))
}

async fn update_me(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::user::UpdateProfileDto>,
) -> Result<Json<crate::dto::user::UserProfileDto>, AppError> {
    let password_changed = payload.password.is_some();
    let profile = crate::services::user::UserService::update_user(&db, user.id, payload)
        .await
        .map_err(|e| match e {
            crate::services::auth::AuthError::UserAlreadyExists => AppError::BadRequest("Bu kullanıcı adı veya e-posta zaten kullanılıyor".to_string()),
            crate::services::auth::AuthError::InvalidCredentials => AppError::BadRequest("Mevcut şifre hatalı".to_string()),
            _ => AppError::Internal("Profil güncellenirken bir hata oluştu".to_string()),
        })?;

    // Şifre değiştiyse ve güvenlik e-postaları tercihi açıksa e-posta bildirimi gönder
    if password_changed && profile.email_security.unwrap_or(false) {
        if let Some(ref email) = profile.email {
            let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
            let to_email = email.clone();
            let username = profile.username.clone();
            tokio::spawn(async move {
                let _ = email_service.send_security_alert(
                    &to_email,
                    &username,
                    "Hesap Şifreniz Değiştirildi",
                    "Hesabınızın giriş şifresi ayarlar sayfası üzerinden başarıyla güncellendi. Eski oturumlarınız güvenlik gereği sonlandırıldı.",
                ).await;
            });
        }
    }

    Ok(Json(profile))
}

async fn delete_me(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::user::DeleteAccountDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    // SA-12: Hesap silme step-up auth ister — mevcut şifre doğrulanmadan silme yapılmaz.
    let db_user = Users::find_by_id(user.id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Kullanıcı bulunamadı.".to_string()))?;

    let hash_clone = db_user.password_hash.clone();
    let password = payload.password.clone();
    let is_valid = tokio::task::spawn_blocking(move || {
        bcrypt::verify(&password, &hash_clone).unwrap_or(false)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Blocking task failed: {}", e)))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Şifre hatalı. Hesap silme işlemi iptal edildi.".to_string()));
    }

    UserService::delete_user(&db, user.id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn get_sessions(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    user: AuthenticatedUser,
    req_headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    use sea_orm::QueryOrder;
    let mut current_jti = None;
    if let Some(refresh_token) = req_headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .map(|pair| pair.trim())
                .find(|pair| pair.starts_with("kepce_refresh_token="))
                .map(|pair| pair["kepce_refresh_token=".len()..].to_string())
        })
    {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-refresh"]);
        validation.validate_exp = false;
        
        if let Ok(td) = jsonwebtoken::decode::<crate::extractors::auth::RefreshClaims>(
            &refresh_token,
            &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        ) {
            current_jti = Some(td.claims.jti);
        }
    }

    let sessions = shared::entities::prelude::UserSessions::find()
        .filter(shared::entities::user_sessions::Column::UserId.eq(user.id))
        .order_by_desc(shared::entities::user_sessions::Column::LastUsedAt)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result: Vec<_> = sessions.into_iter().map(|s| {
        serde_json::json!({
            "id": s.id,
            "ip_address": s.ip_address,
            "user_agent": s.user_agent,
            "last_used_at": s.last_used_at,
            "created_at": s.created_at,
            "is_current": current_jti == Some(s.id)
        })
    }).collect();

    Ok(Json(result))
}

async fn revoke_session(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let session = shared::entities::prelude::UserSessions::find_by_id(session_id)
        .filter(shared::entities::user_sessions::Column::UserId.eq(user.id))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if session.is_some() {
        let _ = shared::entities::prelude::UserSessions::delete_by_id(session_id)
            .exec(&db)
            .await;
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn upload_avatar(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut file_data = None;
    let mut ext = "jpg".to_string();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            let content_type = field.content_type().unwrap_or_default().to_string();
            if content_type != "image/jpeg" && content_type != "image/png" && content_type != "image/webp" {
                return Err(AppError::BadRequest("Sadece JPEG, PNG ve WebP formatları desteklenmektedir.".to_string()));
            }

            ext = match content_type.as_str() {
                "image/png" => "png".to_string(),
                "image/webp" => "webp".to_string(),
                _ => "jpg".to_string(),
            };

            let mut data = Vec::new();
            while let Some(chunk) = field.chunk().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
                data.extend_from_slice(&chunk);
                if data.len() > 2 * 1024 * 1024 {
                    return Err(AppError::BadRequest("Avatar boyutu 2MB'tan büyük olamaz.".to_string()));
                }
            }

            let valid_signature = match content_type.as_str() {
                "image/png" => data.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]),
                "image/jpeg" => data.starts_with(&[0xFF, 0xD8]),
                "image/webp" => data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP",
                _ => false,
            };

            if !valid_signature {
                return Err(AppError::BadRequest("Dosya imzası doğrulanamadı. Geçersiz veya bozuk resim dosyası.".to_string()));
            }

            file_data = Some(data);
            break;
        }
    }

    let data = file_data.ok_or_else(|| AppError::BadRequest("File field missing".to_string()))?;

    // Create the avatars folder if it doesn't exist
    let _ = tokio::fs::create_dir_all("static/avatars").await;

    // Delete existing avatar files to avoid dangling files before saving new one
    let extensions = ["jpg", "png", "webp"];
    for old_ext in &extensions {
        let path = format!("static/avatars/{}.{}", user.id, old_ext);
        let _ = tokio::fs::remove_file(path).await;
    }

    let avatar_path = format!("static/avatars/{}.{}", user.id, ext);
    tokio::fs::write(&avatar_path, &data).await.map_err(|e| {
        tracing::error!("Failed to write avatar: {}", e);
        AppError::Internal("Avatar could not be saved".to_string())
    })?;

    let avatar_url = format!("/static/avatars/{}.{}", user.id, ext);
    UserService::update_avatar_url(&db, user.id, Some(avatar_url.clone()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update avatar url in db: {:?}", e);
            AppError::Internal("Database error updating avatar".to_string())
        })?;

    Ok(Json(serde_json::json!({ "avatar_url": avatar_url })))
}

async fn delete_avatar(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let extensions = ["jpg", "png", "webp"];
    for ext in &extensions {
        let path = format!("static/avatars/{}.{}", user.id, ext);
        let _ = tokio::fs::remove_file(path).await;
    }

    UserService::update_avatar_url(&db, user.id, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete avatar url in db: {:?}", e);
            AppError::Internal("Database error deleting avatar".to_string())
        })?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

async fn verify_email(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let already_verified = AuthService::verify_email_token(&db, &config.jwt_secret, &query.token).await?;
    if already_verified {
        Ok(Json(serde_json::json!({ "status": "already_verified" })))
    } else {
        Ok(Json(serde_json::json!({ "status": "success" })))
    }
}

async fn resend_verification(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check if user is already verified
    let db_user = shared::entities::prelude::Users::find_by_id(user.id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Kullanıcı bulunamadı.".to_string()))?;

    if db_user.is_verified {
        return Err(AppError::BadRequest("E-postanız zaten onaylı.".to_string()));
    }

    // Check cooldown (24 hours)
    {
        let mut map = crate::services::auth::get_resend_cooldowns().lock().unwrap();
        if map.len() > 1000 {
            let now = std::time::Instant::now();
            map.retain(|_, expires_at| *expires_at > now);
        }
        
        if let Some(expires_at) = map.get(&user.id) {
            if std::time::Instant::now() < *expires_at {
                return Err(AppError::TooManyRequests("Lütfen yeni bir onay e-postası istemeden önce 24 saat bekleyiniz.".to_string()));
            }
        }
        
        map.insert(user.id, std::time::Instant::now() + std::time::Duration::from_secs(24 * 3600));
    }

    // Send email
    let verify_token = AuthService::generate_verification_token(user.id, &config.jwt_secret)?;
    let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
    let email_clone = db_user.email;
    
    tokio::spawn(async move {
        if let Err(e) = email_service.send_verification_email(&email_clone, &verify_token).await {
            tracing::error!("Failed to resend verification email to {}: {:?}", email_clone, e);
        } else {
            tracing::info!("Verification email resent successfully to {}", email_clone);
        }
    });

    Ok(Json(serde_json::json!({ "status": "success", "message": "Onay e-postası tekrar gönderildi." })))
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct ForgotPasswordDto {
    #[validate(email)]
    pub email: String,
}

async fn forgot_password(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    ValidatedJson(payload): ValidatedJson<ForgotPasswordDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_opt = Users::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(user) = user_opt {
        // SA-2: Reset token ASLA loglanmaz. Token üretilir ancak e-posta servisi
        // (Faz 3A/3C — Resend) devreye girene kadar sadece bellekte tutulur.
        // Token türü artık `kepce-reset` (SA-5): access/verify token'larıyla karışmaz.
        let reset_token = AuthService::generate_reset_token(user.id, &config.jwt_secret)?;
        
        let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
        let email_clone = payload.email.clone();
        
        // E-posta gönderimini bloklamaması için arka planda çalıştır
        tokio::spawn(async move {
            if let Err(e) = email_service.send_reset_password_email(&email_clone, &reset_token).await {
                tracing::error!("Failed to send password reset email to {}: {:?}", email_clone, e);
            } else {
                tracing::info!("Password reset email sent successfully to {}", email_clone);
            }
        });
        
        tracing::info!("Password reset requested for user_id {}", user.id);
    } else {
        tracing::info!("Password reset requested for non-existent email {}", payload.email);
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct ResetPasswordDto {
    pub token: String,
    
    #[validate(length(min = 8, max = 128, message = "Şifre 8-128 karakter arasında olmalıdır"))]
    pub new_password: String,
}

async fn reset_password(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    ValidatedJson(payload): ValidatedJson<ResetPasswordDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = AuthService::reset_password(&db, &config.jwt_secret, &payload.token, &payload.new_password).await?;
    
    // Şifre sıfırlandıktan sonra kullanıcıya güvenlik e-postası gönder
    if user.email_security {
        let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
        let to_email = user.email.clone();
        let username = user.username.clone();
        tokio::spawn(async move {
            let _ = email_service.send_security_alert(
                &to_email,
                &username,
                "Hesap Şifreniz Sıfırlandı",
                "Hesabınızın giriş şifresi, şifre sıfırlama bağlantısı kullanılarak başarıyla yenilendi. Eski oturumlarınız güvenlik gereği sonlandırıldı.",
            ).await;
        });
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn google_login(
    State(config): State<std::sync::Arc<Config>>,
) -> Result<(HeaderMap, Redirect), AppError> {
    let client_id = config.google_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("Google Client ID tanımlı değil.".to_string()))?;
    let redirect_uri = config.google_redirect_uri.as_ref()
        .ok_or_else(|| AppError::Internal("Google Redirect URI tanımlı değil.".to_string()))?;
    
    // Güvenli rastgele state parametresi üret (OAuth CSRF koruması)
    let mut random_bytes = [0u8; 16];
    rand::thread_rng().fill(&mut random_bytes);
    let state_token: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope=openid%20email%20profile&state={}",
        client_id, redirect_uri, state_token
    );
    
    let secure_flag = if config.cookie_secure { "; Secure" } else { "" };
    let cookie_header = format!(
        "kepce_oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300{}",
        state_token, secure_flag
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie_header.parse().map_err(|e: http::header::InvalidHeaderValue| AppError::Internal(e.to_string()))?,
    );
    
    Ok((headers, Redirect::temporary(&auth_url)))
}

#[derive(serde::Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

async fn google_callback(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<(HeaderMap, Redirect), AppError> {
    let base_callback_url = format!("{}/oauth/callback", config.base_url);
    
    let secure_flag = if config.cookie_secure { "; Secure" } else { "" };
    let clear_state_cookie = format!(
        "kepce_oauth_state=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
        secure_flag
    );
    let mut clear_headers = HeaderMap::new();
    if let Ok(val) = clear_state_cookie.parse() {
        clear_headers.insert(SET_COOKIE, val);
    }

    if let Some(err_msg) = query.error {
        tracing::warn!("Google OAuth error from Google: {}", err_msg);
        return Ok((clear_headers, Redirect::temporary(&format!("{}?error={}", base_callback_url, err_msg))));
    }

    // State parametresi eşleşme kontrolü (CSRF Koruması)
    let cookie_header = req_headers.get(axum::http::header::COOKIE).and_then(|h| h.to_str().ok()).unwrap_or_default();
    let expected_state = cookie_header.split(';').find_map(|c| {
        let parts: Vec<&str> = c.trim().split('=').collect();
        if parts.len() == 2 && parts[0] == "kepce_oauth_state" {
            Some(parts[1])
        } else {
            None
        }
    });

    let state_matches = matches!((query.state.as_deref(), expected_state), (Some(qs), Some(es)) if !qs.is_empty() && qs == es);

    if !state_matches {
        tracing::warn!("Google OAuth state eşleşmedi veya state çerezi eksik");
        return Ok((clear_headers, Redirect::temporary(&format!("{}?error=invalid_oauth_state", base_callback_url))));
    }
    
    let code = match query.code {
        Some(c) => c,
        None => {
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=missing_code", base_callback_url))));
        }
    };
    
    let client_id = config.google_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("Google Client ID tanımlı değil.".to_string()))?;
    let client_secret = config.google_client_secret.as_ref()
        .ok_or_else(|| AppError::Internal("Google Client Secret tanımlı değil.".to_string()))?;
    let redirect_uri = config.google_redirect_uri.as_ref()
        .ok_or_else(|| AppError::Internal("Google Redirect URI tanımlı değil.".to_string()))?;
    
    // HTTP Client
    let client = reqwest::Client::new();
    
    // 1. Exchange code for access token
    let token_res = client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await;
        
    let token_response = match token_res {
        Ok(res) => {
            if !res.status().is_success() {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                tracing::error!("Google token exchange failed: {} - {}", status, body);
                return Ok((clear_headers, Redirect::temporary(&format!("{}?error=token_exchange_failed", base_callback_url))));
            }
            res
        }
        Err(e) => {
            tracing::error!("Network error during token exchange: {:?}", e);
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=network_error", base_callback_url))));
        }
    };
    
    #[derive(serde::Deserialize)]
    struct GoogleTokenResponse {
        access_token: String,
    }
    
    let token_data: GoogleTokenResponse = match token_response.json().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse Google token JSON: {:?}", e);
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=invalid_token_response", base_callback_url))));
        }
    };
    
    // 2. Fetch User Info
    let user_info_res = client.get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token_data.access_token)
        .send()
        .await;
        
    let user_info_response = match user_info_res {
        Ok(res) => {
            if !res.status().is_success() {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                tracing::error!("Google userinfo request failed: {} - {}", status, body);
                return Ok((clear_headers, Redirect::temporary(&format!("{}?error=userinfo_failed", base_callback_url))));
            }
            res
        }
        Err(e) => {
            tracing::error!("Network error during userinfo request: {:?}", e);
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=network_error", base_callback_url))));
        }
    };
    
    #[derive(serde::Deserialize)]
    struct GoogleUserInfo {
        email: String,
        email_verified: Option<bool>,
        name: Option<String>,
        picture: Option<String>,
    }
    
    let user_info: GoogleUserInfo = match user_info_response.json().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse Google userinfo JSON: {:?}", e);
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=invalid_userinfo_response", base_callback_url))));
        }
    };
    
    if user_info.email_verified != Some(true) {
        return Ok((clear_headers, Redirect::temporary(&format!("{}?error=email_not_verified", base_callback_url))));
    }
    
    let email = user_info.email.to_lowercase();
    
    // 3. Authenticate or Register User
    let ip_address = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts).map(|ip| ip.to_string())
    };
    let user_agent = req_headers.get(axum::http::header::USER_AGENT).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    
    let register_res = AuthService::register_or_login_oauth(
        &db, 
        &config.jwt_secret, 
        &email, 
        user_info.name.as_deref(), 
        user_info.picture.as_deref(),
        ip_address, 
        user_agent
    ).await;
    
    let (access_token, refresh_token, is_new) = match register_res {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("OAuth register/login failed: {:?}", e);
            return Ok((clear_headers, Redirect::temporary(&format!("{}?error=auth_failed", base_callback_url))));
        }
    };
    
    // 4. Set cookies and redirect (state çerezini de temizle)
    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(SET_COOKIE, clear_state_cookie.parse().unwrap());
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=900",
            access_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );
    
    let refresh_cookie = format!(
        "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}",
        refresh_token,
        if secure { "Secure" } else { "" }
    );
    let logged_in_cookie = format!(
        "kepce_logged_in=true; Path=/; SameSite=Strict; {}",
        if secure { "Secure" } else { "" }
    );
    
    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());
    headers.append(SET_COOKIE, logged_in_cookie.parse().unwrap());
    
    let redirect_url = format!("{}?is_new={}", base_callback_url, is_new);
    Ok((headers, Redirect::temporary(&redirect_url)))
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct DishToggleDto {
    pub dish_id: i32,
}

async fn get_favorites(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<i32>>, AppError> {
    let favs = UserService::get_favorites(&db, user.id).await?;
    Ok(Json(favs))
}

async fn toggle_favorite(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<DishToggleDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let added = UserService::toggle_favorite(&db, user.id, payload.dish_id).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "action": if added { "added" } else { "removed" }
    })))
}

async fn toggle_pinned(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<DishToggleDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let added = UserService::toggle_pinned(&db, user.id, payload.dish_id).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "action": if added { "added" } else { "removed" }
    })))
}

use crate::dto::user::{NotificationDto, NotificationMarkReadDto};
use shared::entities::prelude::Notifications;
use sea_orm::{QueryOrder, ActiveModelTrait};
use sea_orm::ActiveValue::Set;

async fn get_notifications(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<NotificationDto>>, AppError> {
    let notifs = Notifications::find()
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .order_by_desc(shared::entities::notifications::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let dtos = notifs.into_iter().map(|n| NotificationDto {
        id: n.id,
        r#type: n.r#type,
        title: n.title,
        message: n.message,
        is_read: n.is_read.unwrap_or(false),
        action_label: n.action_label,
        action_href: n.action_href,
        created_at: n.created_at,
    }).collect();

    Ok(Json(dtos))
}

async fn mark_notification_read(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<NotificationMarkReadDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let notif = Notifications::find_by_id(payload.id)
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(n) = notif {
        let mut active: shared::entities::notifications::ActiveModel = n.into();
        active.is_read = Set(Some(true));
        let _ = active.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn mark_all_notifications_read(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    shared::entities::notifications::Entity::update_many()
        .col_expr(shared::entities::notifications::Column::IsRead, sea_orm::sea_query::Expr::value(true))
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

// ===================================================
// Developer Portal Handlers
// ===================================================

use crate::services::developer::DeveloperService;
use crate::dto::developer::{CreateProjectDto, ProjectResponseDto, CreateApiKeyDto, ApiKeyResponseDto, ApiUsageDto};

async fn get_projects(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ProjectResponseDto>>, AppError> {
    let projs = DeveloperService::get_projects(&db, user.id).await?;
    Ok(Json(projs))
}

async fn create_project(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreateProjectDto>,
) -> Result<Json<ProjectResponseDto>, AppError> {
    let proj = DeveloperService::create_project(&db, user.id, payload.name).await?;
    Ok(Json(proj))
}

async fn update_project(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    axum::extract::Path(project_id): axum::extract::Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateProjectDto>,
) -> Result<Json<ProjectResponseDto>, AppError> {
    let proj = DeveloperService::update_project(&db, user.id, project_id, payload.name).await?;
    Ok(Json(proj))
}

async fn delete_project(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    axum::extract::Path(project_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    DeveloperService::delete_project(&db, user.id, project_id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

#[derive(serde::Deserialize)]
struct ApiUsageQuery {
    project_id: Option<String>,
    days: Option<i32>,
}

async fn get_api_usage(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Query(query): Query<ApiUsageQuery>,
) -> Result<Json<Vec<ApiUsageDto>>, AppError> {
    let project_id = query.project_id.unwrap_or_else(|| "all".to_string());
    let days = query.days.unwrap_or(28);
    let usage = DeveloperService::get_api_usage(&db, user.id, &project_id, days).await?;
    Ok(Json(usage))
}

async fn get_api_keys(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ApiKeyResponseDto>>, AppError> {
    let keys = DeveloperService::get_api_keys(&db, user.id).await?;
    Ok(Json(keys))
}

async fn create_api_key(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreateApiKeyDto>,
) -> Result<Json<ApiKeyResponseDto>, AppError> {
    let key = DeveloperService::create_api_key(&db, user.id, payload.project_id, payload.name).await?;
    Ok(Json(key))
}

async fn revoke_api_key(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    DeveloperService::revoke_api_key(&db, user.id, key_id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn request_passwordless(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    ValidatedJson(payload): ValidatedJson<PasswordlessRequestDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    // E-postaya sahip kullanıcıyı bul
    let user = Users::find()
        .filter(users::Column::Email.eq(payload.email.to_lowercase()))
        .one(&db)
        .await
        .map_err(|_| AppError::Internal("Veritabanı hatası".into()))?;

    // Güvenlik: Kullanıcı bulunamasa bile her zaman başarılı dönmeliyiz ki e-posta taraması (enumeration) yapılamasın.
    if let Some(user) = user {
        // Kullanıcı bulunduysa token üret ve e-posta at
        let token = AuthService::generate_passwordless_token(user.id, &config.jwt_secret)
            .map_err(|_| AppError::Internal("Token üretilemedi".into()))?;
        
        let email_service = crate::services::email::EmailService::new(config.resend_api_key.clone(), config.base_url.clone());
        
        // E-postayı arka planda asenkron gönder
        tokio::spawn(async move {
            if let Err(e) = email_service.send_passwordless_login(&user.email, &token).await {
                tracing::error!("Şifresiz giriş e-postası gönderilemedi: {:?}", e);
            }
        });
    }

    Ok(Json(serde_json::json!({
        "message": "Eğer e-posta adresi kayıtlıysa, şifresiz giriş bağlantısı gönderildi."
    })))
}

async fn passwordless_login(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
    ValidatedJson(payload): ValidatedJson<PasswordlessLoginDto>,
) -> Result<(HeaderMap, Json<AuthResponseDto>), AppError> {
    let ip_address = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts).map(|ip| ip.to_string())
    };
    let user_agent = req_headers.get(axum::http::header::USER_AGENT).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    let (access_token, refresh_token, user) = AuthService::passwordless_login(
        &db, 
        &config.jwt_secret, 
        &payload.token, 
        ip_address, 
        user_agent
    ).await?;

    let secure = config.cookie_secure;
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "kepce_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=900",
            access_token,
            if secure { "Secure" } else { "" }
        )
        .parse()
        .unwrap(),
    );

    let (refresh_cookie, logged_in_cookie) = (
        format!(
            "kepce_refresh_token={}; Path=/; HttpOnly; SameSite=Strict; {}; Max-Age=31536000",
            refresh_token,
            if secure { "Secure" } else { "" }
        ),
        format!(
            "kepce_logged_in=true; Path=/; SameSite=Strict; {}; Max-Age=31536000",
            if secure { "Secure" } else { "" }
        )
    );

    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());
    headers.append(SET_COOKIE, logged_in_cookie.parse().unwrap());

    Ok((headers, Json(AuthResponseDto { user })))
}

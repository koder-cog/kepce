//! Kimlik doğrulama, kayıt, oturum yönetimi ve kriptografik kurtarma anahtarı endpoint'leri.
use crate::config::Config;
use crate::dto::user::{
    AuthResponseDto, ConfirmRecoveryDto, LoginRequestDto, RegenerateRecoveryDto,
    RegisterRequestDto, ResetPasswordWithRecoveryDto, UserProfileDto, UserRole,
};
use crate::error::AppError;
use crate::extractors::auth::AuthenticatedUser;
use crate::extractors::validated::ValidatedJson;
use crate::services::auth::{AuthError, AuthService};
use crate::services::user::UserService;
use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, header::SET_COOKIE},
    routing::{delete, get, post, put},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use shared::entities::{prelude::Users, users};
use uuid::Uuid;

pub fn router() -> Router<crate::config::AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/confirm-recovery", post(confirm_recovery))
        .route("/regenerate-recovery", post(regenerate_recovery))
        .route("/password-reset", post(reset_password_with_recovery))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(get_me).put(update_me).delete(delete_me))
        .route("/me/favorites", get(get_favorites))
        .route("/me/favorites/toggle", post(toggle_favorite))
        .route("/me/pinned/toggle", post(toggle_pinned))
        .route("/me/sessions", get(get_sessions))
        .route("/me/sessions/:id", delete(revoke_session))
        .route(
            "/me/notifications",
            get(get_notifications).delete(delete_all_notifications),
        )
        .route("/me/notifications/:id", delete(delete_notification))
        .route("/me/notifications/mark-read", post(mark_notification_read))
        .route(
            "/me/notifications/mark-all-read",
            post(mark_all_notifications_read),
        )
        .route("/avatar", post(upload_avatar).delete(delete_avatar))
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
            AuthError::UserAlreadyExists => AppError::BadRequest(
                "Bu kullanıcı adı sisteme zaten kayıtlı.".to_string(),
            ),
            AuthError::InvalidCredentials => AppError::Unauthorized(
                "Giriş bilgileri hatalı. Lütfen bilgilerinizi kontrol edin.".to_string(),
            ),
            AuthError::InvalidUsername(msg) => AppError::BadRequest(msg),
            AuthError::EmailInputNotSupported => AppError::BadRequest(
                "Kepçe artık e-posta tutmamaktadır. Lütfen kullanıcı adınız ve şifrenizle giriş yapınız.".to_string(),
            ),
            AuthError::InvalidRecoveryKey => AppError::Unauthorized(
                "Kurtarma kodu hatalı veya geçersiz.".to_string(),
            ),
            AuthError::InvalidRecallBlock => AppError::BadRequest(
                "Doğrulama başarısız: Son blok hatalı girildi.".to_string(),
            ),
            AuthError::AccountDisabled => AppError::Forbidden(
                "Hesabınız askıya alınmış veya yasaklanmış. Destek ile iletişime geçin."
                    .to_string(),
            ),
            AuthError::DatabaseError(e) => {
                tracing::error!("Database error in AuthService: {}", e);
                AppError::Internal(
                    "Veritabanına ulaşılamıyor. Lütfen daha sonra tekrar deneyin.".to_string(),
                )
            }
            AuthError::HashError(e) => {
                tracing::error!("Bcrypt error in AuthService: {}", e);
                AppError::Internal(
                    "Güvenlik modülü yanıt vermiyor. Lütfen daha sonra tekrar deneyin.".to_string(),
                )
            }
            AuthError::TokenError(msg) => {
                tracing::error!("JWT error in AuthService: {}", msg);
                AppError::Internal(
                    "Oturum anahtarı oluşturulamadı. Lütfen daha sonra tekrar deneyin.".to_string(),
                )
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
    let client_ip = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts)
    };
    let subnet_hash = client_ip.as_ref().and_then(|ip| {
        let prefix = shared::domain::quality::ContentGuard::extract_subnet_prefix(&ip.to_string());
        crate::core::crypto::compute_subnet_hash(&prefix, &config.subnet_blind_key).ok()
    });

    let (access_token, refresh_token, user, recovery_key) = AuthService::register(
        &db,
        &config.jwt_secret,
        payload,
        subnet_hash,
    )
    .await?;

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

    Ok((
        headers,
        Json(AuthResponseDto {
            user,
            token: Some(access_token),
            recovery_key: Some(recovery_key),
        }),
    ))
}

async fn login(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
    ValidatedJson(payload): ValidatedJson<LoginRequestDto>,
) -> Result<(HeaderMap, Json<AuthResponseDto>), AppError> {
    let remember = payload.remember.unwrap_or(false);

    let client_ip = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts)
    };
    let subnet_hash = client_ip.as_ref().and_then(|ip| {
        let prefix = shared::domain::quality::ContentGuard::extract_subnet_prefix(&ip.to_string());
        crate::core::crypto::compute_subnet_hash(&prefix, &config.subnet_blind_key).ok()
    });

    let (access_token, refresh_token, user) = AuthService::login(
        &db,
        &config.jwt_secret,
        payload,
        subnet_hash,
    )
    .await?;

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

    Ok((
        headers,
        Json(AuthResponseDto {
            user,
            token: Some(access_token),
            recovery_key: None,
        }),
    ))
}

async fn refresh(
    State(db): State<sea_orm::DatabaseConnection>,
    State(config): State<std::sync::Arc<Config>>,
    req_headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    let refresh_token = req_headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
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
    )
    .map_err(|e| {
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
        .ok_or_else(|| {
            AppError::Unauthorized(
                "Oturumunuz sonlandırılmış. Lütfen tekrar giriş yapın.".to_string(),
            )
        })?;

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
        return Err(AppError::Forbidden(
            "Hesabınız askıya alınmış veya yasaklanmış.".to_string(),
        ));
    }

    let role = match user.role {
        shared::entities::sea_orm_active_enums::UserRoleEnum::Admin => UserRole::Admin,
        shared::entities::sea_orm_active_enums::UserRoleEnum::SystemBot => UserRole::SystemBot,
        shared::entities::sea_orm_active_enums::UserRoleEnum::User => UserRole::User,
    };

    let client_ip = {
        let mut exts = axum::http::Extensions::new();
        if let Some(conn) = connect_info {
            exts.insert(conn);
        }
        crate::middleware::rate_limiter::get_client_ip(&req_headers, &exts)
    };
    let subnet_hash = client_ip.as_ref().and_then(|ip| {
        let prefix = shared::domain::quality::ContentGuard::extract_subnet_prefix(&ip.to_string());
        crate::core::crypto::compute_subnet_hash(&prefix, &config.subnet_blind_key).ok()
    });

    let access_token =
        AuthService::generate_token(user.id, &user.nickname, &role, &config.jwt_secret)
            .map_err(|e| AppError::Internal(format!("Failed to generate access token: {:?}", e)))?;
    let new_refresh_token = AuthService::generate_refresh_token(
        &db,
        user.id,
        &config.jwt_secret,
        remember,
        subnet_hash,
    )
    .await
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
    if let Some(refresh_token) = req_headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
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
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::user::UpdateProfileDto>,
) -> Result<Json<crate::dto::user::UserProfileDto>, AppError> {
    let profile = crate::services::user::UserService::update_user(
        &db,
        user.id,
        payload,
    )
    .await
    .map_err(|e| match e {
        crate::services::auth::AuthError::UserAlreadyExists => {
            AppError::BadRequest("Bu kullanıcı adı zaten kullanılıyor".to_string())
        }
        crate::services::auth::AuthError::InvalidCredentials => {
            AppError::BadRequest("Mevcut şifre hatalı".to_string())
        }
        crate::services::auth::AuthError::InvalidUsername(msg) => AppError::BadRequest(msg),
        _ => AppError::Internal("Profil güncellenirken bir hata oluştu".to_string()),
    })?;

    Ok(Json(profile))
}

async fn delete_me(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<crate::dto::user::DeleteAccountDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    // SA-12: Hesap silme step-up auth ister - mevcut şifre doğrulanmadan silme yapılmaz.
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
        return Err(AppError::Unauthorized(
            "Şifre hatalı. Hesap silme işlemi iptal edildi.".to_string(),
        ));
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
    if let Some(refresh_token) = req_headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
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

    let result: Vec<_> = sessions
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "subnet_hash": s.subnet_hash,
                "last_used_at": s.last_used_at,
                "created_at": s.created_at,
                "is_current": current_jti == Some(s.id)
            })
        })
        .collect();

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
    let mut image_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Form verisi okunamadı: {}", e))
    })? {
        if field.name() == Some("avatar") || field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Dosya okunamadı: {}", e))
            })?;
            if bytes.len() > 5 * 1024 * 1024 {
                return Err(AppError::BadRequest("Görsel boyutu en fazla 5MB olabilir".to_string()));
            }
            image_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let bytes = image_bytes.ok_or_else(|| AppError::BadRequest("Avatar dosyası bulunamadı".to_string()))?;

    // RAM üzerinde EXIF metadata soyma ve 256x256 WebP'ye indirgeme
    let processed_bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, AppError> {
        let img = image::load_from_memory(&bytes).map_err(|e| {
            AppError::BadRequest(format!("Geçersiz görsel formatı: {}", e))
        })?;

        let resized = img.resize_to_fill(256, 256, image::imageops::FilterType::Lanczos3);
        let mut out = std::io::Cursor::new(Vec::new());
        resized.write_to(&mut out, image::ImageFormat::WebP).map_err(|e| {
            AppError::Internal(format!("WebP dönüştürme hatası: {}", e))
        })?;
        Ok(out.into_inner())
    })
    .await
    .map_err(|e| AppError::Internal(format!("Blocking task failed: {}", e)))??;

    tokio::fs::create_dir_all("static/avatars").await.map_err(|e| {
        AppError::Internal(format!("Dizin oluşturulamadı: {}", e))
    })?;

    let file_path = format!("static/avatars/{}.webp", user.id);
    tokio::fs::write(&file_path, &processed_bytes).await.map_err(|e| {
        AppError::Internal(format!("Avatar kaydedilemedi: {}", e))
    })?;

    let avatar_url = format!("/static/avatars/{}.webp", user.id);
    let mut user_model: users::ActiveModel = users::Entity::find_by_id(user.id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Kullanıcı bulunamadı".to_string()))?
        .into();

    user_model.avatar_url = Set(Some(avatar_url.clone()));
    user_model.updated_at = Set(Some(chrono::Utc::now().into()));
    user_model.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "avatar_url": avatar_url
    })))
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

    let mut user_model: users::ActiveModel = users::Entity::find_by_id(user.id)
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Kullanıcı bulunamadı".to_string()))?
        .into();

    user_model.avatar_url = Set(None);
    user_model.updated_at = Set(Some(chrono::Utc::now().into()));
    user_model.update(&db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn confirm_recovery(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<ConfirmRecoveryDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let verified = AuthService::confirm_recovery(&db, user.id, &payload.last_block).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "is_verified": verified
    })))
}

async fn regenerate_recovery(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<RegenerateRecoveryDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let new_key = AuthService::regenerate_recovery_key(&db, user.id, &payload.password).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "recovery_key": new_key,
        "new_recovery_key": new_key
    })))
}

async fn reset_password_with_recovery(
    State(db): State<sea_orm::DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<ResetPasswordWithRecoveryDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let new_key = AuthService::reset_password_with_recovery_key(&db, payload).await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "recovery_key": new_key,
        "new_recovery_key": new_key
    })))
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
use sea_orm::QueryOrder;
use shared::entities::prelude::Notifications;

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

    let dtos = notifs
        .into_iter()
        .map(|n| NotificationDto {
            id: n.id,
            r#type: n.r#type,
            title: n.title,
            message: n.message,
            is_read: n.is_read.unwrap_or(false),
            action_label: n.action_label,
            action_href: n.action_href,
            created_at: n.created_at,
        })
        .collect();

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
        let _ = active
            .update(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn mark_all_notifications_read(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    shared::entities::notifications::Entity::update_many()
        .col_expr(
            shared::entities::notifications::Column::IsRead,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn delete_notification(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let notif = Notifications::find_by_id(id)
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .one(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(n) = notif {
        let active: shared::entities::notifications::ActiveModel = n.into();
        active
            .delete(&db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}

async fn delete_all_notifications(
    State(db): State<sea_orm::DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    shared::entities::notifications::Entity::delete_many()
        .filter(shared::entities::notifications::Column::UserId.eq(user.id))
        .exec(&db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

// Geliştirici portalı (Developer Portal) uç noktaları

use crate::dto::developer::{
    ApiKeyResponseDto, ApiUsageDto, CreateApiKeyDto, CreateProjectDto, ProjectResponseDto,
};
use crate::services::developer::DeveloperService;

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
    let key =
        DeveloperService::create_api_key(&db, user.id, payload.project_id, payload.name).await?;
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


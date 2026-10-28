use crate::dto::user::{LoginRequestDto, RegisterRequestDto, UserProfileDto, UserRole};
use crate::extractors::auth::Claims;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::*;
use shared::entities::{prelude::*, users};
use uuid::Uuid;

/// AuthService için özel Hata Tipleri
#[derive(Debug)]
pub enum AuthError {
    UserAlreadyExists,
    InvalidCredentials,
    EmailInputNotSupported,
    InvalidRecoveryKey,
    InvalidRecallBlock,
    InvalidUsername(String),
    AccountDisabled,
    DatabaseError(DbErr),
    HashError(bcrypt::BcryptError),
    TokenError(String),
}

pub struct AuthService;

impl AuthService {
    pub fn is_reserved_username(name: &str) -> bool {
        let lower = name.trim().to_lowercase();
        matches!(
            lower.as_str(),
            "silinmis"
                | "silinmiş"
                | "deleted"
                | "anonim"
                | "anonymous"
                | "admin"
                | "kepce"
                | "kepçe"
                | "moderator"
                | "moderasyon"
                | "destek"
                | "support"
                | "system"
                | "sistem"
                | "bot"
        )
    }

    pub fn validate_nickname(name: &str) -> Result<(), AuthError> {
        let trimmed = name.trim();
        let char_count = trimmed.chars().count();
        if !(3..=25).contains(&char_count) {
            return Err(AuthError::InvalidUsername(
                "Kullanıcı adı en az 3, en fazla 25 karakter olmalıdır.".to_string(),
            ));
        }
        if Self::is_reserved_username(trimmed) {
            return Err(AuthError::InvalidUsername(
                "Bu kullanıcı adı sistem tarafından rezerve edilmiştir ve kullanılamaz.".to_string(),
            ));
        }
        // Sadece küçük harf, rakam ve alt çizgi (/^[a-z0-9_]+$/)
        let is_valid = trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !is_valid {
            return Err(AuthError::InvalidUsername(
                "Kullanıcı adı yalnızca İngilizce küçük harf, rakam ve alt çizgi (_) içerebilir."
                    .to_string(),
            ));
        }
        if trimmed.starts_with("05") || trimmed.starts_with("5") && trimmed.len() >= 10 {
            return Err(AuthError::InvalidUsername(
                "Kullanıcı adı telefon numarası içeremez.".to_string(),
            ));
        }
        Ok(())
    }

    /// JWT Erişim Token'ı Üretici (15 dakika)
    pub fn generate_token(
        user_id: Uuid,
        nickname: &str,
        role: &UserRole,
        jwt_secret: &str,
    ) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user_id,
            username: nickname.to_string(),
            role: role.clone(),
            exp: (Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-web".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenError(e.to_string()))
    }

    /// Uzun süreli Oturum Yenileme (Refresh) Token'ı
    pub async fn generate_refresh_token(
        db: &DatabaseConnection,
        user_id: Uuid,
        jwt_secret: &str,
        remember: bool,
        subnet_hash: Option<String>,
    ) -> Result<String, AuthError> {
        let jti = Uuid::new_v4();
        let days = if remember { 90 } else { 30 };
        let expires_at = Utc::now() + chrono::Duration::days(days);

        let claims = crate::extractors::auth::RefreshClaims {
            sub: user_id,
            exp: expires_at.timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-refresh".to_string(),
            rem: remember,
            jti,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenError(e.to_string()))?;

        let session = shared::entities::user_sessions::ActiveModel {
            id: Set(jti),
            user_id: Set(user_id),
            expires_at: Set(expires_at.into()),
            subnet_hash: Set(subnet_hash),
            last_used_at: Set(Some(Utc::now().into())),
            ..Default::default()
        };
        session.insert(db).await.map_err(AuthError::DatabaseError)?;

        Ok(token)
    }

    /// Kullanıcı Kaydı (Register)
    /// 130-bit Base32 kurtarma anahtarı üretir, hash'ler ve açık kodu yanıtta döner.
    pub async fn register(
        db: &DatabaseConnection,
        jwt_secret: &str,
        dto: RegisterRequestDto,
        subnet_hash: Option<String>,
    ) -> Result<(String, String, UserProfileDto, String), AuthError> {
        let clean_nickname = dto.nickname.trim().to_lowercase();
        Self::validate_nickname(&clean_nickname)?;

        // Rumuz benzersizlik kontrolü
        let exists = Users::find()
            .filter(
                sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::sea_query::Expr::col(users::Column::Nickname),
                ))
                .eq(&clean_nickname),
            )
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?;

        if exists.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        // 125-bit tek seferlik kurtarma anahtarı üretimi ve BLAKE3 keyed hashing
        let recovery_key = crate::core::crypto::generate_secure_recovery_key();
        let full_hash = crate::core::crypto::hash_recovery_key(&recovery_key);
        let last_block = crate::core::crypto::extract_last_block(&recovery_key).unwrap_or("");
        let recall_hash = crate::core::crypto::hash_recall_block(last_block);
        let recovery_key_hash = format!("{}:{}", full_hash, recall_hash);

        // Şifreyi hash'le (Bcrypt)
        let password_clone = dto.password.clone();
        let hashed_password =
            tokio::task::spawn_blocking(move || hash(&password_clone, DEFAULT_COST))
                .await
                .map_err(|e| {
                    AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e)))
                })?
                .map_err(AuthError::HashError)?;

        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            nickname: Set(clean_nickname),
            password_hash: Set(hashed_password),
            recovery_key_hash: Set(Some(recovery_key_hash)),
            role: Set(shared::entities::sea_orm_active_enums::UserRoleEnum::User),
            account_status: Set(shared::entities::sea_orm_active_enums::AccountStatusEnum::Active),
            karma_score: Set(0),
            is_verified: Set(false),
            avatar_url: Set(None),
            default_city_slug: Set(dto.default_city_slug),
            level: Set(1),
            level_progress: Set(0),
            opt_out_statistics: Set(false),
            notif_replies: Set(false),
            notif_interactions: Set(false),
            notif_system: Set(false),
            notif_breakfast_enabled: Set(false),
            notif_breakfast_time: Set("07:30".to_string()),
            notif_dinner_enabled: Set(false),
            notif_dinner_time: Set("17:30".to_string()),
            pinned_badges: Set(None),
            report_trust: Set(sea_orm::entity::prelude::Decimal::from(1)),
            token_version: Set(0),
            created_at: Set(Some(Utc::now().into())),
            updated_at: Set(Some(Utc::now().into())),
        };

        let inserted_user = match new_user.insert(db).await {
            Ok(u) => u,
            Err(e) => {
                if crate::utils::db::is_unique_constraint_violation(&e) {
                    return Err(AuthError::UserAlreadyExists);
                }
                return Err(AuthError::DatabaseError(e));
            }
        };

        let role = UserRole::User;
        let access_token =
            Self::generate_token(inserted_user.id, &inserted_user.nickname, &role, jwt_secret)?;
        let refresh_token =
            Self::generate_refresh_token(db, inserted_user.id, jwt_secret, false, subnet_hash)
                .await?;

        let user_profile = crate::services::user::UserService::build_profile(db, inserted_user, true)
            .await
            .map_err(|e| match e {
                crate::services::user::UserError::DatabaseError(db_err) => {
                    AuthError::DatabaseError(db_err)
                }
                _ => AuthError::InvalidCredentials,
            })?;

        Ok((access_token, refresh_token, user_profile, recovery_key))
    }

    /// Kullanıcı Girişi (Login)
    /// Kullanıcı e-posta girmeye çalışırsa özel bilgilendirme hatası döner.
    pub async fn login(
        db: &DatabaseConnection,
        jwt_secret: &str,
        dto: LoginRequestDto,
        subnet_hash: Option<String>,
    ) -> Result<(String, String, UserProfileDto), AuthError> {
        let identifier_trimmed = dto.identifier.trim();

        // E-posta girildiyse kullanıcıya açıklayıcı mesaj ver
        if identifier_trimmed.contains('@') {
            return Err(AuthError::EmailInputNotSupported);
        }

        let lower = identifier_trimmed.to_lowercase();
        let user = Users::find()
            .filter(
                sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::sea_query::Expr::col(users::Column::Nickname),
                ))
                .eq(&lower),
            )
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
            return Err(AuthError::AccountDisabled);
        }

        let password_clone = dto.password.clone();
        let hash_clone = user.password_hash.clone();
        let is_valid = tokio::task::spawn_blocking(move || verify(&password_clone, &hash_clone))
            .await
            .map_err(|e| {
                AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e)))
            })?
            .map_err(AuthError::HashError)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let role = crate::services::user::UserService::map_role(&user.role);
        let user_id = user.id;

        let profile = crate::services::user::UserService::build_profile(db, user, true)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        let access_token = Self::generate_token(user_id, &profile.nickname, &role, jwt_secret)?;
        let refresh_token = Self::generate_refresh_token(
            db,
            user_id,
            jwt_secret,
            dto.remember.unwrap_or(false),
            subnet_hash,
        )
        .await?;

        Ok((access_token, refresh_token, profile))
    }

    /// Kurtarma Kodu Aktif Hatırlama Teyidi (`POST /api/v1/auth/confirm-recovery`)
    /// Kullanıcının girdiği son 5 karakteri kriptografik olarak teyit eder ve hesabı `is_verified = true` yapar.
    pub async fn confirm_recovery(
        db: &DatabaseConnection,
        user_id: Uuid,
        last_block: &str,
    ) -> Result<bool, AuthError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.is_verified {
            return Ok(true);
        }

        let stored_raw = user
            .recovery_key_hash
            .as_deref()
            .ok_or(AuthError::InvalidCredentials)?;

        let full_hash = if stored_raw.contains(':') {
            let mut parts = stored_raw.split(':');
            let full = parts.next().unwrap_or(stored_raw);
            let expected_recall = parts.next().unwrap_or("");
            let actual_recall = crate::core::crypto::hash_recall_block(last_block);

            use subtle::ConstantTimeEq;
            if expected_recall.as_bytes().ct_eq(actual_recall.as_bytes()).unwrap_u8() != 1 {
                return Err(AuthError::InvalidRecallBlock);
            }
            full.to_string()
        } else {
            // Recall parçası bulunmuyorsa (önceden onaylı veya eski hesap)
            stored_raw.to_string()
        };

        let mut active: users::ActiveModel = user.into();
        active.is_verified = Set(true);
        active.recovery_key_hash = Set(Some(full_hash));
        active.updated_at = Set(Some(Utc::now().into()));
        active.update(db).await.map_err(AuthError::DatabaseError)?;

        Ok(true)
    }

    /// Onaysız hesaplar veya kodunu kaybedenler için yeni kurtarma kodu üretimi
    pub async fn regenerate_recovery_key(
        db: &DatabaseConnection,
        user_id: Uuid,
        password: &str,
    ) -> Result<String, AuthError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        let password_clone = password.to_string();
        let hash_clone = user.password_hash.clone();
        let is_valid = tokio::task::spawn_blocking(move || verify(&password_clone, &hash_clone))
            .await
            .map_err(|e| {
                AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e)))
            })?
            .map_err(AuthError::HashError)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let new_recovery_key = crate::core::crypto::generate_secure_recovery_key();
        let full_hash = crate::core::crypto::hash_recovery_key(&new_recovery_key);
        let last_block = crate::core::crypto::extract_last_block(&new_recovery_key).unwrap_or("");
        let recall_hash = crate::core::crypto::hash_recall_block(last_block);
        let stored_hash = format!("{}:{}", full_hash, recall_hash);

        let mut active: users::ActiveModel = user.into();
        active.recovery_key_hash = Set(Some(stored_hash));
        active.is_verified = Set(false);
        active.updated_at = Set(Some(Utc::now().into()));
        active.update(db).await.map_err(AuthError::DatabaseError)?;

        Ok(new_recovery_key)
    }

    /// Kurtarma Anahtarı ile Şifre Sıfırlama (`POST /api/v1/auth/password-reset`)
    /// 1. Kullanıcı adı ve kurtarma kodunu BLAKE3 ile doğrular.
    /// 2. Eski kodu anında geçersiz kılar ve SIFIRDAN YENİ bir 125-bit kod üretir.
    /// 3. Şifreyi günceller, hesabı onaysız duruma (`is_verified = false`) alır.
    /// 4. Kullanıcıya yeni kurtarma kodunu teslim eder.
    pub async fn reset_password_with_recovery_key(
        db: &DatabaseConnection,
        dto: crate::dto::user::ResetPasswordWithRecoveryDto,
    ) -> Result<String, AuthError> {
        let lower = dto.nickname.trim().to_lowercase();
        let user = Users::find()
            .filter(
                sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::sea_query::Expr::col(users::Column::Nickname),
                ))
                .eq(&lower),
            )
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        let stored_raw = user
            .recovery_key_hash
            .as_deref()
            .ok_or(AuthError::InvalidCredentials)?;

        let stored_full = stored_raw.split(':').next().unwrap_or(stored_raw);
        let incoming_hash = crate::core::crypto::hash_recovery_key(&dto.recovery_key);

        use subtle::ConstantTimeEq;
        if stored_full.as_bytes().ct_eq(incoming_hash.as_bytes()).unwrap_u8() != 1 {
            return Err(AuthError::InvalidCredentials);
        }

        // Yeni kurtarma anahtarı üret
        let new_recovery_key = crate::core::crypto::generate_secure_recovery_key();
        let full_hash = crate::core::crypto::hash_recovery_key(&new_recovery_key);
        let last_block = crate::core::crypto::extract_last_block(&new_recovery_key).unwrap_or("");
        let recall_hash = crate::core::crypto::hash_recall_block(last_block);
        let new_stored_hash = format!("{}:{}", full_hash, recall_hash);

        // Yeni şifreyi hash'le
        let new_password_clone = dto.new_password.clone();
        let hashed_password =
            tokio::task::spawn_blocking(move || hash(&new_password_clone, DEFAULT_COST))
                .await
                .map_err(|e| {
                    AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e)))
                })?
                .map_err(AuthError::HashError)?;

        let mut active: users::ActiveModel = user.clone().into();
        active.password_hash = Set(hashed_password);
        active.recovery_key_hash = Set(Some(new_stored_hash));
        active.is_verified = Set(false);
        active.token_version = Set(active.token_version.unwrap() + 1);
        active.updated_at = Set(Some(Utc::now().into()));
        active.update(db).await.map_err(AuthError::DatabaseError)?;

        // Önceki tüm aktif oturumları iptal et
        let _ = shared::entities::prelude::UserSessions::delete_many()
            .filter(shared::entities::user_sessions::Column::UserId.eq(user.id))
            .exec(db)
            .await;

        Ok(new_recovery_key)
    }

    /// Refresh Token ile Yeni Erişim Token'ı Alma
    pub async fn refresh(
        db: &DatabaseConnection,
        jwt_secret: &str,
        refresh_token: &str,
    ) -> Result<String, AuthError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-refresh"]);

        let token_data = jsonwebtoken::decode::<crate::extractors::auth::RefreshClaims>(
            refresh_token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AuthError::TokenError(e.to_string()))?;

        let session = shared::entities::prelude::UserSessions::find_by_id(token_data.claims.jti)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if session.user_id != token_data.claims.sub {
            return Err(AuthError::InvalidCredentials);
        }

        let user = Users::find_by_id(token_data.claims.sub)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
            return Err(AuthError::AccountDisabled);
        }

        let role = crate::services::user::UserService::map_role(&user.role);
        Self::generate_token(user.id, &user.nickname, &role, jwt_secret)
    }

    /// Oturum Kapatma (Logout)
    pub async fn logout(
        db: &DatabaseConnection,
        jwt_secret: &str,
        refresh_token: &str,
    ) -> Result<(), AuthError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-refresh"]);

        if let Ok(token_data) = jsonwebtoken::decode::<crate::extractors::auth::RefreshClaims>(
            refresh_token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ) {
            let _ = shared::entities::prelude::UserSessions::delete_by_id(token_data.claims.jti)
                .exec(db)
                .await;
        }

        Ok(())
    }
}

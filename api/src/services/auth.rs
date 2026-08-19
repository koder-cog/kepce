use sea_orm::*;
use shared::entities::{prelude::*, users};
use chrono::Utc;
use uuid::Uuid;
use rand::{distributions::Alphanumeric, Rng};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::extractors::auth::Claims;
use crate::dto::user::{LoginRequestDto, RegisterRequestDto, UserRole, UserProfileDto};
use sha2::{Sha256, Digest};
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use std::time::Instant;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn get_resend_cooldowns() -> &'static Mutex<HashMap<Uuid, Instant>> {
    static RESEND_COOLDOWNS: OnceLock<Mutex<HashMap<Uuid, Instant>>> = OnceLock::new();
    RESEND_COOLDOWNS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// AuthService için özel Hata Tipleri (Strongly Typed Errors)
#[derive(Debug)]
pub enum AuthError {
    UserAlreadyExists,
    InvalidCredentials,
    /// Hesap yasaklanmış veya askıya alınmış (account_status != Active)
    AccountDisabled,
    DatabaseError(DbErr),
    HashError(bcrypt::BcryptError),
    TokenError(String),
}

pub struct AuthService;

impl AuthService {
    /// JWT Token Üretici
    pub fn generate_token(user_id: Uuid, username: &str, role: &UserRole, jwt_secret: &str) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            role: role.clone(),
            exp: (Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-web".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).map_err(|e| AuthError::TokenError(e.to_string()))
    }

    pub async fn generate_refresh_token(
        db: &DatabaseConnection,
        user_id: Uuid,
        jwt_secret: &str,
        remember: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String, AuthError> {
        let jti = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::days(30);

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
        ).map_err(|e| AuthError::TokenError(e.to_string()))?;

        let session = shared::entities::user_sessions::ActiveModel {
            id: Set(jti),
            user_id: Set(user_id),
            expires_at: Set(expires_at.into()),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            last_used_at: Set(Some(Utc::now().into())),
            ..Default::default()
        };
        session.insert(db).await.map_err(AuthError::DatabaseError)?;

        Ok(token)
    }

    /// Kullanıcı Kaydı (Register)
    pub async fn register(
        db: &DatabaseConnection,
        jwt_secret: &str,
        dto: RegisterRequestDto,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, UserProfileDto), AuthError> {
        // 1. Username türetme mantığı
        let final_username = match dto.username {
            Some(u) => u.to_lowercase(), // Kullanıcı elle girdiyse onu kullan ve küçült
            None => {
                // Elle girmediyse e-postadan üret
                let base_name = dto.email.split('@').next().unwrap_or("user").to_lowercase();
                
                // Profesyonel yaklaşım: 'Ya tutarsa' yerine benzersizlik sağlanana kadar döngü kur.
                let mut final_username = base_name.clone();
                
                // Race condition koruması: Maksimum 5 deneme
                let max_retries = 5;
                let mut attempt = 0;
                
                loop {
                    let exists = Users::find()
                        .filter(users::Column::Username.eq(&final_username))
                        .one(db)
                        .await
                        .map_err(AuthError::DatabaseError)?;
                    
                    if exists.is_none() {
                        // Veritabanında yok, güvenle kullanabiliriz.
                        break final_username;
                    }
                    
                    attempt += 1;
                    if attempt >= max_retries {
                        return Err(AuthError::UserAlreadyExists);
                    }
                    
                    // İsim alınmışsa sonuna 5 haneli rastgele küçük harf/rakam ekleyip tekrar dene.
                    let random_suffix: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(5)
                        .map(char::from)
                        .collect();
                    
                    final_username = format!("{}_{}", base_name, random_suffix.to_lowercase());
                }
            }
        };

        // 2. Şifreyi Hashle (Bcrypt ile gerçek güvenlik)
        // DİKKAT: bcrypt hash işlemi CPU-bound'dur ve tokio iş parçacıklarını kilitler (thread starvation).
        // Bu yüzden spawn_blocking ile arka plan havuzuna paslıyoruz.
        let password_clone = dto.password.clone();
        let hashed_password = tokio::task::spawn_blocking(move || {
            hash(&password_clone, DEFAULT_COST)
        })
        .await
        .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?
        .map_err(AuthError::HashError)?;

        // 3. Veritabanı Modeline (ActiveModel) dönüştür
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(final_username),
            email: Set(dto.email),
            password_hash: Set(hashed_password),
            role: Set(shared::entities::sea_orm_active_enums::UserRoleEnum::User),
            karma_score: Set(0),
            is_verified: Set(false),
            default_city_slug: Set(dto.default_city_slug),
            email_security: Set(dto.email_security.unwrap_or(false)),
            notif_replies: Set(false),
            notif_interactions: Set(false),
            notif_system: Set(false),
            notif_breakfast_enabled: Set(false),
            notif_breakfast_time: Set("07:30".to_string()),
            notif_dinner_enabled: Set(false),
            notif_dinner_time: Set("17:30".to_string()),
            email_newsletter: Set(false),
            email_updates: Set(false),
            ..Default::default()
        };

        // 4. Veritabanına kaydet (Race Condition Önlemli Insert)
        let inserted_user = match new_user.insert(db).await {
            Ok(u) => u,
            Err(e) => {
                if crate::utils::db::is_unique_constraint_violation(&e) {
                    return Err(AuthError::UserAlreadyExists);
                }
                return Err(AuthError::DatabaseError(e));
            }
        };

        // 5. Dışarıya güvenli DTO dön (Şifre vb. gizli)
        let role = UserRole::User;
        let access_token = Self::generate_token(inserted_user.id, &inserted_user.username, &role, jwt_secret)?;
        let refresh_token = Self::generate_refresh_token(db, inserted_user.id, jwt_secret, false, ip_address, user_agent).await?;
        
        let user = crate::services::user::UserService::build_profile(db, inserted_user, true)
            .await
            .map_err(|e| match e {
                crate::services::user::UserError::DatabaseError(db_err) => AuthError::DatabaseError(db_err),
                crate::services::user::UserError::NotFound => AuthError::InvalidCredentials,
            })?;
        Ok((access_token, refresh_token, user))
    }

    /// Kullanıcı Girişi (Login)
    pub async fn login(
        db: &DatabaseConnection,
        jwt_secret: &str,
        dto: LoginRequestDto,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, UserProfileDto), AuthError> {
        // 1. E-posta mı yoksa Kullanıcı adı mı? (Her halükarda küçük harfle ara)
        let identifier_lower = dto.identifier.to_lowercase();
        let condition = if identifier_lower.contains('@') {
            users::Column::Email.eq(&identifier_lower)
        } else {
            users::Column::Username.eq(&identifier_lower)
        };

        // 2. Kullanıcıyı bul
        let user = Users::find()
            .filter(condition)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        // 2b. Hesap durumu kontrolü (ban/suspend enforce — SA-3)
        if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
            return Err(AuthError::AccountDisabled);
        }

        // 3. Şifreyi doğrula
        // DİKKAT: bcrypt verify işlemi CPU-bound'dur, tokio pool'u bloklamaması için arka plana atıyoruz.
        let password_clone = dto.password.clone();
        let hash_clone = user.password_hash.clone();
        let is_valid = tokio::task::spawn_blocking(move || {
            verify(&password_clone, &hash_clone)
        })
        .await
        .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?
        .map_err(AuthError::HashError)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // 4. Profil verisini çekmeden önce rolü modelden al (UserProfileDto.role artık Option)
        let role = crate::services::user::UserService::map_role(&user.role);
        let user_id = user.id;

        let profile = crate::services::user::UserService::build_profile(db, user, true)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        // 5. JWT Token üret
        let access_token = Self::generate_token(user_id, &profile.username, &role, jwt_secret)?;
        let refresh_token = Self::generate_refresh_token(db, user_id, jwt_secret, dto.remember.unwrap_or(false), ip_address, user_agent).await?;

        Ok((access_token, refresh_token, profile))
    }

    /// E-posta doğrulama token'ı (aud: kepce-verify — SA-5 token-type ayrımı)
    pub fn generate_verification_token(user_id: Uuid, jwt_secret: &str) -> Result<String, AuthError> {
        let claims = VerificationClaims {
            sub: user_id,
            exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-verify".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).map_err(|e| AuthError::TokenError(e.to_string()))
    }

    /// Şifre sıfırlama token'ı (aud: kepce-reset — doğrulama token'ından farklı tür)
    pub fn generate_reset_token(user_id: Uuid, jwt_secret: &str) -> Result<String, AuthError> {
        let claims = VerificationClaims {
            sub: user_id,
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-reset".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).map_err(|e| AuthError::TokenError(e.to_string()))
    }

    /// Şifresiz giriş token'ı (aud: kepce-passwordless — 15 dakika geçerli)
    pub fn generate_passwordless_token(user_id: Uuid, jwt_secret: &str) -> Result<String, AuthError> {
        let claims = VerificationClaims {
            sub: user_id,
            exp: (Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
            iss: "kepce".to_string(),
            aud: "kepce-passwordless".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).map_err(|e| AuthError::TokenError(e.to_string()))
    }

    /// Token'ın daha önce kullanılıp kullanılmadığını veritabanından sorgular
    pub async fn is_token_used(db: &DatabaseConnection, token: &str) -> Result<bool, DbErr> {
        let hash = hash_token(token);
        let stmt = Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT EXISTS(SELECT 1 FROM used_tokens WHERE token_hash = $1);",
            vec![hash.into()],
        );
        let row = db.query_one(stmt).await?;
        match row {
            Some(r) => r.try_get_by_index::<bool>(0),
            None => Ok(false),
        }
    }

    /// Token'ı kullanılmış olarak veritabanına kaydeder ve süresi dolmuşları temizler
    pub async fn mark_token_used(
        db: &DatabaseConnection,
        token: &str,
        token_type: &str,
        user_id: Uuid,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), DbErr> {
        let hash = hash_token(token);
        let stmt = Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            INSERT INTO used_tokens (token_hash, token_type, user_id, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (token_hash) DO NOTHING;
            "#,
            vec![
                hash.into(),
                token_type.into(),
                user_id.into(),
                expires_at.into(),
            ],
        );
        db.execute(stmt).await?;

        // Fırsatçı (opportunistic) temizlik: Süresi dolmuş token kayıtlarını temizle
        let cleanup_stmt = Statement::from_sql_and_values(
            db.get_database_backend(),
            "DELETE FROM used_tokens WHERE expires_at < NOW();",
            vec![],
        );
        let _ = db.execute(cleanup_stmt).await;

        Ok(())
    }

    pub async fn passwordless_login(
        db: &DatabaseConnection,
        jwt_secret: &str,
        token: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, UserProfileDto), AuthError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-passwordless"]);
        let token_data = jsonwebtoken::decode::<VerificationClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ).map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Token blacklist kontrolü (Kalıcı DB kontrolü)
        if Self::is_token_used(db, token).await.map_err(AuthError::DatabaseError)? {
            return Err(AuthError::TokenError("Bu giriş bağlantısı zaten kullanılmış. Lütfen yeni bir bağlantı isteyin.".to_string()));
        }

        let user = Users::find_by_id(token_data.claims.sub)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
            return Err(AuthError::AccountDisabled);
        }

        // Token'ı kullanılmış olarak kaydet (15 dk geçerlilik)
        Self::mark_token_used(
            db,
            token,
            "passwordless",
            user.id,
            Utc::now() + chrono::Duration::minutes(15),
        ).await.map_err(AuthError::DatabaseError)?;

        let role = crate::services::user::UserService::map_role(&user.role);
        let user_id = user.id;

        let profile = crate::services::user::UserService::build_profile(db, user, true)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        let access_token = Self::generate_token(user_id, &profile.username, &role, jwt_secret)?;
        // Magic link her zaman 'remember: true' gibi uzun süreli refresh token verebilir veya vermeyebilir. Biz false verelim.
        let refresh_token = Self::generate_refresh_token(db, user_id, jwt_secret, true, ip_address, user_agent).await?;

        Ok((access_token, refresh_token, profile))
    }

    pub async fn verify_email_token(
        db: &DatabaseConnection,
        jwt_secret: &str,
        token: &str,
    ) -> Result<bool, AuthError> {
        // SADECE kepce-verify türündeki token'lar kabul edilir; access/refresh
        // token'ları artık e-posta doğrulamak için kullanılamaz (SA-5).
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-verify"]);
        let token_data = jsonwebtoken::decode::<VerificationClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ).map_err(|e| AuthError::TokenError(e.to_string()))?;

        let user = Users::find_by_id(token_data.claims.sub)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        // Eğer kullanıcı zaten onaylıysa, hiçbir şey yapmadan "zaten onaylı" döndür
        if user.is_verified {
            return Ok(true);
        }

        // Token blacklist kontrolü (Kalıcı DB kontrolü - 24 saat)
        if Self::is_token_used(db, token).await.map_err(AuthError::DatabaseError)? {
            return Err(AuthError::TokenError("Bu doğrulama bağlantısı geçersiz veya daha önce kullanılmış.".to_string()));
        }

        // Token'ı kullanılmış olarak kaydet (24 saat geçerlilik)
        Self::mark_token_used(
            db,
            token,
            "verify",
            user.id,
            Utc::now() + chrono::Duration::hours(24),
        ).await.map_err(AuthError::DatabaseError)?;

        let mut active: users::ActiveModel = user.into();
        active.is_verified = Set(true);
        active.updated_at = Set(Some(Utc::now().into()));
        active.update(db).await.map_err(AuthError::DatabaseError)?;

        Ok(false)
    }

    pub async fn reset_password(
        db: &DatabaseConnection,
        jwt_secret: &str,
        token: &str,
        new_password: &str,
    ) -> Result<shared::entities::users::Model, AuthError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_issuer(&["kepce"]);
        validation.set_audience(&["kepce-reset"]);
        
        let token_data = jsonwebtoken::decode::<VerificationClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ).map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Token blacklist kontrolü (Kalıcı DB kontrolü - 1 saat)
        if Self::is_token_used(db, token).await.map_err(AuthError::DatabaseError)? {
            return Err(AuthError::TokenError("Bu şifre sıfırlama bağlantısı zaten kullanılmış veya geçersiz.".to_string()));
        }

        let user = Users::find_by_id(token_data.claims.sub)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        let password_clone = new_password.to_string();
        let hashed_password = tokio::task::spawn_blocking(move || {
            bcrypt::hash(&password_clone, bcrypt::DEFAULT_COST)
        })
        .await
        .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?
        .map_err(AuthError::HashError)?;

        // Token'ı kullanılmış olarak kaydet (1 saat geçerlilik)
        Self::mark_token_used(
            db,
            token,
            "reset",
            user.id,
            Utc::now() + chrono::Duration::hours(1),
        ).await.map_err(AuthError::DatabaseError)?;

        let mut active: users::ActiveModel = user.into();
        active.password_hash = Set(hashed_password);
        // SA-11: Şifre değiştiğinde token_version'ı artır (Kullanıcı tüm oturumlardan düşer, güvenlidir)
        active.token_version = Set(active.token_version.clone().unwrap() + 1);
        let updated_user = active.update(db).await.map_err(AuthError::DatabaseError)?;

        // Ayrıca aktif session'ları da sil (Refresh token'lar tamamen iptal)
        let _ = shared::entities::prelude::UserSessions::delete_many()
            .filter(shared::entities::user_sessions::Column::UserId.eq(token_data.claims.sub))
            .exec(db)
            .await;

        Ok(updated_user)
    }

    /// OAuth Giriş veya Kayıt Akışı
    pub async fn register_or_login_oauth(
        db: &DatabaseConnection,
        jwt_secret: &str,
        email: &str,
        _name: Option<&str>,
        picture: Option<&str>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, bool), AuthError> {
        let email_lower = email.to_lowercase();
        
        // 1. E-posta adresiyle kullanıcı var mı bak
        let existing_user = Users::find()
            .filter(users::Column::Email.eq(&email_lower))
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?;
            
        if let Some(user) = existing_user {
            // Kullanıcı hesabı aktif mi?
            if user.account_status != shared::entities::sea_orm_active_enums::AccountStatusEnum::Active {
                return Err(AuthError::AccountDisabled);
            }
            
            // Eğer avatar yoksa ve OAuth sağlayıcısı avatar verdiyse güncelle
            if user.avatar_url.is_none() && picture.is_some() {
                let mut active_user: users::ActiveModel = user.clone().into();
                active_user.avatar_url = Set(picture.map(|s| s.to_string()));
                let _ = active_user.update(db).await;
            }
            
            let role = match user.role {
                shared::entities::sea_orm_active_enums::UserRoleEnum::Admin => UserRole::Admin,
                shared::entities::sea_orm_active_enums::UserRoleEnum::SystemBot => UserRole::SystemBot,
                shared::entities::sea_orm_active_enums::UserRoleEnum::User => UserRole::User,
            };
            
            let access_token = Self::generate_token(user.id, &user.username, &role, jwt_secret)?;
            let refresh_token = Self::generate_refresh_token(db, user.id, jwt_secret, false, ip_address, user_agent).await?;
            
            return Ok((access_token, refresh_token, false));
        }
        
        // 2. Kullanıcı yoksa oluştur
        let base_name = email_lower.split('@').next().unwrap_or("user").to_lowercase();
        let mut final_username = base_name.clone();
        
        let max_retries = 5;
        let mut attempt = 0;
        
        let final_username = loop {
            let exists = Users::find()
                .filter(users::Column::Username.eq(&final_username))
                .one(db)
                .await
                .map_err(AuthError::DatabaseError)?;
                
            if exists.is_none() {
                break final_username;
            }
            
            attempt += 1;
            if attempt >= max_retries {
                final_username = format!("{}_{}", base_name, Uuid::new_v4().to_string()[..5].to_lowercase());
                break final_username;
            }
            
            let random_suffix: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect();
            final_username = format!("{}_{}", base_name, random_suffix.to_lowercase());
        };
        
        // Şifresiz girişler için rastgele geçici bir bcrypt parola hash'i üret
        let dummy_pass = Uuid::new_v4().to_string();
        let hashed_password = tokio::task::spawn_blocking(move || {
            hash(&dummy_pass, DEFAULT_COST)
        })
        .await
        .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?
        .map_err(AuthError::HashError)?;
        
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(final_username),
            email: Set(email_lower),
            password_hash: Set(hashed_password),
            avatar_url: Set(picture.map(|s| s.to_string())),
            role: Set(shared::entities::sea_orm_active_enums::UserRoleEnum::User),
            karma_score: Set(0),
            is_verified: Set(true), // Google doğruladığı için true
            email_security: Set(false),
            notif_replies: Set(false),
            notif_interactions: Set(false),
            notif_system: Set(false),
            notif_breakfast_enabled: Set(false),
            notif_breakfast_time: Set("07:30".to_string()),
            notif_dinner_enabled: Set(false),
            notif_dinner_time: Set("17:30".to_string()),
            email_newsletter: Set(false),
            email_updates: Set(false),
            ..Default::default()
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
        let access_token = Self::generate_token(inserted_user.id, &inserted_user.username, &role, jwt_secret)?;
        let refresh_token = Self::generate_refresh_token(db, inserted_user.id, jwt_secret, false, ip_address, user_agent).await?;
        
        Ok((access_token, refresh_token, true))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerificationClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}



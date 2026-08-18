// Kepçe API — Service: Kullanıcı Servisi
// ========================================
//
// Kullanıcı profili, karma, rozetler ve hesap yönetimi.
//
// Sorumlulukları:
//   1. Herkese açık profil bilgisi (nickname ile arama)
//   2. Karma hesaplama ve seviye belirleme
//   3. Rozet kontrolü ve atama
//   4. Favori yemekhaneler / sabitlenmiş yemekler
//   5. Kullanıcı engelleme (block/unblock)
//   6. Hesap güncelleme / silme

use sea_orm::*;
use chrono::Utc;
use uuid::Uuid;
use shared::entities::{
    prelude::*, users, user_badges, badges, sea_orm_active_enums::UserRoleEnum,
};
use crate::dto::user::{UserProfileDto, UserBadgeDto, UserRole};

#[derive(Debug)]
pub enum UserError {
    NotFound,
    DatabaseError(DbErr),
}

pub struct UserService;

impl UserService {
    /// Veritabanı rolünü DTO rolüne dönüştürür
    pub fn map_role(db_role: &UserRoleEnum) -> UserRole {
        match db_role {
            UserRoleEnum::User => UserRole::User,
            UserRoleEnum::Admin => UserRole::Admin,
            UserRoleEnum::SystemBot => UserRole::SystemBot,
        }
    }

    /// ID ile kullanıcı profilini ve kazandığı tüm rozetleri çeker
    pub async fn get_user_profile_by_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<UserProfileDto, UserError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?
            .ok_or(UserError::NotFound)?;
            
        Self::build_profile(db, user, true).await
    }

    /// Kullanıcı adıyla (username) profil arama ve çekme
    pub async fn get_user_profile_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<UserProfileDto, UserError> {
        let user = Users::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?
            .ok_or(UserError::NotFound)?;
            
        Self::build_profile(db, user, false).await
    }

    /// Ortak profil oluşturucu fonksiyon
    pub(crate) async fn build_profile(
        db: &DatabaseConnection,
        user: users::Model,
        include_private: bool,
    ) -> Result<UserProfileDto, UserError> {
        // Tüm rozetleri çekiyoruz (locked durumlarını belirlemek için)
        let all_db_badges = badges::Entity::find()
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;

        // Kullanıcının kazandığı rozetleri çekiyoruz
        let earned_user_badges = user_badges::Entity::find()
            .filter(user_badges::Column::UserId.eq(user.id))
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;
            
        let mut dto_badges = Vec::new();
        let mut badge_count = 0;
        
        for badge in all_db_badges {
            let user_badge_opt = earned_user_badges.iter().find(|ub| ub.badge_id == badge.id);
            let unlocked = user_badge_opt.is_some();
            if unlocked {
                badge_count += 1;
            }
            
            // Kategori tespiti (veya varsayılan)
            let category = match badge.name.to_lowercase().as_str() {
                n if n.contains("sadakat") || n.contains("level") || n.contains("seviye") => "sadakat",
                n if n.contains("sosyal") || n.contains("yorum") || n.contains("beğeni") => "sosyal",
                n if n.contains("denetim") || n.contains("rapor") || n.contains("şikayet") => "denetim",
                n if n.contains("veri") || n.contains("ekleme") || n.contains("yemekhane") => "veri",
                _ => "sadakat",
            }.to_string();
            
            let icon = match badge.name.to_lowercase().as_str() {
                n if n.contains("sadakat") || n.contains("seviye") => Some("heart".to_string()),
                n if n.contains("sosyal") || n.contains("yorum") => Some("messageSquare".to_string()),
                n if n.contains("denetim") || n.contains("rapor") => Some("shield".to_string()),
                n if n.contains("veri") || n.contains("ekleme") => Some("database".to_string()),
                _ => Some("starFilled".to_string()),
            };

            let awarded_at = user_badge_opt.and_then(|ub| ub.awarded_at.map(|dt| dt.into()));

            dto_badges.push(UserBadgeDto {
                name: badge.name,
                icon,
                icon_url: badge.icon_url,
                description: badge.description,
                category,
                awarded_at,
                unlocked,
                karma_reward: 10, // Varsayılan karma ödülü
                count: if unlocked { 1 } else { 0 },
                is_repeatable: false,
            });
        }

        // Seviye ve ilerleme hesaplama
        let karma = user.karma_score.max(0);
        let level = 1 + ((karma as f64 / 10.0).sqrt().floor() as i32);
        let start = (level - 1) * (level - 1) * 10;
        let end = level * level * 10;
        let current = karma - start;
        let target = end - start;
        let percent = if target > 0 {
            ((current as f64 / target as f64) * 100.0).round() as i32
        } else {
            0
        };

        let title = match level {
            1 => "Çırak",
            2 => "Kalfalık Yolunda",
            3 => "Kalfa",
            4 => "Usta Adayı",
            5 => "Usta",
            6 => "Şef",
            7 => "Gurme",
            8 => "Kepçe Ustası",
            9 => "Yemekhane Gurusu",
            _ => "Efsane Şef",
        }.to_string();

        let level_progress = crate::dto::user::LevelProgressDto {
            level,
            title,
            progress_percent: percent,
            karma_in_level: current,
            karma_for_next: target,
        };

        let is_admin = user.role == UserRoleEnum::Admin;
        let total_badges = dto_badges.len() as i32;
        let created_at_time = user.created_at.unwrap_or_else(|| Utc::now().into()).into();

        Ok(UserProfileDto {
            id: user.id,
            username: user.username.clone(),
            role: if include_private { Some(Self::map_role(&user.role)) } else { None },
            karma_score: user.karma_score,
            is_verified: user.is_verified,
            email: if include_private { Some(user.email) } else { None },
            joined_at: created_at_time,
            created_at: created_at_time,
            avatar_url: user.avatar_url,
            bio: user.bio,
            default_city_slug: user.default_city_slug,
            level,
            level_progress,
            google_id: if include_private { user.google_id } else { None },
            is_admin: if include_private { Some(is_admin) } else { None },
            badge_count,
            total_badges,
            badges: dto_badges,
            opt_out_statistics: user.opt_out_statistics,
            notif_replies: if include_private { Some(user.notif_replies) } else { None },
            notif_interactions: if include_private { Some(user.notif_interactions) } else { None },
            notif_system: if include_private { Some(user.notif_system) } else { None },
            email_newsletter: if include_private { Some(user.email_newsletter) } else { None },
            email_security: if include_private { Some(user.email_security) } else { None },
            email_updates: if include_private { Some(user.email_updates) } else { None },
        })
    }

    /// Profil dashboard istatistikleri (stub / minimal implementasyon)
    pub async fn get_dashboard_stats(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<crate::dto::user::UserDashboardStatsDto, UserError> {
        use shared::entities::{
            user_pinned_dishes, user_favorites, vote_reactions, comments, users as users_table, sea_orm_active_enums::ReactionTypeEnum
        };
        use sea_orm::{QuerySelect, QueryFilter, ColumnTrait, EntityTrait, QueryOrder, Statement, FromQueryResult};

        // Pinned meals
        let pinned = user_pinned_dishes::Entity::find()
            .filter(user_pinned_dishes::Column::UserId.eq(user_id))
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;
            
        let pinned_dish_ids: Vec<i32> = pinned.into_iter().map(|p| p.dish_id).collect();

        // Favorite meals
        let favs = user_favorites::Entity::find()
            .filter(user_favorites::Column::UserId.eq(user_id))
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;

        let fav_dish_ids: Vec<i32> = favs.into_iter().map(|f| f.dish_id).collect();

        let mut all_dish_ids = pinned_dish_ids.clone();
        for id in &fav_dish_ids {
            if !all_dish_ids.contains(id) {
                all_dish_ids.push(*id);
            }
        }

        let mut pinned_meals = Vec::new();
        let mut favorite_meals = Vec::new();

        if !all_dish_ids.is_empty() {
            let dishes_list = shared::entities::dishes::Entity::find()
                .filter(shared::entities::dishes::Column::Id.is_in(all_dish_ids))
                .all(db)
                .await
                .map_err(UserError::DatabaseError)?;

            let dish_map: std::collections::HashMap<i32, String> = dishes_list
                .into_iter()
                .map(|d| (d.id, d.name))
                .collect();

            pinned_meals = pinned_dish_ids
                .into_iter()
                .filter_map(|id| {
                    dish_map.get(&id).map(|name| crate::dto::user::SimpleDishDto {
                        dish_id: id,
                        name: name.clone(),
                    })
                })
                .collect();

            favorite_meals = fav_dish_ids
                .into_iter()
                .filter_map(|id| {
                    dish_map.get(&id).map(|name| crate::dto::user::SimpleDishDto {
                        dish_id: id,
                        name: name.clone(),
                    })
                })
                .collect();
        }

        // Favorite comments
        let recent_votes = vote_reactions::Entity::find()
            .filter(vote_reactions::Column::UserId.eq(user_id))
            .filter(vote_reactions::Column::ReactionType.eq(ReactionTypeEnum::Upvote))
            .order_by_desc(vote_reactions::Column::CreatedAt)
            .limit(10)
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;

        let comment_ids: Vec<Uuid> = recent_votes.into_iter().map(|v| v.comment_id).collect();
        let mut favorite_comments = Vec::new();
        if !comment_ids.is_empty() {
            let raw_comments = comments::Entity::find()
                .filter(comments::Column::Id.is_in(comment_ids))
                .find_also_related(users_table::Entity)
                .all(db)
                .await
                .map_err(UserError::DatabaseError)?;

            if let Ok(enriched) = crate::services::comment::CommentService::enrich_comments(db, raw_comments, Some(user_id), &[]).await {
                favorite_comments = enriched.into_iter().map(|(dto, _)| dto).collect();
            }
        }

        // Favorite authors
        #[derive(Debug, FromQueryResult)]
        struct AuthorCount {
            username: String,
            favorite_count: i64,
        }
        
        let authors_res = AuthorCount::find_by_statement(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            SELECT u.username, COUNT(v.id) as favorite_count
            FROM vote_reactions v
            JOIN comments c ON c.id = v.comment_id
            JOIN users u ON u.id = c.user_id
            WHERE v.user_id = $1 AND v.reaction_type = 'upvote'
            GROUP BY u.username
            ORDER BY favorite_count DESC
            LIMIT 5
            "#,
            vec![user_id.into()]
        )).all(db).await.map_err(UserError::DatabaseError)?;

        let favorite_authors = authors_res.into_iter().map(|r| crate::dto::user::FavoriteAuthorDto {
            username: r.username,
            favorite_count: r.favorite_count as i32,
        }).collect();

        Ok(crate::dto::user::UserDashboardStatsDto {
            favorite_meals,
            pinned_meals,
            favorite_comments,
            favorite_authors,
        })
    }

    /// Kullanıcı bilgilerini günceller
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        dto: crate::dto::user::UpdateProfileDto,
    ) -> Result<UserProfileDto, crate::services::auth::AuthError> {
        use crate::services::auth::AuthError;
        
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        let mut user_model: users::ActiveModel = user.clone().into();

        // SA-12: Kullanıcı adı, e-posta veya şifre değişikliği step-up auth ister.
        // (Eskiden kullanıcı adı şifresiz değiştirilebiliyordu.)
        let needs_password_check = dto.username.is_some() || dto.email.is_some() || dto.password.is_some();
        if needs_password_check {
            let current_password = dto.current_password.clone().ok_or(AuthError::InvalidCredentials)?;
            let hash_clone = user.password_hash.clone();
            let is_valid = tokio::task::spawn_blocking(move || {
                bcrypt::verify(&current_password, &hash_clone).unwrap_or(false)
            })
            .await
            .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?;

            if !is_valid {
                return Err(AuthError::InvalidCredentials);
            }
        }

        if let Some(username) = dto.username {
            // Check if username exists
            let exists = Users::find()
                .filter(users::Column::Username.eq(&username))
                .filter(users::Column::Id.ne(user_id))
                .one(db)
                .await
                .map_err(AuthError::DatabaseError)?;
            if exists.is_some() {
                return Err(AuthError::UserAlreadyExists);
            }
            user_model.username = Set(username);
        }

        if let Some(ref email) = dto.email {
            // Check if email exists
            let exists = Users::find()
                .filter(users::Column::Email.eq(email))
                .filter(users::Column::Id.ne(user_id))
                .one(db)
                .await
                .map_err(AuthError::DatabaseError)?;
            if exists.is_some() {
                return Err(AuthError::UserAlreadyExists);
            }
            user_model.email = Set(email.clone());
        }

        if let Some(password) = dto.password {
            let hashed_password = tokio::task::spawn_blocking(move || {
                bcrypt::hash(&password, bcrypt::DEFAULT_COST)
            })
            .await
            .map_err(|e| AuthError::DatabaseError(DbErr::Custom(format!("Blocking task failed: {}", e))))?
            .map_err(AuthError::HashError)?;

            user_model.password_hash = Set(hashed_password);

            // SA-4: Şifre değişiminde tüm mevcut oturumları (refresh token'ları) geçersiz kıl
            user_model.token_version = Set(user.token_version + 1);
        }

        if let Some(bio) = dto.bio {
            let sanitized_bio = shared::services::content_guard::ContentGuard::sanitize_html(&bio);
            user_model.bio = Set(Some(sanitized_bio));
        }

        if let Some(default_city_slug) = dto.default_city_slug {
            user_model.default_city_slug = Set(Some(default_city_slug));
        }

        if let Some(opt_out) = dto.opt_out_statistics {
            user_model.opt_out_statistics = Set(opt_out);
        }

        if let Some(val) = dto.notif_replies {
            user_model.notif_replies = Set(val);
        }
        if let Some(val) = dto.notif_interactions {
            user_model.notif_interactions = Set(val);
        }
        if let Some(val) = dto.notif_system {
            user_model.notif_system = Set(val);
        }
        if let Some(val) = dto.email_newsletter {
            user_model.email_newsletter = Set(val);
        }
        if let Some(val) = dto.email_security {
            user_model.email_security = Set(val);
        }
        if let Some(val) = dto.email_updates {
            user_model.email_updates = Set(val);
        }

        user_model.updated_at = Set(Some(Utc::now().into()));
        let updated_user = user_model.update(db).await.map_err(AuthError::DatabaseError)?;

        Self::build_profile(db, updated_user, true).await.map_err(|_| AuthError::InvalidCredentials)
    }

    /// Kullanıcıyı siler
    pub async fn delete_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), UserError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?
            .ok_or(UserError::NotFound)?;
            
        user.delete(db).await.map_err(UserError::DatabaseError)?;
        Ok(())
    }

    /// Kullanıcının oturum sürümünü artırır — tüm refresh token'ları geçersiz kılar (SA-4).
    /// Şifre değişimi, ban/suspend ve logout sırasında çağrılır.
    pub async fn bump_token_version(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), UserError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?
            .ok_or(UserError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.token_version = Set(active.token_version.unwrap() + 1);
        active.update(db).await.map_err(UserError::DatabaseError)?;
        Ok(())
    }

    /// Avatar URL günceller
    pub async fn update_avatar_url(
        db: &DatabaseConnection,
        user_id: Uuid,
        avatar_url: Option<String>,
    ) -> Result<(), UserError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?
            .ok_or(UserError::NotFound)?;
            
        let mut active: users::ActiveModel = user.into();
        active.avatar_url = Set(avatar_url);
        active.updated_at = Set(Some(Utc::now().into()));
        active.update(db).await.map_err(UserError::DatabaseError)?;
        Ok(())
    }

    /// Favori yemekleri listeler
    pub async fn get_favorites(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<i32>, UserError> {
        use shared::entities::user_favorites;
        let favs = user_favorites::Entity::find()
            .filter(user_favorites::Column::UserId.eq(user_id))
            .all(db)
            .await
            .map_err(UserError::DatabaseError)?;
        Ok(favs.into_iter().map(|f| f.dish_id).collect())
    }

    /// Favori durumunu değiştirir (varsa siler, yoksa ekler)
    pub async fn toggle_favorite(
        db: &DatabaseConnection,
        user_id: Uuid,
        dish_id: i32,
    ) -> Result<bool, UserError> {
        use shared::entities::{user_favorites, user_pinned_dishes};
        let existing = user_favorites::Entity::find()
            .filter(user_favorites::Column::UserId.eq(user_id))
            .filter(user_favorites::Column::DishId.eq(dish_id))
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?;

        if let Some(fav) = existing {
            fav.delete(db).await.map_err(UserError::DatabaseError)?;
            
            // Profil sabitlenenlerini de senkron temizle
            let existing_pin = user_pinned_dishes::Entity::find()
                .filter(user_pinned_dishes::Column::UserId.eq(user_id))
                .filter(user_pinned_dishes::Column::DishId.eq(dish_id))
                .one(db)
                .await
                .map_err(UserError::DatabaseError)?;
            if let Some(pin) = existing_pin {
                let _ = pin.delete(db).await;
            }
            Ok(false)
        } else {
            let active = user_favorites::ActiveModel {
                user_id: Set(user_id),
                dish_id: Set(dish_id),
                created_at: Set(Some(Utc::now().into())),
            };
            active.insert(db).await.map_err(UserError::DatabaseError)?;

            // Profil sabitlenenlerine de senkron ekle
            let existing_pin = user_pinned_dishes::Entity::find()
                .filter(user_pinned_dishes::Column::UserId.eq(user_id))
                .filter(user_pinned_dishes::Column::DishId.eq(dish_id))
                .one(db)
                .await
                .map_err(UserError::DatabaseError)?;
            if existing_pin.is_none() {
                let active_pin = user_pinned_dishes::ActiveModel {
                    user_id: Set(user_id),
                    dish_id: Set(dish_id),
                    created_at: Set(Some(Utc::now().into())),
                };
                let _ = active_pin.insert(db).await;
            }
            Ok(true)
        }
    }

    /// Sabitlenmiş yemek durumunu değiştirir (varsa siler, yoksa ekler)
    pub async fn toggle_pinned(
        db: &DatabaseConnection,
        user_id: Uuid,
        dish_id: i32,
    ) -> Result<bool, UserError> {
        use shared::entities::user_pinned_dishes;
        let existing = user_pinned_dishes::Entity::find()
            .filter(user_pinned_dishes::Column::UserId.eq(user_id))
            .filter(user_pinned_dishes::Column::DishId.eq(dish_id))
            .one(db)
            .await
            .map_err(UserError::DatabaseError)?;

        if let Some(pin) = existing {
            pin.delete(db).await.map_err(UserError::DatabaseError)?;
            Ok(false)
        } else {
            let active = user_pinned_dishes::ActiveModel {
                user_id: Set(user_id),
                dish_id: Set(dish_id),
                created_at: Set(Some(Utc::now().into())),
            };
            active.insert(db).await.map_err(UserError::DatabaseError)?;
            Ok(true)
        }
    }
}

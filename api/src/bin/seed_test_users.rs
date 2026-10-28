use anyhow::Result;
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use shared::entities::{
    sea_orm_active_enums::{AccountStatusEnum, UserRoleEnum},
    users,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://kepce_admin:SIFRE@127.0.0.1:5432/kepce".to_string());

    println!("Veritabanına bağlanılıyor: {}", db_url);
    let db = Database::connect(&db_url).await?;

    let test_users = vec![
        (
            "admin",
            "Admin123456!",
            "kpc-ADMIN-00001-ALPHA-BETA0-OMEGA",
            true,
            UserRoleEnum::Admin,
            None,
        ),
        (
            "tatli_nohut_4821",
            "Ogrenci123!",
            "kpc-H5K9A-B2M7X-8QW4Z-C3D9Y-E1F2G",
            true,
            UserRoleEnum::User,
            Some("istanbul"),
        ),
        (
            "eksi_tabldot_1034",
            "Tabldot2026!",
            "kpc-R7T2P-N4K9M-X6W8L-Y3J5H-Z9Q8V",
            false,
            UserRoleEnum::User,
            Some("istanbul"),
        ),
        (
            "obur_pilav_9921",
            "EskiSifre123!",
            "kpc-B3C5D-F7G9H-J2K4M-N6P8Q-T1V3W",
            true,
            UserRoleEnum::User,
            None,
        ),
        (
            "glutensiz_gurme_34",
            "ColyakDiyeti2026!",
            "kpc-M2N4P-R6T8V-W1X3Y-Z5A7B-C9D1E",
            true,
            UserRoleEnum::User,
            Some("adana"),
        ),
    ];

    for (nickname, password, recovery_key, is_verified, role, city) in test_users {
        let password_hash = hash(password, DEFAULT_COST)?;
        let full_hash = api::core::crypto::hash_recovery_key(recovery_key);
        let recovery_key_hash = if !is_verified {
            let last_block = api::core::crypto::extract_last_block(recovery_key).unwrap_or("");
            let recall_hash = api::core::crypto::hash_recall_block(last_block);
            format!("{}:{}", full_hash, recall_hash)
        } else {
            full_hash
        };

        if let Some(existing) = users::Entity::find()
            .filter(users::Column::Nickname.eq(nickname))
            .one(&db)
            .await?
        {
            let mut active: users::ActiveModel = existing.into();
            active.password_hash = Set(password_hash);
            active.recovery_key_hash = Set(Some(recovery_key_hash));
            active.is_verified = Set(is_verified);
            active.role = Set(role);
            active.default_city_slug = Set(city.map(|s| s.to_string()));
            active.updated_at = Set(Some(Utc::now().into()));
            active.update(&db).await?;
            println!("Güncellendi: {}", nickname);
        } else {
            let new_user = users::ActiveModel {
                id: Set(Uuid::new_v4()),
                nickname: Set(nickname.to_string()),
                password_hash: Set(password_hash),
                recovery_key_hash: Set(Some(recovery_key_hash)),
                role: Set(role),
                account_status: Set(AccountStatusEnum::Active),
                karma_score: Set(10),
                is_verified: Set(is_verified),
                avatar_url: Set(None),
                default_city_slug: Set(city.map(|s| s.to_string())),
                level: Set(1),
                level_progress: Set(0),
                opt_out_statistics: Set(false),
                notif_replies: Set(true),
                notif_interactions: Set(true),
                notif_system: Set(true),
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
            new_user.insert(&db).await?;
            println!("Oluşturuldu: {}", nickname);
        }
    }

    println!("Tüm test kullanıcıları başarıyla tohumlandı.");
    Ok(())
}

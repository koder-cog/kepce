use sea_orm::*;
use uuid::Uuid;
use chrono::NaiveDate;
use serde_json::json;
use shared::entities::{
    prelude::*,
    reports,
    user_blocks,
    users,
    cities,
    menus,
    menu_dishes,
    dish_aliases,
    dishes,
    sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum},
};
use crate::dto::moderation::{BlockUserDto, InjectBotCommentEntryDto};

#[derive(Debug, Clone, Default)]
pub struct BlockedRelations {
    pub my_blocked_ids: Vec<Uuid>,
    pub blocked_me_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub enum ModerationError {
    CommentNotFound,
    UserNotFound,
    SelfBlockNotAllowed,
    SelfReportNotAllowed,
    AlreadyReported,
    AlreadyBlocked,
    CommentAlreadyDeleted,
    DatabaseError(DbErr),
    CityNotFound,
    NoMenusForMonth,
    InvalidMonth(String),
    DateParseError(String),
}

pub struct ModerationService;

impl ModerationService {
    /// Kullanıcının bir yorumu şikayet etmesi.
    pub async fn report_comment(
        db: &DatabaseConnection,
        reporter_id: Uuid,
        reported_comment_id: Uuid,
        reason: String,
    ) -> Result<(), ModerationError> {
        
        let comment = Comments::find_by_id(reported_comment_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CommentNotFound)?;

        if comment.user_id.is_none() {
            return Err(ModerationError::CommentAlreadyDeleted);
        }

        if comment.user_id == Some(reporter_id) {
            return Err(ModerationError::SelfReportNotAllowed);
        }

        // Aynı kişinin aynı yorumu defalarca şikayet etmesini engelliyoruz (Spam koruması)
        let existing_report = Reports::find()
            .filter(reports::Column::ReporterId.eq(reporter_id))
            .filter(reports::Column::ReportedCommentId.eq(reported_comment_id))
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        if existing_report.is_some() {
            return Err(ModerationError::AlreadyReported);
        }

        let new_report = reports::ActiveModel {
            reporter_id: Set(reporter_id),
            reported_comment_id: Set(Some(reported_comment_id)),
            reason: Set(Some(reason)),
            status: Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Pending),
            ..Default::default()
        };

        new_report.insert(db).await.map_err(ModerationError::DatabaseError)?;

        Ok(())
    }

    /// Bir kullanıcıyı engelleme işlemi.
    pub async fn block_user(
        db: &DatabaseConnection,
        blocker_id: Uuid,
        dto: BlockUserDto,
    ) -> Result<(), ModerationError> {
        
        if blocker_id == dto.blocked_user_id {
            return Err(ModerationError::SelfBlockNotAllowed);
        }

        // Engellenecek kullanıcı gerçekten var mı?
        let user_exists = Users::find_by_id(dto.blocked_user_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .is_some();

        if !user_exists {
            return Err(ModerationError::UserNotFound);
        }

        let new_block = user_blocks::ActiveModel {
            blocker_id: Set(blocker_id),
            blocked_id: Set(dto.blocked_user_id),
            ..Default::default()
        };

        // Eğer zaten engelliyse Unique Constraint patlayacak.
        match new_block.insert(db).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if crate::utils::db::is_unique_constraint_violation(&e) {
                    Err(ModerationError::AlreadyBlocked)
                } else {
                    Err(ModerationError::DatabaseError(e))
                }
            }
        }
    }

    /// Bir kullanıcının engelini kaldırma işlemi.
    pub async fn unblock_user(
        db: &DatabaseConnection,
        blocker_id: Uuid,
        blocked_user_id: Uuid,
    ) -> Result<(), ModerationError> {
        
        let block = UserBlocks::find()
            .filter(user_blocks::Column::BlockerId.eq(blocker_id))
            .filter(user_blocks::Column::BlockedId.eq(blocked_user_id))
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        if let Some(b) = block {
            b.delete(db).await.map_err(ModerationError::DatabaseError)?;
        }
        
        Ok(())
    }

    /// Çift taraflı izolasyon: Kullanıcının engellediği ve kullanıcıyı engelleyenlerin ID listesini döner.
    pub async fn get_blocked_user_ids(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, ModerationError> {
        let blocks = UserBlocks::find()
            .filter(
                sea_orm::Condition::any()
                    .add(user_blocks::Column::BlockerId.eq(user_id))
                    .add(user_blocks::Column::BlockedId.eq(user_id))
            )
            .all(db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        // Kullanıcının id'si blocker ise blocked'ı ekle, blocked ise blocker'ı ekle
        let mut ids: Vec<Uuid> = blocks.into_iter().map(|b| {
            if b.blocker_id == user_id {
                b.blocked_id
            } else {
                b.blocker_id
            }
        }).collect();

        // Aynı kişiyi karşılıklı engellemiş olabilirler (nadir de olsa), deduplicate yapalım.
        ids.sort();
        ids.dedup();

        Ok(ids)
    }

    /// Çift taraflı engelleme ilişkilerini ayrıştırılmış olarak döner
    pub async fn get_blocked_relations(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<BlockedRelations, ModerationError> {
        let blocks = UserBlocks::find()
            .filter(
                sea_orm::Condition::any()
                    .add(user_blocks::Column::BlockerId.eq(user_id))
                    .add(user_blocks::Column::BlockedId.eq(user_id))
            )
            .all(db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        let mut my_blocked_ids = Vec::new();
        let mut blocked_me_ids = Vec::new();

        for b in blocks {
            if b.blocker_id == user_id {
                my_blocked_ids.push(b.blocked_id);
            } else if b.blocked_id == user_id {
                blocked_me_ids.push(b.blocker_id);
            }
        }

        Ok(BlockedRelations {
            my_blocked_ids,
            blocked_me_ids,
        })
    }

    /// Kullanıcı hesap durumunu günceller (Admin/Mod).
    pub async fn update_user_status(
        db: &DatabaseConnection,
        user_id: Uuid,
        status_str: &str,
    ) -> Result<(), ModerationError> {
        use shared::entities::sea_orm_active_enums::AccountStatusEnum;

        let parsed_status = match status_str.to_lowercase().as_str() {
            "active" => AccountStatusEnum::Active,
            "suspended" => AccountStatusEnum::Suspended,
            "banned" => AccountStatusEnum::Banned,
            _ => return Err(ModerationError::DatabaseError(DbErr::Custom("Geçersiz statü değeri".into()))),
        };

        let mut user: users::ActiveModel = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::UserNotFound)?
            .into();

        user.account_status = Set(parsed_status.clone());

        // SA-4: Ban/suspend sırasında oturum sürümünü artır — kullanıcının tüm
        // refresh token'ları anında geçersizleşir (access token'lar da extractor'daki
        // account_status kontrolüne takılır).
        if parsed_status != AccountStatusEnum::Active {
            let current_version = user.token_version.clone().unwrap();
            user.token_version = Set(current_version + 1);
        }

        user.update(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    /// Kullanıcının yetkilendirme, onay ve ban durumunu günceller.
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        is_verified: Option<bool>,
        is_admin: Option<bool>,
        is_banned: Option<bool>,
    ) -> Result<(), ModerationError> {
        use shared::entities::sea_orm_active_enums::{UserRoleEnum, AccountStatusEnum};

        let mut user: users::ActiveModel = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::UserNotFound)?
            .into();

        if let Some(verified) = is_verified {
            user.is_verified = Set(verified);
        }

        if let Some(admin) = is_admin {
            let role = if admin {
                UserRoleEnum::Admin
            } else {
                UserRoleEnum::User
            };
            user.role = Set(role);
        }

        if let Some(banned) = is_banned {
            let status = if banned {
                AccountStatusEnum::Banned
            } else {
                AccountStatusEnum::Active
            };
            user.account_status = Set(status.clone());

            if status != AccountStatusEnum::Active {
                let current_version = user.token_version.clone().unwrap();
                user.token_version = Set(current_version + 1);
            }
        }

        user.update(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    /// Bir raporu/şikayeti çözümler (Admin/Mod).
    pub async fn resolve_report(
        db: &DatabaseConnection,
        report_id: i32,
        _action_taken: String,
    ) -> Result<(), ModerationError> {
        let mut report: reports::ActiveModel = Reports::find_by_id(report_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CommentNotFound)? // Rapor bulunamadı için spesifik bir error eklenebilir ama idare eder
            .into();

        report.status = Set(shared::entities::sea_orm_active_enums::ReportStatusEnum::Resolved);
        report.resolved_at = Set(Some(chrono::Utc::now().into()));

        report.update(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    /// Yeni bir sistem olayı (incident) oluşturur.
    pub async fn create_incident(
        db: &DatabaseConnection,
        dto: crate::dto::moderation::CreateIncidentDto,
    ) -> Result<i32, ModerationError> {
        let new_incident = shared::entities::system_incidents::ActiveModel {
            component: Set(dto.component),
            title: Set(dto.title),
            message: Set(dto.message),
            status: Set("investigating".to_string()),
            impact: Set(dto.impact),
            created_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        let result = new_incident.insert(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(result.id)
    }

    /// Bir sistem olayının (incident) durumunu günceller.
    pub async fn update_incident(
        db: &DatabaseConnection,
        incident_id: i32,
        dto: crate::dto::moderation::UpdateIncidentDto,
    ) -> Result<(), ModerationError> {
        let mut incident: shared::entities::system_incidents::ActiveModel = shared::entities::system_incidents::Entity::find_by_id(incident_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CommentNotFound)? // Re-using error enum for simplicity
            .into();

        incident.status = Set(dto.status.clone());
        if dto.status == "resolved" {
            incident.resolved_at = Set(Some(chrono::Utc::now().into()));
        }

        incident.update(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    /// Bir sistem olayını tamamen siler.
    pub async fn delete_incident(
        db: &DatabaseConnection,
        incident_id: i32,
    ) -> Result<(), ModerationError> {
        let incident = shared::entities::system_incidents::Entity::find_by_id(incident_id)
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CommentNotFound)?;

        incident.delete(db).await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    /// Update menu items (menu_dishes)
    pub async fn update_menu_items(
        db: &DatabaseConnection,
        menu_id: i32,
        dish_alias_ids: Vec<i32>,
    ) -> Result<(), ModerationError> {
        let txn = db.begin().await.map_err(ModerationError::DatabaseError)?;

        // Delete existing menu_dishes for this menu
        shared::entities::menu_dishes::Entity::delete_many()
            .filter(shared::entities::menu_dishes::Column::MenuId.eq(menu_id))
            .exec(&txn)
            .await
            .map_err(ModerationError::DatabaseError)?;

        // Insert new ones
        if !dish_alias_ids.is_empty() {
            let mut inserts = Vec::new();
            for (index, alias_id) in dish_alias_ids.into_iter().enumerate() {
                inserts.push(shared::entities::menu_dishes::ActiveModel {
                    menu_id: Set(menu_id),
                    dish_alias_id: Set(alias_id),
                    order_index: Set(index as i32),
                    is_alternative: Set(false), // Assuming false for now based on default usecase
                    package_name: Set("Standard".to_string()),
                    ..Default::default()
                });
            }
            shared::entities::menu_dishes::Entity::insert_many(inserts)
                .exec(&txn)
                .await
                .map_err(ModerationError::DatabaseError)?;
        }

        txn.commit().await.map_err(ModerationError::DatabaseError)?;
        Ok(())
    }

    // --- Kepçe Bot: Aylık batch (export-monthly / inject) yardımcıları ---

    /// "2026-04" -> (2026-04-01, 2026-05-01)
    fn parse_month_range(month: &str) -> Result<(NaiveDate, NaiveDate), ModerationError> {
        let parts: Vec<&str> = month.split('-').collect();
        if parts.len() != 2 {
            return Err(ModerationError::InvalidMonth(month.to_string()));
        }
        let year: i32 = parts[0]
            .parse()
            .map_err(|_| ModerationError::InvalidMonth(month.to_string()))?;
        let mon: u32 = parts[1]
            .parse()
            .map_err(|_| ModerationError::InvalidMonth(month.to_string()))?;
        let start = NaiveDate::from_ymd_opt(year, mon, 1)
            .ok_or_else(|| ModerationError::InvalidMonth(month.to_string()))?;
        let end = if mon == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, mon + 1, 1)
        }
        .ok_or_else(|| ModerationError::InvalidMonth(month.to_string()))?;
        Ok((start, end))
    }

    /// "2026-04-01" (ISO) veya "1 Nisan 2026" (Türkçe) -> NaiveDate
    fn parse_bot_date(s: &str) -> Option<NaiveDate> {
        let s = s.trim();
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(d);
        }
        const TR_MONTHS: [&str; 12] = [
            "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos",
            "Eylül", "Ekim", "Kasım", "Aralık",
        ];
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() == 3 {
            let day: u32 = parts[0].parse().ok()?;
            let month_idx = TR_MONTHS.iter().position(|m| m.eq_ignore_ascii_case(parts[1]))?;
            let year: i32 = parts[2].parse().ok()?;
            return NaiveDate::from_ymd_opt(year, (month_idx + 1) as u32, day);
        }
        None
    }

    fn meal_label(m: &MealTypeEnum) -> &'static str {
        match m {
            MealTypeEnum::Breakfast => "Kahvaltı",
            MealTypeEnum::Lunch => "Öğle Yemeği",
            MealTypeEnum::Dinner => "Akşam Yemeği",
        }
    }

    /// Aylık menüyü düz metne çevirir (bot girdisi için).
    /// Yalnızca onaylı (Approved) menüleri içerir.
    pub async fn export_monthly_menu_for_bot(
        db: &DatabaseConnection,
        city_slug: &str,
        month: &str,
    ) -> Result<String, ModerationError> {
        let city = Cities::find()
            .filter(cities::Column::Slug.eq(city_slug))
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CityNotFound)?;

        let (start, end) = Self::parse_month_range(month)?;

        let menus_list = Menus::find()
            .filter(menus::Column::CityId.eq(city.id))
            .filter(menus::Column::ServeDate.gte(start))
            .filter(menus::Column::ServeDate.lt(end))
            .filter(menus::Column::Status.eq(MenuStatusEnum::Approved))
            .order_by_asc(menus::Column::ServeDate)
            .order_by_asc(menus::Column::MealType)
            .all(db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        if menus_list.is_empty() {
            return Err(ModerationError::NoMenusForMonth);
        }

        let menu_dishes_groups = menus_list
            .load_many(
                menu_dishes::Entity::find().order_by_asc(menu_dishes::Column::OrderIndex),
                db,
            )
            .await
            .map_err(ModerationError::DatabaseError)?;

        let flat_menu_dishes: Vec<menu_dishes::Model> =
            menu_dishes_groups.iter().flatten().cloned().collect();
        let dish_aliases_opts = flat_menu_dishes
            .load_one(dish_aliases::Entity, db)
            .await
            .map_err(ModerationError::DatabaseError)?;
        let flat_dish_aliases: Vec<dish_aliases::Model> =
            dish_aliases_opts.iter().flatten().cloned().collect();
        let dishes_opts = flat_dish_aliases
            .load_one(dishes::Entity, db)
            .await
            .map_err(ModerationError::DatabaseError)?;

        let mut text = String::new();
        let mut alias_idx = 0usize;
        let mut dish_idx = 0usize;
        let mut last_date: Option<NaiveDate> = None;

        for (i, menu) in menus_list.iter().enumerate() {
            if last_date != Some(menu.serve_date) {
                text.push_str(&format!("\n=== {} ===\n", menu.serve_date));
                last_date = Some(menu.serve_date);
            }
            text.push_str(&format!("--- {} ---\n", Self::meal_label(&menu.meal_type)));
            for md in &menu_dishes_groups[i] {
                let alias_opt = &dish_aliases_opts[alias_idx];
                alias_idx += 1;
                if let Some(alias) = alias_opt {
                    let _ = &dishes_opts[dish_idx];
                    dish_idx += 1;
                    let tag = if md.is_alternative { " (alternatif)" } else { "" };
                    text.push_str(&format!("- {}{}\n", alias.name, tag));
                }
            }
        }

        Ok(text)
    }

    /// Her günün yorumunu, o günün TÜM öğün kayıtlarına (kahvaltı + akşam) yazar.
    pub async fn inject_bot_comments(
        db: &DatabaseConnection,
        city_slug: &str,
        comments: &[InjectBotCommentEntryDto],
    ) -> Result<usize, ModerationError> {
        let city = Cities::find()
            .filter(cities::Column::Slug.eq(city_slug))
            .one(db)
            .await
            .map_err(ModerationError::DatabaseError)?
            .ok_or(ModerationError::CityNotFound)?;

        let txn = db.begin().await.map_err(ModerationError::DatabaseError)?;
        let mut updated: usize = 0;

        for c in comments {
            let date = Self::parse_bot_date(&c.date)
                .ok_or_else(|| ModerationError::DateParseError(c.date.clone()))?;

            let day_menus = Menus::find()
                .filter(menus::Column::CityId.eq(city.id))
                .filter(menus::Column::ServeDate.eq(date))
                .all(&txn)
                .await
                .map_err(ModerationError::DatabaseError)?;

            for m in day_menus {
                let commentary_json = json!({ "yorum": c.commentary }).to_string();
                let active = menus::ActiveModel {
                    id: Set(m.id),
                    bot_commentary: Set(Some(commentary_json)),
                    ..Default::default()
                };
                active
                    .update(&txn)
                    .await
                    .map_err(ModerationError::DatabaseError)?;
                updated += 1;
            }
        }

        txn.commit().await.map_err(ModerationError::DatabaseError)?;
        Ok(updated)
    }

    /// Yerel BERT NLP Moderasyon Servisi Kontrolü
    pub async fn check_text_ai(text: &str) -> Result<AiModerationResponse, String> {
        let moderator_url = std::env::var("MODERATOR_URL").unwrap_or_else(|_| "http://moderator:8002".to_string());
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(800))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .post(format!("{}/check", moderator_url))
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let res: AiModerationResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(res)
        } else {
            Err(format!("Moderator returned status: {}", resp.status()))
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AiModerationResponse {
    pub is_toxic: bool,
    pub label: String,
    pub score: f32,
}

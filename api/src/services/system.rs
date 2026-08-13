use sea_orm::*;
use shared::entities::{menus, menu_dishes};
use crate::dto::system::{VerifyTreeResponseDto, SystemStatusDto, ComponentHistoryDto, StatusDayDto};
use sha2::{Sha256, Digest};
use chrono::{Utc, Duration};

pub struct SystemService;

impl SystemService {
    pub async fn verify_chain(db: &DatabaseConnection) -> Result<VerifyTreeResponseDto, DbErr> {
        let mut corrupted_hashes = Vec::new();
        let mut node_count = 0;

        let cities = shared::entities::cities::Entity::find().all(db).await?;
        let meal_types = vec![
            shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner,
        ];

        for city in &cities {
            for meal_type in &meal_types {
                let menus_list = menus::Entity::find()
                    .filter(menus::Column::CityId.eq(city.id))
                    .filter(menus::Column::MealType.eq(meal_type.clone()))
                    .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved))
                    .order_by_asc(menus::Column::ServeDate)
                    .all(db)
                    .await?;

                let mut expected_prev_hash: Option<String> = None;

                for menu in menus_list {
                    node_count += 1;

                    let dishes = menu_dishes::Entity::find()
                        .filter(menu_dishes::Column::MenuId.eq(menu.id))
                        .order_by_asc(menu_dishes::Column::OrderIndex)
                        .all(db)
                        .await?;

                    let sorted_dish_ids: Vec<i32> = dishes.iter().map(|d| d.dish_alias_id).collect();

                    let meal_type_str = match &menu.meal_type {
                        shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast",
                        shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch",
                        shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner",
                    };

                    let calculated_hash = shared::services::immutable_store::ImmutableStore::compute_menu_hash(
                        menu.serve_date,
                        menu.city_id,
                        meal_type_str,
                        &sorted_dish_ids,
                        expected_prev_hash.as_deref(),
                    );

                    let mut is_corrupted = false;

                    if let Some(ref stored_hash) = menu.merkle_root {
                        if stored_hash != &calculated_hash {
                            is_corrupted = true;
                        }
                    } else {
                        is_corrupted = true;
                    }

                    if menu.previous_hash != expected_prev_hash {
                        is_corrupted = true;
                    }

                    if is_corrupted {
                        corrupted_hashes.push(format!(
                            "Menu ID {}: {} - {} on {} is corrupted (Stored: {:?}, Calculated: {}, Prev Stored: {:?}, Expected Prev: {:?})",
                            menu.id,
                            city.name,
                            meal_type_str,
                            menu.serve_date.format("%Y-%m-%d"),
                            menu.merkle_root,
                            calculated_hash,
                            menu.previous_hash,
                            expected_prev_hash
                        ));
                    }

                    expected_prev_hash = Some(calculated_hash);
                }
            }
        }

        let is_valid = corrupted_hashes.is_empty();
        let corrupted_count = corrupted_hashes.len() as i32;

        Ok(VerifyTreeResponseDto {
            is_valid,
            node_count,
            corrupted_count,
            corrupted_hashes,
        })
    }

    pub async fn get_system_health(db: &DatabaseConnection) -> Result<crate::dto::system::SystemHealthResponseDto, DbErr> {
        let active_incidents_count = shared::entities::system_incidents::Entity::find()
            .filter(shared::entities::system_incidents::Column::Status.ne("resolved"))
            .count(db)
            .await?;

        let status = if active_incidents_count > 0 { "unhealthy".to_string() } else { "healthy".to_string() };

        let mut node_counts = std::collections::HashMap::new();
        let menu_count = menus::Entity::find().count(db).await? as i64;
        let comment_count = shared::entities::comments::Entity::find().count(db).await? as i64;
        let user_count = shared::entities::users::Entity::find().count(db).await? as i64;
        let report_count = shared::entities::reports::Entity::find().count(db).await? as i64;

        node_counts.insert("Menü node".to_string(), menu_count);
        node_counts.insert("Yorum node".to_string(), comment_count);
        node_counts.insert("Kullanıcı node".to_string(), user_count);
        node_counts.insert("Rapor node".to_string(), report_count);

        let total_nodes = menu_count + comment_count + user_count + report_count;

        let cities = shared::entities::cities::Entity::find().all(db).await?;
        let meal_types = vec![
            shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch,
            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner,
        ];

        let mut heads = Vec::new();
        let mut head_hashes = Vec::new();

        for city in &cities {
            for meal_type in &meal_types {
                let latest_menu = menus::Entity::find()
                    .filter(menus::Column::CityId.eq(city.id))
                    .filter(menus::Column::MealType.eq(meal_type.clone()))
                    .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved))
                    .order_by_desc(menus::Column::ServeDate)
                    .one(db)
                    .await?;

                if let Some(menu) = latest_menu {
                    if let Some(hash) = menu.merkle_root {
                        let meal_type_str = match &meal_type {
                            shared::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast",
                            shared::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch",
                            shared::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner",
                        };
                        let key = format!("menu:{}:{}", city.slug, meal_type_str);
                        heads.push(crate::dto::system::HeadDto {
                            key,
                            hash: hash.clone(),
                        });
                        head_hashes.push(hash);
                    }
                }
            }
        }

        let global_fingerprint = if head_hashes.is_empty() {
            "GENESIS_STATE".to_string()
        } else {
            head_hashes.sort();
            let combined = head_hashes.join(":");
            let mut hasher = Sha256::new();
            hasher.update(combined.as_bytes());
            hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>()
        };

        Ok(crate::dto::system::SystemHealthResponseDto {
            status,
            global_fingerprint,
            node_counts,
            heads,
            total_nodes,
        })
    }

    pub async fn get_system_status(db: &DatabaseConnection) -> Result<SystemStatusDto, DbErr> {
        let today = Utc::now();
        let thirty_days_ago = today - Duration::days(30);

        let recent_incidents = shared::entities::system_incidents::Entity::find()
            .filter(shared::entities::system_incidents::Column::CreatedAt.gte(thirty_days_ago.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())))
            .order_by_desc(shared::entities::system_incidents::Column::CreatedAt)
            .all(db)
            .await?;

        let mut genel_durum = "aktif".to_string();
        for inc in &recent_incidents {
            if inc.status != "resolved" {
                if inc.impact == "kesinti" {
                    genel_durum = "kesinti".to_string();
                } else if genel_durum != "kesinti" {
                    genel_durum = "yavas".to_string();
                }
            }
        }

        let incidents_list: Vec<crate::dto::system::IncidentDto> = recent_incidents.into_iter().map(|i| crate::dto::system::IncidentDto {
            id: Some(i.id),
            component: i.component,
            title: i.title,
            message: i.message,
            started_at: i.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            ended_at: i.resolved_at.map(|d| d.to_rfc3339()),
            status: i.impact,
        }).collect();

        // Fetch son_aktivite from menus
        let latest_menu = menus::Entity::find()
            .order_by_desc(menus::Column::CreatedAt)
            .one(db)
            .await?;
        
        let last_activity = latest_menu.and_then(|m| m.created_at.map(|d| d.to_rfc3339()));

        Ok(SystemStatusDto {
            status: genel_durum,
            last_activity,
            incidents: incidents_list,
        })
    }

    pub async fn get_status_history(db: &DatabaseConnection) -> Result<Vec<ComponentHistoryDto>, DbErr> {
        let today = Utc::now();
        let ninety_days_ago = today - Duration::days(90);

        let incidents = shared::entities::system_incidents::Entity::find()
            .filter(shared::entities::system_incidents::Column::CreatedAt.gte(ninety_days_ago.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())))
            .all(db)
            .await?;

        let build_history = |component_name: &str| -> ComponentHistoryDto {
            let mut days = Vec::new();
            for i in (0..90).rev() {
                let date = today - Duration::days(i);
                let is_affected = incidents.iter().any(|inc| {
                    if inc.component != component_name { return false; }
                    let start = inc.created_at.unwrap_or_default().with_timezone(&Utc);
                    let end = inc.resolved_at.map(|d| d.with_timezone(&Utc)).unwrap_or(today);
                    date.date_naive() >= start.date_naive() && date.date_naive() <= end.date_naive()
                });

                let status = if is_affected {
                    let impact = incidents.iter().find(|inc| {
                        if inc.component != component_name { return false; }
                        let start = inc.created_at.unwrap_or_default().with_timezone(&Utc);
                        let end = inc.resolved_at.map(|d| d.with_timezone(&Utc)).unwrap_or(today);
                        date.date_naive() >= start.date_naive() && date.date_naive() <= end.date_naive()
                    }).map(|inc| inc.impact.clone()).unwrap_or("yavas".to_string());
                    impact
                } else {
                    "aktif".to_string()
                };

                days.push(StatusDayDto {
                    date: date.to_rfc3339(),
                    status,
                });
            }
            ComponentHistoryDto {
                name: component_name.to_string(),
                days,
            }
        };

        Ok(vec![
            build_history("API Sunucusu"),
            build_history("Veritabanı"),
            build_history("Botlar"),
        ])
    }
}

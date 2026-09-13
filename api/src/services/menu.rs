//! Menü verilerini okuma, filtreleme ve DTO formatlama servisi.
//!
//! Sorumluluklar ve Mimari İlkeler:
//! - Salt Okunur (Read-Only): Menü yazma ve ayrıştırma (upsert/ingest) işlemleri
//!   worker katmanındadır; API servisi yalnızca okuma, sorgulama ve formatlama yapar.
//! - Dinamik Fiyatlandırma: Fiyatlar veritabanındaki menü tablosuna yazılmaz; okuma
//!   anında `pricing::get_pricing_info_for_city` servisi üzerinden şehre, tarihe ve
//!   yemek kategorisine göre dinamik hesaplanır (Temmuz ve Ağustos aylarında bilinçli
//!   olarak gizlenir).
//! - Toplu Yükleme (Batch Loading): N+1 sorgu maliyetlerini engellemek adına menülere
//!   bağlı yemekler, takma adlar, ana yemekler, yorum ve oy sayıları toplu olarak çekilip
//!   bellekte eşleştirilir.


use sea_orm::*;
use std::collections::{HashMap, HashSet};
use chrono::NaiveDate;
use sea_orm::sea_query::Expr;
use shared::entities::{
    prelude::*, menus, menu_dishes, dish_aliases, dishes, cities, sea_orm_active_enums::MealTypeEnum, comments, menu_votes, dish_votes, menu_history,
};
use crate::dto::menu::{MenuResponseDto, MenuItemDto, DishMasterDataDto, MealType, ArchiveHighlightDto, AlternativeMenuDto};

#[derive(Debug)]
pub enum MenuError {
    NotFound,
    DatabaseError(DbErr),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DishVoteStats {
    pub total: i32,
    pub positive: i32,
    pub negative: i32,
    pub dislike_ratio: Option<f64>,
    pub like_ratio: Option<f64>,
}

pub struct MenuService;

impl MenuService {
    /// Şehir parametresini hem plaka ID (örn: 34) hem de slug (örn: istanbul) üzerinden çözer.
    pub async fn resolve_city(
        db: &DatabaseConnection,
        city_param: &str,
    ) -> Result<Option<cities::Model>, DbErr> {
        if let Ok(id) = city_param.parse::<i32>() {
            cities::Entity::find_by_id(id).one(db).await
        } else {
            cities::Entity::find()
                .filter(cities::Column::Slug.eq(city_param.to_lowercase()))
                .one(db)
                .await
        }
    }

    /// Veritabanı MealType enumunu DTO enumuna çevirir
    fn map_meal_type(db_meal_type: &MealTypeEnum) -> MealType {
        match db_meal_type {
            MealTypeEnum::Breakfast => MealType::Breakfast,
            MealTypeEnum::Lunch => MealType::Lunch,
            MealTypeEnum::Dinner => MealType::Dinner,
        }
    }

    fn map_menu_status(status: &shared::entities::sea_orm_active_enums::MenuStatusEnum) -> String {
        match status {
            shared::entities::sea_orm_active_enums::MenuStatusEnum::Pending => "pending".to_string(),
            shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved => "approved".to_string(),
            shared::entities::sea_orm_active_enums::MenuStatusEnum::Rejected => "rejected".to_string(),
        }
    }

    fn format_calorie_range(min: Option<i32>, max: Option<i32>) -> Option<String> {
        match (min, max) {
            (Some(min_val), Some(max_val)) => Some(format!("{} - {} kcal", min_val, max_val)),
            (Some(min_val), None) => Some(format!("{} kcal", min_val)),
            (None, Some(max_val)) => Some(format!("{} kcal", max_val)),
            (None, None) => None,
        }
    }

    fn parse_alternative_from_history(
        hist: &shared::entities::menu_history::Model,
        meal_type: MealType,
    ) -> Option<AlternativeMenuDto> {
        let mut items = Vec::new();
        let payload = &hist.dishes_payload;

        let dish_list_opt: Option<&Vec<serde_json::Value>> = payload
            .get("dishes")
            .and_then(|d| d.as_array())
            .or_else(|| payload.as_array());

        if let Some(arr) = dish_list_opt {
            for (idx, val) in arr.iter().enumerate() {
                let name = if let Some(s) = val.as_str() {
                    s.to_string()
                } else if let Some(n) = val.get("name").and_then(|s| s.as_str()) {
                    n.to_string()
                } else if let Some(r) = val.get("raw_name").and_then(|s| s.as_str()) {
                    r.to_string()
                } else {
                    continue;
                };

                if shared::services::content_guard::ContentGuard::is_junk_dish_text(&name) {
                    continue;
                }

                let is_alt = val.get("is_alternative").and_then(|b| b.as_bool()).unwrap_or(false);

                items.push(MenuItemDto {
                    order_index: idx as i32,
                    raw_name: name,
                    is_alternative: is_alt,
                    amount: None,
                    calories: None,
                    price: None,
                    category: None,
                    master_data: None,
                });
            }
        }

        if items.is_empty() {
            return None;
        }

        Some(AlternativeMenuDto {
            id: Some(hist.id),
            source_type: hist.source_type.clone(),
            meal_type,
            items,
            calories: None,
        })
    }

    fn compute_menu_dishes_signature(items: &[MenuItemDto]) -> Vec<String> {
        let mut names: Vec<String> = items
            .iter()
            .map(|it| {
                it.raw_name
                    .trim()
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .collect();
        names.sort();
        names
    }

    pub(crate) fn calculate_total_calories(items: &[MenuItemDto]) -> Option<i32> {
        let mut total = 0;
        let mut has_calories = false;
        for item in items {
            // Alternatif yemekler aynı öğünde seçilemeyen ek seçenekler olduğundan
            // tabldot toplam kalorisine çift eklenmez.
            if !item.is_alternative {
                if let Some(cal) = item.calories {
                    total += cal;
                    has_calories = true;
                }
            }
        }
        if has_calories { Some(total) } else { None }
    }

    async fn get_dish_vote_stats_map(
        db: &DatabaseConnection,
        dish_ids: &[i32],
    ) -> HashMap<i32, DishVoteStats> {
        if dish_ids.is_empty() {
            return HashMap::new();
        }

        let vote_stats_list: Vec<(i32, i64, i64, i64)> = dish_votes::Entity::find()
            .select_only()
            .column(dish_votes::Column::DishId)
            .column_as(dish_votes::Column::Id.count(), "total_votes")
            .column_as(
                Expr::cust("COALESCE(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END), 0)"),
                "positive_votes",
            )
            .column_as(
                Expr::cust("COALESCE(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END), 0)"),
                "negative_votes",
            )
            .filter(dish_votes::Column::DishId.is_in(dish_ids.to_vec()))
            .group_by(dish_votes::Column::DishId)
            .into_tuple()
            .all(db)
            .await
            .unwrap_or_default();

        let mut map = HashMap::new();
        for (d_id, total, pos, neg) in vote_stats_list {
            let total_i32 = total as i32;
            let pos_i32 = pos as i32;
            let neg_i32 = neg as i32;
            let (like_ratio, dislike_ratio) = if total_i32 > 0 {
                (
                    Some((pos as f64) / (total as f64)),
                    Some((neg as f64) / (total as f64)),
                )
            } else {
                (None, None)
            };
            map.insert(
                d_id,
                DishVoteStats {
                    total: total_i32,
                    positive: pos_i32,
                    negative: neg_i32,
                    dislike_ratio,
                    like_ratio,
                },
            );
        }
        map
    }

    /// Belirtilen bir menüyü (ID ile) ve içindeki yemekleri tam hiyerarşiyle getirir
    pub async fn get_menu_with_items(
        db: &DatabaseConnection,
        menu_id: i32,
        dietary_type: Option<String>,
        user_id: Option<uuid::Uuid>,
    ) -> Result<MenuResponseDto, MenuError> {
        let (menu, city_opt) = Menus::find_by_id(menu_id)
            .find_also_related(cities::Entity)
            .one(db)
            .await
            .map_err(MenuError::DatabaseError)?
            .ok_or(MenuError::NotFound)?;
            
        if menu.status != shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved {
            return Err(MenuError::NotFound);
        }
            
        let city = city_opt.ok_or(MenuError::NotFound)?;

        let menu_dishes_with_aliases = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(menu_id))
            .find_also_related(dish_aliases::Entity)
            .order_by_asc(menu_dishes::Column::OrderIndex)
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)?;
            
        let dish_ids: Vec<i32> = menu_dishes_with_aliases
            .iter()
            .filter_map(|(_, alias_opt)| alias_opt.as_ref().and_then(|a| a.dish_id))
            .collect();
            
        let master_dishes = if !dish_ids.is_empty() {
            dishes::Entity::find()
                .filter(dishes::Column::Id.is_in(dish_ids.clone()))
                .all(db)
                .await
                .map_err(MenuError::DatabaseError)?
        } else {
            vec![]
        };
        
        let mut master_map = HashMap::new();
        for dish in master_dishes {
            master_map.insert(dish.id, dish);
        }
        let dish_stats_map = Self::get_dish_vote_stats_map(db, &dish_ids).await;
        
        let mut items = Vec::new();
        let mut takeaway_map: HashMap<String, Vec<MenuItemDto>> = HashMap::new();
        for (md, alias_opt) in menu_dishes_with_aliases {
            let alias = alias_opt.ok_or_else(|| MenuError::DatabaseError(DbErr::Custom("Yabancı anahtar bozuk: Alias bulunamadı".into())))?;
            if shared::services::content_guard::ContentGuard::is_junk_dish_text(&alias.name) {
                continue;
            }
            
            let master_data = alias.dish_id.and_then(|did| master_map.get(&did)).map(|dish| {
                let stats = dish_stats_map
                    .get(&dish.id)
                    .copied()
                    .unwrap_or_default();

                DishMasterDataDto {
                    dish_id: dish.id,
                    name: dish.name.clone(),
                    is_celiac: dish.is_celiac,
                    is_vegan: dish.is_vegan,
                    is_vegetarian: dish.is_vegetarian,
                    estimated_calories: dish.estimated_calories,
                    total_votes: stats.total,
                    positive_votes: stats.positive,
                    negative_votes: stats.negative,
                    dislike_ratio: stats.dislike_ratio,
                    like_ratio: stats.like_ratio,
                }
            });
            
            let dish_is_celiac = master_data.as_ref().is_some_and(|m| m.is_celiac);
            let pkg_upper = md.package_name.to_uppercase();
            let is_celiac_pkg = pkg_upper.contains("ÇÖLYAK") || pkg_upper.contains("COLYAK");
            let is_takeaway_pkg = md.package_name != "NORMAL" && !is_celiac_pkg;
            let is_celiac_mode = dietary_type.as_deref() == Some("celiac");
            
            let dish_category = alias.dish_id.and_then(|did| master_map.get(&did)).and_then(|dish| dish.category.clone());
            let meal_type_str = match menu.meal_type {
                MealTypeEnum::Breakfast => "breakfast",
                MealTypeEnum::Lunch => "lunch",
                MealTypeEnum::Dinner => "dinner",
            };

            let dish_name_for_pricing = alias.dish_id.and_then(|did| master_map.get(&did)).map(|d| d.name.as_str()).unwrap_or(alias.name.as_str());

            let price_info = crate::services::pricing::get_pricing_info_for_city(
                &city.slug,
                Some(menu.serve_date),
                meal_type_str,
                dish_category.as_deref(),
                dish_name_for_pricing
            );

            let amount = md.amount.clone().or_else(|| price_info.as_ref().map(|p| p.amount.clone()));
            let price = price_info.map(|p| p.price as f64);

            let item_dto = MenuItemDto {
                order_index: md.order_index,
                raw_name: alias.name.clone(),
                is_alternative: md.is_alternative,
                amount,
                calories: md.calories,
                price,
                category: dish_category,
                master_data,
            };
            
            if is_celiac_mode {
                if is_celiac_pkg || (md.package_name == "NORMAL" && dish_is_celiac) {
                    if is_takeaway_pkg {
                        takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                    } else {
                        items.push(item_dto);
                    }
                }
            } else {
                if !is_celiac_pkg {
                    if md.package_name != "NORMAL" {
                        takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                    } else {
                        items.push(item_dto);
                    }
                }
            }
        }
        
        let mut takeaways = Vec::new();
        for (name, mut t_items) in takeaway_map {
            t_items.sort_by_key(|i| (i.order_index, i.is_alternative));
            takeaways.push(crate::dto::menu::TakeawayMenuDto { name, items: t_items });
        }
        takeaways.sort_by(|a, b| a.name.cmp(&b.name));
        
        let comment_count = shared::entities::comments::Entity::find()
            .filter(shared::entities::comments::Column::MenuId.eq(menu.id))
            .filter(shared::entities::comments::Column::IsDeleted.eq(false))
            .count(db)
            .await.unwrap_or(0) as i32;

        let vote_stats: Option<(i32, i64, i64)> = menu_votes::Entity::find()
            .select_only()
            .column(menu_votes::Column::MenuId)
            .column_as(menu_votes::Column::Id.count(), "vote_count")
            .column_as(
                Expr::cust("SUM(CASE WHEN sentiment = 'positive' THEN 1 WHEN sentiment = 'negative' THEN -1 ELSE 0 END)"),
                "rating_sum",
            )
            .filter(menu_votes::Column::MenuId.eq(menu.id))
            .group_by(menu_votes::Column::MenuId)
            .into_tuple()
            .one(db)
            .await.unwrap_or_default();

        let my_vote = if let Some(uid) = user_id {
            menu_votes::Entity::find()
                .filter(menu_votes::Column::MenuId.eq(menu.id))
                .filter(menu_votes::Column::UserId.eq(uid))
                .one(db)
                .await
                .unwrap_or_default()
                .map(|v| match v.sentiment {
                    shared::entities::sea_orm_active_enums::SentimentEnum::Positive => "positive".to_string(),
                    shared::entities::sea_orm_active_enums::SentimentEnum::Negative => "negative".to_string(),
                    shared::entities::sea_orm_active_enums::SentimentEnum::Neutral => "neutral".to_string(),
                })
        } else {
            None
        };

        let rating_sum = vote_stats.map(|v| v.2 as i32).unwrap_or(0);
        let vote_count = vote_stats.map(|v| v.1 as i32).unwrap_or(0);

        let calorie_range = Self::format_calorie_range(menu.calorie_range_min, menu.calorie_range_max);
        let calculated_calories = Self::calculate_total_calories(&items);

        Ok(MenuResponseDto {
            id: menu.id,
            city_name: city.name,
            city_slug: city.slug.clone(),
            serve_date: menu.serve_date,
            meal_type: Self::map_meal_type(&menu.meal_type),
            source_type: menu.source_type.unwrap_or_else(|| "unknown".to_string()),
            status: Self::map_menu_status(&menu.status),
            bot_commentary: menu.bot_commentary.clone(),
            comment_count,
            rating_sum,
            vote_count,
            my_vote,
            items,
            takeaways,
            alternatives: vec![],
            calorie_range_min: menu.calorie_range_min,
            calorie_range_max: menu.calorie_range_max,
            calorie_range,
            calculated_calories,
        })
    }

    pub async fn get_daily_menus(
        db: &DatabaseConnection,
        city_id: i32,
        date: NaiveDate,
        dietary_type: Option<String>,
        _user_id: Option<uuid::Uuid>,
    ) -> Result<Vec<MenuResponseDto>, MenuError> {
        let menus_with_cities = Menus::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date))
            .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved))
            .order_by_asc(menus::Column::MealType)
            .find_also_related(cities::Entity)
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)?;
            
        if menus_with_cities.is_empty() {
            return Ok(vec![]);
        }
        
        let city = menus_with_cities[0].1.clone().ok_or(MenuError::NotFound)?;
        let menus: Vec<menus::Model> = menus_with_cities.into_iter().map(|(m, _)| m).collect();
        
        // N+1 sorgularını önlemek için menülere ait yemek, takma ad ve ana yemek kayıtları
        // hiyerarşik olarak tek seferde toplu yüklenir.
        let menu_dishes_groups = menus.load_many(
            menu_dishes::Entity::find().order_by_asc(menu_dishes::Column::OrderIndex),
            db
        ).await.map_err(MenuError::DatabaseError)?;

        let flat_menu_dishes: Vec<menu_dishes::Model> = menu_dishes_groups.iter().flatten().cloned().collect();

        let dish_aliases_opts = flat_menu_dishes.load_one(dish_aliases::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let flat_dish_aliases: Vec<dish_aliases::Model> = dish_aliases_opts.iter().flatten().cloned().collect();

        let dishes_opts = flat_dish_aliases.load_one(dishes::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let menu_ids: Vec<i32> = menus.iter().map(|m| m.id).collect();
        let comment_counts_map: HashMap<i32, i32> = if !menu_ids.is_empty() {
            comments::Entity::find()
                .select_only()
                .column(comments::Column::MenuId)
                .column_as(comments::Column::Id.count(), "count")
                .filter(comments::Column::MenuId.is_in(menu_ids.clone()))
                .filter(comments::Column::IsDeleted.eq(false))
                .group_by(comments::Column::MenuId)
                .into_tuple()
                .all(db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|(m_id, count): (i32, i64)| (m_id, count as i32))
                .collect()
        } else {
            HashMap::new()
        };

        let vote_stats_list: Vec<(i32, i64, i64)> = menu_votes::Entity::find()
            .select_only()
            .column(menu_votes::Column::MenuId)
            .column_as(menu_votes::Column::Id.count(), "vote_count")
            .column_as(
                Expr::cust("COALESCE(SUM(CASE WHEN sentiment = 'positive' THEN 1 WHEN sentiment = 'negative' THEN -1 ELSE 0 END), 0)"),
                "rating_sum",
            )
            .filter(menu_votes::Column::MenuId.is_in(menu_ids.clone()))
            .group_by(menu_votes::Column::MenuId)
            .into_tuple()
            .all(db)
            .await
            .unwrap_or_default();

        let mut vote_stats_map: HashMap<i32, (i32, i32)> = HashMap::new();
        for (m_id, count, sum) in vote_stats_list {
            vote_stats_map.insert(m_id, (count as i32, sum as i32));
        }

        let my_votes_map: HashMap<i32, String> = if let Some(uid) = _user_id {
            let user_votes = menu_votes::Entity::find()
                .filter(menu_votes::Column::MenuId.is_in(menu_ids.clone()))
                .filter(menu_votes::Column::UserId.eq(uid))
                .all(db)
                .await
                .unwrap_or_default();

            user_votes.into_iter().map(|v| {
                let sent_str = match v.sentiment {
                    shared::entities::sea_orm_active_enums::SentimentEnum::Positive => "positive".to_string(),
                    shared::entities::sea_orm_active_enums::SentimentEnum::Negative => "negative".to_string(),
                    shared::entities::sea_orm_active_enums::SentimentEnum::Neutral => "neutral".to_string(),
                };
                (v.menu_id, sent_str)
            }).collect()
        } else {
            HashMap::new()
        };

        let history_records: Vec<menu_history::Model> = MenuHistory::find()
            .filter(menu_history::Column::CityId.eq(city_id))
            .filter(menu_history::Column::ServeDate.eq(date))
            .order_by_desc(menu_history::Column::Id)
            .all(db)
            .await
            .unwrap_or_default();

        let mut history_map: HashMap<String, Vec<menu_history::Model>> = HashMap::new();
        for hist in history_records {
            history_map.entry(hist.meal_type.to_lowercase()).or_default().push(hist);
        }

        let mut result = Vec::with_capacity(menus.len());
        let mut alias_idx = 0;
        let mut dish_idx = 0;
        let is_celiac_mode = dietary_type.as_deref() == Some("celiac");

        let dish_ids: Vec<i32> = dishes_opts.iter().flatten().map(|d| d.id).collect();
        let dish_stats_map = Self::get_dish_vote_stats_map(db, &dish_ids).await;

        for (i, menu) in menus.into_iter().enumerate() {
            let mut items = Vec::with_capacity(menu_dishes_groups[i].len());
            let mut takeaway_map: HashMap<String, Vec<MenuItemDto>> = HashMap::new();
            
            for md in &menu_dishes_groups[i] {
                let alias_opt = &dish_aliases_opts[alias_idx];
                alias_idx += 1;
                
                if let Some(alias) = alias_opt {
                    let dish_opt = &dishes_opts[dish_idx];
                    dish_idx += 1;

                    if shared::services::content_guard::ContentGuard::is_junk_dish_text(&alias.name) {
                        continue;
                    }
                    
                    let master_data = dish_opt.as_ref().map(|dish| {
                        let stats = dish_stats_map
                            .get(&dish.id)
                            .copied()
                            .unwrap_or_default();

                        DishMasterDataDto {
                            dish_id: dish.id,
                            name: dish.name.clone(),
                            is_celiac: dish.is_celiac,
                            is_vegan: dish.is_vegan,
                            is_vegetarian: dish.is_vegetarian,
                            estimated_calories: dish.estimated_calories,
                            total_votes: stats.total,
                            positive_votes: stats.positive,
                            negative_votes: stats.negative,
                            dislike_ratio: stats.dislike_ratio,
                            like_ratio: stats.like_ratio,
                        }
                    });
                    
                    let dish_is_celiac = master_data.as_ref().is_some_and(|m| m.is_celiac);
                    let pkg_upper = md.package_name.to_uppercase();
                    let is_celiac_pkg = pkg_upper.contains("ÇÖLYAK") || pkg_upper.contains("COLYAK");
                    let is_takeaway_pkg = md.package_name != "NORMAL" && !is_celiac_pkg;
                    
                    let dish_category = dish_opt.as_ref().and_then(|dish| dish.category.clone());
                    let meal_type_str = match menu.meal_type {
                        MealTypeEnum::Breakfast => "breakfast",
                        MealTypeEnum::Lunch => "lunch",
                        MealTypeEnum::Dinner => "dinner",
                    };

                    let dish_name_for_pricing = dish_opt.as_ref().map(|d| d.name.as_str()).unwrap_or(alias.name.as_str());

                    let price_info = crate::services::pricing::get_pricing_info_for_city(
                        &city.slug,
                        Some(date),
                        meal_type_str,
                        dish_category.as_deref(),
                        dish_name_for_pricing
                    );

                    let amount = md.amount.clone().or_else(|| price_info.as_ref().map(|p| p.amount.clone()));
                    let price = price_info.map(|p| p.price as f64);
                    
                    let item_dto = MenuItemDto {
                        order_index: md.order_index,
                        raw_name: alias.name.clone(),
                        is_alternative: md.is_alternative,
                        amount,
                        calories: md.calories,
                        price,
                        category: dish_category,
                        master_data,
                    };
                    
                    if is_celiac_mode {
                        if is_celiac_pkg || (md.package_name == "NORMAL" && dish_is_celiac) {
                            if is_takeaway_pkg {
                                takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                            } else {
                                items.push(item_dto);
                            }
                        }
                    } else {
                        if !is_celiac_pkg {
                            if md.package_name != "NORMAL" {
                                takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                            } else {
                                items.push(item_dto);
                            }
                        }
                    }
                } else {
                    return Err(MenuError::DatabaseError(DbErr::Custom("Yabancı anahtar bozuk: Alias bulunamadı".into())));
                }
            }
            
            let mut takeaways = Vec::new();
            for (name, mut t_items) in takeaway_map {
                t_items.sort_by_key(|i| (i.order_index, i.is_alternative));
                takeaways.push(crate::dto::menu::TakeawayMenuDto { name, items: t_items });
            }
            takeaways.sort_by(|a, b| a.name.cmp(&b.name));
            
            let calorie_range = Self::format_calorie_range(menu.calorie_range_min, menu.calorie_range_max);
            let calculated_calories = Self::calculate_total_calories(&items);

            let (vote_count, rating_sum) = vote_stats_map.get(&menu.id).copied().unwrap_or((0, 0));
            let my_vote = my_votes_map.get(&menu.id).cloned();
            if items.is_empty() && takeaways.is_empty() {
                continue;
            }

            let meal_type_enum = Self::map_meal_type(&menu.meal_type);
            let meal_type_str = match menu.meal_type {
                MealTypeEnum::Breakfast => "breakfast",
                MealTypeEnum::Lunch => "lunch",
                MealTypeEnum::Dinner => "dinner",
            };

            let mut alternatives = Vec::new();
            if let Some(hist_list) = history_map.get(meal_type_str) {
                let current_src = menu.source_type.as_deref().unwrap_or("unknown");
                let main_sig = Self::compute_menu_dishes_signature(&items);
                let mut seen_signatures = HashSet::new();
                if !main_sig.is_empty() {
                    seen_signatures.insert(main_sig);
                }
                let mut seen_sources = HashSet::new();

                for hist in hist_list {
                    if hist.source_type != current_src && seen_sources.insert(hist.source_type.clone()) {
                        if let Some(alt_dto) = Self::parse_alternative_from_history(hist, meal_type_enum.clone()) {
                            let alt_sig = Self::compute_menu_dishes_signature(&alt_dto.items);
                            if !alt_sig.is_empty() && seen_signatures.insert(alt_sig) {
                                alternatives.push(alt_dto);
                            }
                        }
                    }
                }
            }

            result.push(MenuResponseDto {
                id: menu.id,
                city_name: city.name.clone(),
                city_slug: city.slug.clone(),
                serve_date: menu.serve_date,
                meal_type: meal_type_enum,
                source_type: menu.source_type.unwrap_or_else(|| "unknown".to_string()),
                status: Self::map_menu_status(&menu.status),
                bot_commentary: menu.bot_commentary.clone(),
                comment_count: *comment_counts_map.get(&menu.id).unwrap_or(&0),
                rating_sum,
                vote_count,
                my_vote,
                items,
                takeaways,
                alternatives,
                calorie_range_min: menu.calorie_range_min,
                calorie_range_max: menu.calorie_range_max,
                calorie_range,
                calculated_calories,
            });
        }

        Ok(result)
    }

    /// Filtrelenmiş menüleri çeker (city_slug, date, dietary_type, year, month)
    pub async fn get_menus_by_filter(
        db: &DatabaseConnection,
        city_slug: Option<String>,
        date: Option<NaiveDate>,
        dietary_type: Option<String>,
        year: Option<i32>,
        month: Option<u32>,
        user_id: Option<uuid::Uuid>,
    ) -> Result<Vec<MenuResponseDto>, MenuError> {
        let mut query = Menus::find()
            .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved));

        if let Some(slug) = city_slug {
            let city = Self::resolve_city(db, &slug).await.map_err(MenuError::DatabaseError)?;
                
            if let Some(c) = city {
                query = query.filter(menus::Column::CityId.eq(c.id));
            } else {
                return Ok(vec![]); // Şehir bulunamazsa boş dön
            }
        }

        if let Some(d) = date {
            query = query.filter(menus::Column::ServeDate.eq(d));
        } else if let (Some(y), Some(m)) = (year, month) {
            if let Some(start_date) = NaiveDate::from_ymd_opt(y, m, 1) {
                let next_m = if m == 12 { 1 } else { m + 1 };
                let next_y = if m == 12 { y + 1 } else { y };
                if let Some(end_date) = NaiveDate::from_ymd_opt(next_y, next_m, 1) {
                    query = query
                        .filter(menus::Column::ServeDate.gte(start_date))
                        .filter(menus::Column::ServeDate.lt(end_date));
                }
            }
        }

        let menus_with_cities = query
            .order_by_asc(menus::Column::ServeDate)
            .order_by_asc(menus::Column::MealType)
            .find_also_related(cities::Entity)
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)?;

        if menus_with_cities.is_empty() {
            return Ok(vec![]);
        }

        let menus: Vec<menus::Model> = menus_with_cities.iter().map(|(m, _)| m.clone()).collect();

        let dates: Vec<NaiveDate> = menus.iter().map(|m| m.serve_date).collect();
        let city_ids: Vec<i32> = menus.iter().map(|m| m.city_id).collect();
        let history_records: Vec<menu_history::Model> = if !dates.is_empty() && !city_ids.is_empty() {
            MenuHistory::find()
                .filter(menu_history::Column::CityId.is_in(city_ids))
                .filter(menu_history::Column::ServeDate.is_in(dates))
                .order_by_desc(menu_history::Column::Id)
                .all(db)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        let mut history_map: HashMap<(i32, NaiveDate, String), Vec<menu_history::Model>> = HashMap::new();
        for hist in history_records {
            let key = (hist.city_id, hist.serve_date, hist.meal_type.to_lowercase());
            history_map.entry(key).or_default().push(hist);
        }

        // N+1 sorgularını önlemek için menülerin yemekleri, takma adları, ana yemekleri
        // ve oy/yorum istatistikleri toplu olarak sorgulanır.
        let menu_dishes_groups = menus.load_many(
            menu_dishes::Entity::find().order_by_asc(menu_dishes::Column::OrderIndex),
            db
        ).await.map_err(MenuError::DatabaseError)?;

        let flat_menu_dishes: Vec<menu_dishes::Model> = menu_dishes_groups.iter().flatten().cloned().collect();

        let dish_aliases_opts = flat_menu_dishes.load_one(dish_aliases::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let flat_dish_aliases: Vec<dish_aliases::Model> = dish_aliases_opts.iter().flatten().cloned().collect();

        let dishes_opts = flat_dish_aliases.load_one(dishes::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let menu_ids: Vec<i32> = menus.iter().map(|m| m.id).collect();
        let comment_counts_map: HashMap<i32, i32> = if !menu_ids.is_empty() {
            comments::Entity::find()
                .select_only()
                .column(comments::Column::MenuId)
                .column_as(comments::Column::Id.count(), "count")
                .filter(comments::Column::MenuId.is_in(menu_ids.clone()))
                .filter(comments::Column::IsDeleted.eq(false))
                .group_by(comments::Column::MenuId)
                .into_tuple()
                .all(db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|(m_id, count): (i32, i64)| (m_id, count as i32))
                .collect()
        } else {
            HashMap::new()
        };

        let mut rating_sums = HashMap::new();
        let mut vote_counts = HashMap::new();
        if !menu_ids.is_empty() {
            let stats: Vec<(i32, i64, i64)> = menu_votes::Entity::find()
                .select_only()
                .column(menu_votes::Column::MenuId)
                .column_as(menu_votes::Column::Id.count(), "vote_count")
                .column_as(
                    Expr::cust("SUM(CASE WHEN sentiment = 'positive' THEN 1 WHEN sentiment = 'negative' THEN -1 ELSE 0 END)"),
                    "rating_sum",
                )
                .filter(menu_votes::Column::MenuId.is_in(menu_ids.clone()))
                .group_by(menu_votes::Column::MenuId)
                .into_tuple()
                .all(db)
                .await
                .unwrap_or_default();
            for (m_id, count, sum) in stats {
                vote_counts.insert(m_id, count as i32);
                rating_sums.insert(m_id, sum as i32);
            }
        }

        let mut my_votes_map = HashMap::new();
        if let Some(uid) = user_id {
            if !menu_ids.is_empty() {
                let votes = menu_votes::Entity::find()
                    .filter(menu_votes::Column::MenuId.is_in(menu_ids.clone()))
                    .filter(menu_votes::Column::UserId.eq(uid))
                    .all(db)
                    .await
                    .unwrap_or_default();
                for v in votes {
                    let sent_str = match v.sentiment {
                        shared::entities::sea_orm_active_enums::SentimentEnum::Positive => "positive".to_string(),
                        shared::entities::sea_orm_active_enums::SentimentEnum::Negative => "negative".to_string(),
                        shared::entities::sea_orm_active_enums::SentimentEnum::Neutral => "neutral".to_string(),
                    };
                    my_votes_map.insert(v.menu_id, sent_str);
                }
            }
        }

        let mut result = Vec::with_capacity(menus.len());
        let mut alias_idx = 0;
        let mut dish_idx = 0;

        let is_celiac_mode = dietary_type.as_deref() == Some("celiac");

        let dish_ids: Vec<i32> = dishes_opts.iter().flatten().map(|d| d.id).collect();
        let dish_stats_map = Self::get_dish_vote_stats_map(db, &dish_ids).await;

        for (i, (menu, city_opt)) in menus_with_cities.into_iter().enumerate() {
            if let Some(city) = city_opt {
                let mut items = Vec::with_capacity(menu_dishes_groups[i].len());
                let mut takeaway_map: HashMap<String, Vec<MenuItemDto>> = HashMap::new();
                
                for md in &menu_dishes_groups[i] {
                    let alias_opt = &dish_aliases_opts[alias_idx];
                    alias_idx += 1;
                    
                    if let Some(alias) = alias_opt {
                        let dish_opt = &dishes_opts[dish_idx];
                        dish_idx += 1;

                        if shared::services::content_guard::ContentGuard::is_junk_dish_text(&alias.name) {
                            continue;
                        }
                        
                        let master_data = dish_opt.as_ref().map(|dish| {
                            let stats = dish_stats_map
                                .get(&dish.id)
                                .copied()
                                .unwrap_or_default();

                            DishMasterDataDto {
                                dish_id: dish.id,
                                name: dish.name.clone(),
                                is_celiac: dish.is_celiac,
                                is_vegan: dish.is_vegan,
                                is_vegetarian: dish.is_vegetarian,
                                estimated_calories: dish.estimated_calories,
                                total_votes: stats.total,
                                positive_votes: stats.positive,
                                negative_votes: stats.negative,
                                dislike_ratio: stats.dislike_ratio,
                                like_ratio: stats.like_ratio,
                            }
                        });
                        
                        let dish_is_celiac = master_data.as_ref().is_some_and(|m| m.is_celiac);
                        let pkg_upper = md.package_name.to_uppercase();
                        let is_celiac_pkg = pkg_upper.contains("ÇÖLYAK") || pkg_upper.contains("COLYAK");
                        let is_takeaway_pkg = md.package_name != "NORMAL" && !is_celiac_pkg;
                        
                        let dish_category = dish_opt.as_ref().and_then(|dish| dish.category.clone());
                        let meal_type_str = match menu.meal_type {
                            MealTypeEnum::Breakfast => "breakfast",
                            MealTypeEnum::Lunch => "lunch",
                            MealTypeEnum::Dinner => "dinner",
                        };

                        let dish_name_for_pricing = dish_opt.as_ref().map(|d| d.name.as_str()).unwrap_or(alias.name.as_str());

                        let price_info = crate::services::pricing::get_pricing_info_for_city(
                            &city.slug,
                            Some(menu.serve_date),
                            meal_type_str,
                            dish_category.as_deref(),
                            dish_name_for_pricing
                        );

                        let amount = md.amount.clone().or_else(|| price_info.as_ref().map(|p| p.amount.clone()));
                        let price = price_info.map(|p| p.price as f64);
                        
                        let item_dto = MenuItemDto {
                            order_index: md.order_index,
                            raw_name: alias.name.clone(),
                            is_alternative: md.is_alternative,
                            amount,
                            calories: md.calories,
                            price,
                            category: dish_category,
                            master_data,
                        };
                        
                        if is_celiac_mode {
                            if is_celiac_pkg || (md.package_name == "NORMAL" && dish_is_celiac) {
                                if is_takeaway_pkg {
                                    takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                                } else {
                                    items.push(item_dto);
                                }
                            }
                        } else {
                            if !is_celiac_pkg {
                                if md.package_name != "NORMAL" {
                                    takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                                } else {
                                    items.push(item_dto);
                                }
                            }
                        }
                    } else {
                        return Err(MenuError::DatabaseError(DbErr::Custom("Yabancı anahtar bozuk: Alias bulunamadı".into())));
                    }
                }
                
                let mut takeaways = Vec::new();
                for (name, mut t_items) in takeaway_map {
                    t_items.sort_by_key(|i| (i.order_index, i.is_alternative));
                    takeaways.push(crate::dto::menu::TakeawayMenuDto { name, items: t_items });
                }
                takeaways.sort_by(|a, b| a.name.cmp(&b.name));
                
                let calorie_range = Self::format_calorie_range(menu.calorie_range_min, menu.calorie_range_max);
                let calculated_calories = Self::calculate_total_calories(&items);

                if items.is_empty() && takeaways.is_empty() {
                    continue;
                }

                let meal_type_enum = Self::map_meal_type(&menu.meal_type);
                let meal_type_str = match menu.meal_type {
                    MealTypeEnum::Breakfast => "breakfast",
                    MealTypeEnum::Lunch => "lunch",
                    MealTypeEnum::Dinner => "dinner",
                };

                let mut alternatives = Vec::new();
                if let Some(hist_list) = history_map.get(&(menu.city_id, menu.serve_date, meal_type_str.to_string())) {
                    let current_src = menu.source_type.as_deref().unwrap_or("unknown");
                    let main_sig = Self::compute_menu_dishes_signature(&items);
                    let mut seen_signatures = HashSet::new();
                    if !main_sig.is_empty() {
                        seen_signatures.insert(main_sig);
                    }
                    let mut seen_sources = HashSet::new();

                    for hist in hist_list {
                        if hist.source_type != current_src && seen_sources.insert(hist.source_type.clone()) {
                            if let Some(alt_dto) = Self::parse_alternative_from_history(hist, meal_type_enum.clone()) {
                                let alt_sig = Self::compute_menu_dishes_signature(&alt_dto.items);
                                if !alt_sig.is_empty() && seen_signatures.insert(alt_sig) {
                                    alternatives.push(alt_dto);
                                }
                            }
                        }
                    }
                }

                result.push(MenuResponseDto {
                    id: menu.id,
                    city_name: city.name,
                    city_slug: city.slug.clone(),
                    serve_date: menu.serve_date,
                    meal_type: meal_type_enum,
                    source_type: menu.source_type.unwrap_or_else(|| "unknown".to_string()),
                    status: Self::map_menu_status(&menu.status),
                    bot_commentary: menu.bot_commentary.clone(),
                    comment_count: *comment_counts_map.get(&menu.id).unwrap_or(&0),
                    rating_sum: *rating_sums.get(&menu.id).unwrap_or(&0),
                    vote_count: *vote_counts.get(&menu.id).unwrap_or(&0),
                    my_vote: my_votes_map.get(&menu.id).cloned(),
                    items,
                    takeaways,
                    alternatives,
                    calorie_range_min: menu.calorie_range_min,
                    calorie_range_max: menu.calorie_range_max,
                    calorie_range,
                    calculated_calories,
                });
            } else {
                // Şehir kaydı bulunamayan menü yanıttan çıkarılsa da düzleştirilmiş dizi indekslerinin senkron kalması için sayaçlar ilerletilir.
                for _ in &menu_dishes_groups[i] {
                    if dish_aliases_opts[alias_idx].is_some() {
                        dish_idx += 1;
                    }
                    alias_idx += 1;
                }
            }
        }

        Ok(result)
    }


    pub async fn get_archive_years(
        db: &DatabaseConnection,
        city_slug: Option<String>,
    ) -> Result<Vec<i32>, MenuError> {
        let mut query = Menus::find()
            .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved));
        
        if let Some(slug) = city_slug {
            let city = Self::resolve_city(db, &slug).await.map_err(MenuError::DatabaseError)?;
                
            if let Some(c) = city {
                query = query.filter(menus::Column::CityId.eq(c.id));
            } else {
                return Ok(vec![]);
            }
        }

        use sea_orm::{QuerySelect, QueryOrder, sea_query::Expr};
        let res: Vec<(i32,)> = query
            .select_only()
            .column_as(Expr::cust("EXTRACT(YEAR FROM serve_date)::int"), "year")
            .group_by(Expr::cust("EXTRACT(YEAR FROM serve_date)::int"))
            .order_by_desc(Expr::cust("EXTRACT(YEAR FROM serve_date)::int"))
            .into_tuple()
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)?;

        Ok(res.into_iter().map(|(y,)| y).collect())
    }

    /// Onaylı menüsü bulunan şehirlerin en son menü yıl ve ay bilgisini döner.
    /// Arşiv sayfasındaki hızlı keşif kartlarını dinamik ve geçerli verilerle besler.
    pub async fn get_archive_highlights(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<ArchiveHighlightDto>, MenuError> {
        let safe_limit = limit.clamp(1, 12) as i64;
        let sql = match db.get_database_backend() {
            DatabaseBackend::Sqlite => r#"
                SELECT c.slug AS city_slug, c.name AS city_name,
                       CAST(strftime('%Y', m.max_date) AS INTEGER) AS year,
                       CAST(strftime('%m', m.max_date) AS INTEGER) AS month
                FROM (
                    SELECT city_id, MAX(serve_date) as max_date
                    FROM menus
                    WHERE status = 'approved'
                    GROUP BY city_id
                ) m
                JOIN cities c ON c.id = m.city_id
                ORDER BY RANDOM()
                LIMIT ?
            "#,
            _ => r#"
                SELECT c.slug AS city_slug, c.name AS city_name,
                       EXTRACT(YEAR FROM m.max_date)::int AS year,
                       EXTRACT(MONTH FROM m.max_date)::int AS month
                FROM (
                    SELECT city_id, MAX(serve_date) as max_date
                    FROM menus
                    WHERE status = 'approved'
                    GROUP BY city_id
                ) m
                JOIN cities c ON c.id = m.city_id
                ORDER BY RANDOM()
                LIMIT $1
            "#,
        };

        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            vec![safe_limit.into()],
        );

        ArchiveHighlightDto::find_by_statement(query)
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::menu::{DishMasterDataDto, MenuItemDto};

    fn make_test_item(order_index: i32, is_alternative: bool, calories: Option<i32>) -> MenuItemDto {
        MenuItemDto {
            order_index,
            raw_name: "Test Yemek".into(),
            is_alternative,
            amount: None,
            calories,
            price: None,
            category: None,
            master_data: calories.map(|c| DishMasterDataDto {
                dish_id: 1,
                name: "Test Yemek".into(),
                is_celiac: false,
                is_vegan: false,
                is_vegetarian: false,
                estimated_calories: Some(c),
                total_votes: 0,
                positive_votes: 0,
                negative_votes: 0,
                dislike_ratio: None,
                like_ratio: None,
            }),
        }
    }

    #[test]
    fn test_calculate_total_calories_empty() {
        assert_eq!(MenuService::calculate_total_calories(&[]), None);
    }

    #[test]
    fn test_calculate_total_calories_no_calories() {
        let items = vec![
            make_test_item(1, false, None),
            make_test_item(2, false, None),
        ];
        assert_eq!(MenuService::calculate_total_calories(&items), None);
    }

    #[test]
    fn test_calculate_total_calories_sums_standard_items() {
        let items = vec![
            make_test_item(1, false, Some(250)),
            make_test_item(2, false, Some(450)),
            make_test_item(3, false, None),
        ];
        assert_eq!(MenuService::calculate_total_calories(&items), Some(700));
    }

    #[test]
    fn test_calculate_total_calories_ignores_alternatives() {
        let items = vec![
            make_test_item(1, false, Some(250)),
            make_test_item(2, true, Some(600)),
            make_test_item(3, false, Some(350)),
        ];
        assert_eq!(MenuService::calculate_total_calories(&items), Some(600));
    }

    #[test]
    fn test_parse_alternative_from_history_dishes_array() {
        let hist = shared::entities::menu_history::Model {
            id: 10,
            city_id: 34,
            serve_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 13).unwrap(),
            meal_type: "dinner".into(),
            source_type: "kykyemek".into(),
            submitted_by: None,
            dishes_payload: serde_json::json!({
                "dishes": [
                    { "name": "Mercimek Çorbası", "is_alternative": false },
                    { "name": "Kuru Fasulye", "is_alternative": false }
                ]
            }),
            created_at: None,
        };

        let alt = MenuService::parse_alternative_from_history(&hist, MealType::Dinner).unwrap();
        assert_eq!(alt.id, Some(10));
        assert_eq!(alt.source_type, "kykyemek");
        assert_eq!(alt.meal_type, MealType::Dinner);
        assert_eq!(alt.items.len(), 2);
        assert_eq!(alt.items[0].raw_name, "Mercimek Çorbası");
        assert_eq!(alt.items[1].raw_name, "Kuru Fasulye");
    }

    #[test]
    fn test_parse_alternative_from_history_filters_junk() {
        let hist = shared::entities::menu_history::Model {
            id: 11,
            city_id: 34,
            serve_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 13).unwrap(),
            meal_type: "dinner".into(),
            source_type: "kykmenu".into(),
            submitted_by: None,
            dishes_payload: serde_json::json!([
                "Veri yok. Menüye sahipseniz bize yazın",
                "Ezogelin Çorbası"
            ]),
            created_at: None,
        };

        let alt = MenuService::parse_alternative_from_history(&hist, MealType::Dinner).unwrap();
        assert_eq!(alt.items.len(), 1);
        assert_eq!(alt.items[0].raw_name, "Ezogelin Çorbası");
    }

    #[test]
    fn test_compute_menu_dishes_signature_deduplication() {
        let items1 = vec![
            MenuItemDto {
                order_index: 0,
                raw_name: "Mercimek Çorbası".into(),
                is_alternative: false,
                amount: None,
                calories: None,
                price: None,
                category: None,
                master_data: None,
            },
            MenuItemDto {
                order_index: 1,
                raw_name: "Pirinç Pilavı".into(),
                is_alternative: false,
                amount: None,
                calories: None,
                price: None,
                category: None,
                master_data: None,
            },
        ];

        let items2 = vec![
            MenuItemDto {
                order_index: 0,
                raw_name: "  pirinç pilavı  ".into(),
                is_alternative: false,
                amount: None,
                calories: None,
                price: None,
                category: None,
                master_data: None,
            },
            MenuItemDto {
                order_index: 1,
                raw_name: "Mercimek Çorbası.".into(),
                is_alternative: false,
                amount: None,
                calories: None,
                price: None,
                category: None,
                master_data: None,
            },
        ];

        assert_eq!(
            MenuService::compute_menu_dishes_signature(&items1),
            MenuService::compute_menu_dishes_signature(&items2)
        );
    }
}


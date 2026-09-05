// Kepçe API - Service: Menü Servisi
// ===================================
//
// Menü verilerini okuma ve formatlama işlemleri.
//
// Sorumlulukları:
//   1. Bugünkü menüyü getirme (şehre göre)
//   2. Arşiv menülerini listeleme (tarih aralığı, sayfalama)
//   3. Tek menü detayı (ID ile)
//   4. Menü öğelerini dish ilişkileriyle birlikte çekme
//   5. MenuDto'ya dönüştürme
//
// Menü yazma (upsert) işlemleri worker tarafındadır,
// burada sadece okuma + formatlama var.

use sea_orm::*;
use std::collections::HashMap;
use chrono::NaiveDate;
use sea_orm::sea_query::Expr;
use shared::entities::{
    prelude::*, menus, menu_dishes, dish_aliases, dishes, cities, sea_orm_active_enums::MealTypeEnum, comments, menu_votes, dish_votes,
};
use crate::dto::menu::{MenuResponseDto, MenuItemDto, DishMasterDataDto, MealType};

#[derive(Debug)]
pub enum MenuError {
    NotFound,
    DatabaseError(DbErr),
}

pub struct MenuService;

impl MenuService {
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

    fn calculate_total_calories(items: &[MenuItemDto]) -> Option<i32> {
        let mut total = 0;
        let mut has_calories = false;
        for item in items {
            if let Some(cal) = item.calories {
                total += cal;
                has_calories = true;
            }
        }
        if has_calories { Some(total) } else { None }
    }

    async fn get_dish_vote_stats_map(
        db: &DatabaseConnection,
        dish_ids: &[i32],
    ) -> HashMap<i32, (i32, i32, i32, Option<f64>, Option<f64>)> {
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
            map.insert(d_id, (total_i32, pos_i32, neg_i32, dislike_ratio, like_ratio));
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

        // Menüdeki yemekleri (order_index'e göre sıralı) ve o yemeğin "Alias" verisini çekiyoruz
        let menu_dishes_with_aliases = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(menu_id))
            .find_also_related(dish_aliases::Entity)
            .order_by_asc(menu_dishes::Column::OrderIndex)
            .all(db)
            .await
            .map_err(MenuError::DatabaseError)?;
            
        // Master (asıl) yemekleri bulk olarak çekmek için id'leri topluyoruz
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
        
        // Response formatına (DTO) çeviriyoruz
        let mut items = Vec::new();
        let mut takeaway_map: HashMap<String, Vec<MenuItemDto>> = HashMap::new();
        for (md, alias_opt) in menu_dishes_with_aliases {
            let alias = alias_opt.ok_or_else(|| MenuError::DatabaseError(DbErr::Custom("Yabancı anahtar bozuk: Alias bulunamadı".into())))?;
            
            let master_data = alias.dish_id.and_then(|did| master_map.get(&did)).map(|dish| {
                let (tot, pos, neg, d_ratio, l_ratio) = dish_stats_map
                    .get(&dish.id)
                    .copied()
                    .unwrap_or((0, 0, 0, None, None));

                DishMasterDataDto {
                    dish_id: dish.id,
                    name: dish.name.clone(),
                    is_celiac: dish.is_celiac,
                    is_vegan: dish.is_vegan,
                    is_vegetarian: dish.is_vegetarian,
                    estimated_calories: dish.estimated_calories,
                    total_votes: tot,
                    positive_votes: pos,
                    negative_votes: neg,
                    dislike_ratio: d_ratio,
                    like_ratio: l_ratio,
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
            let price = price_info.map(|p| format!("{:.2} ₺", p.price));

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
                // Standard Mode
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
        
        // 1. MenuDishes load
        let menu_dishes_groups = menus.load_many(
            menu_dishes::Entity::find().order_by_asc(menu_dishes::Column::OrderIndex),
            db
        ).await.map_err(MenuError::DatabaseError)?;

        let flat_menu_dishes: Vec<menu_dishes::Model> = menu_dishes_groups.iter().flatten().cloned().collect();

        // 2. DishAliases load
        let dish_aliases_opts = flat_menu_dishes.load_one(dish_aliases::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let flat_dish_aliases: Vec<dish_aliases::Model> = dish_aliases_opts.iter().flatten().cloned().collect();

        // 3. Dishes load
        let dishes_opts = flat_dish_aliases.load_one(dishes::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        // 4. Comments and Votes load
        let menu_ids: Vec<i32> = menus.iter().map(|m| m.id).collect();
        let mut comment_counts = Vec::with_capacity(menus.len());
        for m in &menus {
            let c = comments::Entity::find()
                .filter(comments::Column::MenuId.eq(m.id))
                .filter(comments::Column::IsDeleted.eq(false))
                .count(db)
                .await.unwrap_or(0);
            comment_counts.push(c as i32);
        }

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
                    
                    let master_data = dish_opt.as_ref().map(|dish| {
                        let (tot, pos, neg, d_ratio, l_ratio) = dish_stats_map
                            .get(&dish.id)
                            .copied()
                            .unwrap_or((0, 0, 0, None, None));

                        DishMasterDataDto {
                            dish_id: dish.id,
                            name: dish.name.clone(),
                            is_celiac: dish.is_celiac,
                            is_vegan: dish.is_vegan,
                            is_vegetarian: dish.is_vegetarian,
                            estimated_calories: dish.estimated_calories,
                            total_votes: tot,
                            positive_votes: pos,
                            negative_votes: neg,
                            dislike_ratio: d_ratio,
                            like_ratio: l_ratio,
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
                    let price = price_info.map(|p| format!("{:.2} ₺", p.price));
                    
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
                        // Standard Mode
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

            result.push(MenuResponseDto {
                id: menu.id,
                city_name: city.name.clone(),
                city_slug: city.slug.clone(),
                serve_date: menu.serve_date,
                meal_type: Self::map_meal_type(&menu.meal_type),
                source_type: menu.source_type.unwrap_or_else(|| "unknown".to_string()),
                status: Self::map_menu_status(&menu.status),
                bot_commentary: menu.bot_commentary.clone(),
                comment_count: comment_counts[i],
                rating_sum,
                vote_count,
                my_vote,
                items,
                takeaways,
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
        _dietary_type: Option<String>,
        year: Option<i32>,
        month: Option<u32>,
        user_id: Option<uuid::Uuid>,
    ) -> Result<Vec<MenuResponseDto>, MenuError> {
        let mut query = Menus::find()
            .filter(menus::Column::Status.eq(shared::entities::sea_orm_active_enums::MenuStatusEnum::Approved));

        if let Some(slug) = city_slug {
            let city = cities::Entity::find()
                .filter(cities::Column::Slug.eq(&slug))
                .one(db)
                .await
                .map_err(MenuError::DatabaseError)?;
                
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

        // V2'de dietary_type menus tablosundan kalktı.
        // let diet = dietary_type.unwrap_or_else(|| "standard".to_string());
        // query = query.filter(menus::Column::DietaryType.eq(diet));

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

        // 1. MenuDishes load
        let menu_dishes_groups = menus.load_many(
            menu_dishes::Entity::find().order_by_asc(menu_dishes::Column::OrderIndex),
            db
        ).await.map_err(MenuError::DatabaseError)?;

        let flat_menu_dishes: Vec<menu_dishes::Model> = menu_dishes_groups.iter().flatten().cloned().collect();

        // 2. DishAliases load
        let dish_aliases_opts = flat_menu_dishes.load_one(dish_aliases::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        let flat_dish_aliases: Vec<dish_aliases::Model> = dish_aliases_opts.iter().flatten().cloned().collect();

        // 3. Dishes load
        let dishes_opts = flat_dish_aliases.load_one(dishes::Entity, db)
            .await.map_err(MenuError::DatabaseError)?;

        // 4. Comments load
        let menu_ids: Vec<i32> = menus.iter().map(|m| m.id).collect();
        let mut comment_counts = Vec::with_capacity(menus.len());
        for m in &menus {
            let c = comments::Entity::find()
                .filter(comments::Column::MenuId.eq(m.id))
                .filter(comments::Column::IsDeleted.eq(false))
                .count(db)
                .await.unwrap_or(0);
            comment_counts.push(c as i32);
        }

        // 5. Vote stats load
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
                .await.unwrap_or_default();
            for (m_id, count, sum) in stats {
                vote_counts.insert(m_id, count as i32);
                rating_sums.insert(m_id, sum as i32);
            }
        }

        // 6. My Votes load
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

        let is_celiac_mode = _dietary_type.as_deref() == Some("celiac");

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
                        
                        let master_data = dish_opt.as_ref().map(|dish| {
                            let (tot, pos, neg, d_ratio, l_ratio) = dish_stats_map
                                .get(&dish.id)
                                .copied()
                                .unwrap_or((0, 0, 0, None, None));

                            DishMasterDataDto {
                                dish_id: dish.id,
                                name: dish.name.clone(),
                                is_celiac: dish.is_celiac,
                                is_vegan: dish.is_vegan,
                                is_vegetarian: dish.is_vegetarian,
                                estimated_calories: dish.estimated_calories,
                                total_votes: tot,
                                positive_votes: pos,
                                negative_votes: neg,
                                dislike_ratio: d_ratio,
                                like_ratio: l_ratio,
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
                        let price = price_info.map(|p| format!("{:.2} ₺", p.price));
                        
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

                        tracing::info!("DEBUG: celiac_mode={}, pkg={}, is_celiac_pkg={}, is_takeaway={}", 
                            is_celiac_mode, md.package_name, is_celiac_pkg, is_takeaway_pkg);
                        
                        if is_celiac_mode {
                            if is_celiac_pkg || (md.package_name == "NORMAL" && dish_is_celiac) {
                                if is_takeaway_pkg {
                                    takeaway_map.entry(md.package_name.clone()).or_default().push(item_dto);
                                } else {
                                    items.push(item_dto);
                                }
                            }
                        } else {
                            // Standard Mode
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

                result.push(MenuResponseDto {
                    id: menu.id,
                    city_name: city.name,
                    city_slug: city.slug.clone(),
                    serve_date: menu.serve_date,
                    meal_type: Self::map_meal_type(&menu.meal_type),
                    source_type: menu.source_type.unwrap_or_else(|| "unknown".to_string()),
                    status: Self::map_menu_status(&menu.status),
                    bot_commentary: menu.bot_commentary.clone(),
                    comment_count: comment_counts[i],
                    rating_sum: *rating_sums.get(&menu.id).unwrap_or(&0),
                    vote_count: *vote_counts.get(&menu.id).unwrap_or(&0),
                    my_vote: my_votes_map.get(&menu.id).cloned(),
                    items,
                    takeaways,
                    calorie_range_min: menu.calorie_range_min,
                    calorie_range_max: menu.calorie_range_max,
                    calorie_range,
                    calculated_calories,
                });
            } else {
                // If city is None, skip it but still consume indices correctly
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
            let city = cities::Entity::find()
                .filter(cities::Column::Slug.eq(&slug))
                .one(db)
                .await
                .map_err(MenuError::DatabaseError)?;
                
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
}

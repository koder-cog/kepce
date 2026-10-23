use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};
use shared::entities::{
    cities, dish_aliases, menu_dishes, menus,
    sea_orm_active_enums::{MealTypeEnum, MenuStatusEnum},
};

use crate::dto::internal_ingest::{
    InternalIngestResponseDto, InternalKykyemekIngestRequest, InternalMenuDto,
};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum DishSlotKey {
    Primary(String, i32),
    Alternative(String, i32, i32),
}

fn sanitize_dish_name(name: &str) -> String {
    static RE_TAG: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE_TAG.get_or_init(|| regex::Regex::new(r"</?[a-zA-Z0-9]+(?:\s+[^>]*)?>").unwrap());
    let result = re.replace_all(name, "").into_owned();

    let decoded = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    static RE_PLUS: OnceLock<regex::Regex> = OnceLock::new();
    let re_plus = RE_PLUS.get_or_init(|| regex::Regex::new(r"\s*\+\s*").unwrap());
    let spaced = re_plus.replace_all(&decoded, " + ").into_owned();

    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_dish_calories(raw: &Option<String>) -> Option<i32> {
    let s = raw.as_ref()?;
    let cleaned = s
        .to_lowercase()
        .replace("kcal", "")
        .replace("kkal", "")
        .replace("kalori", "")
        .trim()
        .to_string();
    if let Ok(v) = cleaned.parse::<i32>() {
        return Some(v);
    }
    if cleaned.contains('/') {
        for part in cleaned.split('/') {
            if let Ok(v) = part.trim().parse::<i32>() {
                return Some(v);
            }
        }
    }
    let parts: Vec<&str> = cleaned.split(&['-', '–'][..]).map(|p| p.trim()).collect();
    if parts.len() == 2
        && let (Ok(a), Ok(b)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
    {
        return Some((a + b) / 2);
    }
    None
}

async fn get_or_create_dish_alias(
    txn: &sea_orm::DatabaseTransaction,
    raw_name: &str,
    category: Option<String>,
) -> anyhow::Result<(i32, i32)> {
    let sanitized = sanitize_dish_name(raw_name);
    let canonical = shared::services::normalizer::normalize_food_name(&sanitized);
    let final_dish_name = if canonical.is_empty() {
        sanitized.clone()
    } else {
        canonical
    };
    let final_category =
        category.or_else(|| shared::services::categorizer::categorize_dish(&final_dish_name));

    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        r#"
        WITH upsert_dish AS (
            INSERT INTO dishes (name, category) VALUES ($1, $2)
            ON CONFLICT ((LOWER(TRIM(name)))) DO UPDATE SET category = COALESCE(dishes.category, EXCLUDED.category)
            RETURNING id
        )
        INSERT INTO dish_aliases (name, dish_id)
        VALUES ($3, (SELECT id FROM upsert_dish))
        ON CONFLICT (name) DO UPDATE SET dish_id = COALESCE(dish_aliases.dish_id, EXCLUDED.dish_id)
        RETURNING id, dish_id;
        "#,
        vec![
            final_dish_name.into(),
            final_category.into(),
            sanitized.into(),
        ],
    );

    let query_res = txn.query_one(stmt).await?;
    if let Some(row) = query_res {
        let alias_id: i32 = row.try_get("", "id")?;
        let dish_id: i32 = row.try_get("", "dish_id")?;
        Ok((alias_id, dish_id))
    } else {
        Err(anyhow::anyhow!("Upsert işlemi alias ID döndüremedi"))
    }
}

pub struct InternalIngestService;

impl InternalIngestService {
    pub async fn ingest_kykyemek(
        db: &DatabaseConnection,
        payload: InternalKykyemekIngestRequest,
    ) -> anyhow::Result<InternalIngestResponseDto> {
        let source_type = payload
            .source_type
            .unwrap_or_else(|| "kykyemek".to_string());

        let total_received = payload.menus.len();
        let mut total_inserted = 0;
        let mut total_updated = 0;
        let mut total_skipped = 0;
        let mut errors = Vec::new();

        // Şehir eşlemesi için yerel önbellek
        let mut city_cache: HashMap<String, i32> = HashMap::new();

        for menu_dto in payload.menus {
            match Self::process_single_menu(db, &mut city_cache, &source_type, menu_dto).await {
                Ok(Some(true)) => total_inserted += 1,
                Ok(Some(false)) => total_updated += 1,
                Ok(None) => total_skipped += 1,
                Err(err) => {
                    tracing::warn!("İç menü aktarım hatası: {:?}", err);
                    errors.push(err.to_string());
                }
            }
        }

        Ok(InternalIngestResponseDto {
            total_received,
            total_inserted,
            total_updated,
            total_skipped,
            errors,
        })
    }

    async fn process_single_menu(
        db: &DatabaseConnection,
        city_cache: &mut HashMap<String, i32>,
        source_type: &str,
        menu_dto: InternalMenuDto,
    ) -> anyhow::Result<Option<bool>> {
        let city_id = if let Some(&id) = city_cache.get(&menu_dto.city_slug) {
            id
        } else {
            let city_opt = cities::Entity::find()
                .filter(cities::Column::Slug.eq(&menu_dto.city_slug))
                .one(db)
                .await?;
            let city = city_opt
                .ok_or_else(|| anyhow::anyhow!("Şehir slug bulunamadı: {}", menu_dto.city_slug))?;
            city_cache.insert(menu_dto.city_slug.clone(), city.id);
            city.id
        };

        let date = NaiveDate::parse_from_str(&menu_dto.serve_date, "%Y-%m-%d").map_err(|e| {
            anyhow::anyhow!("Geçersiz tarih formatı ({}): {:?}", menu_dto.serve_date, e)
        })?;

        let meal_type = match menu_dto.meal_type.to_lowercase().as_str() {
            "breakfast" | "kahvaltı" => MealTypeEnum::Breakfast,
            "lunch" | "öğle" => MealTypeEnum::Lunch,
            "dinner" | "akşam" => MealTypeEnum::Dinner,
            other => return Err(anyhow::anyhow!("Bilinmeyen öğün türü: {}", other)),
        };

        // 1. ContentGuard ile çöp metin filtresi
        let dishes: Vec<Vec<_>> = menu_dto
            .dishes
            .into_iter()
            .map(|group| {
                group
                    .into_iter()
                    .filter(|c| {
                        let trimmed = c.name.trim();
                        !trimmed.is_empty()
                            && !shared::services::content_guard::ContentGuard::is_junk_dish_text(
                                trimmed,
                            )
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|group| !group.is_empty())
            .collect();

        let celiac_dishes: Vec<Vec<_>> = menu_dto
            .celiac_dishes
            .into_iter()
            .map(|group| {
                group
                    .into_iter()
                    .filter(|c| {
                        let trimmed = c.name.trim();
                        !trimmed.is_empty()
                            && !shared::services::content_guard::ContentGuard::is_junk_dish_text(
                                trimmed,
                            )
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|group| !group.is_empty())
            .collect();

        let takeaways: Vec<(String, Vec<Vec<_>>)> = menu_dto
            .takeaways
            .into_iter()
            .map(|pkg| {
                let filtered_groups: Vec<Vec<_>> = pkg
                    .dishes
                    .into_iter()
                    .map(|group| {
                        group
                            .into_iter()
                            .filter(|c| {
                                let trimmed = c.name.trim();
                                !trimmed.is_empty()
                                    && !shared::services::content_guard::ContentGuard::is_junk_dish_text(
                                        trimmed,
                                    )
                            })
                            .collect::<Vec<_>>()
                    })
                    .filter(|group| !group.is_empty())
                    .collect();
                (pkg.package_name, filtered_groups)
            })
            .filter(|(_, groups)| !groups.is_empty())
            .collect();

        if dishes.len() < 2 && celiac_dishes.is_empty() && takeaways.is_empty() {
            tracing::warn!(
                "Yetersiz/çöp menü atlandı (city_id: {}, tarih: {}, öğün: {:?})",
                city_id,
                date,
                meal_type
            );
            return Ok(None);
        }

        let meta = shared::services::source_registry::SourceRegistry::resolve(source_type);
        let target_status = if meta.is_auto_approvable {
            MenuStatusEnum::Approved
        } else {
            MenuStatusEnum::Pending
        };

        let txn = db.begin().await?;

        let existing_menu = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::ServeDate.eq(date))
            .filter(menus::Column::MealType.eq(meal_type.clone()))
            .one(&txn)
            .await?;

        let mut existing_map = HashMap::new();
        let mut existing_dishes_list = Vec::new();

        if let Some(ref m) = existing_menu {
            if m.status == MenuStatusEnum::Rejected && m.source_type.as_deref() == Some(source_type)
            {
                tracing::debug!(
                    "Menü reddedilmiş olduğundan güncelleme atlandı (city_id: {}, tarih: {}, öğün: {:?})",
                    city_id,
                    date,
                    meal_type
                );
                txn.rollback().await?;
                return Ok(None);
            }

            let existing_source = m.source_type.as_deref().unwrap_or("");
            let existing_meta =
                shared::services::source_registry::SourceRegistry::resolve(existing_source);
            let incoming_meta =
                shared::services::source_registry::SourceRegistry::resolve(source_type);

            // Saha Gerçeği Koruması: Mevcut menü saha teyitliyse (GroundTruth), harici kazıyıcı ezemez.
            if existing_meta.tier == shared::services::source_registry::TrustTier::GroundTruth
                && incoming_meta.tier < shared::services::source_registry::TrustTier::GroundTruth
            {
                tracing::debug!(
                    "Saha gerçeği koruması: Mevcut menü ({}) GroundTruth, gelen ({}) ezemez. Güncelleme atlandı.",
                    existing_source,
                    source_type
                );
                txn.rollback().await?;
                return Ok(None);
            }

            let existing_dishes = menu_dishes::Entity::find()
                .filter(menu_dishes::Column::MenuId.eq(m.id))
                .find_also_related(dish_aliases::Entity)
                .all(&txn)
                .await?;

            for (d, _) in &existing_dishes {
                let key = if d.is_alternative {
                    DishSlotKey::Alternative(d.package_name.clone(), d.order_index, d.dish_alias_id)
                } else {
                    DishSlotKey::Primary(d.package_name.clone(), d.order_index)
                };
                existing_map.insert(key, d.clone());
            }
            existing_dishes_list = existing_dishes;
        }

        // target_map oluştur
        let mut target_map: HashMap<DishSlotKey, (i32, i32, Option<String>, Option<i32>)> =
            HashMap::new();
        let mut seen_package_dishes: HashSet<(String, i32, i32)> = HashSet::new();
        let mut seen_package_aliases: HashSet<(String, i32)> = HashSet::new();

        for (i, dish_group) in dishes.into_iter().enumerate() {
            let order_index = i as i32;
            for (j, comp) in dish_group.into_iter().enumerate() {
                let is_alternative = j > 0;
                let (alias_id, dish_id) =
                    get_or_create_dish_alias(&txn, &comp.name, comp.category).await?;
                let package_name = "NORMAL".to_string();
                let cals = parse_dish_calories(&comp.calories);

                if !seen_package_dishes.insert((package_name.clone(), order_index, dish_id)) {
                    continue;
                }

                let key = if is_alternative {
                    DishSlotKey::Alternative(package_name.clone(), order_index, alias_id)
                } else {
                    DishSlotKey::Primary(package_name.clone(), order_index)
                };

                if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                    *existing_amt = comp
                        .amount
                        .filter(|s| !s.trim().is_empty())
                        .or(existing_amt.take());
                    *existing_cals = cals.or(*existing_cals);
                } else if seen_package_aliases.insert((package_name, alias_id)) {
                    target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
                }
            }
        }

        for (i, dish_group) in celiac_dishes.into_iter().enumerate() {
            let order_index = i as i32;
            for (j, comp) in dish_group.into_iter().enumerate() {
                let is_alternative = j > 0;
                let (alias_id, dish_id) =
                    get_or_create_dish_alias(&txn, &comp.name, comp.category).await?;
                let package_name = "ÇÖLYAK MENÜSÜ".to_string();
                let cals = parse_dish_calories(&comp.calories);

                if !seen_package_dishes.insert((package_name.clone(), order_index, dish_id)) {
                    continue;
                }

                let key = if is_alternative {
                    DishSlotKey::Alternative(package_name.clone(), order_index, alias_id)
                } else {
                    DishSlotKey::Primary(package_name.clone(), order_index)
                };

                if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                    *existing_amt = comp
                        .amount
                        .filter(|s| !s.trim().is_empty())
                        .or(existing_amt.take());
                    *existing_cals = cals.or(*existing_cals);
                } else if seen_package_aliases.insert((package_name, alias_id)) {
                    target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
                }
            }
        }

        for (package, package_dishes) in takeaways.into_iter() {
            let sanitized_package = sanitize_dish_name(&package);
            for (i, dish_group) in package_dishes.into_iter().enumerate() {
                let order_index = i as i32;
                for (j, comp) in dish_group.into_iter().enumerate() {
                    let is_alternative = j > 0;
                    let (alias_id, dish_id) =
                        get_or_create_dish_alias(&txn, &comp.name, comp.category).await?;
                    let cals = parse_dish_calories(&comp.calories);

                    if !seen_package_dishes.insert((
                        sanitized_package.clone(),
                        order_index,
                        dish_id,
                    )) {
                        continue;
                    }

                    let key = if is_alternative {
                        DishSlotKey::Alternative(sanitized_package.clone(), order_index, alias_id)
                    } else {
                        DishSlotKey::Primary(sanitized_package.clone(), order_index)
                    };

                    if let Some((_, _, existing_amt, existing_cals)) = target_map.get_mut(&key) {
                        *existing_amt = comp
                            .amount
                            .filter(|s| !s.trim().is_empty())
                            .or(existing_amt.take());
                        *existing_cals = cals.or(*existing_cals);
                    } else if seen_package_aliases.insert((sanitized_package.clone(), alias_id)) {
                        target_map.insert(key, (alias_id, dish_id, comp.amount, cals));
                    }
                }
            }
        }

        let is_inserted;
        let menu_id = if let Some(ref m) = existing_menu {
            let same_len = existing_map.len() == target_map.len();
            let same_dishes = same_len
                && target_map
                    .iter()
                    .all(|(key, (alias_id, _dish_id, amt, cals))| {
                        if let Some(existing) = existing_map.get(key) {
                            existing.dish_alias_id == *alias_id
                                && (amt.is_none() || amt.as_deref() == existing.amount.as_deref())
                                && (cals.is_none() || *cals == existing.calories)
                        } else {
                            false
                        }
                    });
            let same_calories = m.calorie_range_min == menu_dto.calorie_range_min
                && m.calorie_range_max == menu_dto.calorie_range_max;
            let same_source = m.source_type.as_deref() == Some(source_type);

            if same_dishes && same_calories && same_source {
                txn.rollback().await?;
                return Ok(None);
            }

            // Mevcut durumu geçmişe arşivle
            let payload = serde_json::json!(
                existing_dishes_list
                    .iter()
                    .map(|(md, alias)| {
                        serde_json::json!({
                            "name": alias.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                            "package_name": md.package_name.clone(),
                            "order_index": md.order_index,
                            "is_alternative": md.is_alternative
                        })
                    })
                    .collect::<Vec<_>>()
            );

            let hist = shared::entities::menu_history::ActiveModel {
                city_id: Set(m.city_id),
                serve_date: Set(m.serve_date),
                meal_type: Set(match m.meal_type {
                    MealTypeEnum::Breakfast => "breakfast".to_string(),
                    MealTypeEnum::Lunch => "lunch".to_string(),
                    MealTypeEnum::Dinner => "dinner".to_string(),
                }),
                source_type: Set(m
                    .source_type
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())),
                submitted_by: Set(m.submitted_by),
                dishes_payload: Set(payload),
                ..Default::default()
            };
            hist.insert(&txn).await?;

            let mut update_m: menus::ActiveModel = m.clone().into();
            update_m.source_type = Set(Some(source_type.to_string()));
            update_m.status = Set(target_status.clone());
            if menu_dto.calorie_range_min.is_some() || menu_dto.calorie_range_max.is_some() {
                update_m.calorie_range_min =
                    Set(menu_dto.calorie_range_min.or(m.calorie_range_min));
                update_m.calorie_range_max =
                    Set(menu_dto.calorie_range_max.or(m.calorie_range_max));
            }
            update_m.update(&txn).await?;

            is_inserted = false;
            m.id
        } else {
            let new_menu = menus::ActiveModel {
                city_id: Set(city_id),
                serve_date: Set(date),
                meal_type: Set(meal_type),
                source_type: Set(Some(source_type.to_string())),
                submitted_by: Set(None),
                status: Set(target_status.clone()),
                calorie_range_min: Set(menu_dto.calorie_range_min),
                calorie_range_max: Set(menu_dto.calorie_range_max),
                ..Default::default()
            };
            let res = new_menu.insert(&txn).await?;
            is_inserted = true;
            res.id
        };

        // Slot uzlaşması (In-place diff & preservation)
        let mut matched_existing_ids = HashSet::new();
        let mut existing_alias_owner: HashMap<(String, i32), i32> = HashMap::new();
        let mut existing_dish_in_slot: HashMap<(String, i32, i32), i32> = HashMap::new();

        for (d, alias) in &existing_dishes_list {
            existing_alias_owner.insert((d.package_name.clone(), d.dish_alias_id), d.id);
            if let Some(a) = alias
                && let Some(did) = a.dish_id
            {
                existing_dish_in_slot.insert((d.package_name.clone(), d.order_index, did), d.id);
            }
        }

        for (key, (alias_id, dish_id, amount, calories)) in target_map.into_iter() {
            let (package_name, order_index, is_alternative) = match &key {
                DishSlotKey::Primary(pkg, idx) => (pkg.clone(), *idx, false),
                DishSlotKey::Alternative(pkg, idx, _) => (pkg.clone(), *idx, true),
            };

            if let Some(existing) = existing_map.get(&key) {
                matched_existing_ids.insert(existing.id);

                if let Some(owner_id) = existing_alias_owner.get(&(package_name.clone(), alias_id))
                    && *owner_id != existing.id
                {
                    continue;
                }

                if let Some(owner_id) =
                    existing_dish_in_slot.get(&(package_name.clone(), order_index, dish_id))
                    && *owner_id != existing.id
                {
                    continue;
                }

                let final_amount = amount
                    .filter(|s| !s.trim().is_empty())
                    .or(existing.amount.clone());
                let final_calories = calories.or(existing.calories);

                let mut active: menu_dishes::ActiveModel = existing.clone().into();
                active.dish_alias_id = Set(alias_id);
                active.amount = Set(final_amount);
                active.calories = Set(final_calories);
                active.update(&txn).await?;
                continue;
            }

            if existing_dish_in_slot.contains_key(&(package_name.clone(), order_index, dish_id)) {
                continue;
            }

            let stmt = sea_orm::Statement::from_sql_and_values(
                sea_orm::DbBackend::Postgres,
                r#"
                INSERT INTO menu_dishes (menu_id, dish_alias_id, order_index, is_alternative, package_name, amount, calories)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (menu_id, dish_alias_id, package_name)
                DO UPDATE SET order_index = EXCLUDED.order_index,
                              is_alternative = EXCLUDED.is_alternative,
                              amount = COALESCE(EXCLUDED.amount, menu_dishes.amount),
                              calories = COALESCE(EXCLUDED.calories, menu_dishes.calories)
                "#,
                vec![
                    menu_id.into(),
                    alias_id.into(),
                    order_index.into(),
                    is_alternative.into(),
                    package_name.into(),
                    amount.into(),
                    calories.into(),
                ],
            );
            txn.execute(stmt).await?;
        }

        let obsolete_ids: Vec<i32> = existing_dishes_list
            .iter()
            .map(|(d, _)| d.id)
            .filter(|id| !matched_existing_ids.contains(id))
            .collect();

        if !obsolete_ids.is_empty() {
            menu_dishes::Entity::delete_many()
                .filter(menu_dishes::Column::Id.is_in(obsolete_ids))
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;

        if target_status == MenuStatusEnum::Approved {
            let _ = shared::services::immutable_store::ImmutableStore::write_menu_hash(db, menu_id)
                .await;
        }

        Ok(Some(is_inserted))
    }
}

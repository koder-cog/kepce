use sea_orm::*;
use crate::entities::menus;
use sha2::{Sha256, Digest};
use chrono::NaiveDate;

pub struct ImmutableStore;

impl ImmutableStore {
    /// Compute a SHA-256 hash representing a menu block in the Hash Chain
    pub fn compute_menu_hash(
        serve_date: NaiveDate,
        city_id: i32,
        meal_type: &str,
        sorted_dish_ids: &[i32],
        previous_hash: Option<&str>,
    ) -> String {
        let prev = previous_hash.unwrap_or("GENESIS");
        let dishes_str = sorted_dish_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        
        let payload = format!(
            "{}:{}:{}:{}:{}",
            serve_date.format("%Y-%m-%d"),
            city_id,
            meal_type,
            dishes_str,
            prev
        );

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    /// Retrieve the previous menu hash in the chain for the same city and meal type
    pub async fn get_previous_hash(
        db: &DatabaseConnection,
        city_id: i32,
        meal_type: &crate::entities::sea_orm_active_enums::MealTypeEnum,
        serve_date: NaiveDate,
    ) -> Result<Option<String>, DbErr> {
        let prev_menu = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::MealType.eq(meal_type.clone()))
            .filter(menus::Column::ServeDate.lt(serve_date))
            .order_by_desc(menus::Column::ServeDate)
            .one(db)
            .await?;
        
        Ok(prev_menu.and_then(|m| m.merkle_root))
    }

    /// Calculate the menu hash and write it along with previous_hash into the database
    pub async fn write_menu_hash(
        db: &DatabaseConnection,
        menu_id: i32,
    ) -> Result<String, DbErr> {
        use crate::entities::menu_dishes;

        let menu_model = menus::Entity::find_by_id(menu_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Menu not found".to_string()))?;

        let dishes = menu_dishes::Entity::find()
            .filter(menu_dishes::Column::MenuId.eq(menu_id))
            .order_by_asc(menu_dishes::Column::OrderIndex)
            .all(db)
            .await?;

        let sorted_dish_ids: Vec<i32> = dishes.iter().map(|d| d.dish_alias_id).collect();

        let prev_hash = Self::get_previous_hash(db, menu_model.city_id, &menu_model.meal_type, menu_model.serve_date).await?;

        let meal_type_str = match menu_model.meal_type {
            crate::entities::sea_orm_active_enums::MealTypeEnum::Breakfast => "breakfast",
            crate::entities::sea_orm_active_enums::MealTypeEnum::Lunch => "lunch",
            crate::entities::sea_orm_active_enums::MealTypeEnum::Dinner => "dinner",
        };

        let calculated_hash = Self::compute_menu_hash(
            menu_model.serve_date,
            menu_model.city_id,
            meal_type_str,
            &sorted_dish_ids,
            prev_hash.as_deref(),
        );

        let mut active_menu: menus::ActiveModel = menu_model.into();
        active_menu.merkle_root = Set(Some(calculated_hash.clone()));
        active_menu.previous_hash = Set(prev_hash);
        active_menu.update(db).await?;

        Ok(calculated_hash)
    }

    /// Recalculates the hash chain forward from a specific date.
    /// This is crucial for maintaining chain integrity when historical (backfill) menus are inserted.
    pub async fn recalculate_chain_from_date(
        db: &DatabaseConnection,
        city_id: i32,
        meal_type: &crate::entities::sea_orm_active_enums::MealTypeEnum,
        start_date: NaiveDate,
    ) -> Result<(), DbErr> {
        let menus_to_update = menus::Entity::find()
            .filter(menus::Column::CityId.eq(city_id))
            .filter(menus::Column::MealType.eq(meal_type.clone()))
            .filter(menus::Column::ServeDate.gte(start_date))
            .order_by_asc(menus::Column::ServeDate)
            .all(db)
            .await?;

        for menu in menus_to_update {
            // write_menu_hash already relies on get_previous_hash.
            // Since we iterate in ascending order, each menu will automatically pick up 
            // the newly recalculated hash from the previous menu in the loop.
            Self::write_menu_hash(db, menu.id).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_compute_menu_hash_genesis() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        let dishes = vec![1, 2, 3];
        
        let hash1 = ImmutableStore::compute_menu_hash(date, 1, "lunch", &dishes, None);
        let hash2 = ImmutableStore::compute_menu_hash(date, 1, "lunch", &dishes, None);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, "");
    }

    #[test]
    fn test_compute_menu_hash_chain() {
        let date1 = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        let dishes1 = vec![1, 2, 3];
        let hash1 = ImmutableStore::compute_menu_hash(date1, 1, "lunch", &dishes1, None);

        let date2 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let dishes2 = vec![4, 5];
        let hash2 = ImmutableStore::compute_menu_hash(date2, 1, "lunch", &dishes2, Some(&hash1));

        assert_ne!(hash1, hash2);
    }
}



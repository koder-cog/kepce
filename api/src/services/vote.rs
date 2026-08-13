use sea_orm::*;
use shared::entities::{menu_votes, dish_votes, menu_dishes, sea_orm_active_enums::SentimentEnum};

#[derive(Debug)]
pub enum VoteError {
    DatabaseError(DbErr),
    MenuNotFound,
    UnverifiedUser,
}

pub struct VoteService;

impl VoteService {
    pub async fn vote_menu(
        db: &DatabaseConnection,
        menu_id: i32,
        user_id: uuid::Uuid,
        sentiment: SentimentEnum,
    ) -> Result<(), VoteError> {
        // 0. Check if user is verified
        let user = shared::entities::prelude::Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(VoteError::DatabaseError)?
            .ok_or(VoteError::DatabaseError(DbErr::Custom("Kullanıcı bulunamadı".to_string())))?;
        
        if !user.is_verified {
            return Err(VoteError::UnverifiedUser);
        }

        // Start a transaction
        let txn = db.begin().await.map_err(VoteError::DatabaseError)?;

        // 1. Menu vote ops
        if sentiment == SentimentEnum::Neutral {
            // Delete menu vote
            menu_votes::Entity::delete_many()
                .filter(menu_votes::Column::MenuId.eq(menu_id))
                .filter(menu_votes::Column::UserId.eq(user_id))
                .exec(&txn)
                .await
                .map_err(VoteError::DatabaseError)?;
                
            // Delete broadcasted (non-explicit) dish votes
            dish_votes::Entity::delete_many()
                .filter(dish_votes::Column::MenuId.eq(menu_id))
                .filter(dish_votes::Column::UserId.eq(user_id))
                .filter(dish_votes::Column::IsExplicit.eq(false))
                .exec(&txn)
                .await
                .map_err(VoteError::DatabaseError)?;
        } else {
            // Upsert menu vote
            let menu_vote = menu_votes::ActiveModel {
                menu_id: Set(menu_id),
                user_id: Set(user_id),
                sentiment: Set(sentiment.clone()),
                ..Default::default()
            };
            
            // Check if exists
            let existing_menu_vote = menu_votes::Entity::find()
                .filter(menu_votes::Column::MenuId.eq(menu_id))
                .filter(menu_votes::Column::UserId.eq(user_id))
                .one(&txn)
                .await
                .map_err(VoteError::DatabaseError)?;
                
            if let Some(existing) = existing_menu_vote {
                let mut active: menu_votes::ActiveModel = existing.into();
                active.sentiment = Set(sentiment.clone());
                active.update(&txn).await.map_err(VoteError::DatabaseError)?;
            } else {
                menu_votes::Entity::insert(menu_vote).exec(&txn).await.map_err(VoteError::DatabaseError)?;
            }
            
            // Broadcast to dishes!
            // Find all dishes for this menu
            let menu_dishes_list = menu_dishes::Entity::find()
                .filter(menu_dishes::Column::MenuId.eq(menu_id))
                .find_also_related(shared::entities::dish_aliases::Entity)
                .all(&txn)
                .await
                .map_err(VoteError::DatabaseError)?;
                
            for (_, alias_opt) in menu_dishes_list {
                if let Some(alias) = alias_opt {
                    if let Some(dish_id) = alias.dish_id {
                        // Check if explicit vote exists
                        let existing_dish_vote = dish_votes::Entity::find()
                            .filter(dish_votes::Column::DishId.eq(dish_id))
                            .filter(dish_votes::Column::MenuId.eq(menu_id))
                            .filter(dish_votes::Column::UserId.eq(user_id))
                            .one(&txn)
                            .await
                            .map_err(VoteError::DatabaseError)?;
                            
                        if let Some(existing) = existing_dish_vote {
                            // Only overwrite if it's NOT explicit
                            if !existing.is_explicit {
                                let mut active: dish_votes::ActiveModel = existing.into();
                                active.sentiment = Set(sentiment.clone());
                                active.update(&txn).await.map_err(VoteError::DatabaseError)?;
                            }
                        } else {
                            // Insert inherited vote
                            dish_votes::Entity::insert(dish_votes::ActiveModel {
                                dish_id: Set(dish_id),
                                menu_id: Set(menu_id),
                                user_id: Set(user_id),
                                sentiment: Set(sentiment.clone()),
                                is_explicit: Set(false),
                                ..Default::default()
                            }).exec(&txn).await.map_err(VoteError::DatabaseError)?;
                        }
                    }
                }
            }
        }
        
        txn.commit().await.map_err(VoteError::DatabaseError)?;
        Ok(())
    }
}

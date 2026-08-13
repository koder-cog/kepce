//! `SeaORM` Entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "dishes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", unique)]
    pub name: String,
    pub category: Option<String>,
    pub parent_id: Option<i32>,
    pub is_celiac: bool,
    pub is_vegan: bool,
    pub is_vegetarian: bool,
    pub estimated_calories: Option<i32>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comments::Entity")]
    Comments,
    #[sea_orm(has_many = "super::dish_aliases::Entity")]
    DishAliases,
    #[sea_orm(has_many = "super::dish_tags::Entity")]
    DishTags,
    #[sea_orm(has_many = "super::dish_votes::Entity")]
    DishVotes,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::user_favorites::Entity")]
    UserFavorites,
    #[sea_orm(has_many = "super::user_pinned_dishes::Entity")]
    UserPinnedDishes,
}

impl Related<super::comments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comments.def()
    }
}

impl Related<super::dish_aliases::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DishAliases.def()
    }
}

impl Related<super::dish_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DishTags.def()
    }
}

impl Related<super::dish_votes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DishVotes.def()
    }
}

impl Related<super::user_favorites::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserFavorites.def()
    }
}

impl Related<super::user_pinned_dishes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserPinnedDishes.def()
    }
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        super::dish_tags::Relation::Tags.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::dish_tags::Relation::Dishes.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
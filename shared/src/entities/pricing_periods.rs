//! `SeaORM` Entity for pricing_periods

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "pricing_periods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub city_slug: String,
    pub period_start: Date,
    pub period_end: Date,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cities::Entity",
        from = "Column::CitySlug",
        to = "super::cities::Column::Slug",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Cities,
    #[sea_orm(has_many = "super::meal_category_prices::Entity")]
    MealCategoryPrices,
}

impl Related<super::cities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cities.def()
    }
}

impl Related<super::meal_category_prices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MealCategoryPrices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

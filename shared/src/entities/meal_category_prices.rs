//! `SeaORM` Entity for meal_category_prices

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meal_category_prices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pricing_period_id: i32,
    pub meal_type: String,
    pub category_name: String,
    pub portion_amount: Option<String>,
    pub price: Decimal,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pricing_periods::Entity",
        from = "Column::PricingPeriodId",
        to = "super::pricing_periods::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PricingPeriods,
}

impl Related<super::pricing_periods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PricingPeriods.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

//! `SeaORM` Entity for takeaway_slot_items

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "takeaway_slot_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub slot_id: i32,
    pub dish_id: i32,
    pub portion_override: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::takeaway_slots::Entity",
        from = "Column::SlotId",
        to = "super::takeaway_slots::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TakeawaySlots,
    #[sea_orm(
        belongs_to = "super::dishes::Entity",
        from = "Column::DishId",
        to = "super::dishes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Dishes,
}

impl Related<super::takeaway_slots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TakeawaySlots.def()
    }
}

impl Related<super::dishes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dishes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

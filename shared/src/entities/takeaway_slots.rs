//! `SeaORM` Entity for takeaway_slots

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "takeaway_slots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub package_id: i32,
    pub slot_index: i32,
    pub slot_title: Option<String>,
    pub is_required: bool,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::takeaway_packages::Entity",
        from = "Column::PackageId",
        to = "super::takeaway_packages::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TakeawayPackages,
    #[sea_orm(has_many = "super::takeaway_slot_items::Entity")]
    TakeawaySlotItems,
}

impl Related<super::takeaway_packages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TakeawayPackages.def()
    }
}

impl Related<super::takeaway_slot_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TakeawaySlotItems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

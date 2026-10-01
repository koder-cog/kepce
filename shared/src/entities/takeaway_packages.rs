//! `SeaORM` Entity for takeaway_packages

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "takeaway_packages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub city_slug: String,
    pub package_name: String,
    pub academic_year: String,
    pub is_active: bool,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
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
    #[sea_orm(has_many = "super::takeaway_slots::Entity")]
    TakeawaySlots,
}

impl Related<super::cities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cities.def()
    }
}

impl Related<super::takeaway_slots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TakeawaySlots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

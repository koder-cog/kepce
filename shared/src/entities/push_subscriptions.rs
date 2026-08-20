//! `SeaORM` Entity for `push_subscriptions` table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "push_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub city_id: Option<i32>,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub notif_breakfast_enabled: bool,
    pub notif_breakfast_time: String,
    pub notif_dinner_enabled: bool,
    pub notif_dinner_time: String,
    pub user_agent: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::cities::Entity",
        from = "Column::CityId",
        to = "super::cities::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Cities,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::cities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cities.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

//! `SeaORM` Entity

use super::sea_orm_active_enums::ReportStatusEnum;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub category: String,
    pub subject: String,
    #[sea_orm(column_type = "Text")]
    pub message: String,
    pub status: ReportStatusEnum,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub resolved_at: Option<DateTimeWithTimeZone>,
    pub source: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub page_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub user_agent: Option<String>,
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
    #[sea_orm(has_many = "super::contact_message_replies::Entity")]
    Replies,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::contact_message_replies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Replies.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
//! `SeaORM` Entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_message_replies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub contact_message_id: i32,
    pub responder_id: Option<Uuid>,
    #[sea_orm(column_type = "Text")]
    pub reply_body: String,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact_messages::Entity",
        from = "Column::ContactMessageId",
        to = "super::contact_messages::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ContactMessages,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::ResponderId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Users,
}

impl Related<super::contact_messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ContactMessages.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

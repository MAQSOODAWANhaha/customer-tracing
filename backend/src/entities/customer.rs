use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::next_action::NextAction;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "customers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub rate: i32,
    pub user_id: i32,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    pub is_deleted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::customer_track::Entity")]
    CustomerTrack,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::customer_track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CustomerTrack.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerWithLatestTrack {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub rate: i32,
    pub notes: Option<String>,
    pub latest_track_time: Option<ChronoDateTimeUtc>,
    pub latest_next_action: Option<NextAction>,
    pub latest_content: Option<String>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub rate: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub rate: Option<i32>,
}
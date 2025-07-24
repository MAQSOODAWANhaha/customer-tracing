use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::next_action::NextAction;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "customer_tracks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub customer_id: i32,
    pub content: String,
    pub next_action: NextAction,
    pub track_time: ChronoDateTimeUtc,
    pub next_track_time: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::customer::Entity",
        from = "Column::CustomerId",
        to = "super::customer::Column::Id"
    )]
    Customer,
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize)]
pub struct CustomerTrackInfo {
    pub id: i32,
    pub customer_id: i32,
    pub content: String,
    pub next_action: NextAction,
    pub track_time: ChronoDateTimeUtc,
    pub next_track_time: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

impl From<Model> for CustomerTrackInfo {
    fn from(track: Model) -> Self {
        Self {
            id: track.id,
            customer_id: track.customer_id,
            content: track.content,
            next_action: track.next_action,
            track_time: track.track_time,
            next_track_time: track.next_track_time,
            created_at: track.created_at,
            updated_at: track.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTrackRequest {
    pub customer_id: i32,
    pub content: String,
    pub next_action: Option<NextAction>,
    pub track_time: Option<ChronoDateTimeUtc>,
    pub next_track_time: Option<ChronoDateTimeUtc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackRequest {
    pub content: Option<String>,
    pub next_action: Option<NextAction>,
    pub track_time: Option<ChronoDateTimeUtc>,
    pub next_track_time: Option<ChronoDateTimeUtc>,
}
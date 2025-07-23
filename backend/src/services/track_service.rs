use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::entities::{
    customer, customer::Entity as Customer, customer_track,
    customer_track::Entity as CustomerTrack, next_action::NextAction,
};

#[derive(Debug, Deserialize)]
pub struct CreateTrackRequest {
    pub customer_id: i32,
    pub content: String,
    pub next_action: NextAction,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackRequest {
    pub content: Option<String>,
    pub next_action: Option<NextAction>,
}

#[derive(Debug, Serialize)]
pub struct TrackListResponse {
    pub tracks: Vec<customer_track::Model>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

pub struct TrackService;

impl TrackService {
    pub async fn create_track(
        db: &DatabaseConnection,
        user_id: i32,
        request: CreateTrackRequest,
    ) -> Result<customer_track::Model> {
        // 验证客户是否属于当前用户
        let _customer = Customer::find_by_id(request.customer_id)
            .filter(customer::Column::UserId.eq(user_id))
            .filter(customer::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("客户不存在或无权限"))?;

        let track = customer_track::ActiveModel {
            customer_id: Set(request.customer_id),
            content: Set(request.content),
            next_action: Set(request.next_action),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let track = track.insert(db).await?;
        Ok(track)
    }

    pub async fn get_track_by_id(
        db: &DatabaseConnection,
        track_id: i32,
        user_id: i32,
    ) -> Result<Option<customer_track::Model>> {
        let track = CustomerTrack::find_by_id(track_id)
            .find_also_related(Customer)
            .filter(customer::Column::UserId.eq(user_id))
            .filter(customer::Column::IsDeleted.eq(false))
            .one(db)
            .await?;

        Ok(track.map(|(track, _)| track))
    }

    pub async fn list_tracks_by_customer(
        db: &DatabaseConnection,
        customer_id: i32,
        user_id: i32,
        page: u64,
        limit: u64,
    ) -> Result<TrackListResponse> {
        // 验证客户是否属于当前用户
        let _customer = Customer::find_by_id(customer_id)
            .filter(customer::Column::UserId.eq(user_id))
            .filter(customer::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("客户不存在或无权限"))?;

        let paginator = CustomerTrack::find()
            .filter(customer_track::Column::CustomerId.eq(customer_id))
            .order_by_desc(customer_track::Column::CreatedAt)
            .paginate(db, limit);

        let tracks = paginator.fetch_page(page - 1).await?;
        let total = paginator.num_items().await?;

        Ok(TrackListResponse {
            tracks,
            total,
            page,
            limit,
        })
    }

    pub async fn update_track(
        db: &DatabaseConnection,
        track_id: i32,
        user_id: i32,
        request: UpdateTrackRequest,
    ) -> Result<customer_track::Model> {
        let track = CustomerTrack::find_by_id(track_id)
            .find_also_related(Customer)
            .filter(customer::Column::UserId.eq(user_id))
            .filter(customer::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("跟进记录不存在或无权限"))?
            .0;

        let mut active_track: customer_track::ActiveModel = track.into();

        if let Some(content) = request.content {
            active_track.content = Set(content);
        }
        if let Some(next_action) = request.next_action {
            active_track.next_action = Set(next_action);
        }

        active_track.updated_at = Set(chrono::Utc::now());

        let updated_track = active_track.update(db).await?;
        Ok(updated_track)
    }

    pub async fn delete_track(
        db: &DatabaseConnection,
        track_id: i32,
        user_id: i32,
    ) -> Result<()> {
        let track = CustomerTrack::find_by_id(track_id)
            .find_also_related(Customer)
            .filter(customer::Column::UserId.eq(user_id))
            .filter(customer::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("跟进记录不存在或无权限"))?
            .0;

        CustomerTrack::delete_by_id(track.id).exec(db).await?;
        Ok(())
    }
}
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{
        customer::{self, Entity as Customer},
        customer_track::{
            self, Entity as CustomerTrack, CreateTrackRequest, UpdateTrackRequest,
            CustomerTrackInfo,
        },
        next_action::NextAction,
    },
    middleware::auth::CurrentUser,
    handlers::auth::AppState,
};

#[derive(Debug, Deserialize)]
pub struct TrackListQuery {
    pub customer_id: i32,
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 { 1 }
fn default_limit() -> u64 { 20 }

#[derive(Debug, Serialize)]
pub struct TrackListResponse {
    pub tracks: Vec<CustomerTrackInfo>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct CustomerTrackListResponse {
    pub tracks: Vec<CustomerTrackInfo>,
    pub customer: CustomerInfo,
}

#[derive(Debug, Serialize)]
pub struct CustomerInfo {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub rate: i32,
}

impl From<customer::Model> for CustomerInfo {
    fn from(customer: customer::Model) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            rate: customer.rate,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NextActionsResponse {
    pub actions: Vec<String>,
}

pub async fn list_customer_tracks(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerTrackListResponse>, StatusCode> {
    // First verify customer belongs to current user
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get tracking records
    let tracks = CustomerTrack::find()
        .filter(customer_track::Column::CustomerId.eq(customer_id))
        .order_by_desc(customer_track::Column::TrackTime)
        .all(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackListResponse {
        tracks: tracks.into_iter().map(CustomerTrackInfo::from).collect(),
        customer: CustomerInfo::from(customer),
    }))
}

pub async fn create_customer_track(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateTrackRequest>,
) -> Result<Json<CustomerTrackInfo>, StatusCode> {
    // Verify customer belongs to current user
    let _customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let now = Utc::now();
    
    let track = customer_track::ActiveModel {
        customer_id: Set(customer_id),
        content: Set(req.content),
        next_action: Set(req.next_action.unwrap_or(NextAction::Continue)),
        track_time: Set(req.track_time.unwrap_or(now)),
        next_track_time: Set(req.next_track_time),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let track = track
        .insert(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackInfo::from(track)))
}

pub async fn update_customer_track(
    Extension(current_user): Extension<CurrentUser>,
    Path(track_id): Path<i32>,
    State(app_state): State<AppState>,
    Json(req): Json<UpdateTrackRequest>,
) -> Result<Json<CustomerTrackInfo>, StatusCode> {
    // Find the track and verify ownership through customer relationship
    let track = CustomerTrack::find_by_id(track_id)
        .find_also_related(Customer)
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let (track, customer) = track;
    let customer = customer.ok_or(StatusCode::NOT_FOUND)?;

    // Verify the customer belongs to current user
    if customer.user_id != current_user.id || customer.is_deleted {
        return Err(StatusCode::NOT_FOUND);
    }

    // Update track
    let mut track_active: customer_track::ActiveModel = track.into();
    
    if let Some(content) = req.content {
        track_active.content = Set(content);
    }
    if let Some(next_action) = req.next_action {
        track_active.next_action = Set(next_action);
    }
    if let Some(track_time) = req.track_time {
        track_active.track_time = Set(track_time);
    }
    if let Some(next_track_time) = req.next_track_time {
        track_active.next_track_time = Set(Some(next_track_time));
    }
    
    track_active.updated_at = Set(Utc::now());

    let updated_track = track_active
        .update(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackInfo::from(updated_track)))
}

pub async fn delete_customer_track(
    Extension(current_user): Extension<CurrentUser>,
    Path(track_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // Find the track and verify ownership through customer relationship
    let track = CustomerTrack::find_by_id(track_id)
        .find_also_related(Customer)
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let (track, customer) = track;
    let customer = customer.ok_or(StatusCode::NOT_FOUND)?;

    // Verify the customer belongs to current user
    if customer.user_id != current_user.id || customer.is_deleted {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete the track
    CustomerTrack::delete_by_id(track.id)
        .exec(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// 分页查询跟进记录 - 支持前端的 /api/tracks?customer_id=1 请求
pub async fn list_tracks(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TrackListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<TrackListResponse>, StatusCode> {
    // 验证客户是否属于当前用户
    let _customer = Customer::find_by_id(params.customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 分页查询跟进记录
    let paginator = CustomerTrack::find()
        .filter(customer_track::Column::CustomerId.eq(params.customer_id))
        .order_by_desc(customer_track::Column::TrackTime)
        .paginate(&app_state.db, params.limit);

    let tracks_page = paginator
        .fetch_page(params.page - 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = paginator
        .num_items()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TrackListResponse {
        tracks: tracks_page.into_iter().map(CustomerTrackInfo::from).collect(),
        total,
        page: params.page,
        limit: params.limit,
    }))
}

// 创建跟进记录 - 支持前端的 POST /api/tracks 请求
pub async fn create_track(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateTrackRequest>,
) -> Result<Json<CustomerTrackInfo>, StatusCode> {
    // 验证客户是否属于当前用户
    let _customer = Customer::find_by_id(req.customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let now = Utc::now();
    
    let track = customer_track::ActiveModel {
        customer_id: Set(req.customer_id),
        content: Set(req.content),
        next_action: Set(req.next_action.unwrap_or(NextAction::Continue)),
        track_time: Set(req.track_time.unwrap_or(now)),
        next_track_time: Set(req.next_track_time),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let track = track
        .insert(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackInfo::from(track)))
}

pub async fn get_next_actions() -> Json<NextActionsResponse> {
    Json(NextActionsResponse {
        actions: NextAction::variants().into_iter().map(|s| s.to_string()).collect(),
    })
}
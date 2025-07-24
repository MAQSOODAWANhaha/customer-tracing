use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{
        customer::{self, Entity as Customer, CreateCustomerRequest, UpdateCustomerRequest},
        customer_group::CustomerGroup,
        customer_track::{self, Entity as CustomerTrack},
        next_action::NextAction,
    },
    middleware::auth::CurrentUser,
    handlers::auth::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CustomerListQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub search: Option<String>,
    pub status: Option<NextAction>,
}

fn default_page() -> u64 { 1 }
fn default_limit() -> u64 { 20 }

#[derive(Debug, Serialize)]
pub struct CustomerListResponse {
    pub customers: Vec<CustomerWithLatestTrack>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct CustomerWithLatestTrack {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub rate: f32,
    pub notes: Option<String>,
    pub customer_group: CustomerGroup,
    pub next_action: NextAction,
    pub latest_track_time: Option<chrono::DateTime<chrono::Utc>>,
    pub latest_next_action: Option<NextAction>,
    pub latest_content: Option<String>,
    pub track_count: i64,
    pub user_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize)]
pub struct CustomerDetailResponse {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub rate: f32,
    pub customer_group: CustomerGroup,
    pub user_id: i32,
    pub next_action: NextAction,
    pub track_count: i64,
    pub last_track_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
}

pub async fn list_customers(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<CustomerListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerListResponse>, StatusCode> {
    // 第一步：获取所有符合基本条件的客户
    let mut base_query = Customer::find()
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false));

    // 添加基本搜索过滤
    if let Some(search_term) = &params.search {
        base_query = base_query.filter(
            customer::Column::Name.contains(search_term)
                .or(customer::Column::Phone.contains(search_term))
        );
    }

    // 获取所有符合基本条件的客户
    let all_customers = base_query
        .order_by_desc(customer::Column::UpdatedAt)
        .all(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 第二步：获取每个客户的跟进信息并进行状态筛选
    let mut customer_with_tracks = Vec::new();
    for customer in all_customers {
        // 查询该客户的最新跟进记录
        let latest_track = CustomerTrack::find()
            .filter(customer_track::Column::CustomerId.eq(customer.id))
            .order_by_desc(customer_track::Column::TrackTime)
            .one(&app_state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 确定客户的当前状态
        let current_status = latest_track.as_ref().map(|t| t.next_action.clone()).unwrap_or(NextAction::Continue);
        
        // 如果有状态筛选条件，检查是否匹配
        if let Some(filter_status) = &params.status {
            if current_status != *filter_status {
                continue; // 跳过不匹配的客户
            }
        }

        // 查询该客户的跟进记录总数
        let track_count = CustomerTrack::find()
            .filter(customer_track::Column::CustomerId.eq(customer.id))
            .count(&app_state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        customer_with_tracks.push(CustomerWithLatestTrack {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            address: customer.address,
            rate: customer.rate,
            notes: customer.notes,
            customer_group: customer.customer_group,
            next_action: current_status,
            latest_track_time: latest_track.as_ref().map(|t| t.track_time),
            latest_next_action: latest_track.as_ref().map(|t| t.next_action.clone()),
            latest_content: latest_track.as_ref().map(|t| t.content.clone()),
            track_count: track_count as i64,
            user_id: customer.user_id,
            created_at: customer.created_at,
            updated_at: customer.updated_at,
            is_deleted: customer.is_deleted,
        });
    }

    // 第三步：手动分页
    let total = customer_with_tracks.len() as u64;
    let start_index = ((params.page - 1) * params.limit) as usize;
    let end_index = std::cmp::min(start_index + params.limit as usize, customer_with_tracks.len());
    
    let paginated_customers = if start_index < customer_with_tracks.len() {
        customer_with_tracks[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    Ok(Json(CustomerListResponse {
        customers: paginated_customers,
        total,
        page: params.page,
        limit: params.limit,
    }))
}

pub async fn get_customer(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerDetailResponse>, StatusCode> {
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get track count
    let track_count = CustomerTrack::find()
        .filter(customer_track::Column::CustomerId.eq(customer_id))
        .count(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get latest track to determine next_action and last_track_at
    let latest_track = CustomerTrack::find()
        .filter(customer_track::Column::CustomerId.eq(customer_id))
        .order_by_desc(customer_track::Column::TrackTime)
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (next_action, last_track_at) = if let Some(track) = latest_track {
        (track.next_action, Some(track.track_time))
    } else {
        (NextAction::Continue, None) // Default action for customers without tracks
    };

    let response = CustomerDetailResponse {
        id: customer.id,
        name: customer.name,
        phone: customer.phone,
        address: customer.address,
        notes: customer.notes,
        rate: customer.rate,
        customer_group: customer.customer_group,
        user_id: customer.user_id,
        next_action,
        track_count: track_count as i64,
        last_track_at,
        created_at: customer.created_at,
        updated_at: customer.updated_at,
        is_deleted: customer.is_deleted,
    };

    Ok(Json(response))
}

pub async fn create_customer(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<customer::Model>, StatusCode> {
    let now = Utc::now();
    
    let customer = customer::ActiveModel {
        name: Set(req.name),
        phone: Set(req.phone),
        address: Set(req.address),
        notes: Set(req.notes),
        rate: Set(req.rate.unwrap_or(0.0)),
        customer_group: Set(req.customer_group.unwrap_or(CustomerGroup::GroupClass)),
        user_id: Set(current_user.id), // Automatically associate with current user
        created_at: Set(now),
        updated_at: Set(now),
        is_deleted: Set(false),
        ..Default::default()
    };

    let customer = customer
        .insert(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(customer))
}

pub async fn update_customer(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<Json<customer::Model>, StatusCode> {
    // Check if customer belongs to current user
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Update customer
    let mut customer_active: customer::ActiveModel = customer.into();
    
    if let Some(name) = req.name {
        customer_active.name = Set(name);
    }
    if let Some(phone) = req.phone {
        customer_active.phone = Set(if phone.is_empty() { None } else { Some(phone) });
    }
    if let Some(address) = req.address {
        customer_active.address = Set(if address.is_empty() { None } else { Some(address) });
    }
    if let Some(notes) = req.notes {
        customer_active.notes = Set(if notes.is_empty() { None } else { Some(notes) });
    }
    if let Some(rate) = req.rate {
        customer_active.rate = Set(rate);
    }
    if let Some(customer_group) = req.customer_group {
        customer_active.customer_group = Set(customer_group);
    }
    
    customer_active.updated_at = Set(Utc::now());

    let updated_customer = customer_active
        .update(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_customer))
}

pub async fn delete_customer(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // Check if customer belongs to current user
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Soft delete the customer
    let mut customer_active: customer::ActiveModel = customer.into();
    customer_active.is_deleted = Set(true);
    customer_active.updated_at = Set(Utc::now());

    customer_active
        .update(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
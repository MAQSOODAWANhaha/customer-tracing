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

#[derive(Debug, Serialize)]
pub struct CustomerWithLatestTrack {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub rate: i32,
    pub notes: Option<String>,
    pub latest_track_time: Option<chrono::DateTime<chrono::Utc>>,
    pub latest_next_action: Option<NextAction>,
    pub latest_content: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_customers(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<CustomerListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerListResponse>, StatusCode> {
    let mut query = Customer::find()
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false));

    // Add search filter if provided
    if let Some(search_term) = &params.search {
        query = query.filter(
            customer::Column::Name.contains(search_term)
                .or(customer::Column::Phone.contains(search_term))
                .or(customer::Column::Email.contains(search_term))
                .or(customer::Column::Company.contains(search_term))
        );
    }

    let paginator = query
        .order_by_desc(customer::Column::UpdatedAt)
        .paginate(&app_state.db, params.limit);

    let customers_page = paginator
        .fetch_page(params.page - 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = paginator
        .num_items()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // For now, return customers without latest track info
    // In a production app, you'd want to optimize this with a JOIN query
    let customer_with_tracks = customers_page
        .into_iter()
        .map(|customer| CustomerWithLatestTrack {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            rate: customer.rate,
            notes: customer.notes,
            latest_track_time: None, // TODO: Optimize with JOIN
            latest_next_action: None,
            latest_content: None,
            created_at: customer.created_at,
        })
        .collect();

    Ok(Json(CustomerListResponse {
        customers: customer_with_tracks,
        total,
        page: params.page,
        limit: params.limit,
    }))
}

pub async fn get_customer(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<Json<customer::Model>, StatusCode> {
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(customer))
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
        email: Set(req.email),
        company: Set(req.company),
        address: Set(req.address),
        notes: Set(req.notes),
        rate: Set(req.rate.unwrap_or(0)),
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
        customer_active.phone = Set(Some(phone));
    }
    if let Some(email) = req.email {
        customer_active.email = Set(Some(email));
    }
    if let Some(company) = req.company {
        customer_active.company = Set(Some(company));
    }
    if let Some(address) = req.address {
        customer_active.address = Set(Some(address));
    }
    if let Some(notes) = req.notes {
        customer_active.notes = Set(Some(notes));
    }
    if let Some(rate) = req.rate {
        customer_active.rate = Set(rate);
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
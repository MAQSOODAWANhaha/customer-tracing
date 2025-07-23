use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{user, user::Entity as User},
    middleware::auth::CurrentUser,
    utils::{jwt::generate_jwt_token, password::verify_password},
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<user::Model> for UserInfo {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            name: user.name,
            last_login_at: user.last_login_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
    pub jwt_expire_hours: i64,
}

impl AsRef<String> for AppState {
    fn as_ref(&self) -> &String {
        &self.jwt_secret
    }
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by username
    let user = User::find()
        .filter(user::Column::Username.eq(&req.username))
        .filter(user::Column::IsActive.eq(true))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let token_pair = generate_jwt_token(
        user.id,
        &user.username,
        &user.name,
        &app_state.jwt_secret,
        app_state.jwt_expire_hours,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update last_login_at
    let mut user_active: user::ActiveModel = user.clone().into();
    user_active.last_login_at = Set(Some(Utc::now()));
    let updated_user = user_active
        .update(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token: token_pair.access_token,
        expires_in: token_pair.expires_in,
        user: UserInfo::from(updated_user),
    }))
}

pub async fn logout(
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<LogoutResponse>, StatusCode> {
    // In a JWT-based system, logout is typically handled client-side
    // by removing the token from storage. Here we just return a success message.
    Ok(Json(LogoutResponse {
        message: "Successfully logged out".to_string(),
    }))
}

pub async fn refresh_token(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    // Generate new JWT token with current user info
    let token_pair = generate_jwt_token(
        current_user.id,
        &current_user.username,
        &current_user.name,
        &app_state.jwt_secret,
        app_state.jwt_expire_hours,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RefreshTokenResponse {
        token: token_pair.access_token,
        expires_in: token_pair.expires_in,
    }))
}

pub async fn get_current_user(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<UserInfo>, StatusCode> {
    // Get fresh user data from database
    let user = User::find_by_id(current_user.id)
        .filter(user::Column::IsActive.eq(true))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(UserInfo::from(user)))
}
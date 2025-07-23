use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::entities::{user, user::Entity as User};
use crate::utils::password::verify_password;
use crate::utils::jwt::generate_jwt_token;

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
    pub created_at: String,
    pub last_login_at: Option<String>,
}

pub struct AuthService;

impl AuthService {
    pub async fn authenticate_user(
        db: &DatabaseConnection,
        login_request: LoginRequest,
    ) -> Result<LoginResponse> {
        // 查找用户
        let user = User::find()
            .filter(user::Column::Username.eq(&login_request.username))
            .filter(user::Column::IsActive.eq(true))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户名或密码错误"))?;

        // 验证密码
        if !verify_password(&login_request.password, &user.password_hash)? {
            return Err(anyhow::anyhow!("用户名或密码错误"));
        }

        // 更新最后登录时间
        let mut active_user: user::ActiveModel = user.clone().into();
        active_user.last_login_at = Set(Some(chrono::Utc::now()));
        active_user.updated_at = Set(chrono::Utc::now());
        let updated_user = active_user.update(db).await?;

        // 生成 JWT Token
        // 需要一个JWT_SECRET - 这里暂时使用硬编码，实际应该从环境变量获取
        let jwt_secret = "your-secret-key"; // TODO: 从环境变量获取
        let token_pair = generate_jwt_token(
            updated_user.id,
            &updated_user.username,
            &updated_user.name,
            jwt_secret,
            24, // 24小时过期
        )?;

        Ok(LoginResponse {
            token: token_pair.access_token,
            expires_in: token_pair.expires_in,
            user: UserInfo {
                id: updated_user.id,
                username: updated_user.username,
                name: updated_user.name,
                created_at: updated_user.created_at.to_rfc3339(),
                last_login_at: updated_user.last_login_at.map(|dt| dt.to_rfc3339()),
            },
        })
    }

    pub async fn get_user_by_id(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<Option<UserInfo>> {
        let user = User::find_by_id(user_id)
            .filter(user::Column::IsActive.eq(true))
            .one(db)
            .await?;

        Ok(user.map(|u| UserInfo {
            id: u.id,
            username: u.username,
            name: u.name,
            created_at: u.created_at.to_rfc3339(),
            last_login_at: u.last_login_at.map(|dt| dt.to_rfc3339()),
        }))
    }
}
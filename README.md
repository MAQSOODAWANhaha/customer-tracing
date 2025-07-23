# 客户追踪系统需求文档

## 📋 项目概述

### 项目名称
客户追踪系统 (Customer Tracking System)

### 项目背景
开发一个简单高效的客户追踪系统，帮助用户记录和管理客户信息，跟踪客户跟进状态，提高客户服务质量和销售转化率。支持多用户使用，每个用户只能管理自己创建的客户信息。

### 项目目标
- 提供安全的用户认证和权限控制系统
- 提供简洁易用的客户信息管理界面
- 支持客户跟进记录的时间线展示
- 优化移动端浏览器体验
- 提供快速的数据查询和展示
- 确保数据隔离，每个用户只能访问自己的数据

---

## 🎯 功能需求

### 1. 用户认证管理
- **用户登录页面**：提供用户名/密码登录界面
- **JWT认证**：基于JWT Token的身份验证
- **自动登录**：记住登录状态，支持Token自动续期
- **登出功能**：安全退出并清除认证信息
- **路由保护**：未认证用户自动跳转到登录页
- **命令行用户管理**：支持通过CLI添加用户

### 2. 客户信息管理
- **客户列表展示**：分页显示当前用户的客户基本信息
- **客户信息录入**：添加新客户信息（自动关联当前用户）
- **客户信息编辑**：修改客户基本信息（仅限自己创建的客户）
- **客户信息删除**：删除客户记录（仅限自己创建的客户，软删除）
- **数据隔离**：用户只能查看和操作自己创建的客户

### 3. 客户追踪管理  
- **追踪记录添加**：为自己的客户添加跟进记录
- **追踪记录查看**：时间线方式展示跟进历史
- **追踪记录编辑**：修改跟进记录内容
- **下次跟进动作**：选择继续跟进或结束跟进
- **权限控制**：只能操作自己客户的跟进记录

---

## 💻 技术架构

### 后端技术栈
```yaml
语言: Rust Edition 2021/2024
Web框架: Axum 0.8.x
数据库: SQLite (最新版本)
ORM: Sea-ORM (最新版本) 
命令行: Clap 4.x
认证: JWT Token (jsonwebtoken)
密码加密: bcrypt
中间件: axum-extra (JWT middleware)
日志: tracing
配置: config/dotenv
```

### 前端技术栈
```yaml
框架: Vue 3.x (Composition API)
UI组件库: Naive UI
状态管理: Pinia
路由: Vue Router 4.x (路由守卫)
HTTP客户端: Axios (请求/响应拦截器)
构建工具: Vite
类型检查: TypeScript
本地存储: localStorage (Token存储)
```

### 部署环境
```yaml
目标设备: 移动端浏览器优先
兼容性: 现代浏览器 (Chrome, Safari, Firefox)
响应式: 移动端优先设计
```

---

## 🗃️ 数据模型设计

### 1. 用户表 (users)
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE,
    last_login_at TIMESTAMP NULL
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_is_active ON users(is_active);
```

### 2. 客户表 (customers)
```sql
CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    address TEXT,
    notes TEXT,
    rate INTEGER DEFAULT 0 CHECK(rate >= 0 AND rate <= 5),
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_customers_user_id ON customers(user_id);
CREATE INDEX idx_customers_is_deleted ON customers(is_deleted);
CREATE INDEX idx_customers_name ON customers(name);
```

### 3. 客户追踪记录表 (customer_tracks)
```sql
CREATE TABLE customer_tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    next_action VARCHAR(64) DEFAULT '继续跟进' CHECK(next_action IN ('继续跟进', '结束跟进')),
    track_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    next_track_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX idx_customer_tracks_customer_id ON customer_tracks(customer_id);
CREATE INDEX idx_customer_tracks_track_time ON customer_tracks(track_time);
```

### 4. Sea-ORM 实体定义

#### User Entity
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    pub is_active: bool,
    pub last_login_at: Option<ChronoDateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::customer::Entity")]
    Customer,
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

#### Customer Entity
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "customers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
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
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
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
```

#### NextAction Enum
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(64))")]
pub enum NextAction {
    #[sea_orm(string_value = "继续跟进")]
    Continue,
    #[sea_orm(string_value = "结束跟进")]
    End,
}

impl Default for NextAction {
    fn default() -> Self {
        NextAction::Continue
    }
}

impl NextAction {
    pub fn as_str(&self) -> &str {
        match self {
            NextAction::Continue => "继续跟进",
            NextAction::End => "结束跟进",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "继续跟进" => Some(NextAction::Continue),
            "结束跟进" => Some(NextAction::End),
            _ => None,
        }
    }
    
    pub fn variants() -> Vec<&'static str> {
        vec!["继续跟进", "结束跟进"]
    }
}
```

#### CustomerTrack Entity
```rust
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
    #[sea_orm(belongs_to = "super::customer::Entity", from = "Column::CustomerId", to = "super::customer::Column::Id")]
    Customer,
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

---

## 🔌 API接口设计

### 1. JWT认证中间件
```rust
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub name: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub name: String,
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(app_state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let current_user = CurrentUser {
        id: claims.claims.user_id,
        username: claims.claims.username,
        name: claims.claims.name,
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
```

### 2. 认证相关API
```rust
// POST /api/auth/login
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
    pub last_login_at: Option<ChronoDateTimeUtc>,
}

// POST /api/auth/refresh - 刷新Token
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub expires_in: i64,
}

// POST /api/auth/logout
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

// GET /api/auth/me - 获取当前用户信息
pub async fn get_current_user(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UserInfo>, StatusCode> {
    Ok(Json(UserInfo {
        id: current_user.id,
        username: current_user.username,
        name: current_user.name,
        last_login_at: None, // 从数据库获取
    }))
}
```

### 3. 客户管理API（带权限控制）
```rust
use crate::entities::next_action::NextAction;
use axum::Extension;

// GET /api/customers?page=1&limit=20&search=
#[derive(Debug, Serialize)]
pub struct CustomerListResponse {
    pub customers: Vec<CustomerWithLatestTrack>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
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

// 客户列表处理器（只返回当前用户的客户）
pub async fn list_customers(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<CustomerListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerListResponse>, StatusCode> {
    let customers = Customer::find()
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        // 添加搜索过滤
        .apply_if(params.search, |query, search| {
            query.filter(
                customer::Column::Name.contains(&search)
                    .or(customer::Column::Phone.contains(&search))
            )
        })
        .order_by_desc(customer::Column::UpdatedAt)
        .paginate(&app_state.db, params.limit as u64);

    // 获取分页数据和总数
    let customers_page = customers.fetch_page(params.page as u64 - 1).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let total = customers.num_items().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ... 处理最新跟进记录关联查询
    
    Ok(Json(CustomerListResponse {
        customers: customer_with_tracks,
        total,
        page: params.page,
        limit: params.limit,
    }))
}

// POST /api/customers - 创建客户（自动关联当前用户）
#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub rate: Option<i32>,
}

pub async fn create_customer(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, StatusCode> {
    let customer = customer::ActiveModel {
        name: Set(req.name),
        phone: Set(req.phone),
        address: Set(req.address),
        notes: Set(req.notes),
        rate: Set(req.rate.unwrap_or(0)),
        user_id: Set(current_user.id), // 自动关联当前用户
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let customer = customer.insert(&app_state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(customer))
}

// PUT /api/customers/:id - 更新客户（只能更新自己的客户）
pub async fn update_customer(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, StatusCode> {
    // 检查客户是否属于当前用户
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 执行更新操作
    let mut customer: customer::ActiveModel = customer.into();
    
    if let Some(name) = req.name {
        customer.name = Set(name);
    }
    // ... 其他字段更新
    customer.updated_at = Set(Utc::now());

    let updated_customer = customer.update(&app_state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_customer))
}
```

### 4. 客户追踪API（带权限控制）
```rust
// GET /api/customers/:id/tracks - 获取客户跟进记录
pub async fn list_customer_tracks(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<Json<CustomerTrackListResponse>, StatusCode> {
    // 首先验证客户是否属于当前用户
    let customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 获取跟进记录
    let tracks = CustomerTrack::find()
        .filter(customer_track::Column::CustomerId.eq(customer_id))
        .order_by_desc(customer_track::Column::TrackTime)
        .all(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackListResponse {
        tracks: tracks.into_iter().map(|t| CustomerTrackInfo::from(t)).collect(),
        customer: CustomerInfo::from(customer),
    }))
}

// POST /api/customers/:id/tracks - 创建跟进记录
pub async fn create_customer_track(
    Extension(current_user): Extension<CurrentUser>,
    Path(customer_id): Path<i32>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateTrackRequest>,
) -> Result<Json<CustomerTrackInfo>, StatusCode> {
    // 验证客户是否属于当前用户
    let _customer = Customer::find_by_id(customer_id)
        .filter(customer::Column::UserId.eq(current_user.id))
        .filter(customer::Column::IsDeleted.eq(false))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let track = customer_track::ActiveModel {
        customer_id: Set(customer_id),
        content: Set(req.content),
        next_action: Set(req.next_action.unwrap_or(NextAction::Continue)),
        track_time: Set(req.track_time.unwrap_or_else(|| Utc::now())),
        next_track_time: Set(req.next_track_time),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let track = track.insert(&app_state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CustomerTrackInfo::from(track)))
}
```

### 5. Axum路由定义
```rust
use axum::{
    routing::{get, post, put, delete},
    Router, middleware,
};
use tower_http::cors::CorsLayer;

pub fn create_routes(app_state: AppState) -> Router {
    Router::new()
        // 公开路由（无需认证）
        .route("/api/auth/login", post(auth::login))
        .route("/api/health", get(|| async { "OK" }))
        
        // 受保护路由（需要认证）
        .route("/api/auth/me", get(auth::get_current_user))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh_token))
        
        // 客户路由
        .route("/api/customers", get(customer::list).post(customer::create))
        .route("/api/customers/:id", 
            get(customer::get)
            .put(customer::update)
            .delete(customer::delete)
        )
        
        // 客户追踪路由
        .route("/api/customers/:id/tracks", 
            get(customer_track::list)
            .post(customer_track::create)
        )
        .route("/api/tracks/:id", 
            put(customer_track::update)
            .delete(customer_track::delete)
        )
        .route("/api/tracks/actions", get(customer_track::get_next_actions))
        
        // 对受保护路由应用认证中间件
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware))
        
        // CORS中间件
        .layer(CorsLayer::permissive())
        
        // 静态文件服务
        .fallback_service(ServeDir::new("dist"))
        
        .with_state(app_state)
}

// 需要将公开路由从认证中间件中排除
pub fn create_routes_with_auth(app_state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/health", get(|| async { "OK" }))
        .with_state(app_state.clone());

    let protected_routes = Router::new()
        .route("/api/auth/me", get(auth::get_current_user))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh_token))
        .route("/api/customers", get(customer::list).post(customer::create))
        .route("/api/customers/:id", 
            get(customer::get).put(customer::update).delete(customer::delete))
        .route("/api/customers/:id/tracks", 
            get(customer_track::list).post(customer_track::create))
        .route("/api/tracks/:id", 
            put(customer_track::update).delete(customer_track::delete))
        .route("/api/tracks/actions", get(customer_track::get_next_actions))
        .layer(middleware::from_fn_with_state(
            app_state.clone(), 
            auth_middleware
        ))
        .with_state(app_state.clone());

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("dist"))
}
```

---

## 🎨 前端界面设计

### 1. TypeScript 类型定义
```typescript
// types/auth.ts
export interface User {
  id: number
  username: string
  name: string
  last_login_at?: string
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  expires_in: number
  user: User
}

export interface AuthState {
  token: string | null
  user: User | null
  isAuthenticated: boolean
}

// types/customer.ts
export interface Customer {
  id: number
  name: string
  phone?: string
  address?: string
  notes?: string
  rate: number
  user_id: number
  created_at: string
  updated_at: string
}

export interface CustomerWithLatestTrack extends Customer {
  latest_track_time?: string
  latest_next_action?: NextAction
  latest_content?: string
}
```

### 2. 认证状态管理 (Pinia Store)
```typescript
// stores/auth.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/services/auth'
import type { User, AuthState, LoginRequest } from '@/types/auth'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const user = ref<User | null>(null)

  const isAuthenticated = computed(() => !!token.value && !!user.value)

  // 初始化认证状态
  const initAuth = async () => {
    const savedToken = localStorage.getItem('token')
    if (savedToken) {
      token.value = savedToken
      try {
        const response = await authApi.getCurrentUser()
        user.value = response.data
        return true
      } catch (error) {
        // Token无效，清除本地存储
        logout()
        return false
      }
    }
    return false
  }

  // 登录
  const login = async (credentials: LoginRequest) => {
    try {
      const response = await authApi.login(credentials)
      token.value = response.data.token
      user.value = response.data.user
      
      // 保存到本地存储
      localStorage.setItem('token', response.data.token)
      
      return { success: true, user: response.data.user }
    } catch (error: any) {
      return { 
        success: false, 
        message: error.response?.data?.message || '登录失败' 
      }
    }
  }

  // 登出
  const logout = async () => {
    try {
      if (token.value) {
        await authApi.logout()
      }
    } catch (error) {
      // 忽略登出API错误
    } finally {
      token.value = null
      user.value = null
      localStorage.removeItem('token')
    }
  }

  // 刷新Token
  const refreshToken = async () => {
    try {
      const response = await authApi.refreshToken()
      token.value = response.data.token
      localStorage.setItem('token', response.data.token)
      return true
    } catch (error) {
      logout()
      return false
    }
  }

  return {
    token,
    user,
    isAuthenticated,
    initAuth,
    login,
    logout,
    refreshToken
  }
})
```

### 3. 登录页面
```vue
<!-- views/Login.vue -->
<template>
  <div class="login-container">
    <div class="login-card">
      <n-card title="客户追踪系统" size="large">
        <template #header-extra>
          <n-icon :component="PersonCircleOutline" size="24" />
        </template>
        
        <n-form
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginFormRules"
          size="large"
          :show-require-mark="false"
        >
          <n-form-item path="username">
            <n-input
              v-model:value="loginForm.username"
              placeholder="请输入用户名"
              clearable
              @keydown.enter="handleLogin"
            >
              <template #prefix>
                <n-icon :component="PersonOutline" />
              </template>
            </n-input>
          </n-form-item>
          
          <n-form-item path="password">
            <n-input
              v-model:value="loginForm.password"
              type="password"
              placeholder="请输入密码"
              show-password-on="click"
              @keydown.enter="handleLogin"
            >
              <template #prefix>
                <n-icon :component="LockClosedOutline" />
              </template>
            </n-input>
          </n-form-item>
          
          <n-form-item>
            <n-checkbox v-model:checked="rememberMe">
              记住登录状态
            </n-checkbox>
          </n-form-item>
        </n-form>
        
        <template #action>
          <n-space vertical size="large" style="width: 100%">
            <n-button
              type="primary"
              size="large"
              block
              :loading="loginLoading"
              @click="handleLogin"
            >
              登录
            </n-button>
            
            <n-text depth="3" style="text-align: center; display: block">
              客户追踪管理系统 v1.0
            </n-text>
          </n-space>
        </template>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useMessage, type FormInst, type FormRules } from 'naive-ui'
import { PersonCircleOutline, PersonOutline, LockClosedOutline } from '@vicons/ionicons5'
import { useAuthStore } from '@/stores/auth'
import type { LoginRequest } from '@/types/auth'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const authStore = useAuthStore()

const loginFormRef = ref<FormInst | null>(null)
const loginLoading = ref(false)
const rememberMe = ref(true)

const loginForm = reactive<LoginRequest>({
  username: '',
  password: ''
})

const loginFormRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: ['input', 'blur'] },
    { min: 3, max: 50, message: '用户名长度为 3-50 个字符', trigger: ['input', 'blur'] }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: ['input', 'blur'] },
    { min: 6, message: '密码长度不能少于 6 个字符', trigger: ['input', 'blur'] }
  ]
}

const handleLogin = async () => {
  if (!loginFormRef.value) return
  
  try {
    await loginFormRef.value.validate()
    loginLoading.value = true
    
    const result = await authStore.login(loginForm)
    
    if (result.success) {
      message.success(`欢迎回来，${result.user?.name}！`)
      
      // 跳转到目标页面或首页
      const redirect = route.query.redirect as string || '/customers'
      router.push(redirect)
    } else {
      message.error(result.message || '登录失败')
    }
  } catch (error) {
    // 表单验证失败
  } finally {
    loginLoading.value = false
  }
}

// 页面加载时检查是否已登录
onMounted(() => {
  if (authStore.isAuthenticated) {
    router.push('/customers')
  }
})
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.login-card :deep(.n-card) {
  background: rgba(255, 255, 255, 0.95);
}

.login-card :deep(.n-card-header) {
  text-align: center;
  font-weight: 600;
  font-size: 20px;
  color: #333;
}

@media (max-width: 480px) {
  .login-container {
    padding: 16px;
  }
  
  .login-card {
    max-width: none;
  }
}
</style>
```

### 4. 路由守卫
```typescript
// router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import Login from '@/views/Login.vue'
import CustomerList from '@/views/CustomerList.vue'
import CustomerDetail from '@/views/CustomerDetail.vue'

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: Login,
    meta: { requiresAuth: false }
  },
  {
    path: '/',
    redirect: '/customers'
  },
  {
    path: '/customers',
    name: 'CustomerList',
    component: CustomerList,
    meta: { requiresAuth: true }
  },
  {
    path: '/customers/:id',
    name: 'CustomerDetail',
    component: CustomerDetail,
    meta: { requiresAuth: true }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()
  
  // 如果路由需要认证
  if (to.meta.requiresAuth) {
    // 检查是否已认证
    if (!authStore.isAuthenticated) {
      // 尝试从本地存储恢复认证状态
      const restored = await authStore.initAuth()
      
      if (!restored) {
        // 跳转到登录页，并保存目标路由
        next({
          path: '/login',
          query: { redirect: to.fullPath }
        })
        return
      }
    }
  }
  
  // 如果已登录用户访问登录页，跳转到客户列表
  if (to.path === '/login' && authStore.isAuthenticated) {
    next('/customers')
    return
  }
  
  next()
})

export default router
```

### 5. HTTP请求拦截器
```typescript
// utils/request.ts
import axios, { type AxiosResponse, type AxiosError } from 'axios'
import { useAuthStore } from '@/stores/auth'
import { useMessage } from 'naive-ui'
import router from '@/router'

const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
  timeout: 10000
})

// 请求拦截器 - 添加认证头
request.interceptors.request.use(
  (config) => {
    const authStore = useAuthStore()
    
    if (authStore.token) {
      config.headers.Authorization = `Bearer ${authStore.token}`
    }
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器 - 处理认证错误
request.interceptors.response.use(
  (response: AxiosResponse) => {
    return response
  },
  async (error: AxiosError) => {
    const authStore = useAuthStore()
    const message = useMessage()
    
    if (error.response) {
      const status = error.response.status
      
      switch (status) {
        case 401:
          // Token过期或无效
          message.error('登录已过期，请重新登录')
          await authStore.logout()
          router.push('/login')
          break
          
        case 403:
          message.error('没有权限执行此操作')
          break
          
        case 404:
          message.error('请求的资源不存在')
          break
          
        case 500:
          message.error('服务器内部错误')
          break
          
        default:
          message.error(error.response.data?.message || '请求失败')
      }
    } else if (error.request) {
      message.error('网络连接错误')
    } else {
      message.error('请求配置错误')
    }
    
    return Promise.reject(error)
  }
)

export default request
```

### 6. 更新后的客户列表页面
```vue
<!-- views/CustomerList.vue -->
<template>
  <div class="customer-list-container">
    <!-- 页面头部 -->
    <n-page-header @back="null">
      <template #title>
        客户管理
      </template>
      <template #extra>
        <n-space>
          <n-text depth="3">
            欢迎，{{ authStore.user?.name }}
          </n-text>
          <n-dropdown :options="userMenuOptions" @select="handleUserMenuSelect">
            <n-button quaternary circle>
              <n-icon :component="PersonOutline" size="20" />
            </n-button>
          </n-dropdown>
        </n-space>
      </template>
    </n-page-header>

    <!-- 搜索和过滤 -->
    <n-space class="search-bar" justify="space-between">
      <n-input 
        v-model:value="searchQuery"
        placeholder="搜索客户姓名或电话"
        clearable
        @input="handleSearch"
        style="max-width: 300px"
      >
        <template #prefix>
          <n-icon :component="SearchOutlined" />
        </template>
      </n-input>
      
      <n-button type="primary" @click="showCreateModal = true">
        <template #icon>
          <n-icon :component="PlusOutlined" />
        </template>
        添加客户
      </n-button>
    </n-space>

    <!-- 客户列表 -->
    <n-spin :show="loading">
      <n-list bordered v-if="customers.length > 0">
        <n-list-item v-for="customer in customers" :key="customer.id">
          <CustomerCard 
            :customer="customer" 
            @click="goToDetail(customer.id)"
            @edit="handleEditCustomer"
            @delete="handleDeleteCustomer"
          />
        </n-list-item>
      </n-list>
      
      <n-empty 
        v-else-if="!loading" 
        description="暂无客户数据"
        size="large"
        style="margin: 40px 0"
      >
        <template #extra>
          <n-button size="small" @click="showCreateModal = true">
            创建第一个客户
          </n-button>
        </template>
      </n-empty>
    </n-spin>

    <!-- 分页 -->
    <n-pagination
      v-if="customers.length > 0"
      v-model:page="currentPage"
      :page-count="totalPages"
      show-size-picker
      :page-sizes="[10, 20, 50]"
      :page-slot="7"
      @update:page="handlePageChange"
      @update:page-size="handlePageSizeChange"
    />

    <!-- 创建/编辑客户弹窗 -->
    <CustomerFormModal 
      v-model:show="showCreateModal"
      :customer="editingCustomer"
      @success="handleCustomerSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { 
  SearchOutlined, 
  PlusOutlined, 
  PersonOutline,
  LogOutOutline,
  SettingsOutline 
} from '@vicons/ionicons5'
import { useAuthStore } from '@/stores/auth'
import { customerApi } from '@/services/customer'
import type { CustomerWithLatestTrack } from '@/types/customer'
import CustomerCard from '@/components/CustomerCard.vue'
import CustomerFormModal from '@/components/CustomerFormModal.vue'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()

// 状态管理
const loading = ref(false)
const customers = ref<CustomerWithLatestTrack[]>([])
const searchQuery = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const totalCount = ref(0)
const showCreateModal = ref(false)
const editingCustomer = ref<CustomerWithLatestTrack | null>(null)

// 计算属性
const totalPages = computed(() => Math.ceil(totalCount.value / pageSize.value))

// 用户菜单选项
const userMenuOptions = [
  {
    label: '个人设置',
    key: 'settings',
    icon: () => h(SettingsOutline)
  },
  {
    type: 'divider'
  },
  {
    label: '退出登录',
    key: 'logout',
    icon: () => h(LogOutOutline)
  }
]

// 处理用户菜单选择
const handleUserMenuSelect = (key: string) => {
  switch (key) {
    case 'settings':
      // TODO: 打开设置页面
      message.info('个人设置功能开发中')
      break
    case 'logout':
      handleLogout()
      break
  }
}

// 退出登录
const handleLogout = async () => {
  await authStore.logout()
  message.success('已成功退出登录')
  router.push('/login')
}

// 加载客户列表
const loadCustomers = async () => {
  try {
    loading.value = true
    const response = await customerApi.getCustomers({
      page: currentPage.value,
      limit: pageSize.value,
      search: searchQuery.value
    })
    
    customers.value = response.data.customers
    totalCount.value = response.data.total
  } catch (error) {
    message.error('加载客户列表失败')
  } finally {
    loading.value = false
  }
}

// 其他事件处理函数...
const goToDetail = (customerId: number) => {
  router.push(`/customers/${customerId}`)
}

const handleEditCustomer = (customer: CustomerWithLatestTrack) => {
  editingCustomer.value = customer
  showCreateModal.value = true
}

const handleDeleteCustomer = async (customer: CustomerWithLatestTrack) => {
  // TODO: 实现删除确认对话框
  try {
    await customerApi.deleteCustomer(customer.id)
    message.success('客户删除成功')
    loadCustomers()
  } catch (error) {
    message.error('删除客户失败')
  }
}

const handleCustomerSaved = () => {
  showCreateModal.value = false
  editingCustomer.value = null
  loadCustomers()
}

const handleSearch = () => {
  currentPage.value = 1
  loadCustomers()
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  loadCustomers()
}

const handlePageSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  loadCustomers()
}

// 页面加载
onMounted(() => {
  loadCustomers()
})
</script>

<style scoped>
.customer-list-container {
  padding: 16px;
  max-width: 1200px;
  margin: 0 auto;
}

.search-bar {
  margin: 24px 0;
  flex-direction: column;
}

@media (min-width: 768px) {
  .search-bar {
    flex-direction: row;
  }
}

.n-pagination {
  margin-top: 24px;
  justify-content: center;
}
</style>
```

---

## 📁 项目结构

```
customer-tracker/
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── cli.rs              # 命令行工具
│   │   ├── config.rs           # 配置管理
│   │   ├── database.rs         # 数据库连接
│   │   ├── entities/           # Sea-ORM实体
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── customer.rs
│   │   │   ├── customer_track.rs
│   │   │   └── next_action.rs  # 枚举定义
│   │   ├── handlers/           # HTTP处理器
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs         # 认证相关
│   │   │   ├── customer.rs
│   │   │   └── customer_track.rs
│   │   ├── services/           # 业务逻辑层
│   │   │   ├── mod.rs
│   │   │   ├── auth_service.rs
│   │   │   ├── customer_service.rs
│   │   │   └── track_service.rs
│   │   ├── middleware/         # 中间件
│   │   │   ├── mod.rs
│   │   │   └── auth.rs         # JWT认证中间件
│   │   ├── utils/              # 工具函数
│   │   │   ├── mod.rs
│   │   │   ├── password.rs
│   │   │   ├── jwt.rs          # JWT工具
│   │   │   └── validation.rs   # 数据验证
│   │   └── routes.rs           # 路由定义
│   ├── migrations/             # 数据库迁移
│   │   ├── 001_create_users.sql
│   │   ├── 002_create_customers.sql
│   │   └── 003_create_customer_tracks.sql
│   ├── Cargo.toml
│   └── .env
├── frontend/
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── router/
│   │   │   └── index.ts        # 路由配置和守卫
│   │   ├── stores/             # Pinia状态管理
│   │   │   ├── auth.ts         # 认证状态
│   │   │   ├── customer.ts
│   │   │   └── track.ts
│   │   ├── views/              # 页面组件
│   │   │   ├── Login.vue       # 登录页面
│   │   │   ├── CustomerList.vue # 客户列表
│   │   │   └── CustomerDetail.vue
│   │   ├── components/         # 公共组件
│   │   │   ├── CustomerCard.vue
│   │   │   ├── CustomerFormModal.vue
│   │   │   ├── TrackTimeline.vue
│   │   │   ├── NextActionSelect.vue
│   │   │   └── ProtectedRoute.vue # 路由保护组件
│   │   ├── services/           # API服务
│   │   │   ├── api.ts
│   │   │   ├── auth.ts         # 认证API
│   │   │   ├── customer.ts
│   │   │   └── track.ts
│   │   ├── types/              # TypeScript类型定义
│   │   │   ├── auth.ts         # 认证相关类型
│   │   │   ├── customer.ts
│   │   │   └── track.ts
│   │   └── utils/              # 工具函数
│   │       ├── format.ts
│   │       ├── request.ts      # HTTP请求拦截器
│   │       └── storage.ts      # 本地存储工具
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── .env.development
├── README.md
├── docker-compose.yml          # 部署配置
└── .github/workflows/          # CI/CD配置
    └── deploy.yml
```

---

## 🛠️ 开发计划

### 第一阶段：认证系统搭建 (1.5周)
- [ ] 项目初始化和依赖安装
- [ ] 数据库模型设计和迁移
- [ ] JWT认证中间件实现
- [ ] 用户登录/登出API
- [ ] 前端登录页面和路由守卫
- [ ] CLI用户管理工具

### 第二阶段：核心功能开发 (2周)
- [ ] 客户信息CRUD API（带权限控制）
- [ ] 客户追踪记录API（带权限控制）
- [ ] NextAction枚举类型实现
- [ ] 前端认证状态管理
- [ ] 客户列表页面（显示当前用户的客户）

### 第三阶段：界面完善和优化 (1.5周)
- [ ] 客户详情页面和时间线
- [ ] NextAction状态显示优化
- [ ] 移动端响应式优化
- [ ] 用户体验优化
- [ ] 错误处理和加载状态

### 第四阶段：测试和部署 (1周)
- [ ] 单元测试和集成测试
- [ ] 权限控制边界测试
- [ ] 性能优化
- [ ] 安全性测试
- [ ] Docker部署配置
- [ ] 文档完善

---

## 🔧 开发环境配置

### 后端开发环境
```toml
# Cargo.toml
[dependencies]
axum = "0.8"
axum-extra = { version = "0.9", features = ["typed-header"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.0", features = ["derive"] }
jsonwebtoken = "9.0"
bcrypt = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"
anyhow = "1.0"
thiserror = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### 前端开发环境
```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "pinia": "^2.1.0",
    "naive-ui": "^2.35.0",
    "@vicons/ionicons5": "^0.12.0",
    "axios": "^1.6.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.5.0",
    "vite": "^5.0.0",
    "typescript": "^5.2.0",
    "@types/node": "^20.8.0",
    "unplugin-auto-import": "^0.17.0",
    "unplugin-vue-components": "^0.26.0"
  }
}
```

### 环境变量配置
```bash
# backend/.env
DATABASE_URL=sqlite://./customer_tracker.db
JWT_SECRET=your-super-secret-jwt-key-here-change-in-production-min-256-bits
JWT_EXPIRE_HOURS=24
LOG_LEVEL=debug
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# 默认管理员账户（仅开发环境）
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin123
ADMIN_NAME=系统管理员

# CORS配置
CORS_ORIGIN=http://localhost:5173
```

```bash
# frontend/.env.development
VITE_API_BASE_URL=http://localhost:3000
VITE_APP_TITLE=客户追踪系统
VITE_TOKEN_STORAGE_KEY=customer_tracker_token
```

### CLI命令示例
```rust
// src/cli.rs
use clap::{Args, Parser, Subcommand};
use crate::entities::next_action::NextAction;

#[derive(Parser)]
#[command(name = "customer-tracker")]
#[command(about = "客户追踪系统管理工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 用户管理
    User(UserArgs),
    /// 数据库管理
    Database(DatabaseArgs),
    /// 服务器管理
    Server(ServerArgs),
}

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub action: UserAction,
}

#[derive(Subcommand)]
pub enum UserAction {
    /// 创建新用户
    Create {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        name: String,
    },
    /// 列出所有用户
    List {
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// 重置用户密码
    ResetPassword {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    /// 禁用/启用用户
    Toggle {
        #[arg(short, long)]
        username: String,
    },
}

#[derive(Args)]
pub struct DatabaseArgs {
    #[command(subcommand)]
    pub action: DatabaseAction,
}

#[derive(Subcommand)]
pub enum DatabaseAction {
    /// 运行数据库迁移
    Migrate,
    /// 创建迁移文件
    CreateMigration {
        #[arg(short, long)]
        name: String,
    },
    /// 数据库状态
    Status,
}

#[derive(Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub action: ServerAction,
}

#[derive(Subcommand)]
pub enum ServerAction {
    /// 启动服务器
    Start {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// 生成JWT密钥
    GenerateJwtSecret,
}

// 使用示例：
// cargo run -- user create -u admin -p password123 -n "系统管理员"
// cargo run -- user list --limit 20
// cargo run -- database migrate
// cargo run -- server start --port 8080
// cargo run -- server generate-jwt-secret
```

### 开发启动脚本
```bash
# scripts/dev.sh
#!/bin/bash

echo "启动客户追踪系统开发环境..."

# 检查是否安装了必要的工具
command -v cargo >/dev/null 2>&1 || { echo "请先安装 Rust"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "请先安装 Node.js"; exit 1; }

# 启动后端
echo "启动后端服务..."
cd backend
if [ ! -f ".env" ]; then
    echo "创建后端环境变量文件..."
    cp .env.example .env
fi
cargo run -- database migrate
cargo run -- user create -u admin -p admin123 -n "管理员" 2>/dev/null || true
cargo run -- server start &
BACKEND_PID=$!

# 等待后端启动
sleep 3

# 启动前端
echo "启动前端服务..."
cd ../frontend
if [ ! -f ".env.development" ]; then
    echo "创建前端环境变量文件..."
    cp .env.development.example .env.development
fi
npm install
npm run dev &
FRONTEND_PID=$!

echo "开发环境已启动："
echo "- 后端服务: http://localhost:3000"
echo "

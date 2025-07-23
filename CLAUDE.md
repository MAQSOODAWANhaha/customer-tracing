# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个客户追踪系统，使用 Rust (Axum + Sea-ORM) 作为后端，Vue 3 + TypeScript + Naive UI 作为前端。支持多用户使用，具有严格的数据隔离 - 每个用户只能访问自己创建的客户信息。

## 开发命令

### 后端 (Rust)
```bash
# 项目初始化
cargo init backend
cd backend

# 运行数据库迁移
cargo run -- database migrate

# 创建用户（开发阶段）
cargo run -- user create -u admin -p admin123 -n "管理员"

# 启动开发服务器
cargo run -- server start --port 3000

# 生成JWT密钥
cargo run -- server generate-jwt-secret
```

### 前端 (Vue 3)
```bash
# 项目初始化
npm create vue@latest frontend
cd frontend

# 安装依赖
npm install

# 开发服务器
npm run dev

# 构建生产版本
npm run build
```

## 核心架构

### 认证系统
- JWT Token 认证，中间件保护API路由
- 严格的权限控制：用户只能操作自己创建的数据
- 前端路由守卫，自动跳转到登录页
- Token存储在localStorage，支持自动续期

### 数据模型
- **users**: 用户表，包含认证信息
- **customers**: 客户表，通过user_id关联到用户（软删除）
- **customer_tracks**: 客户跟进记录表，支持"继续跟进"/"结束跟进"状态

### 关键特性
1. **数据隔离**: 所有客户相关API都通过user_id过滤，确保用户只能访问自己的数据
2. **NextAction枚举**: 跟进记录支持"继续跟进"和"结束跟进"两种状态
3. **移动端优先**: 响应式设计，优化移动端浏览器体验
4. **CLI管理工具**: 支持命令行管理用户和数据库

### 技术栈重点
- **后端**: Axum 0.8.x + Sea-ORM + SQLite + JWT + bcrypt
- **前端**: Vue 3 Composition API + Pinia + Naive UI + TypeScript + Vite
- **认证**: JWT Token + 路由守卫 + 请求拦截器

### 开发注意事项
- 所有API都需要权限验证，确保数据隔离
- NextAction枚举使用中文值："继续跟进"、"结束跟进"
- 前端使用TypeScript，需要定义完整的类型
- 遵循RESTful API设计原则
- 支持分页查询和搜索功能

### 环境变量
后端需要配置：DATABASE_URL, JWT_SECRET, SERVER_PORT
前端需要配置：VITE_API_BASE_URL
# 客户追踪系统 - 快速开始 🚀

## 🎯 项目概述

这是一个基于 Rust + Vue3 的客户追踪管理系统，支持多用户数据隔离，每个用户只能管理自己的客户信息。

## 🛠️ 技术栈

### 后端
- **语言**: Rust Edition 2021
- **框架**: Axum 0.8.x
- **数据库**: SQLite + Sea-ORM
- **认证**: JWT Token

### 前端
- **框架**: Vue 3 + TypeScript
- **UI库**: Naive UI
- **状态管理**: Pinia
- **路由**: Vue Router 4

## 📦 快速启动

### 1. 环境要求
```bash
# 检查 Rust 安装
cargo --version

# 检查 Node.js 安装
npm --version
```

### 2. 一键启动
```bash
# 进入项目目录
cd customer-tracing

# 使用开发脚本启动（推荐）
./start-dev.sh
```

### 3. 手动启动

#### 启动后端
```bash
cd backend

# 初始化数据库和创建默认用户
cargo run --bin migrate
cargo run --bin cli -- create-user admin admin123 "管理员"

# 启动后端服务
cargo run
```

#### 启动前端
```bash
cd frontend

# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

## 🌐 访问应用

- **前端界面**: http://localhost:5173
- **后端API**: http://localhost:3000

## 🔑 默认账户

- **用户名**: admin
- **密码**: admin123

## 💡 主要功能

### ✅ 已完成功能

1. **用户认证系统**
   - JWT Token 认证
   - 自动登录和token刷新
   - 路由保护

2. **客户信息管理**
   - 客户列表展示（分页、搜索）
   - 客户信息CRUD操作
   - 数据隔离（用户只能访问自己的客户）

3. **客户跟进追踪**
   - 跟进记录时间线展示
   - 添加/编辑/删除跟进记录
   - 下一步行动状态管理（继续跟进/结束跟进）

4. **响应式设计**
   - 移动端优先设计
   - 现代化UI界面

5. **CLI工具**
   - 用户管理命令行工具
   - 数据库迁移工具

### 🎨 界面预览

- **登录页面**: 渐变背景 + 卡片式登录
- **客户列表**: 网格布局卡片展示
- **客户详情**: 客户信息 + 跟进时间线
- **跟进记录**: 时间线展示 + 快捷模板

## 📁 项目结构

```
customer-tracing/
├── backend/                 # Rust 后端
│   ├── src/
│   │   ├── entities/       # Sea-ORM 实体
│   │   ├── handlers/       # API 处理器
│   │   ├── middleware/     # JWT 中间件
│   │   └── ...
│   └── migrations/         # 数据库迁移
├── frontend/               # Vue3 前端
│   ├── src/
│   │   ├── views/         # 页面组件
│   │   ├── components/    # 公共组件
│   │   ├── stores/        # Pinia 状态管理
│   │   └── ...
├── start-dev.sh           # 开发启动脚本
└── README.md              # 详细文档
```

## 🔧 开发指南

### 添加新用户
```bash
cd backend
cargo run --bin cli -- create-user <username> <password> <name>
```

### 数据库迁移
```bash
cd backend
cargo run --bin migrate
```

### 构建生产版本
```bash
# 后端
cd backend
cargo build --release

# 前端
cd frontend
npm run build
```

## 🐛 常见问题

1. **端口冲突**: 确保 3000 和 5173 端口未被占用
2. **数据库错误**: 删除 `data/customer_tracker.db` 后重新迁移
3. **依赖安装失败**: 检查网络连接，使用国内镜像源

## 📄 详细文档

查看 [README.md](./README.md) 获取完整的技术文档和API说明。

## 🎉 开始使用

1. 启动系统：`./start-dev.sh`
2. 访问：http://localhost:5173
3. 登录：admin / admin123
4. 开始管理您的客户！
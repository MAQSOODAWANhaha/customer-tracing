# Docker 部署指南 🐳

## 🚀 快速开始

### 一键部署（推荐）

```bash
# 克隆项目
git clone <repository-url>
cd customer-tracing

# 一键部署开发环境
./deploy.sh deploy

# 访问应用
# 前端: http://localhost
# 后端API: http://localhost:3000
# 默认账户: admin / admin123
```

## 📋 部署选项

### 1. 开发环境部署

```bash
# 使用部署脚本（推荐）
./deploy.sh deploy dev

# 或手动执行
docker-compose up -d --build
```

**开发环境特点：**
- 前端和后端分别暴露端口
- 详细的调试日志
- 热重载支持
- 开发工具集成

### 2. 生产环境部署

```bash
# 配置生产环境变量
cp .env.example .env
# 编辑 .env 文件，设置生产环境参数

# 部署生产环境
./deploy.sh deploy prod

# 或手动执行
docker-compose -f docker-compose.prod.yaml up -d --build
```

**生产环境特点：**
- Nginx 反向代理
- 资源优化和缓存
- 安全头设置
- 健康检查
- 资源限制

## 🛠️ 部署脚本命令

```bash
# 部署
./deploy.sh deploy [dev|prod]     # 部署指定环境
./deploy.sh stop [dev|prod]       # 停止服务
./deploy.sh logs [dev|prod] [service]  # 查看日志
./deploy.sh cleanup               # 清理所有资源
./deploy.sh help                  # 显示帮助
```

### 使用示例

```bash
# 部署开发环境
./deploy.sh deploy dev

# 查看后端日志
./deploy.sh logs dev backend

# 部署生产环境
./deploy.sh deploy prod

# 停止生产环境
./deploy.sh stop prod

# 清理所有 Docker 资源
./deploy.sh cleanup
```

## ⚙️ 环境变量配置

### 开发环境 (.env)

```bash
# JWT 配置
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRE_HOURS=24

# 后端配置
RUST_LOG=info
CORS_ORIGIN=http://localhost

# 前端配置（Docker环境使用相对路径）
API_BASE_URL=/api
```

### 本地开发环境 (frontend/.env.local)

```bash
# 本地npm run dev时使用
VITE_API_BASE_URL=http://localhost:3000
VITE_APP_TITLE=客户追踪系统
VITE_TOKEN_STORAGE_KEY=customer_tracker_token
```

### 生产环境 (.env)

```bash
# JWT 配置（必须修改）
JWT_SECRET=your-production-jwt-secret-min-256-bits
JWT_EXPIRE_HOURS=24

# 后端配置
RUST_LOG=warn
CORS_ORIGIN=https://your-domain.com

# 前端配置（通过Nginx代理）
API_BASE_URL=/api
```

## 🏗️ 架构说明

### 开发环境架构

```
外部访问:
  http://localhost ──→ Frontend Container (Nginx:80)
  http://localhost:3000 ──→ Backend Container (Rust:3000)

容器间通信:
┌─────────────────┐    /api/*     ┌─────────────────┐
│   Frontend      │──────────────→│    Backend      │
│   (Vue3/Nginx)  │               │    (Rust)       │
│   Port: 80      │←──────────────│   Port: 3000    │
│   Nginx Proxy   │   Response    │   Axum + SQLite │
└─────────────────┘               └─────────────────┘
       │                                    │
       │                                    │
    Docker Network: customer-tracker-network
    Frontend uses service name 'backend:3000'
```

### 生产环境架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Nginx Proxy   │    │   Frontend      │    │    Backend      │
│   Port: 80/443  │────│   (Vue3)        │────│    (Rust)       │
│   Load Balancer │    │   Static Files  │    │   Axum + SQLite │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📁 目录结构

```
customer-tracing/
├── docker-compose.yaml          # 开发环境配置
├── docker-compose.prod.yaml     # 生产环境配置
├── .env.example                 # 环境变量模板
├── deploy.sh                    # 一键部署脚本
├── backend/
│   ├── Dockerfile              # 后端镜像构建
│   └── .dockerignore
├── frontend/
│   ├── Dockerfile              # 前端镜像构建
│   ├── nginx.conf              # Nginx 配置
│   └── .dockerignore
├── data/                       # 数据持久化目录
├── logs/                       # 日志目录
└── ssl/                        # SSL 证书目录
```

## 🔧 手动部署步骤

### 1. 前置要求

```bash
# 检查 Docker 版本
docker --version
docker-compose --version

# 创建必要目录
mkdir -p data logs ssl
```

### 2. 构建镜像

```bash
# 构建后端镜像
docker build -t customer-tracker-backend ./backend

# 构建前端镜像
docker build -t customer-tracker-frontend ./frontend
```

### 3. 运行容器

```bash
# 启动开发环境
docker-compose up -d

# 启动生产环境
docker-compose -f docker-compose.prod.yaml up -d
```

## 📊 监控和日志

### 查看服务状态

```bash
# 开发环境
docker-compose ps

# 生产环境
docker-compose -f docker-compose.prod.yaml ps
```

### 查看日志

```bash
# 查看所有日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f backend
docker-compose logs -f frontend

# 查看实时日志
./deploy.sh logs dev backend
```

### 健康检查

```bash
# 检查后端健康状态
curl http://localhost:3000/api/health

# 检查前端健康状态
curl http://localhost/health
```

## 🔒 安全配置

### 生产环境安全建议

1. **修改默认密码**
   ```bash
   # 进入后端容器
   docker exec -it customer-tracker-backend cli reset-password admin new-secure-password
   ```

2. **使用强 JWT 密钥**
   ```bash
   # 生成安全的 JWT 密钥
   openssl rand -base64 64
   ```

3. **配置 SSL 证书**
   ```bash
   # 将证书文件放入 ssl/ 目录
   cp your-cert.pem ssl/
   cp your-key.pem ssl/
   ```

4. **网络安全**
   - 使用防火墙限制端口访问
   - 配置 CORS 只允许信任的域名
   - 定期更新镜像和依赖

## 🚨 故障排除

### 常见问题

1. **端口被占用**
   ```bash
   # 检查端口占用
   lsof -i :80
   lsof -i :3000
   
   # 修改端口映射
   # 编辑 docker-compose.yaml 中的 ports 配置
   ```

2. **数据库连接失败**
   ```bash
   # 检查数据目录权限
   ls -la data/
   
   # 重新初始化数据库
   docker exec customer-tracker-backend migrate
   ```

3. **前端无法访问后端**
   ```bash
   # 检查网络连接
   docker network ls
   docker network inspect customer-tracker_customer-tracker-network
   ```

4. **容器无法启动**
   ```bash
   # 查看详细错误日志
   docker-compose logs backend
   docker-compose logs frontend
   
   # 检查镜像构建
   docker images | grep customer-tracker
   ```

### 清理和重置

```bash
# 停止所有服务
./deploy.sh stop

# 清理所有资源（谨慎使用）
./deploy.sh cleanup

# 重新构建和部署
./deploy.sh deploy
```

## 📈 性能优化

### 资源限制

生产环境已配置资源限制：

```yaml
deploy:
  resources:
    limits:
      memory: 512M      # 后端最大内存
      cpus: '0.5'       # 后端最大CPU
    reservations:
      memory: 256M      # 后端保留内存
      cpus: '0.25'      # 后端保留CPU
```

### 缓存优化

- 静态资源启用1年缓存
- API 响应启用适当缓存
- Nginx gzip 压缩
- 镜像分层优化

## 📝 升级指南

### 应用更新

```bash
# 拉取最新代码
git pull origin main

# 重新构建和部署
./deploy.sh deploy prod

# 或分步执行
docker-compose -f docker-compose.prod.yaml down
docker-compose -f docker-compose.prod.yaml build --no-cache
docker-compose -f docker-compose.prod.yaml up -d
```

### 数据备份

```bash
# 备份数据库
cp data/customer_tracker.db data/customer_tracker.db.backup.$(date +%Y%m%d_%H%M%S)

# 备份配置文件
tar -czf config_backup_$(date +%Y%m%d_%H%M%S).tar.gz .env docker-compose*.yaml
```

## 🎯 生产部署清单

- [ ] 修改默认JWT密钥
- [ ] 配置生产环境域名和CORS
- [ ] 修改默认管理员密码
- [ ] 配置SSL证书（可选）
- [ ] 设置防火墙规则
- [ ] 配置日志轮转
- [ ] 设置监控告警
- [ ] 数据备份策略
- [ ] 性能测试验证
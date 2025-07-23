# 多阶段构建 - 构建阶段
FROM docker.m.daocloud.io/node:18-alpine as builder

WORKDIR /app

# 配置 npm 中国镜像源
RUN npm config set registry https://registry.npmmirror.com && \
    npm config set cache /tmp/.npm

# 复制 package 文件
COPY package*.json ./

# 安装依赖
RUN npm ci --only=production

# 复制源代码
COPY . .

# 设置构建时的环境变量
ARG VITE_API_BASE_URL=/api
ARG VITE_TOKEN_STORAGE_KEY=customer_tracker_token
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL
ENV VITE_TOKEN_STORAGE_KEY=$VITE_TOKEN_STORAGE_KEY

# 构建应用
RUN npm run build

# 运行阶段 - 使用 nginx 提供静态文件服务
FROM docker.m.daocloud.io/nginx:alpine

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制 nginx 配置
COPY nginx.conf /etc/nginx/conf.d/default.conf

# 暴露端口
EXPOSE 80

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
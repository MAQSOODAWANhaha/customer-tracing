# 多阶段构建 - 构建阶段
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 来预编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# 复制源代码
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN touch src/main.rs && cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/customer-tracker /usr/local/bin/customer-tracker
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate
COPY --from=builder /app/target/release/cli /usr/local/bin/cli

# 复制迁移文件
COPY migrations ./migrations

# 创建数据目录
RUN mkdir -p /app/data

# 设置环境变量
ENV DATABASE_URL=sqlite:///app/data/customer_tracker.db
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
ENV RUST_LOG=info

# 暴露端口
EXPOSE 3000

# 创建启动脚本
RUN echo '#!/bin/bash\n\
set -e\n\
echo "Running database migrations..."\n\
migrate\n\
echo "Creating default admin user if not exists..."\n\
cli create-user admin admin123 "管理员" 2>/dev/null || echo "Admin user already exists"\n\
echo "Starting server..."\n\
exec customer-tracker' > /usr/local/bin/start.sh

RUN chmod +x /usr/local/bin/start.sh

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["/usr/local/bin/start.sh"]
# 多阶段构建 - 构建阶段
FROM docker.m.daocloud.io/rust:1.75-slim as builder

# 配置中国镜像源
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && \
    sed -i 's/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# 配置 Rust 中国镜像源
RUN mkdir -p ~/.cargo && \
    echo '[source.crates-io]' > ~/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> ~/.cargo/config.toml && \
    echo '[source.ustc]' >> ~/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> ~/.cargo/config.toml

WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码和迁移文件
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN cargo build --release

# 运行阶段
FROM docker.m.daocloud.io/debian:bookworm-slim

# 配置中国镜像源
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && \
    sed -i 's/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/customer-tracker /app/customer-tracker

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
/app/customer-tracker database migrate\n\
echo "Creating default admin user if not exists..."\n\
/app/customer-tracker user create -u admin -p admin123 -n "管理员" 2>/dev/null || echo "Admin user already exists"\n\
echo "Starting server..."\n\
exec /app/customer-tracker server start --port 3000 --host 0.0.0.0' > /usr/local/bin/start.sh

RUN chmod +x /usr/local/bin/start.sh

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["/usr/local/bin/start.sh"]
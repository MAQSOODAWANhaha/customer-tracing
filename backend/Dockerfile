# 多阶段构建 - 构建阶段
FROM docker.m.daocloud.io/rust:1.88.0-slim as builder


# 配置中国镜像源
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && \
    sed -i 's/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# 配置 Cargo 中国镜像源 (关键修复)
RUN mkdir -p $CARGO_HOME && \
    echo '[source.crates-io]' > $CARGO_HOME/config.toml && \
    echo 'replace-with = "ustc"' >> $CARGO_HOME/config.toml && \
    echo '' >> $CARGO_HOME/config.toml && \
    echo '[source.ustc]' >> $CARGO_HOME/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> $CARGO_HOME/config.toml && \
    echo '' >> $CARGO_HOME/config.toml && \
    echo '[net]' >> $CARGO_HOME/config.toml && \
    echo 'git-fetch-with-cli = true' >> $CARGO_HOME/config.toml

# 或者使用以下环境变量方式 (推荐)
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_REGISTRIES_CRATES_IO_INDEX=https://mirrors.ustc.edu.cn/crates.io-index/

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

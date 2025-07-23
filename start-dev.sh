#!/bin/bash

# 客户追踪系统开发启动脚本
echo "🚀 启动客户追踪系统开发环境"

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ 未找到 Rust，请先安装 Rust 开发环境"
    echo "   访问 https://rustup.rs/ 安装"
    exit 1
fi

# 检查是否安装了 Node.js
if ! command -v npm &> /dev/null; then
    echo "❌ 未找到 Node.js，请先安装 Node.js 开发环境"
    echo "   访问 https://nodejs.org/ 安装"
    exit 1
fi

# 创建日志目录
mkdir -p logs

# 后端启动函数
start_backend() {
    echo "🦀 启动后端服务..."
    cd backend
    
    # 检查数据库是否存在，不存在则创建
    if [ ! -f "../data/customer_tracker.db" ]; then
        echo "📦 初始化数据库..."
        mkdir -p ../data
        cargo run --bin migrate
        
        # 创建默认管理员用户
        echo "👤 创建默认管理员用户..."
        cargo run --bin cli -- create-user admin admin123 "管理员"
    fi
    
    # 启动后端服务
    cargo run 2>&1 | tee ../logs/backend.log &
    BACKEND_PID=$!
    echo "✅ 后端服务启动成功 (PID: $BACKEND_PID)"
    
    cd ..
}

# 前端启动函数
start_frontend() {
    echo "🌐 启动前端服务..."
    cd frontend
    
    # 检查依赖是否安装
    if [ ! -d "node_modules" ]; then
        echo "📦 安装前端依赖..."
        npm install
    fi
    
    # 确保使用本地开发环境配置
    if [ ! -f ".env.local" ]; then
        echo "📝 创建本地开发环境配置..."
        cat > .env.local << EOF
# 本地开发环境配置（本地npm run dev时使用）
VITE_API_BASE_URL=http://localhost:3000
VITE_APP_TITLE=客户追踪系统
VITE_TOKEN_STORAGE_KEY=customer_tracker_token
EOF
    fi
    
    # 启动前端服务
    npm run dev 2>&1 | tee ../logs/frontend.log &
    FRONTEND_PID=$!
    echo "✅ 前端服务启动成功 (PID: $FRONTEND_PID)"
    
    cd ..
}

# 清理函数
cleanup() {
    echo
    echo "🛑 正在停止服务..."
    
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null
        echo "✅ 后端服务已停止"
    fi
    
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null
        echo "✅ 前端服务已停止"
    fi
    
    echo "👋 再见！"
    exit 0
}

# 捕获中断信号
trap cleanup SIGINT SIGTERM

# 启动服务
start_backend
sleep 3  # 等待后端启动完成
start_frontend

echo
echo "🎉 开发环境启动完成！"
echo "📊 后端 API: http://localhost:3000"
echo "🌐 前端界面: http://localhost:5173"
echo "📝 日志文件: logs/backend.log, logs/frontend.log"
echo
echo "💡 默认管理员账户："
echo "   用户名: admin"
echo "   密码: admin123"
echo
echo "按 Ctrl+C 停止所有服务"

# 等待用户中断
wait
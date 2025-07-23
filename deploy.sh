#!/bin/bash

# 客户追踪系统 Docker 部署脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 Docker 和 Docker Compose
check_dependencies() {
    print_info "检查系统依赖..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装。请先安装 Docker。"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose 未安装。请先安装 Docker Compose。"
        exit 1
    fi
    
    print_success "系统依赖检查通过"
}

# 创建必要的目录
create_directories() {
    print_info "创建必要目录..."
    mkdir -p data logs/nginx nginx/conf.d ssl
    print_success "目录创建完成"
}

# 生成 JWT 密钥
generate_jwt_secret() {
    if [ ! -f .env ]; then
        print_info "创建环境变量文件..."
        cp .env.example .env
        
        # 生成随机 JWT 密钥
        JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
        sed -i "s/your-super-secret-jwt-key-here-change-in-production-min-256-bits/$JWT_SECRET/" .env
        
        print_success "环境变量文件已创建，JWT 密钥已生成"
    else
        print_warning "环境变量文件已存在，跳过创建"
    fi
}

# 部署应用
deploy() {
    local env_type=${1:-"dev"}
    
    print_info "开始部署客户追踪系统 ($env_type 环境)..."
    
    if [ "$env_type" = "prod" ]; then
        # 生产环境部署
        print_info "使用生产环境配置部署..."
        docker-compose -f docker-compose.prod.yaml down --remove-orphans
        docker-compose -f docker-compose.prod.yaml build --no-cache
        docker-compose -f docker-compose.prod.yaml up -d
        
        print_info "等待服务启动..."
        sleep 30
        
        # 检查服务状态
        if docker-compose -f docker-compose.prod.yaml ps | grep -q "Up"; then
            print_success "生产环境部署成功！"
            print_info "访问地址: http://localhost"
        else
            print_error "部署失败，请检查日志"
            docker-compose -f docker-compose.prod.yaml logs
            exit 1
        fi
    else
        # 开发环境部署
        print_info "使用开发环境配置部署..."
        docker-compose down --remove-orphans
        docker-compose build --no-cache
        docker-compose up -d
        
        print_info "等待服务启动..."
        sleep 30
        
        # 检查服务状态
        if docker-compose ps | grep -q "Up"; then
            print_success "开发环境部署成功！"
            print_info "前端访问地址: http://localhost"
            print_info "后端API地址: http://localhost:3000"
        else
            print_error "部署失败，请检查日志"
            docker-compose logs
            exit 1
        fi
    fi
}

# 停止服务
stop() {
    local env_type=${1:-"dev"}
    
    print_info "停止客户追踪系统 ($env_type 环境)..."
    
    if [ "$env_type" = "prod" ]; then
        docker-compose -f docker-compose.prod.yaml down
    else
        docker-compose down
    fi
    
    print_success "服务已停止"
}

# 查看日志
logs() {
    local env_type=${1:-"dev"}
    local service=${2:-""}
    
    if [ "$env_type" = "prod" ]; then
        if [ -n "$service" ]; then
            docker-compose -f docker-compose.prod.yaml logs -f "$service"
        else
            docker-compose -f docker-compose.prod.yaml logs -f
        fi
    else
        if [ -n "$service" ]; then
            docker-compose logs -f "$service"
        else
            docker-compose logs -f
        fi
    fi
}

# 清理资源
cleanup() {
    print_warning "这将删除所有容器、镜像和数据，操作不可逆！"
    read -p "确定要继续吗？(y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "清理 Docker 资源..."
        
        # 停止并删除容器
        docker-compose down --remove-orphans
        docker-compose -f docker-compose.prod.yaml down --remove-orphans 2>/dev/null || true
        
        # 删除镜像
        docker rmi $(docker images "customer-tracker*" -q) 2>/dev/null || true
        
        # 删除数据目录
        rm -rf data logs
        
        print_success "清理完成"
    else
        print_info "取消清理操作"
    fi
}

# 显示帮助信息
show_help() {
    echo "客户追踪系统 Docker 部署脚本"
    echo
    echo "用法:"
    echo "  $0 [命令] [参数]"
    echo
    echo "命令:"
    echo "  deploy [dev|prod]     部署应用 (默认: dev)"
    echo "  stop [dev|prod]       停止应用 (默认: dev)"
    echo "  logs [dev|prod] [service]  查看日志 (默认: dev, 所有服务)"
    echo "  cleanup               清理所有 Docker 资源"
    echo "  help                  显示此帮助信息"
    echo
    echo "示例:"
    echo "  $0 deploy             # 部署开发环境"
    echo "  $0 deploy prod        # 部署生产环境"
    echo "  $0 logs dev backend   # 查看开发环境后端日志"
    echo "  $0 stop prod          # 停止生产环境"
    echo
    echo "默认访问地址:"
    echo "  开发环境: http://localhost (前端) + http://localhost:3000 (后端)"
    echo "  生产环境: http://localhost"
    echo
    echo "默认登录账户:"
    echo "  用户名: admin"
    echo "  密码: admin123"
}

# 主函数
main() {
    local command=${1:-"help"}
    
    case $command in
        "deploy")
            check_dependencies
            create_directories
            generate_jwt_secret
            deploy $2
            ;;
        "stop")
            stop $2
            ;;
        "logs")
            logs $2 $3
            ;;
        "cleanup")
            cleanup
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# 执行主函数
main "$@"
#!/bin/bash

# 客户追踪系统 Docker 一键部署脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

print_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# 全局变量
PUBLIC_IP=""
DOMAIN=""
PROTOCOL="http"

# 获取用户输入的公网IP或域名
get_public_address() {
    print_step "配置公网访问地址"
    echo
    
    # 尝试自动获取公网IP
    print_info "正在自动检测公网IP..."
    AUTO_IP=""
    
    # 多种方式获取公网IP
    for service in "ifconfig.me" "ipinfo.io/ip" "icanhazip.com" "ident.me"; do
        if AUTO_IP=$(curl -s --max-time 5 "$service" 2>/dev/null); then
            if [[ $AUTO_IP =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
                print_success "检测到公网IP: $AUTO_IP"
                break
            fi
        fi
    done
    
    echo
    echo "请选择配置方式："
    echo "1) 使用检测到的公网IP: $AUTO_IP"
    echo "2) 手动输入公网IP"
    echo "3) 使用域名"
    echo "4) 本地部署 (localhost)"
    echo
    
    while true; do
        read -p "请选择 [1-4]: " choice
        case $choice in
            1)
                if [ -n "$AUTO_IP" ]; then
                    PUBLIC_IP="$AUTO_IP"
                    DOMAIN="$AUTO_IP"
                    break
                else
                    print_error "未能自动检测到有效的公网IP，请选择其他选项"
                fi
                ;;
            2)
                while true; do
                    read -p "请输入公网IP地址: " ip
                    if [[ $ip =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
                        PUBLIC_IP="$ip"
                        DOMAIN="$ip"
                        break 2
                    else
                        print_error "请输入有效的IP地址格式 (例如: 192.168.1.100)"
                    fi
                done
                ;;
            3)
                while true; do
                    read -p "请输入域名 (例如: example.com): " domain
                    if [[ $domain =~ ^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$ ]]; then
                        DOMAIN="$domain"
                        PROTOCOL="https"
                        print_info "使用域名: $domain (将使用HTTPS)"
                        break 2
                    else
                        print_error "请输入有效的域名格式"
                    fi
                done
                ;;
            4)
                PUBLIC_IP="127.0.0.1"
                DOMAIN="localhost"
                print_info "使用本地部署模式"
                break
                ;;
            *)
                print_error "请输入有效选项 [1-4]"
                ;;
        esac
    done
    
    print_success "配置地址: ${PROTOCOL}://${DOMAIN}"
}

# 检查 Docker 和 Docker Compose
check_dependencies() {
    print_step "检查系统依赖"
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装。请先安装 Docker。"
        print_info "安装命令: curl -fsSL https://get.docker.com | sh"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose 未安装。请先安装 Docker Compose。"
        exit 1
    fi
    
    # 检查Docker服务是否运行
    if ! docker info &> /dev/null; then
        print_error "Docker 服务未运行。请启动 Docker 服务。"
        print_info "启动命令: sudo systemctl start docker"
        exit 1
    fi
    
    print_success "系统依赖检查通过"
}

# 创建必要的目录
create_directories() {
    print_step "创建必要目录"
    
    # 创建数据和日志目录
    mkdir -p data/backend logs/nginx nginx/conf.d ssl
    
    # 设置权限
    chmod 755 data logs nginx ssl
    
    print_success "目录创建完成"
}

# 自动生成所有环境变量配置
generate_env_config() {
    print_step "生成环境变量配置"
    
    # 备份已存在的.env文件
    if [ -f .env ]; then
        print_warning "发现已存在的.env文件，备份为.env.backup"
        cp .env .env.backup
    fi
    
    # 生成随机 JWT 密钥
    print_info "生成安全的JWT密钥..."
    JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
    
    # 确定API基础URL - 前端使用相对路径，后端CORS使用完整地址
    VITE_API_BASE_URL="/api"  # 前端总是使用相对路径
    if [ "$DOMAIN" = "localhost" ]; then
        CORS_ORIGIN="http://localhost"
    else
        CORS_ORIGIN="${PROTOCOL}://${DOMAIN}"
    fi
    
    # 创建生产环境配置文件
    cat > .env << EOF
# 自动生成的环境配置文件
# 生成时间: $(date)
# 部署地址: ${PROTOCOL}://${DOMAIN}

# JWT 配置
JWT_SECRET=${JWT_SECRET}
JWT_EXPIRE_HOURS=24

# 后端配置
RUST_LOG=warn
CORS_ORIGIN=${CORS_ORIGIN}

# 前端配置
VITE_API_BASE_URL=${VITE_API_BASE_URL}
VITE_TOKEN_STORAGE_KEY=customer_tracker_token

# 部署配置
PUBLIC_IP=${PUBLIC_IP}
DOMAIN=${DOMAIN}
PROTOCOL=${PROTOCOL}
EOF
    
    print_success "环境变量配置已生成"
    print_info "API地址: ${VITE_API_BASE_URL}"
    print_info "CORS源: ${CORS_ORIGIN}"
}

# 初始化数据库和默认用户
init_database() {
    print_step "初始化数据库和默认用户"
    
    print_info "等待后端服务完全启动..."
    
    # 等待后端服务健康检查通过
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if docker-compose -f docker-compose.prod.yaml exec -T backend /app/customer-tracker database status > /dev/null 2>&1; then
            print_success "后端服务已就绪"
            break
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            print_error "后端服务启动超时"
            return 1
        fi
        
        print_info "等待后端服务启动... ($attempt/$max_attempts)"
        sleep 5
        ((attempt++))
    done
    
    # 运行数据库迁移
    print_info "运行数据库迁移..."
    docker-compose -f docker-compose.prod.yaml exec -T backend /app/customer-tracker database migrate
    
    # 创建默认管理员用户
    print_info "创建默认管理员用户..."
    docker-compose -f docker-compose.prod.yaml exec -T backend /app/customer-tracker user create -u admin -p admin123 -n "管理员" || {
        print_warning "用户可能已存在，跳过创建"
    }
    
    print_success "数据库初始化完成"
}

# 健康检查
health_check() {
    print_step "执行健康检查"
    
    local max_attempts=60
    local attempt=1
    local all_healthy=false
    
    while [ $attempt -le $max_attempts ] && [ "$all_healthy" = false ]; do
        local backend_healthy=false
        local frontend_healthy=false
        local nginx_healthy=false
        
        # 检查后端健康状态
        if curl -f -s "${PROTOCOL}://${DOMAIN}/api/health" > /dev/null 2>&1; then
            backend_healthy=true
        fi
        
        # 检查前端健康状态
        if curl -f -s "${PROTOCOL}://${DOMAIN}/health" > /dev/null 2>&1; then
            frontend_healthy=true
        fi
        
        # 检查Nginx健康状态
        if curl -f -s "${PROTOCOL}://${DOMAIN}" > /dev/null 2>&1; then
            nginx_healthy=true
        fi
        
        if [ "$backend_healthy" = true ] && [ "$frontend_healthy" = true ] && [ "$nginx_healthy" = true ]; then
            all_healthy=true
            break
        fi
        
        if [ $((attempt % 10)) -eq 0 ]; then
            print_info "健康检查进行中... ($attempt/$max_attempts)"
            print_info "后端: $([ "$backend_healthy" = true ] && echo "✓" || echo "✗") 前端: $([ "$frontend_healthy" = true ] && echo "✓" || echo "✗") Nginx: $([ "$nginx_healthy" = true ] && echo "✓" || echo "✗")"
        fi
        
        sleep 5
        ((attempt++))
    done
    
    if [ "$all_healthy" = true ]; then
        print_success "所有服务健康检查通过"
        return 0
    else
        print_error "服务健康检查失败"
        return 1
    fi
}

# 一键部署主函数
deploy() {
    print_step "开始一键部署客户追踪系统"
    
    # 停止已存在的容器
    print_info "停止现有服务..."
    docker-compose -f docker-compose.prod.yaml down --remove-orphans 2>/dev/null || true
    
    # 清理旧镜像（可选）
    print_info "清理旧镜像..."
    docker image prune -f || true
    
    # 构建和启动服务
    print_info "构建并启动服务..."
    docker-compose -f docker-compose.prod.yaml build --no-cache
    docker-compose -f docker-compose.prod.yaml up -d
    
    # 等待服务启动
    print_info "等待服务启动..."
    sleep 15
    
    # 初始化数据库
    init_database
    
    # 健康检查
    if health_check; then
        print_success "🎉 部署成功！"
        echo
        echo "=================================================================="
        echo "🌟 客户追踪系统部署完成"
        echo "=================================================================="
        echo "📱 访问地址: ${PROTOCOL}://${DOMAIN}"
        echo "🔑 默认账户: admin / admin123"
        echo "📊 管理命令: ./deploy.sh logs    # 查看日志"
        echo "📊 管理命令: ./deploy.sh stop    # 停止服务"
        echo "📊 管理命令: ./deploy.sh status  # 查看状态"
        echo "=================================================================="
        
        # 显示服务状态
        docker-compose -f docker-compose.prod.yaml ps
    else
        print_error "部署失败，请检查日志"
        print_info "查看日志命令: ./deploy.sh logs"
        exit 1
    fi
}

# 停止服务
stop() {
    print_step "停止客户追踪系统"
    
    docker-compose -f docker-compose.prod.yaml down --remove-orphans
    
    print_success "服务已停止"
}

# 查看服务状态
status() {
    print_step "查看服务状态"
    
    echo "Docker Compose 服务状态:"
    docker-compose -f docker-compose.prod.yaml ps
    
    echo
    echo "容器资源使用情况:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" $(docker-compose -f docker-compose.prod.yaml ps -q) 2>/dev/null || echo "无运行中的容器"
    
    echo
    echo "磁盘使用情况:"
    du -sh data/ logs/ 2>/dev/null || echo "数据目录不存在"
}

# 查看日志
logs() {
    local service=${1:-""}
    
    print_step "查看服务日志"
    
    if [ -n "$service" ]; then
        case $service in
            "backend"|"frontend"|"nginx")
                print_info "查看 $service 服务日志 (按 Ctrl+C 退出)"
                docker-compose -f docker-compose.prod.yaml logs -f "$service"
                ;;
            *)
                print_error "无效的服务名。可用服务: backend, frontend, nginx"
                exit 1
                ;;
        esac
    else
        print_info "查看所有服务日志 (按 Ctrl+C 退出)"
        docker-compose -f docker-compose.prod.yaml logs -f
    fi
}

# 重启服务
restart() {
    local service=${1:-""}
    
    if [ -n "$service" ]; then
        print_step "重启 $service 服务"
        docker-compose -f docker-compose.prod.yaml restart "$service"
        print_success "$service 服务已重启"
    else
        print_step "重启所有服务"
        docker-compose -f docker-compose.prod.yaml restart
        print_success "所有服务已重启"
    fi
}

# 更新服务
update() {
    print_step "更新客户追踪系统"
    
    print_warning "这将重新构建并部署最新版本的系统"
    read -p "确定要继续吗？(y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 保存当前配置
        if [ -f .env ]; then
            cp .env .env.update.backup
            print_info "已备份当前配置"
        fi
        
        # 重新部署
        deploy
    else
        print_info "取消更新操作"
    fi
}

# 清理资源
cleanup() {
    print_warning "⚠️  危险操作：这将删除所有容器、镜像、数据和配置文件！"
    print_warning "⚠️  此操作不可逆，所有客户数据将永久丢失！"
    echo
    read -p "请输入 'DELETE ALL DATA' 确认删除: " confirm
    
    if [ "$confirm" = "DELETE ALL DATA" ]; then
        print_info "开始清理所有资源..."
        
        # 停止并删除容器
        docker-compose -f docker-compose.prod.yaml down --remove-orphans --volumes
        
        # 删除镜像
        print_info "删除相关镜像..."
        docker images --format "{{.Repository}}:{{.Tag}}" | grep -E "(customer-tracker|customer_tracker)" | xargs -r docker rmi
        
        # 删除数据和日志目录
        print_info "删除数据和日志文件..."
        rm -rf data/ logs/ .env .env.backup .env.update.backup
        
        # 删除网络
        docker network prune -f
        
        # 删除未使用的卷
        docker volume prune -f
        
        print_success "所有资源已清理完成"
    else
        print_info "取消清理操作"
    fi
}

# 显示帮助信息
show_help() {
    cat << 'EOF'
🚀 客户追踪系统 - Docker 一键部署脚本

📖 用法:
  ./deploy.sh [命令] [参数]

🔧 可用命令:
  deploy       一键部署系统 (自动配置所有环境变量)
  stop         停止所有服务
  status       查看服务状态和资源使用情况
  logs [service]  查看日志 (可选服务: backend, frontend, nginx)
  restart [service]  重启服务 (可选指定单个服务)
  update       更新系统到最新版本
  cleanup      完全清理所有数据和配置 (危险操作)
  help         显示此帮助信息

✨ 部署流程:
  1. 自动检测或手动配置公网IP/域名
  2. 自动生成安全的JWT密钥和环境变量
  3. 自动构建并启动所有服务 (Nginx + 前端 + 后端)
  4. 自动初始化数据库和创建管理员账户
  5. 执行健康检查确保服务正常运行

💡 使用示例:
  ./deploy.sh deploy           # 一键部署 (推荐)
  ./deploy.sh logs backend     # 查看后端日志
  ./deploy.sh status           # 查看服务状态
  ./deploy.sh restart nginx    # 重启Nginx服务
  ./deploy.sh stop             # 停用服务

🔐 默认登录账户:
  用户名: admin
  密码: admin123

📝 部署要求:
  - Docker 和 Docker Compose
  - 公网IP或域名 (可自动检测)
  - 开放80端口 (HTTP) 或443端口 (HTTPS)

⚠️  注意事项:
  - 首次部署会询问公网IP配置
  - 生产环境建议使用域名和HTTPS
  - cleanup 命令会删除所有数据，请谨慎使用
EOF
}

# 快速部署函数 (无参数时的默认行为)
quick_deploy() {
    echo "🚀 欢迎使用客户追踪系统一键部署"
    echo
    echo "此脚本将自动完成以下操作:"
    echo "  ✓ 检查系统依赖"
    echo "  ✓ 配置公网访问地址"  
    echo "  ✓ 生成安全配置"
    echo "  ✓ 构建并启动服务"
    echo "  ✓ 初始化数据库"
    echo "  ✓ 创建管理员账户"
    echo
    read -p "是否开始部署？(Y/n): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        print_info "部署已取消"
        exit 0
    fi
    
    # 执行完整部署流程
    check_dependencies
    get_public_address
    create_directories  
    generate_env_config
    deploy
}

# 主函数
main() {
    local command=${1:-""}
    
    # 如果没有参数，执行快速部署
    if [ $# -eq 0 ]; then
        quick_deploy
        return
    fi
    
    case $command in
        "deploy")
            check_dependencies
            get_public_address
            create_directories
            generate_env_config
            deploy
            ;;
        "stop")
            stop
            ;;
        "status")
            status
            ;;
        "logs")
            logs $2
            ;;
        "restart")
            restart $2
            ;;
        "update")
            update
            ;;
        "cleanup")
            cleanup
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "未知命令: $command"
            echo
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"
#!/bin/bash
#
# 服务器快速部署脚本
# 用于在新服务器上快速搭建 webhook 部署环境
#

set -e

echo "============================================"
echo "GitHub Webhook 部署环境 - 快速安装脚本"
echo "============================================"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

# 检查是否以 root 运行
if [ "$EUID" -ne 0 ]; then
    error "请以 root 用户运行此脚本 (使用 sudo)"
    exit 1
fi

# 检查系统
if [ ! -f /etc/os-release ]; then
    error "无法识别操作系统"
    exit 1
fi

source /etc/os-release
if [[ "$ID" != "ubuntu" && "$ID" != "debian" ]]; then
    warn "此脚本仅针对 Ubuntu/Debian 测试，其他系统可能需要调整"
fi

echo ""
echo "=== 步骤 1: 安装系统依赖 ==="

apt-get update
apt-get install -y \
    nginx \
    rsync \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev

echo ""
echo "=== 步骤 2: 安装 Rust ==="

if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # 使 cargo 立即可用
    source "$HOME/.cargo/env"
    info "Rust 安装完成"
else
    info "Rust 已安装"
fi

echo ""
echo "=== 步骤 3: 创建目录结构 ==="

mkdir -p /var/www/docs-source
mkdir -p /var/www/docs
mkdir -p /opt/webhook-server
mkdir -p /var/log

# 设置权限
chown -R www-data:www-data /var/www
chmod 755 /var/www

info "目录结构创建完成"

echo ""
echo "=== 步骤 4: 编译 Webhook Server ==="

# 检查当前目录是否有源代码
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -f "$SCRIPT_DIR/webhook-server/Cargo.toml" ]; then
    cd "$SCRIPT_DIR/webhook-server"
    info "编译 webhook-server..."

    # 复制依赖文件到目标位置
    cp -r . /opt/webhook-server/
    cd /opt/webhook-server

    # 编译 release 版本
    source "$HOME/.cargo/env"
    cargo build --release

    info "编译完成：/opt/webhook-server/target/release/webhook-server"
else
    error "未找到 webhook-server 源代码，请确保从项目根目录运行此脚本"
    exit 1
fi

echo ""
echo "=== 步骤 5: 配置 systemd 服务 ==="

# 复制服务文件
if [ -f "$SCRIPT_DIR/webhook-server.service" ]; then
    cp "$SCRIPT_DIR/webhook-server.service" /etc/systemd/system/

    # 修改服务文件中的路径
    sed -i "s|WorkingDirectory=.*|WorkingDirectory=/opt/webhook-server|" /etc/systemd/system/webhook-server.service
    sed -i "s|ExecStart=.*|ExecStart=/opt/webhook-server/target/release/webhook-server|" /etc/systemd/system/webhook-server.service

    # 重新加载并启动服务
    systemctl daemon-reload
    systemctl enable webhook-server

    info "systemd 服务已配置"
else
    warn "未找到 webhook-server.service 文件，跳过 systemd 配置"
fi

echo ""
echo "=== 步骤 6: 配置 Nginx ==="

cat > /etc/nginx/sites-available/docs << 'EOF'
server {
    listen 80;
    server_name _;

    root /var/www/docs;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静态文件缓存
    location ~* \.(jpg|jpeg|png|gif|ico|css|js|svg|woff|woff2)$ {
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # 禁止访问隐藏文件
    location ~ /\. {
        deny all;
    }
}
EOF

# 启用站点
if [ ! -f /etc/nginx/sites-enabled/docs ]; then
    ln -s /etc/nginx/sites-available/docs /etc/nginx/sites-enabled/
fi

# 移除 default 站点（如果存在）
if [ -f /etc/nginx/sites-enabled/default ]; then
    rm /etc/nginx/sites-enabled/default
fi

# 测试并重载 nginx
if nginx -t; then
    systemctl restart nginx
    info "Nginx 配置完成"
else
    error "Nginx 配置测试失败"
fi

echo ""
echo "=== 步骤 7: 创建环境配置文件 ==="

# 生成随机 secret
WEBHOOK_SECRET=$(openssl rand -hex 16)

cat > /opt/webhook-server/.env << EOF
# GitHub Webhook 配置
WEBHOOK_SECRET=${WEBHOOK_SECRET}
DEPLOY_SCRIPT=/opt/webhook-server/deploy.sh
PORT=5000

# 路径配置
REPO_DIR=/var/www/docs-source
DEPLOY_DIR=/var/www/docs
LOG_FILE=/var/log/webhook-deploy.log
EOF

chmod 600 /opt/webhook-server/.env
info "环境配置文件已创建：/opt/webhook-server/.env"
info "Webhook Secret: ${WEBHOOK_SECRET}"

echo ""
echo "============================================"
echo "🎉 部署完成!"
echo "============================================"
echo ""
echo "下一步操作:"
echo ""
echo "1. 复制以下 Secret 到 GitHub Webhook 配置:"
echo "   ${WEBHOOK_SECRET}"
echo ""
echo "2. 在 GitHub 中配置 Webhook:"
echo "   - URL: http://<你的服务器 IP>:5000/webhook"
echo "   - Secret: 上面的 Secret"
echo "   - Events: Just the push event"
echo ""
echo "3. 克隆仓库到源码目录:"
echo "   cd /var/www/docs-source"
echo "   git clone <你的仓库 URL> ."
echo ""
echo "4. 复制 deploy.sh 到 webhook-server 目录"
echo ""
echo "5. 启动服务:"
echo "   systemctl start webhook-server"
echo ""
echo "6. 执行首次部署:"
echo "   bash /opt/webhook-server/deploy.sh"
echo ""
echo "============================================"
echo ""
echo "服务状态:"
echo "  - Webhook Server: systemctl status webhook-server"
echo "  - Nginx: systemctl status nginx"
echo ""
echo "日志查看:"
echo "  - Webhook: journalctl -u webhook-server -f"
echo "  - 部署：tail -f /var/log/webhook-deploy.log"
echo ""

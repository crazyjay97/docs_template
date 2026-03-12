# GitHub Webhook 部署方案

## 概述

本方案使用 Rust 编写的 webhook 服务器接收 GitHub push 事件，自动触发文档构建和部署。

## 安全验证机制

| 层级 | 验证方式 | 说明 |
|------|----------|------|
| 1 | **HMAC 签名** | 使用 WEBHOOK_SECRET 验证请求签名 |
| 2 | **IP 段验证** | 验证请求来自 GitHub 官方 IP 段 |
| 3 | **组织白名单** | 只响应指定组织的仓库 |
| 4 | **仓库白名单** | 只响应指定的具体仓库 |

## 架构

```
GitHub (push) → Rust Webhook Server → deploy.sh → nginx
                    ↓
            1. 验证 IP
            2. 验证签名
            3. 验证组织/仓库
```

## 服务器端安装步骤

### 1. 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装系统依赖
sudo apt-get update
sudo apt-get install -y nginx rsync git

# 安装 Python 和依赖（用于构建文档）
sudo apt-get install -y python3 python3-pip python3-venv
```

### 2. 创建目录结构

```bash
sudo mkdir -p /var/www/docs-source    # 源代码目录
sudo mkdir -p /var/www/docs           # nginx 部署目录
sudo mkdir -p /opt/webhook-server     # webhook 服务目录
```

### 3. 部署 Webhook Server

```bash
cd /opt/webhook-server
git clone <your-repo-url> .
cargo build --release
```

### 4. 配置环境变量

```bash
sudo tee /opt/webhook-server/.env > /dev/null << 'EOF'
# 安全配置
WEBHOOK_SECRET=your_secure_secret_token_here

# 白名单配置（重要！）
ALLOWED_ORGS=your-company
ALLOWED_REPOS=your-company/your-docs-repo

# 网络配置
PORT=5000
SKIP_IP_CHECK=false

# 部署配置
DEPLOY_SCRIPT=/opt/webhook-server/deploy.sh
REPO_DIR=/var/www/docs-source
DEPLOY_DIR=/var/www/docs
LOG_FILE=/var/log/webhook-deploy.log
EOF
```

### 5. 配置 systemd 服务

```bash
sudo cp webhook-server.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable webhook-server
sudo systemctl start webhook-server
sudo systemctl status webhook-server
```

### 6. 配置 Nginx

```bash
sudo tee /etc/nginx/sites-available/docs > /dev/null << 'EOF'
server {
    listen 80;
    server_name your-domain.com;

    root /var/www/docs;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/docs /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 7. 初始化仓库

```bash
cd /var/www/docs-source
sudo git clone <your-repo-url> .
sudo chown -R www-data:www-data .
sudo -u www-data bash /opt/webhook-server/deploy.sh
```

## GitHub 端配置

### 创建 Webhook

1. 进入 GitHub 仓库 → Settings → Webhooks → Add webhook

2. 配置：
   - **Payload URL**: `http://your-server-ip:5000/webhook`
   - **Content type**: `application/json`
   - **Secret**: `your_secure_secret_token_here`
   - **Events**: `Just the push event`

3. 点击 Add webhook

## 环境变量说明

| 变量 | 说明 | 示例 |
|------|------|------|
| `WEBHOOK_SECRET` | Webhook 签名验证密钥 | `my_secret_123` |
| `ALLOWED_ORGS` | 允许的组织（逗号分隔） | `my-company,my-org` |
| `ALLOWED_REPOS` | 允许的仓库（逗号分隔） | `my-company/docs` |
| `PORT` | 服务端口 | `5000` |
| `SKIP_IP_CHECK` | 是否跳过 IP 验证 | `false` |
| `DEPLOY_SCRIPT` | 部署脚本路径 | `/opt/deploy.sh` |

## 测试

### 检查服务状态

```bash
# 健康检查
curl http://localhost:5000/health

# 查看部署日志
curl http://localhost:5000/logs

# 查看系统日志
sudo journalctl -u webhook-server -f
```

### 测试 Webhook

在 GitHub webhook 页面点击 "Redeliver" 测试推送。

## 故障排查

```bash
# Webhook 服务日志
sudo journalctl -u webhook-server -f

# 部署日志
sudo tail -f /var/log/webhook-deploy.log

# 检查端口
sudo lsof -i :5000
```

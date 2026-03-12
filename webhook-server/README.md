# Webhook Server

GitHub Webhook 服务器，用于文档自动部署。

## 功能特性

- **安全验证**
  - HMAC SHA256 签名验证 (WEBHOOK_SECRET)
  - GitHub 官方 IP 段验证
  - 组织/仓库白名单验证
- **自动部署**：监听 `push` 事件，自动触发部署脚本
- **日志记录**：记录部署历史，可通过 API 查询
- **健康检查**：提供 `/health` 健康检查端点

## 快速开始

### 1. 编译

```bash
cd webhook-server
cargo build --release
```

编译产物位置：`target/release/webhook-server`

### 2. 配置环境变量

创建 `.env` 文件或通过 systemd 配置：

```bash
# 必需配置
WEBHOOK_SECRET=your_webhook_secret_here    # GitHub Webhook 密钥
DEPLOY_SCRIPT=/opt/webhook-server/deploy.sh  # 部署脚本路径

# 可选配置
PORT=5000                              # 服务监听端口 (默认：5000)
REPO_DIR=/var/www/docs-source          # 源码目录
DEPLOY_DIR=/var/www/docs               # 部署目标目录
LOG_FILE=/var/log/webhook-deploy.log   # 日志文件路径
ALLOWED_ORGS=your-org                  # 允许的组织白名单 (逗号分隔)
ALLOWED_USERS=your-username            # 允许的个人用户白名单 (逗号分隔)
SKIP_IP_CHECK=false                    # 是否跳过 IP 检查 (默认：false)
```

### 3. 运行

#### 方式一：直接运行

```bash
./target/release/webhook-server
```

#### 方式二：使用 systemd 服务

1. 复制服务文件到 systemd 目录：

```bash
sudo cp webhook-server.service /etc/systemd/system/
```

2. 编辑 `/etc/systemd/system/webhook-server.service`，修改 `Environment` 变量：

```ini
Environment="WEBHOOK_SECRET=your_actual_secret"
Environment="DEPLOY_SCRIPT=/path/to/your/deploy.sh"
```

3. 启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable webhook-server
sudo systemctl start webhook-server
sudo systemctl status webhook-server
```

## API 端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/webhook` | POST | GitHub Webhook 接收端点 |
| `/health` | GET | 健康检查 |
| `/logs` | GET | 获取部署日志 |

## GitHub Webhook 配置

1. 进入仓库的 **Settings** > **Webhooks** > **Add webhook**

2. 配置如下：
   - **Payload URL**: `http://your-server-ip:5000/webhook`
   - **Content type**: `application/json`
   - **Secret**: 与 `WEBHOOK_SECRET` 相同的值
   - **Events**: 选择 **Push events**

3. 添加后，GitHub 会发送 ping 事件测试连接

## 部署脚本示例

创建 `deploy.sh`：

```bash
#!/bin/bash
set -e

echo "Starting deployment..."

# 进入源码目录
cd /var/www/docs-source

# 拉取最新代码
git pull origin main

# 构建文档
cd docs
make build

# 复制到部署目录
sudo cp -r _build/html/* /var/www/docs/

echo "Deployment completed!"
```

赋予执行权限：

```bash
chmod +x deploy.sh
```

## 安全建议

1. **始终设置 WEBHOOK_SECRET**：防止未授权请求
2. **配置白名单**：限制允许触发的组织/个人用户
3. **保持 IP 检查启用**：确保请求来自 GitHub
4. **使用 HTTPS**：生产环境建议使用反向代理 (如 Nginx) 启用 HTTPS

## 日志查看

```bash
# 查看 systemd 日志
sudo journalctl -u webhook-server -f

# 查看部署日志 API
curl http://localhost:5000/logs
```

## 故障排查

### 1. 签名验证失败

确保 GitHub Webhook 配置中的 Secret 与 `WEBHOOK_SECRET` 完全一致。

### 2. IP 检查失败

如果服务器在代理后面，可能需要设置 `SKIP_IP_CHECK=true`。

### 3. 部署脚本失败

检查部署脚本的执行权限和路径配置。

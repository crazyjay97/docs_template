#!/bin/bash
#
# GitHub Webhook 部署脚本
# 用于拉取代码、构建文档并部署到 nginx 目录
#

set -e  # 遇到错误立即退出

# ============== 配置 ==============
# 可通过环境变量覆盖
REPO_DIR="${REPO_DIR:-/var/www/docs-source}"
DEPLOY_DIR="${DEPLOY_DIR:-/var/www/docs}"
WEBHOOK_SERVER_DIR="${WEBHOOK_SERVER_DIR:-/opt/webhook-server}"
LOG_FILE="${LOG_FILE:-/var/log/webhook-deploy.log}"
BRANCH="${BRANCH:-main}"

# ============== 日志函数 ==============
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

error_log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "$LOG_FILE" >&2
}

# ============== 部署步骤 ==============
deploy() {
    log "=========================================="
    log "开始部署文档..."
    log "=========================================="

    # 1. 进入仓库目录
    log "步骤 1: 进入仓库目录..."
    if [ -d "$REPO_DIR" ]; then
        cd "$REPO_DIR"
        log "✓ 仓库目录存在：$REPO_DIR"
    else
        error_log "仓库目录不存在：$REPO_DIR"
        exit 1
    fi

    # 2. 拉取最新代码
    log "步骤 2: 拉取最新代码..."
    git fetch origin "$BRANCH"
    git reset --hard "origin/$BRANCH"
    log "✓ 代码已更新到 $BRANCH 分支"

    # 3. 构建文档
    log "步骤 3: 构建文档..."
    cd docs
    if [ -f "build_dist.sh" ]; then
        bash build_dist.sh 2>&1 | tee -a "$LOG_FILE"
        log "✓ 文档构建完成"
    else
        error_log "未找到构建脚本：build_dist.sh"
        exit 1
    fi

    # 4. 同步到部署目录
    log "步骤 4: 同步文件到部署目录..."
    if [ -d "nginx_deploy" ]; then
        # 使用 rsync 同步，避免全量复制
        rsync -av --delete nginx_deploy/ "$DEPLOY_DIR/"
        log "✓ 文件已同步到：$DEPLOY_DIR"
    else
        error_log "未找到部署目录：nginx_deploy"
        exit 1
    fi

    # 5. 设置权限 (nginx 需要)
    log "步骤 5: 设置文件权限..."
    chown -R www-data:www-data "$DEPLOY_DIR" 2>/dev/null || true
    chmod -R 755 "$DEPLOY_DIR"
    log "✓ 权限设置完成"

    log "=========================================="
    log "🎉 部署完成!"
    log "=========================================="
}

# ============== 主程序 ==============
main() {
    # 确保日志目录存在
    mkdir -p "$(dirname "$LOG_FILE")"

    # 执行部署（使用 try-catch 模式）
    (deploy) || {
        error_log "部署失败!"
        exit 1
    }
}

main "$@"

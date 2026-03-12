# 服务器编译问题修复指南

## 问题原因

`build-docs` 命令不存在，因为该命令来自 `esp-docs` 包，但 `requirements.txt` 中没有包含它。

## 解决方案

已修改构建系统，使用标准的 Sphinx 命令替代 `build-docs`。

### 在服务器上执行

```bash
# 1. 拉取最新代码
cd /var/www/docs-source
git pull origin main

# 2. 安装 Python 依赖
cd docs
source ../venv/bin/activate
pip install -r requirements.txt

# 3. 构建文档
make build

# 或者使用 dist 目标（构建并准备部署）
make dist
```

### 或者手动执行完整部署

```bash
# 执行部署脚本
bash /opt/webhook-server/deploy.sh
```

## 修改的文件

| 文件 | 修改内容 |
|------|----------|
| `docs/Makefile` | 改用 `sphinx-build` 命令 |
| `docs/en/Makefile` | 新增（Sphinx 标准 Makefile） |
| `docs/zh_CN/Makefile` | 新增（Sphinx 标准 Makefile） |
| `docs/build_dist.sh` | 改用 `make build` |
| `deploy.sh` | 改用 `make build` |

## 本地测试

在本地可以先测试构建：

```bash
cd docs
make clean
make build

# 查看构建结果
ls -la _build/en/generic/html/
ls -la _build/zh_CN/generic/html/
```

## 常见问题

### 1. `sphinx-build: command not found`

确保已激活虚拟环境并安装依赖：

```bash
source ../venv/bin/activate
pip install -r requirements.txt
```

### 2. 构建失败，提示缺少主题

安装主题：

```bash
pip install sphinx-rtd-theme
```

### 3. 中文构建失败

确保服务器安装了中文字体：

```bash
# Ubuntu/Debian
apt-get install fonts-wqy-zenhei fonts-wqy-microhei

# CentOS/RHEL
yum install wqy-zenhei-fonts wqy-microhei-fonts
```

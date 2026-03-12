# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

基于 ESP-IDF 文档结构的 Sphinx 文档模板工程，支持中英文国际化 (i18n)。

## Project Structure

- **`docs/`** - 文档工程目录，包含所有 Sphinx 配置和源文件
  - `en/` - 英文文档源文件
  - `zh_CN/` - 中文文档源文件
  - `_static/` - 静态资源 (CSS, JS, 图片)
  - `conf_common.py` - 通用 Sphinx 配置
  - `Makefile` - 主构建脚本
  - `build_dist.sh` - 部署构建脚本

## Build Commands

所有命令在 `docs/` 目录下执行，使用 `docs/venv/` 虚拟环境。

### 安装依赖
```bash
cd docs
source venv/bin/activate
pip install -r requirements.txt
```

### 构建文档
```bash
# 构建所有语言
make build

# 只构建英文
make build-en

# 只构建中文
make build-zh

# 清理构建产物
make clean
```

### 构建用于部署
```bash
# 创建 dist/ 目录 (包含 nginx 部署所需文件)
make dist
# 或直接运行脚本
./build_dist.sh
```

### 预览文档
打开浏览器访问：
- `_build/en/html/index.html` - 英文文档
- `_build/zh_CN/html/index.html` - 中文文档

## Adding New Pages

1. 在对应语言目录创建 `.rst` 文件
2. 在 `index.rst` 或相关章节的 `toctree` 中添加路径
3. 在文件顶部添加翻译链接：
   ```rst
   :link_to_translation:`zh_CN:[中文]`  # 英文页面
   :link_to_translation:`en:[English]`  # 中文页面
   ```
4. 在另一语言目录创建对应的翻译文件

## Key Configuration Files

- **`conf_common.py`** - 通用 Sphinx 配置（extensions, theme, static paths）
- **`en/conf.py`** / **`zh_CN/conf.py`** - 语言特定配置（导入 conf_common.py）
- **`requirements.txt`** - Python 依赖 (sphinx, sphinx-rtd-theme, jieba)

## Chinese Search Fix

中文搜索问题已修复（2026-03-12）。问题原因：Sphinx 前端 JavaScript 缺少中文分词器，导致搜索查询无法匹配 jieba 生成的索引。

修复方案：
- 新增 `_static/js/chinese_splitter.js` - 中文查询分词器，生成 n-grams 匹配索引
- 修改 `zh_CN/conf.py` - 在 `setup()` 中注册分词器脚本

测试搜索功能：构建后访问 `_build/zh_CN/html/search.html`，尝试搜索"文档"、"安装"、"搜索"等关键词。

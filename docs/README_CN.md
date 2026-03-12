# 文档模板

基于 ESP-IDF 文档结构的可复用 Sphinx 文档模板，支持中英文国际化 (i18n)。

## 功能特性

- **Sphinx**: 使用标准 Sphinx 构建系统
- **国际化 (i18n)**: 内置中英文支持，可轻松切换语言
- **开箱即用**: 修改内容即可发布文档
- **功能完整**: 包含代码高亮、表格、注释框等核心功能
- **Nginx 部署**: 内置构建脚本，可创建部署用的 `dist/` 目录
- **GitHub Webhook**: 支持自动部署（需配置）

## 快速开始

### 1. 安装依赖

```bash
pip install -r requirements.txt
```

### 2. 构建文档

构建英文文档：
```bash
cd docs/en
make html
```

构建中文文档：
```bash
cd docs/zh_CN
make html
```

### 3. 使用 Makefile 构建（推荐）

在项目根目录（docs/）执行：

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

### 4. 构建用于部署

创建 `dist/` 目录用于 nginx 部署：

```bash
make dist
```

或直接运行脚本：

```bash
./build_dist.sh
```

生成的 `dist/` 目录包含：
- `dist/en/` - 英文文档
- `dist/zh_CN/` - 中文文档
- `dist/index.html` - 语言选择首页

### 5. 查看文档

打开浏览器访问构建产物：
- `_build/en/generic/html/index.html` - 英文
- `_build/zh_CN/generic/html/index.html` - 中文

或使用部署版本：
- `dist/en/index.html`
- `dist/zh_CN/index.html`

## 目录结构

```
docs/
├── Makefile                    # 主 Makefile
├── build_dist.sh               # 部署构建脚本
├── conf_common.py              # 通用 Sphinx 配置
├── requirements.txt            # Python 依赖
├── README.md                   # 英文说明
├── README_CN.md                # 中文说明（本文件）
├── page_redirects.txt          # 页面重定向配置
├── _static/                    # 静态资源
│   ├── css/
│   │   └── theme_overrides.css
│   ├── js/
│   │   ├── chatbot_widget_en.js
│   │   ├── chatbot_widget_cn.js
│   │   ├── enhanced_search.js  # 增强搜索 (Ctrl+K)
│   │   └── version_table.js
│   └── *.png, *.svg            # 图片
├── en/                         # 英文文档
│   ├── Makefile
│   ├── conf.py
│   ├── index.rst
│   └── ...
└── zh_CN/                      # 中文文档
    ├── Makefile
    ├── conf.py
    ├── index.rst
    └── ...
```

## 配置说明

### conf_common.py

通用 Sphinx 配置：

- **extensions**: Sphinx 扩展（copybutton 等）
- **github_repo**: GitHub 仓库地址
- **project_slug**: 项目标识
- **languages**: 支持的语言列表
- **html_static_path**: 静态文件路径

### 语言特定的 conf.py

每个语言目录的 `conf.py`：

- 导入 `conf_common.py`
- 设置 `project` 名称
- 设置 `copyright` 信息
- 设置 `language` 代码

## 添加新页面

1. 在对应语言目录创建 `.rst` 文件
2. 在 `index.rst` 或相关章节的 `toctree` 中添加文件路径
3. 在文件顶部添加翻译链接：
   ```rst
   :link_to_translation:`zh_CN:[中文]`  # 英文页面
   :link_to_translation:`en:[English]`  # 中文页面
   ```
4. 在另一语言目录创建对应的翻译文件

## Sphinx 语法示例

参考 `en/api-guides/features-example.rst` 查看：

- 代码块（带语法高亮）
- 表格（list-table 和 grid tables）
- 注释框（notes, warnings, tips）
- 交叉引用
- 图片和图表
- 条件内容
- 内外链
- 数学公式
- 列表

## 国际化

### 添加翻译链接

在每个 RST 文件顶部添加：

```rst
:link_to_translation:`zh_CN:[中文]`  # 英文页面
:link_to_translation:`en:[English]`  # 中文页面
```

### 创建翻译内容

1. 复制英文 `.rst` 文件到 `zh_CN/` 对应位置
2. 翻译内容
3. 更新 `:link_to_translation:` 指向英文版本

## 自定义

### 主题覆盖

编辑 `_static/css/theme_overrides.css` 自定义外观：

```css
/* 修改主题色 */
.wy-side-nav-search {
    background-color: #your-color;
}
```

### Chatbot Widget

编辑 `_static/js/chatbot_widget_en.js` 和 `_static/js/chatbot_widget_cn.js`：

- 替换 `your-website-id-here` 为实际的 Kapa.ai ID
- 更新品牌颜色和 Logo
- 自定义提示语

### 搜索功能

已集成增强搜索：
- 按 `Ctrl+K` (Mac: `Cmd+K`) 快速聚焦搜索框
- 显示搜索结果数量
- Sphinx 内置全文搜索

### 页面重定向

编辑 `page_redirects.txt` 设置 URL 重定向：

```
old/page/path    new/page/path
```

## 部署

### 本地构建后上传

```bash
# 1. 构建
make dist

# 2. 将 dist/ 目录上传到服务器
scp -r dist/* user@server:/var/www/docs/
```

### 使用 GitHub Webhook 自动部署

项目包含完整的 webhook 部署方案：

1. 服务器端 Rust webhook 服务 (`webhook-server/`)
2. 部署脚本 (`deploy.sh`)
3. systemd 服务配置 (`webhook-server.service`)
4. 详细部署文档 (`DEPLOYMENT.md`)

参考 `DEPLOYMENT.md` 获取完整配置步骤。

## 故障排查

### 构建错误

1. 确认依赖已安装：`pip install -r requirements.txt`
2. 检查 RST 语法错误
3. 确认引用的文件存在

### 图片不显示

确保图片路径相对于 `.rst` 文件正确：

```rst
.. image:: ../../_static/your-image.png
```

### 缺少中文字体

在服务器上安装中文字体：

```bash
# Ubuntu/Debian
apt-get install fonts-wqy-zenhei fonts-wqy-microhei

# CentOS/RHEL
yum install wqy-zenhei-fonts wqy-microhei-fonts
```

## 许可证

本模板按原样提供，用于创建基于 Sphinx 的文档。

## 资源

- [Sphinx 文档](https://www.sphinx-doc.org/)
- [reStructuredText 入门](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html)
- [Read the Docs 主题](https://sphinx-rtd-theme.readthedocs.io/)

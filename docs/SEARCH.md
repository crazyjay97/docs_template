# 文档搜索功能说明

## 当前搜索方案：Sphinx 内置全文搜索

你的项目使用 Sphinx 构建，**已经内置了全文搜索功能**。


### 功能特点

| 功能 | 说明 |
|------|------|
| 全文索引 | 自动构建所有文档的全文索引 |
| 关键词高亮 | 搜索结果中 highlight 匹配关键词 |
| 多语言支持 | 支持英文和中文分词搜索 |
| 离线搜索 | 搜索索引在客户端，无需服务器 |

### 使用方法

1. **点击侧边栏搜索框**
2. **快捷键**: 按 `Ctrl+K` (Windows/Linux) 或 `Cmd+K` (Mac) 快速聚焦搜索框
3. 输入关键词后按回车
4. 在搜索结果页面查看匹配结果

### 搜索索引构建

搜索索引在构建文档时自动生成 (`searchindex.js`)：

```bash
# 构建文档时会自动生成搜索索引
source venv/bin/activate && build-docs
```

### 部署注意事项

确保 `searchindex.js` 文件被正确复制到部署目录：

```bash
# 检查 dist 目录是否包含搜索索引
ls -la dist/en/searchindex.js
ls -la dist/zh_CN/searchindex.js
```

---

## 可选：集成 Algolia DocSearch（更强大的搜索）

如果你需要更好的搜索体验（如即时搜索、模糊匹配、同义词、分析），可以集成 Algolia DocSearch。

### 集成步骤

#### 1. 申请 DocSearch

访问 https://docsearch.algolia.com/ 提交申请

#### 2. 安装爬虫 (本地索引)

```bash
pip install docsearch-scraper
```

#### 3. 配置爬虫

创建 `docsearch.config.json`:

```json
{
  "index_name": "your-project",
  "start_urls": [
    "https://your-domain.com/en/"
  ],
  "sitemap_urls": [
    "https://your-domain.com/sitemap.xml"
  ],
  "selectors": {
    "lvl0": ".wy-menu-vertical li.current > a",
    "lvl1": "h1",
    "lvl2": "h2",
    "lvl3": "h3",
    "lvl4": "h4",
    "text": "p, li, td"
  }
}
```

#### 4. 在页面中添加搜索框

在 `conf_common.py` 中添加：

```python
html_js_files = html_js_files + [
    ('https://cdn.jsdelivr.net/npm/@docsearch/js@3', {
        'integrity': 'sha384-xxx',
        'crossorigin': 'anonymous'
    }),
]

html_css_files = html_css_files + [
    'https://cdn.jsdelivr.net/npm/@docsearch/css@3',
]
```

然后在模板中添加搜索容器或使用 Kapa AI 这样的替代方案。

---

## 当前项目配置

### 已启用的搜索相关配置

1. **Sphinx 搜索** - 默认启用
2. **Kapa AI Chatbot** - AI 助手搜索（需要配置 API）
3. **增强搜索脚本** - `enhanced_search.js` (Ctrl+K 快捷键)

### 配置文件位置

- `docs/conf_common.py` - 通用配置
- `docs/en/conf.py` - 英文配置
- `docs/zh_CN/conf.py` - 中文配置
- `docs/_static/js/enhanced_search.js` - 增强搜索脚本

### 验证搜索功能

构建后访问：
- `http://localhost/en/search.html?q=keyword`
- `http://localhost/zh_CN/search.html?q=关键词`

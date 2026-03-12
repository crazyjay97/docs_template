# 文档搜索功能实现总结

## 已完成的工作

### 1. Sphinx 内置搜索（已启用）

你的项目使用 Sphinx，**自带全文搜索功能**：

- **搜索索引**: `searchindex.js` (构建时自动生成)
- **搜索工具**: `searchtools.js` (Sphinx 提供)
- **搜索页面**: `search.html` (搜索结果显示页)
- **搜索框**: 侧边栏已内置

### 2. 增强搜索脚本（已添加）

创建了 `docs/_static/js/enhanced_search.js`，提供：

| 功能 | 说明 |
|------|------|
| `Ctrl+K` / `Cmd+K` | 快速聚焦搜索框 |
| 结果计数 | 显示找到多少条结果 |
| 更好的占位符 | 显示 "Search docs (Ctrl+K)" |

### 3. 配置更新

已更新 `docs/conf_common.py`:
```python
html_js_files = [
    'js/version_table.js',
    'js/enhanced_search.js',  # 新增
]
```

---

## 使用搜索功能

### 构建文档

```bash
cd docs
source ../venv/bin/activate
build-docs
```

### 搜索方式

1. **侧边栏搜索框** - 直接输入关键词
2. **快捷键** - 按 `Ctrl+K` (Mac: `Cmd+K`) 快速聚焦
3. **URL 直接搜索** - `search.html?q=关键词`

### 部署后访问

```
http://your-domain.com/en/search.html?q=installation
http://your-domain.com/zh_CN/search.html?q=安装
```

---

## 搜索原理

```
源文件 (.rst) → Sphinx 构建 → searchindex.js (全文索引)
                                        ↓
                              用户搜索 → searchtools.js 匹配 → 显示结果
```

Sphinx 在构建时会：
1. 解析所有 `.rst` 文件
2. 提取文本内容建立倒排索引
3. 生成 `searchindex.js` (包含所有文档的关键词索引)
4. 用户搜索时在浏览器端进行匹配

---

## 如果需要更强大的搜索

考虑集成 **Algolia DocSearch** 或 **Kapa AI**（已配置）：

- **Algolia DocSearch**: 云端搜索，支持模糊匹配、同义词、分析
- **Kapa AI**: AI 助手，可以回答问题（需要 API key）

Kapa AI 配置位置：
- `docs/_static/js/chatbot_widget_en.js`
- `docs/_static/js/chatbot_widget_cn.js`

需要替换 `data-website-id` 为你的实际 ID。

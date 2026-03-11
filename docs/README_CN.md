# 文档模板

基于 ESP-IDF 文档结构的可复用 Sphinx 文档模板，支持中英文国际化 (i18n)。

## 功能特性

- **Sphinx + esp-docs**: 使用与 ESP-IDF 文档相同的技术栈
- **国际化 (i18n)**: 内置中英文支持，可轻松切换语言
- **开箱即用**: 只需修改内容即可发布文档
- **功能完整**: 包含 esp-docs 所有核心特性

## 快速开始

### 1. 安装依赖

```bash
pip install -r requirements.txt
```

### 2. 构建文档

英文文档：
```bash
cd en
build-docs
```

中文文档：
```bash
cd zh_CN
build-docs
```

### 3. 查看文档

在浏览器中打开 `_build/html/index.html` 查看生成的文档。

## 目录结构

```
template_docs/
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
│   │   └── version_table.js
│   └── *.png, *.svg            # 图片和图标
├── en/                         # 英文文档
│   ├── conf.py
│   ├── index.rst
│   ├── 404.rst
│   ├── about.rst
│   ├── languages.rst
│   └── api-guides/
│       ├── index.rst
│       └── features-example.rst
└── zh_CN/                      # 中文文档
    ├── conf.py
    ├── index.rst
    ├── 404.rst
    ├── about.rst
    ├── languages.rst
    └── api-guides/
        ├── index.rst
        └── features-example.rst
```

## 配置说明

### conf_common.py

此文件包含所有语言共享的通用 Sphinx 配置：

- **extensions**: Sphinx 扩展（复制按钮、wavedrom 等）
- **github_repo**: GitHub 仓库地址
- **project_slug**: 主题使用的项目 slug
- **versions_url**: 版本选择 URL
- **languages**: 支持的语言列表
- **html_static_path**: 静态文件路径
- **conditional_include_dict**: 条件内容配置

### 语言特定的 conf.py

每个语言文件夹有自己的 `conf.py`：

- 导入 `conf_common.py`
- 设置 `project` 项目名称
- 设置 `copyright` 版权信息
- 设置 `language` 语言代码
- 配置语言特定的 JavaScript 文件

## 添加新页面

1. 在相应语言文件夹中创建新的 `.rst` 文件
2. 将文件添加到 `index.rst` 或相关章节索引的 `toctree` 中
3. 在文件顶部添加翻译链接：`:link_to_translation:`en:[English]``
4. 在另一个语言文件夹中创建对应的翻译文件

## Sphinx 语法示例

查看 `en/api-guides/features-example.rst` 获取 comprehensive 示例：

- 带语法高亮的代码块
- 表格（list-table 和 grid tables）
- 提示框、警告框、建议框和重要信息框
- 交叉引用
- 图片和图表
- 条件内容
- 链接（内部和外部）
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
3. 更新 `:link_to_translation:` 指令指向英文版本

## 自定义

### 主题覆盖

编辑 `_static/css/theme_overrides.css` 自定义外观：

```css
/* 示例：更改主色调 */
.wy-side-nav-search {
    background-color: #your-color;
}
```

### Chatbot 机器人

编辑 `_static/js/chatbot_widget_en.js` 和 `_static/js/chatbot_widget_cn.js` 配置 AI 聊天机器人：

- 将 `your-website-id-here` 替换为实际的 Kapa.ai 网站 ID
- 更新品牌颜色和 logo
- 自定义免责声明消息

### 页面重定向

编辑 `page_redirects.txt` 设置 URL 重定向（适用于移动或重命名的页面）：

```
old/page/path    new/page/path
```

## 高级功能

### 条件内容

使用 `.. only::` 指令条件显示内容：

```rst
.. only:: html

    此内容仅在 HTML 构建中可见。

.. only:: custom_tag

    此内容仅在定义了 custom_tag 时可见。
```

### 版本选择

模板支持版本选择。在 `conf_common.py` 中配置 `versions_url` 启用此功能。

## 常见问题

### 构建错误

如果遇到构建错误：

1. 确保所有依赖已安装：`pip install -r requirements.txt`
2. 检查 RST 语法错误
3. 验证所有引用的文件存在

### 缺少翻译

如果页面没有翻译，用户将被重定向到英文版本。这是预期行为。

## 许可证

本模板按原样提供，用于创建基于 Sphinx 的文档。

## 资源

- [Sphinx 文档](https://www.sphinx-doc.org/)
- [reStructuredText 入门](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html)
- [esp-docs](https://github.com/espressif/esp-docs)

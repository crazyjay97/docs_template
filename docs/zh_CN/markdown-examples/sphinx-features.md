# Markdown 中的 Sphinx 高级功能

:link_to_translation:`en:[English]`

本文档展示使用 MyST-Parser 在 Markdown 中可用的 Sphinx 特有功能。

## 文档目录（Toctree）

```{toctree}
:hidden:

basic-syntax
../api-guides/index
```

## 交叉引用

### 引用章节

可以使用标签引用其他章节：

```markdown
```{ref} section-label
```

或使用 Markdown 链接：

```markdown
[链接文本](section-label)
```

### 创建标签

```markdown
(my-label)=
## 章节标题
```

然后引用它：

```markdown
[跳转到章节](my-label)
```

### 标签示例

(label-example-zh)=
#### 这是带标签的章节

你可以使用 `[链接](label-example-zh)` 链接到这个章节。

## 包含其他文件

### 包含 Markdown 文件

```{include} basic-syntax.md
:start-line: 0
:end-line: 10
```

### 包含代码文件语法

下面展示 `literalinclude` 的写法。为了避免示例页在默认构建时依赖不存在的文件，这里只展示语法，不实际执行：

````markdown
```{literalinclude} path/to/example.py
:language: python
:lines: 1-20
:caption: 示例代码
```
````

## 术语表

定义术语：

```{glossary}

API
  应用程序编程接口 - 一组允许创建应用程序的函数和过程。

框架
  用于开发软件应用程序的平台。
```

引用术语表术语：

- 查看 {term}`API` 文档
- 了解 {term}`框架`

## 版本添加/更改

```{versionadded} 1.0.0
   此功能在 1.0.0 版本中添加。
```

```{versionchanged} 2.0.0
   此功能在 2.0.0 版本中更改。
```

```{deprecated} 3.0.0
   此功能自 3.0.0 版本已弃用。
```

## 参数和返回值

```{py:function} compute(x, y, operation='add')

   根据操作计算结果。

   :param x: 第一个操作数
   :param y: 第二个操作数
   :param operation: 要执行的操作（默认：'add'）
   :type x: int
   :type y: int
   :type operation: str
   :return: 计算结果
   :rtype: int
   :raises ValueError: 如果不支持该操作
```

## 带标题的图

```{figure} ../../_static/get-started.png
:name: figure-example-zh
:alt: 示例图
:width: 400px

这是图的标题。图可以包含 Markdown 和 **粗体文本**。
```

引用图：参见 [图 1](figure-example-zh)。

## 特殊表格功能

### CSV 表格

注意：CSV 表格选项必须从第 0 列开始，不能有前导空格。

````markdown
```{csv-table}
:header: "姓名","年龄","城市"
:widths: 20, 10, 30

"爱丽丝", 25, "纽约"
"鲍勃", 30, "伦敦"
"查理", 35, "东京"
```
````

### 列表表格

```{list-table}
:header-rows: 1
:widths: 25 25 50

* - 列 1
  - 列 2
  - 描述
* - 项目 A
  - 项目 B
  - 这是一个描述
* - 项目 C
  - 项目 D
  - 另一个描述
```

## 选项卡（需要 sphinx-tabs 扩展）

tabs 指令需要 `sphinx-tabs` 扩展。启用方法：

1. 安装：`pip install sphinx-tabs`
2. 在 `conf_common.py` 的 `extensions` 中添加：`'sphinx_tabs.tabs'`

示例语法：

`````markdown
````{tabs}
```{group-tab} Python

```python
def hello():
    print("来自 Python 的问候！")
```
```

```{group-tab} JavaScript

```javascript
function hello() {
    console.log("来自 JavaScript 的问候！");
}
```
```
````
`````

## 自定义指令

### 原始 HTML

```{raw} html
<div style="background-color: #f0f0f0; padding: 10px; border-radius: 5px;">
    <p>这是自定义 HTML 内容。</p>
</div>
```

### 带自定义类的容器

```{container} custom-class

此内容包裹在带有类 "custom-class" 的 div 中。

```

## 索引条目

{index}`single: Python; 示例`

这会为 "Python; 示例" 创建索引条目。

## 脚注

这是一个脚注引用 [^1]。

[^1]: 这是脚注内容。

## 引用（需要 sphinxcontrib-bibtex）

bibliography 指令需要 `sphinxcontrib-bibtex` 扩展。启用方法：

1. 安装：`pip install sphinxcontrib-bibtex`
2. 在 `conf_common.py` 的 `extensions` 中添加：`'sphinxcontrib.bibtex'`
3. 配置参考文献文件

示例语法：

````markdown
```{bibliography}
:style: unsrt
```
````

## 总结

| 功能 | 语法 | 描述 |
|------|------|------|
| 目录树 | `{toctree}` | 文档树 |
| 交叉引用 | `{ref}`, 标签 | 链接到章节 |
| 包含 | `{include}` | 包含文件内容 |
| 术语表 | `{glossary}` | 定义术语 |
| 版本 | `{versionadded}` | 版本信息 |
| 图 | `{figure}` | 带标题的图片 |
| 表格 | `{csv-table}` | 特殊表格 |
| 脚注 | `[^1]` | 脚注引用 |

更多详情，请访问 [Sphinx 文档](https://www.sphinx-doc.org/)。

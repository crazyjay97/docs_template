# Markdown 基础语法

:link_to_translation:`en:[English]`

本文档展示 MyST-Parser 支持的 Markdown 基础语法。

## 标题

使用 `#` 表示标题。`#` 的数量决定标题级别：

```markdown
# 一级标题
## 二级标题
### 三级标题
#### 四级标题
```

## 文本格式

| 语法 | 效果 |
|------|------|
| `*斜体*` | *斜体* |
| `**粗体**` | **粗体** |
| `***粗斜体***` | ***粗斜体*** |
| `~~删除线~~` | ~~删除线~~ |
| `` `行内代码` `` | `行内代码` |

## 列表

### 无序列表

- 项目 1
- 项目 2
  - 嵌套项目 2.1
  - 嵌套项目 2.2
- 项目 3

### 有序列表

1. 第一项
2. 第二项
3. 第三项

### 任务列表

- [x] 已完成的任务
- [ ] 未完成的任务
- [ ] 另一个未完成任务

## 代码块

### Python 代码

```python
def greet(name):
    """向某人打招呼"""
    print(f"你好，{name}!")

greet("世界")
```

### Bash 命令

```bash
# 安装依赖
pip install -r requirements.txt

# 运行应用程序
python main.py
```

### 带行号的代码

````markdown
```{code-block} python
:linenos:

def complex_function():
    result = 0
    for i in range(100):
        result += i * 2
    return result
```
````

## 引用块

> 这是一个引用块。
> 它可以跨越多行。
>
> > 嵌套引用块也受支持。

## 表格

| 列 1 | 列 2 | 列 3 |
|------|------|------|
| 单元格 1 | 单元格 2 | 单元格 3 |
| 单元格 4 | 单元格 5 | 单元格 6 |

### 带对齐的表格

| 左对齐 | 居中对齐 | 右对齐 |
|:-------|:--------:|-------:|
| 文本   | 文本     | 文本   |
| 更多文本 | 更多文本 | 更多文本 |

## 链接和图片

### 链接

- [外部链接](https://www.example.com)
- [内部链接](../index)
- 自动链接：<https://www.example.com>

### 图片

```markdown
![替代文本](../../_static/get-started.png)
```

![示例图片](../../_static/get-started.png)

## 数学公式

### 行内公式

质能等价公式为 $E = mc^2$。

### 块级公式

```{math}
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
```

### 编号公式

```{math}
:label: quadratic-zh

x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
```

参见公式 {math:numref}`quadratic-zh`。

## 提示框

```{note}
这是一个 note 提示框。
```

```{tip}
这是一个 tip 提示框。
```

```{warning}
这是一个 warning 警告框。
```

```{danger}
这是一个 danger 危险框。
```

## 定义列表

术语 1
: 定义 1

术语 2
: 定义 2
: 替代定义

## 水平线

水平线下方是一条分隔线。

---

## 结论

以上涵盖了 Markdown 基础语法。更多高级功能请参阅 [MyST-Parser 文档](https://myst-parser.readthedocs.io/)。

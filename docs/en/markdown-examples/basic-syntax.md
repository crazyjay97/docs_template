# Basic Markdown Syntax

:link_to_translation:`zh_CN:[中文]`

This document demonstrates basic Markdown syntax supported by MyST-Parser.

## Headings

Use `#` for headings. The number of `#` symbols determines the heading level:

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
```

## Text Formatting

| Syntax | Result |
|--------|--------|
| `*italic*` | *italic* |
| `**bold**` | **bold** |
| `***bold italic***` | ***bold italic*** |
| `~~strikethrough~~` | ~~strikethrough~~ |
| `` `inline code` `` | `inline code` |

## Lists

### Unordered List

- Item 1
- Item 2
  - Nested item 2.1
  - Nested item 2.2
- Item 3

### Ordered List

1. First item
2. Second item
3. Third item

### Task List

- [x] Completed task
- [ ] Incomplete task
- [ ] Another incomplete task

## Code Blocks

### Python Code

```python
def greet(name):
    """Say hello to someone."""
    print(f"Hello, {name}!")

greet("World")
```

### Bash Commands

```bash
# Install dependencies
pip install -r requirements.txt

# Run the application
python main.py
```

### Code with Line Numbers

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

## Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> > Nested blockquotes are also supported.

## Tables

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

### Table with Alignment

| Left Aligned | Center Aligned | Right Aligned |
|:-------------|:--------------:|--------------:|
| Text         | Text           | Text          |
| More text    | More text      | More text     |

## Links and Images

### Links

- [External Link](https://www.example.com)
- [Internal Link](../index)
- Auto-link: <https://www.example.com>

### Images

```markdown
![Alt text](../../_static/get-started.png)
```

![Example Image](../../_static/get-started.png)

## Mathematical Expressions

### Inline Math

The mass-energy equivalence is expressed as $E = mc^2$.

### Block Math

```{math}
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
```

### Numbered Equations

```{math}
:label: quadratic

x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
```

See equation {math:numref}`quadratic` above.

## Admonitions

```{note}
This is a note admonition.
```

```{tip}
This is a tip admonition.
```

```{warning}
This is a warning admonition.
```

```{danger}
This is a danger admonition.
```

## Definition Lists

Term 1
: Definition 1

Term 2
: Definition 2
: Alternative definition

## Horizontal Rule

Some text before the horizontal rule.

---

## Conclusion

This covers the basic Markdown syntax. For more advanced features, see the [MyST-Parser documentation](https://myst-parser.readthedocs.io/).

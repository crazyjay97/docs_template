# Advanced Sphinx Features in Markdown

:link_to_translation:`zh_CN:[中文]`

This document demonstrates advanced Sphinx-specific features that work in Markdown using MyST-Parser.

## Table of Contents (Toctree)

```{toctree}
:hidden:

basic-syntax
../api-guides/index
```

## Cross-References

### Referencing Sections

You can reference other sections using labels:

```markdown
```{ref} section-label
```

or using markdown links:

```markdown
[Link text](section-label)
```

### Creating Labels

```markdown
(my-label)=
## Section Title
```

Then reference it:

```markdown
[Go to Section](my-label)
```

### Example Labels

(label-example)=
#### This is a Labeled Section

You can link to this section using `[link](label-example)`.

## Including Other Files

### Include Markdown File

```{include} basic-syntax.md
:start-line: 0
:end-line: 10
```

### Include Code File Syntax

This shows the `literalinclude` syntax only. To keep the example page warning-free in a default build, it does not try to include a real file:

````markdown
```{literalinclude} path/to/example.py
:language: python
:lines: 1-20
:caption: Example Code
```
````

## Glossary Terms

Define terms in a glossary:

```{glossary}

API
  Application Programming Interface - a set of functions and procedures allowing the creation of applications.

Framework
  A platform for developing software applications.
```

Reference glossary terms:

- Check out the {term}`API` documentation
- Learn about the {term}`Framework`

## Version Added/Changed

```{versionadded} 1.0.0
   This feature was added in version 1.0.0.
```

```{versionchanged} 2.0.0
   This feature was changed in version 2.0.0.
```

```{deprecated} 3.0.0
   This feature is deprecated since version 3.0.0.
```

## Parameters and Returns

```{py:function} compute(x, y, operation='add')

   Compute a result based on operation.

   :param x: First operand
   :param y: Second operand
   :param operation: Operation to perform (default: 'add')
   :type x: int
   :type y: int
   :type operation: str
   :return: Computed result
   :rtype: int
   :raises ValueError: If operation is not supported
```

## Figures with Captions

```{figure} ../../_static/get-started.png
:name: figure-example
:alt: Example Figure
:width: 400px

This is a figure caption. Figures can contain markdown and **bold text**.
```

Reference the figure: See [Figure 1](figure-example).

## Tables with Special Features

### CSV Tables

Note: CSV table options must start at column 0 with no leading spaces.

````markdown
```{csv-table}
:header: "Name","Age","City"
:widths: 20, 10, 30

"Alice", 25, "New York"
"Bob", 30, "London"
"Charlie", 35, "Tokyo"
```
````

### List Tables

```{list-table}
:header-rows: 1
:widths: 25 25 50

* - Column 1
  - Column 2
  - Description
* - Item A
  - Item B
  - This is a description
* - Item C
  - Item D
  - Another description
```

## Tabs (requires sphinx-tabs extension)

The tabs directive requires the `sphinx-tabs` extension. To enable it:

1. Install: `pip install sphinx-tabs`
2. Add to `extensions` in `conf_common.py`: `'sphinx_tabs.tabs'`

Example syntax:

`````markdown
````{tabs}
```{group-tab} Python

```python
def hello():
    print("Hello from Python!")
```
```

```{group-tab} JavaScript

```javascript
function hello() {
    console.log("Hello from JavaScript!");
}
```
```
````
`````

## Custom Directives

### Raw HTML

```{raw} html
<div style="background-color: #f0f0f0; padding: 10px; border-radius: 5px;">
    <p>This is custom HTML content.</p>
</div>
```

### Container with Custom Classes

```{container} custom-class

This content is wrapped in a div with class "custom-class".

```

## Index Entries

{index}`single: Python; examples`

This creates an index entry for "Python; examples".

## Footnotes

Here is a footnote reference [^1].

[^1]: This is the footnote content.

## Citations (requires sphinxcontrib-bibtex)

The bibliography directive requires the `sphinxcontrib-bibtex` extension. To enable it:

1. Install: `pip install sphinxcontrib-bibtex`
2. Add to `extensions` in `conf_common.py`: `'sphinxcontrib.bibtex'`
3. Configure a bibliography file

Example syntax:

````markdown
```{bibliography}
:style: unsrt
```
````

## Summary

| Feature | Syntax | Description |
|---------|--------|-------------|
| Toctree | `{toctree}` | Document tree |
| Cross-ref | `{ref}`, labels | Link to sections |
| Include | `{include}` | Include file content |
| Glossary | `{glossary}` | Define terms |
| Version | `{versionadded}` | Version info |
| Figure | `{figure}` | Images with captions |
| Table | `{csv-table}` | Special tables |
| Footnotes | `[^1]` | Footnote references |

For more details, visit the [Sphinx documentation](https://www.sphinx-doc.org/).

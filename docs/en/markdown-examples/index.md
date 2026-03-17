# Markdown Examples

:link_to_translation:`zh_CN:[中文]`

This section provides examples of using Markdown (`.md`) files in Sphinx documentation with MyST-Parser support.

## Available Examples

```{toctree}
:maxdepth: 1

basic-syntax
sphinx-features
```

## Quick Start

Creating a Markdown document is simple:

1. Create a `.md` file in your documentation directory
2. Write content using standard Markdown syntax
3. Add the file to a `toctree` directive
4. Build your documentation

## Why Use Markdown?

- **Familiar syntax**: Markdown is widely known and easy to learn
- **Simpler for basic content**: Less verbose than reStructuredText
- **Better tool support**: Many editors have excellent Markdown preview
- **Migration friendly**: Easier to import existing Markdown documentation

## Markdown vs reStructuredText

Both formats are fully supported. Choose based on your preference:

| Aspect | Markdown | reStructuredText |
|--------|----------|------------------|
| Learning curve | Gentle | Steeper |
| Syntax verbosity | Low | Higher |
| Native Sphinx support | Via MyST-Parser | Native |
| Complex features | Good with MyST | Excellent |
| File extension | `.md` | `.rst` |

## Tips

- Use `.md` for new documentation to reduce migration effort
- Existing `.rst` files continue to work without changes
- Both formats can coexist in the same documentation
- For complex Sphinx features, check the [MyST-Parser documentation](https://myst-parser.readthedocs.io/)

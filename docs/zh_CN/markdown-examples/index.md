# Markdown 示例

:link_to_translation:`en:[English]`

本章节提供在 Sphinx 文档中使用 Markdown（`.md`）文件的示例，由 MyST-Parser 支持。

## 可用示例

```{toctree}
:maxdepth: 1

basic-syntax
sphinx-features
```

## 快速开始

创建 Markdown 文档很简单：

1. 在文档目录中创建 `.md` 文件
2. 使用标准 Markdown 语法编写内容
3. 将文件添加到 `toctree` 指令
4. 构建文档

## 为什么使用 Markdown？

- **熟悉的语法**：Markdown 广为人知，易于学习
- **基本内容更简单**：比 reStructuredText 更简洁
- **更好的工具支持**：许多编辑器有出色的 Markdown 预览功能
- **便于迁移**：更容易导入现有的 Markdown 文档

## Markdown vs reStructuredText

两种格式都完全支持。根据你的偏好选择：

| 方面 | Markdown | reStructuredText |
|------|----------|------------------|
| 学习曲线 | 平缓 | 较陡 |
| 语法简洁性 | 低 | 较高 |
| 原生 Sphinx 支持 | 通过 MyST-Parser | 原生 |
| 复杂功能 | 使用 MyST 良好 | 优秀 |
| 文件扩展名 | `.md` | `.rst` |

## 提示

- 使用 `.md` 编写新文档以减少迁移工作
- 现有的 `.rst` 文件继续工作，无需更改
- 两种格式可以在同一文档中共存
- 对于复杂的 Sphinx 功能，请查阅 [MyST-Parser 文档](https://myst-parser.readthedocs.io/)

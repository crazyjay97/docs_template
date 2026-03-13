Sphinx 特性示例
===============

:link_to_translation:`en:[English]`

本页面演示了各种 Sphinx 功能和 RST 语法，您可以在文档中使用这些功能。

代码块
------

以下是带有语法高亮的代码块示例：

.. code-block:: c

    #include <stdio.h>

    int main(void)
    {
        printf("Hello, World!\n");
        return 0;
    }

您也可以对其他编程语言使用 ``code-block`` 指令：

.. code-block:: python

    def hello():
        print("Hello from Python!")

    if __name__ == "__main__":
        hello()

.. code-block:: bash

    echo "Hello from Bash"

行内代码
--------

使用双反引号表示行内代码：``int x = 5;`` 或 ``function_name()``。

表格
----

简单表格
~~~~~~~~

.. list-table::
   :header-rows: 1

   * - 名称
     - 描述
     - 类型
   * - GPIO
     - 通用输入输出
     - 外设
   * - UART
     - 通用异步收发器
     - 通信
   * - SPI
     - 串行外设接口
     - 通信

网格表格
~~~~~~~~

+----------+----------+----------+
| 表头 1   | 表头 2   | 表头 3   |
+==========+==========+==========+
| 单元格 1 | 单元格 2 | 单元格 3 |
+----------+----------+----------+
| 单元格 4 | 单元格 5 | 单元格 6 |
+----------+----------+----------+

提示和警告
----------

.. note::

    这是提示框。使用提示框提供额外信息或使用技巧。

.. warning::

    这是警告框。使用警告框提醒用户潜在问题。

.. tip::

    这是建议框。使用建议框分享有用的建议。

.. important::

    这是重要信息。用户应该注意这些内容。

.. code-block:: c
    :caption: 带标题的示例

    int example = 42;

交叉引用
--------

您可以使用 ``:ref:`` 角色引用其他章节：

- 引用章节：:ref:`code-blocks`
- 引用其他文档：:doc:`index`

对于 API 文档，您可以使用：

- :func:`function_name` 引用函数
- :class:`ClassName` 引用类
- :meth:`method_name` 引用方法
- :attr:`attribute_name` 引用属性

列表
----

无序列表
~~~~~~~~

- 项目 1
- 项目 2
- 项目 3
    - 嵌套项目 1
    - 嵌套项目 2

有序列表
~~~~~~~~

1. 第一步
2. 第二步
3. 第三步

定义列表
~~~~~~~~

术语 1
    术语 1 的定义

术语 2
    术语 2 的定义

图片
----

.. figure:: ../../_static/get-started.png
    :align: center
    :alt: 图片的替代文本
    :figclass: align-center

    这是图片的标题。您可以使用它来解释图片内容。

条件内容
--------

您可以使用 ``.. only::`` 指令有条件地包含内容：

.. only:: html

    此内容仅在 HTML 构建中可见。

.. only:: latex

    此内容仅在 LaTeX/PDF 构建中可见。

.. only:: some_custom_tag

    此内容仅在定义了 ``some_custom_tag`` 标签时可见。
    您可以在配置中定义自定义标签，或在运行构建命令时定义。

链接
----

外部链接
~~~~~~~~

- `Sphinx 文档 <https://www.sphinx-doc.org/>`_
- `reStructuredText 入门 <https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html>`_

内部链接
~~~~~~~~

- :doc:`index` - 链接到 index 文档
- :ref:`genindex` - 链接到总索引

数学公式
--------

您可以包含数学表达式：

行内数学公式：:math:`E = mc^2`

块级数学公式：

.. math::

    \int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}

注释
----

.. 这是注释。注释在渲染输出中不可见。
   您可以使用注释为自己或其他作者留下备注。

.. |substitution| replace:: 这是文本替换

您也可以通过在上文定义替换，然后在文本中使用 |substitution|。

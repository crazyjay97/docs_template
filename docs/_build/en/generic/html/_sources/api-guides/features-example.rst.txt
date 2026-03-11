Sphinx Features Example
=======================

:link_to_translation:`zh_CN:[中文]`

This page demonstrates various Sphinx features and RST syntax that you can use in your documentation.

Code Blocks
-----------

Here is an example of a code block with syntax highlighting:

.. code-block:: c

    #include <stdio.h>

    int main(void)
    {
        printf("Hello, World!\n");
        return 0;
    }

You can also use the ``code-block`` directive with different languages:

.. code-block:: python

    def hello():
        print("Hello from Python!")

    if __name__ == "__main__":
        hello()

.. code-block:: bash

    echo "Hello from Bash"

Inline Code
-----------

Use double backticks for inline code: ``int x = 5;`` or ``function_name()``.

Tables
------

Simple Table
~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Name
     - Description
     - Type
   * - GPIO
     - General Purpose Input/Output
     - Peripheral
   * - UART
     - Universal Asynchronous Receiver/Transmitter
     - Communication
   * - SPI
     - Serial Peripheral Interface
     - Communication

Grid Table
~~~~~~~~~~

+----------+----------+----------+
| Header 1 | Header 2 | Header 3 |
+==========+==========+==========+
| Cell 1   | Cell 2   | Cell 3   |
+----------+----------+----------+
| Cell 4   | Cell 5   | Cell 6   |
+----------+----------+----------+

Notes and Warnings
------------------

.. note::

    This is a note. Use notes to provide additional information or tips.

.. warning::

    This is a warning. Use warnings to alert users about potential issues.

.. tip::

    This is a tip. Use tips to share helpful suggestions.

.. important::

    This is important information. Users should pay attention to this.

.. code-block:: c
    :caption: Example with caption

    int example = 42;

Cross-References
----------------

You can reference other sections using the ``:ref:`` role:

- Reference to a section: :ref:`code-blocks`
- Reference to another document: :doc:`index`

For API documentation, you can use:

- :func:`function_name` for functions
- :class:`ClassName` for classes
- :meth:`method_name` for methods
- :attr:`attribute_name` for attributes

Lists
-----

Unordered List
~~~~~~~~~~~~~~

- Item 1
- Item 2
- Item 3
    - Nested item 1
    - Nested item 2

Ordered List
~~~~~~~~~~~~

1. First step
2. Second step
3. Third step

Definition List
~~~~~~~~~~~~~~~

Term 1
    Definition of term 1

Term 2
    Definition of term 2

Images
------

.. figure:: ../../_static/get-started.png
    :align: center
    :alt: Alternative text for the image
    :figclass: align-center

    This is a caption for the figure. You can use this to explain the image.

Conditional Content
-------------------

You can include content conditionally using the ``.. only::`` directive:

.. only:: html

    This content is only visible in HTML builds.

.. only:: latex

    This content is only visible in LaTeX/PDF builds.

.. only:: some_custom_tag

    This content is only visible when the ``some_custom_tag`` tag is defined.
    You can define custom tags in your configuration or when running the build command.

Links
-----

External Links
~~~~~~~~~~~~~~

- `Sphinx Documentation <https://www.sphinx-doc.org/>`_
- `reStructuredText Primer <https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html>`_

Internal Links
~~~~~~~~~~~~~~

- :doc:`index` - Link to the index document
- :ref:`genindex` - Link to the general index

Mathematics
-----------

You can include mathematical expressions:

Inline math: :math:`E = mc^2`

Block math:

.. math::

    \int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}

Comments
--------

.. This is a comment. Comments are not visible in the rendered output.
   You can use comments to leave notes for yourself or other authors.

.. |substitution| replace:: This is a text substitution

You can also use |substitution| in your text by defining substitutions above.

快速入门
===========

:link_to_translation:`en:[English]`

本快速入门指南将帮助您在几分钟内开始使用您的项目。

基本用法
-----------

以下是一个简单的入门示例：

.. code-block:: python

    import your_module

    # 初始化模块
    your_module.init()

    # 使用主函数
    result = your_module.do_something()
    print(f"结果：{result}")

配置
-------------

您可以使用环境变量配置模块：

.. code-block:: bash

    export YOUR_MODULE_CONFIG=path/to/config.json
    export YOUR_MODULE_DEBUG=true

或编程方式配置：

.. code-block:: python

    your_module.configure(
        debug=True,
        config_path="path/to/config.json"
    )

下一步
----------

- 查看 :doc:`../api-reference/index` 获取详细的 API 文档
- 阅读 :doc:`../api-guides/index` 了解更多高级使用指南
- 访问 :doc:`../about` 页面了解更多关于项目的信息

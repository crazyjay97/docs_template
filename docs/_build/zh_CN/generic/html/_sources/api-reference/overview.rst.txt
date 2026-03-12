API 概览
============

:link_to_translation:`en:[English]`

本页面提供主要 API 组件的概览。

核心模块
------------

本项目由几个核心模块组成：

- **core**: 主要功能和工具
- **utils**: 辅助函数和工具
- **config**: 配置管理

基本示例
-------------

以下是使用 API 的基本示例：

.. code-block:: python

    from your_module import Core, Config

    # 创建配置
    config = Config()
    config.set_option('key', 'value')

    # 初始化核心模块
    core = Core(config)
    result = core.process()

API 参考
-------------

详细的 API 文档请参阅各个模块页面：

- :mod:`your_module.core` - 核心功能
- :mod:`your_module.utils` - 工具函数
- :mod:`your_module.config` - 配置管理

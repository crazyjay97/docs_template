API Overview
============

:link_to_translation:`zh_CN:[中文]`

This page provides an overview of the main API components.

Core Modules
------------

The project consists of several core modules:

- **core**: Main functionality and utilities
- **utils**: Helper functions and tools
- **config**: Configuration management

Basic Example
-------------

Here's a basic example of using the API:

.. code-block:: python

    from your_module import Core, Config

    # Create configuration
    config = Config()
    config.set_option('key', 'value')

    # Initialize core module
    core = Core(config)
    result = core.process()

API Reference
-------------

For detailed API documentation, see the individual module pages:

- :mod:`your_module.core` - Core functionality
- :mod:`your_module.utils` - Utility functions
- :mod:`your_module.config` - Configuration management

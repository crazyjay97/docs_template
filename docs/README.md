# Documentation Template

A reusable Sphinx documentation template based on the ESP-IDF documentation structure, featuring internationalization (i18n) support for English and Chinese.

## Features

- **Sphinx + esp-docs**: Built on the same technology stack as ESP-IDF documentation
- **Internationalization (i18n)**: Built-in support for English and Chinese with easy language switching
- **Ready to Use**: Just modify the content and publish your documentation
- **Full-Featured**: Includes all core features from esp-docs
- **Nginx Deployment Ready**: Includes build script to create deployable `dist/` directory

## Quick Start

### 1. Install Dependencies

```bash
pip install -r requirements.txt
```

### 2. Build Documentation

For English documentation:
```bash
cd en
build-docs
```

For Chinese documentation:
```bash
cd zh_CN
build-docs
```

### 3. Build for Deployment

To create a `dist/` directory ready for nginx deployment:
```bash
./build_dist.sh
```

The resulting `dist/` directory contains everything needed to serve the documentation directly from a web server.

### 4. View Documentation

Open `_build/html/index.html` in your browser to view the generated documentation.

Or view the deployment-ready version in the `dist/` directory.

## Quick Start

### 1. Install Dependencies

```bash
pip install -r requirements.txt
```

### 2. Build Documentation

For English documentation:
```bash
cd en
build-docs
```

For Chinese documentation:
```bash
cd zh_CN
build-docs
```

### 3. View Documentation

Open `_build/html/index.html` in your browser to view the generated documentation.

## Directory Structure

```
template_docs/
├── conf_common.py              # Common Sphinx configuration
├── requirements.txt            # Python dependencies
├── README.md                   # This file (English)
├── README_CN.md                # Chinese README
├── page_redirects.txt          # Page redirect configuration
├── _static/                    # Static assets
│   ├── css/
│   │   └── theme_overrides.css
│   ├── js/
│   │   ├── chatbot_widget_en.js
│   │   ├── chatbot_widget_cn.js
│   │   └── version_table.js
│   └── *.png, *.svg            # Images and icons
├── en/                         # English documentation
│   ├── conf.py
│   ├── index.rst
│   ├── 404.rst
│   ├── about.rst
│   ├── languages.rst
│   └── api-guides/
│       ├── index.rst
│       └── features-example.rst
└── zh_CN/                      # Chinese documentation
    ├── conf.py
    ├── index.rst
    ├── 404.rst
    ├── about.rst
    ├── languages.rst
    └── api-guides/
        ├── index.rst
        └── features-example.rst
```

## Configuration

### conf_common.py

This file contains the common Sphinx configuration shared by all languages:

- **extensions**: Sphinx extensions (copybutton, wavedrom, etc.)
- **github_repo**: Your GitHub repository
- **project_slug**: Project slug for the theme
- **versions_url**: URL for version selection
- **languages**: List of supported languages
- **html_static_path**: Path to static files
- **conditional_include_dict**: Conditional content configuration

### Language-specific conf.py

Each language folder has its own `conf.py` that:

- Imports `conf_common.py`
- Sets the `project` name
- Sets the `copyright` information
- Sets the `language` code
- Configures language-specific JavaScript files

## Adding New Pages

1. Create a new `.rst` file in the appropriate language folder
2. Add the file to the `toctree` in `index.rst` or relevant section index
3. Add the translation link at the top: `:link_to_translation:`zh_CN:[中文]``
4. Create the corresponding translated file in the other language folder

## Sphinx Syntax Examples

See `en/api-guides/features-example.rst` for comprehensive examples of:

- Code blocks with syntax highlighting
- Tables (list-table and grid tables)
- Notes, warnings, tips, and important boxes
- Cross-references
- Images and figures
- Conditional content
- Links (internal and external)
- Mathematics
- Lists

## Internationalization

### Adding Translation Links

At the top of each RST file, add:

```rst
:link_to_translation:`zh_CN:[中文]`  # For English pages
:link_to_translation:`en:[English]`  # For Chinese pages
```

### Creating Translated Content

1. Copy the English `.rst` file to the corresponding location in `zh_CN/`
2. Translate the content
3. Update the `:link_to_translation:` directive to point to the English version

## Customization

### Theme Overrides

Edit `_static/css/theme_overrides.css` to customize the appearance:

```css
/* Example: Change the primary color */
.wy-side-nav-search {
    background-color: #your-color;
}
```

### Chatbot Widget

Edit `_static/js/chatbot_widget_en.js` and `_static/js/chatbot_widget_cn.js` to configure the AI chatbot:

- Replace `your-website-id-here` with your actual Kapa.ai website ID
- Update branding colors and logos
- Customize the disclaimer message

### Page Redirects

Edit `page_redirects.txt` to set up URL redirects for moved or renamed pages:

```
old/page/path    new/page/path
```

## Advanced Features

### Conditional Content

Use the `.. only::` directive to show content conditionally:

```rst
.. only:: html

    This content is only visible in HTML builds.

.. only:: custom_tag

    This content is only visible when custom_tag is defined.
```

### Version Selection

The template supports version selection. Configure `versions_url` in `conf_common.py` to enable this feature.

## Troubleshooting

### Build Errors

If you encounter build errors:

1. Ensure all dependencies are installed: `pip install -r requirements.txt`
2. Check for RST syntax errors
3. Verify that all referenced files exist

### Missing Translations

If a page doesn't have a translation, users will be directed to the English version. This is expected behavior.

## License

This template is provided as-is for creating Sphinx-based documentation.

## Resources

- [Sphinx Documentation](https://www.sphinx-doc.org/)
- [reStructuredText Primer](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html)
- [esp-docs](https://github.com/espressif/esp-docs)

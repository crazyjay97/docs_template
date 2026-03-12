#
# Common (non-language-specific) configuration for Sphinx
#
# This file is imported from a language-specific conf.py (ie en/conf.py or
# zh_CN/conf.py)
# type: ignore
# pylint: disable=wildcard-import
# pylint: disable=undefined-variable
import os
import re
import sys
from pathlib import Path

# ==============================================================================
# Base Sphinx Configuration (standalone, no esp-docs dependency required)
# ==============================================================================

# Add the parent directory to sys.path for importing
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

# ==============================================================================
# Project Information (override in language-specific conf.py)
# ==============================================================================
# project = 'Your Project Name'  # Set in language-specific conf.py
# copyright = '2016 - {}, Your Company Name'.format(current_year)

# The version info for the project you're documenting
# This can be autocalculated or set manually
release = os.environ.get("PROJECT_RELEASE", "latest")
version = os.environ.get("PROJECT_VERSION", "1.0")

# ==============================================================================
# Supported Languages
# ==============================================================================
languages = ["en", "zh_CN"]

# ==============================================================================
# Sphinx Extensions
# ==============================================================================
extensions = [
    "sphinx_copybutton",
    "jieba_search",  # Chinese word segmentation for better search
    # Note: sphinxcontrib.wavedrom is commented out by default because it requires
    # libxcb.dylib which may not be available on all systems.
    # To enable wavedrom, uncomment the following lines:
    # 'sphinxcontrib.wavedrom',
    # Add more extensions as needed:
    # 'sphinx.ext.intersphinx',
    # 'sphinx.ext.autodoc',
    # 'sphinx.ext.viewcode',
]

# Use wavedrompy as backend, instead of wavedrom-cli
# render_using_wavedrompy = True

# Disable smartquotes to prevent issues with special characters
smartquotes = False

# ==============================================================================
# GitHub Configuration
# ==============================================================================
# Replace with your GitHub repository
github_repo = "your-username/your-repo"
github_version = "main"

# ==============================================================================
# HTML Theme Configuration
# ==============================================================================
# Using sphinx_rtd_theme or sphinx_idf_theme
html_theme = "sphinx_rtd_theme"  # or 'sphinx_idf_theme' if available

# For better nginx deployment
html_extra_path = []  # Additional files to copy to output directory

html_context = {
    "github_user": "your-username",
    "github_repo": "your-repo",
    "github_version": github_version,
    "display_github": True,
}

# Extra options required by sphinx_idf_theme (if using it)
project_slug = "your-project-slug"
versions_url = None  # 'https://your-domain.com/versions.js'

# ==============================================================================
# Static Files Configuration
# ==============================================================================
html_static_path = ["../_static"]
html_css_files = [
    "css/theme_overrides.css",
    "js/chatbot_widget.css",
]

# JavaScript files (can be overridden in language-specific conf.py)
html_js_files = [
    "js/version_table.js",
    "js/enhanced_search.js",  # Enhanced search functionality (Ctrl+K shortcut)
    "js/language_switcher.js",  # Language switcher for nginx subdirectory deployments
]

# ==============================================================================
# HTML Output Options
# ==============================================================================
htmlhelp_basename = "YourProjectDoc"
html_last_updated_fmt = "%b %d, %Y"

# ==============================================================================
# Conditional Content Configuration
# ==============================================================================
# Define which documents to include based on tags/conditions
# Format: {'tag_needed': ['document/path.rst']}
conditional_include_dict = {
    # Example conditional includes:
    # 'FEATURE_A_ENABLED': ['api-guides/feature-a/index.rst'],
    # 'FEATURE_B_ENABLED': ['api-guides/feature-b/index.rst'],
}

# ==============================================================================
# Link Check Configuration
# ==============================================================================
linkcheck_anchors = False
linkcheck_timeout = 30
linkcheck_workers = 1
linkcheck_ignore = [
    # Add URLs that should be ignored by linkcheck
    # 'https://example.com/some-page',
]

linkcheck_exclude_documents = [
    "index",  # May have false positives due to section links
]

# ==============================================================================
# Page Redirects
# ==============================================================================
# Load page redirects from file
page_redirects_file = Path(os.path.dirname(__file__)) / "page_redirects.txt"
html_redirect_pages = []
if page_redirects_file.exists():
    with open(page_redirects_file) as f:
        lines = [
            re.sub(" +", " ", line.strip())
            for line in f.readlines()
            if line.strip() != "" and not line.startswith("#")
        ]
        for line in lines:  # check for well-formed entries
            if len(line.split(" ")) != 2:
                raise RuntimeError(f"Invalid line in page_redirects.txt: {line}")
    html_redirect_pages = [tuple(line.split(" ")) for line in lines]

# ==============================================================================
# Google Analytics (optional)
# ==============================================================================
google_analytics_id = os.environ.get("CI_GOOGLE_ANALYTICS_ID", None)

# ==============================================================================
# Project Homepage
# ==============================================================================
project_homepage = "https://github.com/your-username/your-repo"

# ==============================================================================
# Internationalization (i18n)
# ==============================================================================
locale_dirs = ["../locales/"]
gettext_compact = False

# ==============================================================================
# Options for LaTeX output
# ==============================================================================
latex_elements = {
    "papersize": "a4paper",
    "pointsize": "10pt",
}

# ==============================================================================
# Options for EPUB output
# ==============================================================================
epub_show_urls = "footnote"
epub_description = "Your Project Documentation"


# ==============================================================================
# Setup function for custom initialization
# ==============================================================================
def setup(app):
    """Sphinx setup function."""
    # Add custom tags if needed
    # app.add_config_value('my_custom_option', 'default_value', 'html')

    # Add link_to_translation role for i18n support
    # This creates a simple role that links to translated versions
    from docutils import nodes
    from docutils.parsers.rst import roles

    def link_to_translation_role(
        name, rawtext, text, lineno, inliner, options={}, content=[]
    ):
        """Create a link to translated version using JavaScript for correct path handling."""
        # Parse text like "zh_CN:[中文]" or "en:[English]"
        import re

        match = re.match(r"(\w+):\[(.*?)\]", text)
        if match:
            lang, link_text = match.groups()
            # 直接插入原始 HTML，这样 data-* 属性能被保留
            html = (
                f'<a href="javascript:void(0)" '
                f'class="lang-switch-link" '
                f'data-target-lang="{lang}">'
                f"{link_text}</a>"
            )
            node = nodes.raw("", html, format="html")
            return [node], []
        return [], []

    roles.register_local_role("link_to_translation", link_to_translation_role)


# ==============================================================================
# Callback function for user setup
# ==============================================================================
def user_setup_callback(app, config):
    """
    Callback function for user setup that needs be done after `config-init`-event.

    Override this function in your language-specific conf.py if needed.
    """
    # Example: Set base URL for documentation
    # config.html_baseurl = f'https://docs.your-domain.com/{config.language}/stable/'
    pass

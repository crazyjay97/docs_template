"""
Sphinx Jieba Chinese Word Segmentation Extension

This extension improves Chinese search support by using jieba to tokenize
Chinese text during search index generation.

The key insight is that Sphinx's default search uses simple whitespace splitting,
which doesn't work for Chinese text. This extension patches the word splitting
logic to use jieba for Chinese text.
"""

import os
import re
import sys
from pathlib import Path

# Add the docs directory to path for imports
docs_dir = Path(__file__).parent
if str(docs_dir) not in sys.path:
    sys.path.insert(0, str(docs_dir))

import jieba


class JiebaChineseTextSplitter:
    """
    Chinese text splitter using jieba for Sphinx searchindex generation.
    """

    def __init__(self):
        # Custom stop words to exclude from search index
        self.stop_words = {
            # Common function words
            '的', '了', '在', '是', '我', '你', '他', '她',
            '它', '们', '有', '和', '或', '与', '就',
            '都', '而', '及', '又', '或', '被', '对',
            '以', '于', '上', '下', '中', '内', '外',
            '之', '为', '至', '向', '从', '到', '着',
            '过', '就', '都', '很', '也', '还', '又',
            '并', '且', '但', '而', '则', '即', '仍',
            '更', '才', '却', '已', '再', '此', '那',
            '些', '所', '其', '何', '几', '则', '每',
            '各', '所有', '这些', '那些', '这个', '那个',
            '我们', '你们', '他们', '她们', '它们',
            # Common punctuation as single chars
            '，', '。', '？', '！', '：', '；', '、',
            '（', '）', '【', '】', '『', '』', '「', '」',
            '《', '》', '——', '…', '—', '．',
        }

    def split(self, text, language='zh_CN'):
        """
        Split text into searchable terms using jieba segmentation.

        Args:
            text: Input text (can be mixed Chinese and English)
            language: Language code (currently only zh_CN is handled specially)

        Returns:
            List of terms to index
        """
        if not text:
            return []

        # For Chinese, use jieba segmentation
        if language in ('zh_CN', 'zh'):
            return self._split_chinese(text)
        else:
            # For other languages, use default behavior
            return self._split_default(text)

    def _split_chinese(self, text):
        """
        Split Chinese text using jieba, with English preservation.
        """
        terms = []

        # Pattern to split text into Chinese and non-Chinese parts
        pattern = r'([\u4e00-\u9fff]+)'

        parts = re.split(pattern, text)

        for part in parts:
            if not part:
                continue

            # Check if part is purely Chinese
            if re.fullmatch(r'[\u4e00-\u9fff]+', part):
                # Segment Chinese text using jieba
                words = jieba.lcut(part)
                for word in words:
                    word = word.strip()
                    if word and len(word) > 1 and word not in self.stop_words:
                        terms.append(word)
                        # Also add individual characters for partial matching
                        # This enables searching for single characters
                        if len(word) > 1:
                            terms.extend(list(word))
            else:
                # Process non-Chinese text (English, numbers, etc.)
                # Split on non-alphanumeric characters
                word_pattern = r'\b\w+\b'
                words = re.findall(word_pattern, part)
                for word in words:
                    word = word.strip().lower()
                    if word and len(word) > 1:
                        terms.append(word)

        return terms

    def _split_default(self, text):
        """
        Default word splitting for non-Chinese languages.
        """
        # Split on non-word characters, preserve alphanumeric
        words = re.findall(r'\b\w+\b', text.lower())
        return [w for w in words if len(w) > 2]


# Global instance
_jieba_splitter = None


def get_splitter():
    """Get or create the jieba splitter instance."""
    global _jieba_splitter
    if _jieba_splitter is None:
        _jieba_splitter = JiebaChineseTextSplitter()
    return _jieba_splitter


def patch_sphinx_search_for_chinese():
    """
    Patch Sphinx's search module to use jieba for Chinese text.

    This modifies the behavior at runtime so that Chinese text is properly
    segmented when building the search index.
    """
    try:
        import sphinx.search as search_module

        # Store original _default_word_splitter if it exists
        original_splitter = getattr(search_module, '_word_splitter', None)

        def jieba_word_splitter(text, language='zh_CN'):
            """Wrapper that uses jieba for Chinese text."""
            splitter = get_splitter()
            return splitter.split(text, language)

        # Patch the word splitter
        search_module._word_splitter = jieba_word_splitter

        # Also patch the language-specific splitters if they exist
        if hasattr(search_module, 'languages'):
            # The languages dict contains search language classes
            # We need to ensure they use our splitter
            for lang_code, lang_class in list(search_module.languages.items()):
                # Patch the split method if it exists
                if hasattr(lang_class, 'split'):
                    original_split_method = lang_class.split

                    def patched_split(self, input):
                        # Use jieba for Chinese, default for others
                        if self.lang_code in ('zh_CN', 'zh'):
                            return get_splitter().split(input, self.lang_code)
                        return original_split_method(self, input)

                    lang_class.split = patched_split

        return True
    except Exception as e:
        print(f"Warning: Could not patch Sphinx search: {e}")
        import traceback
        traceback.print_exc()
        return False


def setup(app):
    """
    Sphinx extension setup function.

    Initializes jieba and patches Sphinx's search functionality.
    """
    # Initialize jieba
    jieba.setLogLevel('INFO')

    # Try to load custom dictionary if available
    custom_dict_path = docs_dir / 'custom_dict.txt'
    if custom_dict_path.exists():
        jieba.load_userdict(str(custom_dict_path))

    # Patch Sphinx search for Chinese
    patch_sphinx_search_for_chinese()

    # Register our splitter for use in templates if needed
    app.jieba_splitter = get_splitter()

    return {
        'version': '1.0',
        'parallel_read_safe': True,
        'parallel_write_safe': True,
    }

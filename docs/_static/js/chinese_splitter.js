/**
 * Chinese Query Splitter for Sphinx Search
 *
 * This script overrides the default splitQuery function to properly
 * split Chinese search queries to match how jieba indexes content.
 *
 * Sphinx's jieba extension indexes Chinese text by splitting into
 * meaningful words (e.g., "文档搜索" -> ["文档", "搜索"]).
 *
 * Our splitter uses a conservative chunking strategy for Chinese queries.
 * This is necessary because current Sphinx search treats split terms as
 * required matches. Overlapping n-grams like ["预览", "览功", "功能"] make
 * normal phrases impossible to match unless every n-gram is indexed.
 */

(function() {
    'use strict';

    /**
     * Chinese query splitter that generates conservative Chinese chunks
     * to match against jieba-indexed content
     *
     * @param {string} query - The search query to split
     * @returns {string[]} Array of search terms
     */
    function chineseSplitQuery(query) {
        if (!query) {
            return [];
        }

        var terms = [];

        // Extract Latin words (English, numbers)
        var latinRegex = /[a-zA-Z0-9_]+/g;
        var latinMatches = query.match(latinRegex);
        if (latinMatches) {
            latinMatches.forEach(function(word) {
                terms.push(word);
                var lower = word.toLowerCase();
                if (word !== lower) {
                    terms.push(lower);
                }
            });
        }

        // Extract and process Chinese text
        var chineseText = query.replace(/[a-zA-Z0-9_]/g, ' ')
                               .replace(/[^\u4e00-\u9fff]+/g, ' ')
                               .trim();

        if (chineseText) {
            // Split on spaces first (user-indicated word boundaries)
            var words = chineseText.split(/\s+/).filter(function(w) { return w; });

            words.forEach(function(word) {
                var len = word.length;

                if (len <= 2) {
                    terms.push(word);
                    return;
                }

                // Chinese technical terms are very often 2-char compounds.
                // Use non-overlapping 2-char chunks so queries like
                // "预览功能" become ["预览", "功能"] instead of
                // ["预览", "览功", "功能"].
                if (len % 2 === 0) {
                    for (var i = 0; i < len; i += 2) {
                        terms.push(word.substring(i, i + 2));
                    }
                    return;
                }

                // For odd-length words, keep the full term as a fallback.
                // Project-specific terms should be added to docs/search_dict.txt
                // so the build index and query splitting can agree on the word.
                terms.push(word);
            });
        }

        // Remove duplicates and empty strings
        var uniqueTerms = [];
        var seen = {};
        for (var i = 0; i < terms.length; i++) {
            var term = terms[i];
            if (term && !seen[term]) {
                seen[term] = true;
                uniqueTerms.push(term);
            }
        }

        return uniqueTerms;
    }

    /**
     * Apply overrides to Sphinx search functionality
     */
    function applyOverrides() {
        // Store original if it exists
        if (typeof splitQuery !== 'undefined') {
            window._originalSplitQuery = splitQuery;
        }

        // Override global splitQuery
        window.splitQuery = chineseSplitQuery;

        // Override Search object
        if (typeof Search !== 'undefined') {
            Search.splitQuery = chineseSplitQuery;

            // Override _parseQuery to handle Chinese properly
            // Chinese words should NOT be stemmed
            Search._parseQuery = function(query) {
                var stemmer = new Stemmer();
                var normalizedQuery = query.trim();
                var searchTerms = new Set();
                var excludedTerms = new Set();
                var highlightTerms = new Set();
                var objectTerms = new Set(chineseSplitQuery(query.toLowerCase().trim()));

                chineseSplitQuery(normalizedQuery).forEach(function(queryTerm) {
                    var queryTermLower = queryTerm.toLowerCase();

                    // Skip stopwords and pure numbers
                    if (stopwords.indexOf(queryTermLower) !== -1 ||
                        queryTerm.match(/^\d+$/)) {
                        return;
                    }

                    // For Chinese text, don't stem - add as-is
                    // Stemming is for European languages only
                    var isChinese = /[\u4e00-\u9fff]/.test(queryTerm);

                    var word = isChinese ? queryTerm : stemmer.stemWord(queryTermLower);

                    if (word[0] === '-') {
                        excludedTerms.add(word.substr(1));
                        return;
                    }

                    searchTerms.add(word);
                    highlightTerms.add(isChinese ? queryTerm : queryTermLower);
                });

                if (typeof SPHINX_HIGHLIGHT_ENABLED !== 'undefined' && SPHINX_HIGHLIGHT_ENABLED) {
                    localStorage.setItem(
                        'sphinx_highlight_terms',
                        Array.from(highlightTerms).join(' ')
                    );
                }

                return [normalizedQuery, searchTerms, excludedTerms, highlightTerms, objectTerms];
            };
        }

        console.log('Chinese search splitter activated');
    }

    // Apply with multiple strategies to ensure searchtools.js is loaded
    applyOverrides();

    // Retry after DOMContentLoaded
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            setTimeout(applyOverrides, 100);
        });
    }

    // Retry periodically until Search is available
    var attempts = 0;
    var checkInterval = setInterval(function() {
        attempts++;
        if (typeof Search !== 'undefined' && typeof Search._parseQuery === 'function') {
            applyOverrides();
            clearInterval(checkInterval);
        } else if (attempts > 20) {
            clearInterval(checkInterval);
        }
    }, 100);

})();

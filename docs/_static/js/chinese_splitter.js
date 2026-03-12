/**
 * Chinese Query Splitter for Sphinx Search
 *
 * This script overrides the default splitQuery function to properly
 * split Chinese search queries to match how jieba indexes content.
 *
 * Sphinx's jieba extension indexes Chinese text by splitting into
 * meaningful words (e.g., "文档搜索" -> ["文档", "搜索"]).
 *
 * Our splitter generates multiple n-grams to maximize matching:
 * - Unigrams (single chars) - for partial matches
 * - Bigrams (2-char words) - most common Chinese word length
 * - Trigrams (3-char words) - for longer terms
 * - Full phrase - for exact matches
 */

(function() {
    'use strict';

    /**
     * Chinese query splitter that generates n-grams
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

                // Add the full word
                terms.push(word);

                // Generate all possible bigrams (most important for Chinese)
                for (var i = 0; i < len - 1; i++) {
                    terms.push(word.substring(i, i + 2));
                }

                // Generate trigrams for words of 3+ characters
                for (var i = 0; i < len - 2; i++) {
                    terms.push(word.substring(i, i + 3));
                }

                // Generate 4-grams for words of 4+ characters
                for (var i = 0; i < len - 3; i++) {
                    terms.push(word.substring(i, i + 4));
                }
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
                var searchTerms = new Set();
                var excludedTerms = new Set();
                var highlightTerms = new Set();
                var objectTerms = new Set(chineseSplitQuery(query));

                chineseSplitQuery(query).forEach(function(queryTerm) {
                    var queryTermLower = queryTerm.toLowerCase();

                    // Skip stopwords and pure numbers
                    if (stopwords.indexOf(queryTermLower) !== -1 ||
                        queryTerm.match(/^\d+$/)) {
                        return;
                    }

                    // For Chinese text, don't stem - add as-is
                    // Stemming is for European languages only
                    var isChinese = /[\u4e00-\u9fff]/.test(queryTerm);

                    if (isChinese) {
                        searchTerms.add(queryTerm);
                        highlightTerms.add(queryTerm);
                    } else {
                        // For Latin words, apply stemming
                        searchTerms.add(queryTermLower);
                        highlightTerms.add(queryTerm);
                    }
                });

                return {
                    searchTerms: searchTerms,
                    excludedTerms: excludedTerms,
                    highlightTerms: highlightTerms,
                    objectTerms: objectTerms
                };
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

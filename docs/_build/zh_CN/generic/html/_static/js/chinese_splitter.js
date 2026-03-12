/**
 * Chinese Query Splitter for Sphinx Search
 *
 * This script overrides the default splitQuery function to properly
 * split Chinese search queries into individual words.
 *
 * The default Sphinx splitQuery doesn't handle Chinese well - it keeps
 * Chinese text as one long string. This splitter uses a simple approach:
 * it splits on common Chinese word boundaries and generates n-grams
 * for better matching with the jieba-indexed content.
 */

(function() {
    'use strict';

    /**
     * Chinese query splitter that generates unigrams, bigrams and trigrams
     * This matches how Sphinx's jieba extension indexes Chinese text
     *
     * @param {string} query - The search query to split
     * @returns {string[]} Array of search terms
     */
    var chineseSplitQuery = function(query) {
        if (!query) {
            return [];
        }

        var terms = [];

        // First, extract any Latin alphabet words (English, numbers)
        var latinRegex = /[a-zA-Z0-9_]+/g;
        var latinMatches = query.match(latinRegex);
        if (latinMatches) {
            // Add Latin words in both original and lowercase forms
            latinMatches.forEach(function(word) {
                terms.push(word);
                if (word !== word.toLowerCase()) {
                    terms.push(word.toLowerCase());
                }
            });
        }

        // Remove Latin characters and split on non-Chinese characters
        var chineseText = query.replace(/[a-zA-Z0-9_]/g, ' ');
        chineseText = chineseText.replace(/[^\u4e00-\u9fff]+/g, ' ').trim();

        if (chineseText) {
            // Process Chinese text character by character and generate n-grams
            var chars = chineseText.split('');

            // Generate unigrams (single characters)
            for (var i = 0; i < chars.length; i++) {
                if (chars[i].trim()) {
                    terms.push(chars[i]);
                }
            }

            // Generate bigrams (2-character words)
            for (var i = 0; i < chars.length - 1; i++) {
                if (chars[i].trim() && chars[i + 1].trim()) {
                    terms.push(chars[i] + chars[i + 1]);
                }
            }

            // Generate trigrams (3-character words) for longer phrases
            for (var i = 0; i < chars.length - 2; i++) {
                if (chars[i].trim() && chars[i + 1].trim() && chars[i + 2].trim()) {
                    terms.push(chars[i] + chars[i + 1] + chars[i + 2]);
                }
            }
        }

        // Remove duplicates and empty strings
        var uniqueTerms = [];
        var seen = {};
        for (var i = 0; i < terms.length; i++) {
            if (terms[i] && !seen[terms[i]]) {
                seen[terms[i]] = true;
                uniqueTerms.push(terms[i]);
            }
        }

        return uniqueTerms;
    };

    // Wait for searchtools.js to load, then override splitQuery
    function overrideSplitQuery() {
        if (typeof splitQuery !== 'undefined') {
            // Store reference to original if needed
            window._originalSplitQuery = splitQuery;
        }

        // Override the global splitQuery
        window.splitQuery = chineseSplitQuery;

        // Also update Search object if it exists
        if (typeof Search !== 'undefined') {
            Search.splitQuery = chineseSplitQuery;
            // Also override _parseQuery if needed
            if (typeof Search._parseQuery === 'function') {
                var originalParseQuery = Search._parseQuery;
                Search._parseQuery = function(query) {
                    // Use our custom splitter
                    var objectTerms = new Set(chineseSplitQuery(query.toLowerCase().trim()));
                    var highlightTerms = new Set();
                    var searchTerms = new Set();
                    var excludedTerms = new Set();

                    chineseSplitQuery(query.trim()).forEach(function(queryTerm) {
                        var queryTermLower = queryTerm.toLowerCase();

                        // Skip stopwords
                        if (stopwords.indexOf(queryTermLower) !== -1 || queryTerm.match(/^\d+$/)) {
                            return;
                        }

                        searchTerms.add(queryTermLower);
                        highlightTerms.add(queryTerm);
                    });

                    return {
                        searchTerms: searchTerms,
                        excludedTerms: excludedTerms,
                        highlightTerms: highlightTerms,
                        objectTerms: objectTerms
                    };
                };
            }
        }

        console.log('Chinese query splitter initialized');
    }

    // Try to override immediately, and also after DOMContentLoaded
    overrideSplitQuery();
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', overrideSplitQuery);
    }

    // Also try after a short delay to ensure searchtools.js is fully loaded
    setTimeout(overrideSplitQuery, 100);
})();

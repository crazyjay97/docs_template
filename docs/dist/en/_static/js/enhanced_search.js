/**
 * Enhanced Search Functionality
 *
 * Features:
 * - Keyboard shortcut (Ctrl/Cmd + K) to focus search
 * - Search result preview
 * - Better result highlighting
 */

(function() {
    'use strict';

    // Initialize when DOM is ready
    document.addEventListener('DOMContentLoaded', function() {
        initSearchEnhancements();
    });

    function initSearchEnhancements() {
        // Add keyboard shortcut (Ctrl/Cmd + K)
        document.addEventListener('keydown', function(e) {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                focusSearchInput();
            }
        });

        // Add placeholder text with hint
        var searchInputs = document.querySelectorAll('input[name="q"]');
        searchInputs.forEach(function(input) {
            if (!input.placeholder) {
                input.placeholder = 'Search docs (Ctrl+K)';
            }
            input.setAttribute('aria-label', 'Search documentation');
        });

        // Improve search results page
        enhanceSearchResults();
    }

    function focusSearchInput() {
        var searchInput = document.querySelector('input[name="q"]');
        if (searchInput) {
            searchInput.focus();
            searchInput.select();
        }
    }

    function enhanceSearchResults() {
        // Wait for search to load
        setTimeout(function() {
            var searchResults = document.getElementById('search-results');
            if (!searchResults) return;

            // Add result count
            var observer = new MutationObserver(function(mutations) {
                mutations.forEach(function(mutation) {
                    if (mutation.addedNodes.length) {
                        var results = searchResults.querySelectorAll('li');
                        if (results.length > 0) {
                            var count = document.getElementById('search-result-count');
                            if (!count) {
                                count = document.createElement('p');
                                count.id = 'search-result-count';
                                count.style.color = '#666';
                                count.style.marginBottom = '1rem';
                                searchResults.insertBefore(count, searchResults.firstChild);
                            }
                            count.textContent = results.length + ' result(s) found';
                        }
                    }
                });
            });

            observer.observe(searchResults, { childList: true });
        }, 500);
    }

    // Expose function globally
    window.SearchEnhancements = {
        focus: focusSearchInput
    };
})();

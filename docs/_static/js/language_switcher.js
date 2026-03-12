/**
 * Language Switcher
 *
 * Provides language switching functionality with correct path handling
 * Works with nginx subdirectory deployments
 */

(function() {
    'use strict';

    // Available languages and their paths
    var LANGUAGES = {
        'en': 'English',
        'zh_CN': '简体中文'
    };

    /**
     * Get current language from URL path
     * e.g., /template/zh_CN/index.html -> zh_CN
     */
    function getCurrentLang() {
        var path = window.location.pathname;
        var parts = path.split('/');
        for (var i = 0; i < parts.length; i++) {
            if (LANGUAGES.hasOwnProperty(parts[i])) {
                return parts[i];
            }
        }
        return 'en'; // default
    }

    /**
     * Get current page filename
     * e.g., /template/zh_CN/index.html -> index.html
     */
    function getCurrentPage() {
        var path = window.location.pathname;
        var parts = path.split('/');
        return parts[parts.length - 1] || 'index.html';
    }

    /**
     * Switch to another language
     */
    function switchLanguage(lang) {
        var currentPage = getCurrentPage();
        var path = window.location.pathname;

        // Find the base path (everything before the language code)
        var langIndex = path.indexOf('/' + lang + '/');
        var basePath = '';

        if (langIndex > 0) {
            basePath = path.substring(0, langIndex);
        }

        // Build new URL
        var newUrl = basePath + '/' + lang + '/' + currentPage;

        // Navigate to new URL
        window.location.href = newUrl;
    }

    /**
     * Create language selector dropdown
     */
    function createLanguageSelector() {
        var currentLang = getCurrentLang();

        // Create container
        var container = document.createElement('div');
        container.className = 'language-selector';
        container.style.cssText = 'position:fixed;top:15px;right:15px;z-index:9999;background:#fff;border:1px solid #ddd;padding:8px 12px;border-radius:4px;box-shadow:0 2px 8px rgba(0,0,0,0.15);font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif;';

        // Create label
        var label = document.createElement('span');
        label.textContent = 'Language: ';
        label.style.cssText = 'margin-right:8px;font-size:13px;color:#555;';
        container.appendChild(label);

        // Create links for other languages
        var first = true;
        for (var lang in LANGUAGES) {
            if (lang === currentLang) continue;

            if (!first) {
                var separator = document.createElement('span');
                separator.textContent = '|';
                separator.style.cssText = 'margin:0 8px;color:#ccc;';
                container.appendChild(separator);
            }

            var link = document.createElement('a');
            link.href = '#';
            link.textContent = LANGUAGES[lang];
            link.style.cssText = 'color:#428bca;text-decoration:none;font-size:13px;font-weight:500;transition:color 0.2s;';
            link.onmouseover = function(el) { return function() { el.style.color = '#3071a9'; }; }(link);
            link.onmouseout = function(el) { return function() { el.style.color = '#428bca'; }; }(link);
            link.onclick = function(l) {
                return function(e) {
                    e.preventDefault();
                    switchLanguage(l);
                };
            }(lang);

            container.appendChild(link);
            first = false;
        }

        return container;
    }

    /**
     * Initialize language switcher
     */
    function init() {
        // Add language switcher to the page
        var switcher = createLanguageSelector();
        document.body.appendChild(switcher);

        // Also update any :link_to_translation: links (with data-target-lang attribute)
        var translationLinks = document.querySelectorAll('a.lang-switch-link');
        translationLinks.forEach(function(link) {
            var targetLang = link.getAttribute('data-target-lang');
            if (targetLang) {
                link.onclick = function(e) {
                    e.preventDefault();
                    switchLanguage(targetLang);
                };
            }
        });
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose globally
    window.LanguageSwitcher = {
        switch: switchLanguage,
        getCurrentLang: getCurrentLang
    };
})();

/**
 * Version Table Widget
 *
 * This script handles version selection and display in the documentation.
 */

(function() {
    'use strict';

    // Version configuration
    const VERSION_CONFIG = {
        defaultVersion: 'latest',
        storageKey: 'docs_version_preference',
        versionsUrl: null  // Set this to your versions JSON endpoint
    };

    /**
     * Load versions from the configured URL
     * @returns {Promise<Array>} List of available versions
     */
    async function loadVersions() {
        if (!VERSION_CONFIG.versionsUrl) {
            console.warn('versionsUrl not configured');
            return [];
        }

        try {
            const response = await fetch(VERSION_CONFIG.versionsUrl);
            if (!response.ok) {
                throw new Error('Failed to load versions');
            }
            return await response.json();
        } catch (error) {
            console.error('Error loading versions:', error);
            return [];
        }
    }

    /**
     * Get the current version from the URL or storage
     * @returns {string} Current version
     */
    function getCurrentVersion() {
        // Try to get from URL first
        const urlMatch = window.location.pathname.match(/\/([^/]+)\//);
        if (urlMatch && urlMatch[1]) {
            return urlMatch[1];
        }

        // Fall back to stored preference
        return localStorage.getItem(VERSION_CONFIG.storageKey) || VERSION_CONFIG.defaultVersion;
    }

    /**
     * Save version preference
     * @param {string} version - Version to save
     */
    function saveVersionPreference(version) {
        localStorage.setItem(VERSION_CONFIG.storageKey, version);
    }

    /**
     * Initialize version selector
     */
    function initVersionSelector() {
        const selector = document.querySelector('#version-selector');
        if (!selector) {
            return;
        }

        loadVersions().then(versions => {
            if (versions.length === 0) {
                return;
            }

            const currentVersion = getCurrentVersion();

            versions.forEach(version => {
                const option = document.createElement('option');
                option.value = version.name || version;
                option.textContent = version.display_name || version;

                if (option.value === currentVersion) {
                    option.selected = true;
                }

                selector.appendChild(option);
            });

            selector.addEventListener('change', (e) => {
                saveVersionPreference(e.target.value);
                // Navigate to the selected version
                const newUrl = window.location.pathname.replace(
                    /\/[^/]+\//,
                    `/${e.target.value}/`
                );
                window.location.href = newUrl;
            });
        });
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initVersionSelector);
    } else {
        initVersionSelector();
    }
})();

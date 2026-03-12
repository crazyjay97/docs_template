/**
 * Language Switcher
 *
 * Provides language switching functionality with correct path handling
 * Works with nginx subdirectory deployments
 */

(function () {
  "use strict";

  var LANGUAGES = {
    en: "English",
    zh_CN: "简体中文",
  };

  function getCurrentLang() {
    var path = window.location.pathname;
    var parts = path.split("/");
    for (var i = 0; i < parts.length; i++) {
      if (LANGUAGES.hasOwnProperty(parts[i])) {
        return parts[i];
      }
    }
    return "en";
  }

  function getCurrentPage() {
    var path = window.location.pathname;
    var parts = path.split("/");
    return parts[parts.length - 1] || "index.html";
  }

  function getBasePath() {
    var path = window.location.pathname;
    var currentLang = getCurrentLang();
    var langIndex = path.indexOf("/" + currentLang + "/");
    if (langIndex > 0) {
      return path.substring(0, langIndex);
    }
    return "";
  }

  function switchLanguage(targetLang) {
    var parts = window.location.pathname.split("/");

    for (var i = 0; i < parts.length; i++) {
      if (LANGUAGES.hasOwnProperty(parts[i])) {
        parts[i] = targetLang;
        break;
      }
    }

    window.location.href = parts.join("/");
  }

  function createLanguageSelector() {
    var currentLang = getCurrentLang();
    var container = document.createElement("div");
    container.className = "language-selector";
    container.style.cssText =
      'position:fixed;top:15px;right:15px;z-index:9999;background:#fff;border:1px solid #ddd;padding:8px 12px;border-radius:4px;box-shadow:0 2px 8px rgba(0,0,0,0.15);font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif;';

    var label = document.createElement("span");
    label.textContent = "Language: ";
    label.style.cssText = "margin-right:8px;font-size:13px;color:#555;";
    container.appendChild(label);

    var first = true;
    for (var lang in LANGUAGES) {
      if (lang === currentLang) continue;

      if (!first) {
        var separator = document.createElement("span");
        separator.textContent = "|";
        separator.style.cssText = "margin:0 8px;color:#ccc;";
        container.appendChild(separator);
      }

      var link = document.createElement("a");
      link.href = "javascript:void(0)";
      link.textContent = LANGUAGES[lang];
      link.style.cssText =
        "color:#428bca;text-decoration:none;font-size:13px;font-weight:500;";
      link.onclick = (function (l) {
        return function (e) {
          e.preventDefault();
          switchLanguage(l);
        };
      })(lang);

      container.appendChild(link);
      first = false;
    }

    return container;
  }

  function init() {
    // Add floating language selector
    var switcher = createLanguageSelector();
    document.body.appendChild(switcher);

    // Fix all translation links - intercept clicks on any link containing language codes
    function fixTranslationLinks() {
      var links = document.querySelectorAll(
        'a[href*="../en/"], a[href*="../zh_CN/"], a.lang-switch-link',
      );
      links.forEach(function (link) {
        // Extract target language from href or data attribute
        var href = link.getAttribute("href") || "";
        var targetLang = null;

        if (link.getAttribute("data-target-lang")) {
          targetLang = link.getAttribute("data-target-lang");
        } else if (
          href.indexOf("/zh_CN/") >= 0 ||
          href.indexOf("../zh_CN") >= 0
        ) {
          targetLang = "zh_CN";
        } else if (href.indexOf("/en/") >= 0 || href.indexOf("../en") >= 0) {
          targetLang = "en";
        }

        if (targetLang) {
          link.onclick = function (e) {
            e.preventDefault();
            switchLanguage(targetLang);
            return false;
          };
        }
      });
    }

    fixTranslationLinks();

    // Also fix links that may be loaded dynamically
    setTimeout(fixTranslationLinks, 500);
    setTimeout(fixTranslationLinks, 1000);
  }

  // Initialize
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  window.LanguageSwitcher = {
    switch: switchLanguage,
    getCurrentLang: getCurrentLang,
    getBasePath: getBasePath,
  };
})();

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
    const parts = window.location.pathname.split("/");
    console.log("targetLang:", targetLang);
    for (let i = 0; i < parts.length; i++) {
      if (LANGUAGES.hasOwnProperty(parts[i])) {
        parts[i] = targetLang;
        break;
      }
    }
    console.log("targetHref:", parts.join("/") + window.location.hash);
    window.location.href = parts.join("/") + window.location.hash;
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
    // 全局拦截语言切换链接
    document.addEventListener("click", function (e) {
      const link = e.target.closest(".lang-switch-link");
      if (!link) return;

      e.preventDefault();
      const lang = link.dataset.targetLang;
      switchLanguage(lang);
    });
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

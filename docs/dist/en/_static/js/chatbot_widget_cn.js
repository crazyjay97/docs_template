/**
 * Chatbot Widget - Chinese Version
 *
 * This script loads the Kapa AI chatbot widget for Chinese documentation.
 * Configure the widget settings by modifying the data attributes.
 */

document.addEventListener("DOMContentLoaded", function () {
    var script = document.createElement("script");
    script.src = "https://widget.kapa.ai/kapa-widget.bundle.js";

    // Bot protection mechanism
    script.setAttribute("data-bot-protection-mechanism", "hcaptcha");

    // Website ID (replace with your actual ID)
    script.setAttribute("data-website-id", "your-website-id-here");

    // Project branding
    script.setAttribute("data-modal-title", "项目文档 AI 助手");
    script.setAttribute("data-project-name", "您的项目文档");
    script.setAttribute("data-project-color", "#C62817");
    script.setAttribute("data-project-logo", "https://your-domain.com/logo.png");

    // Button appearance
    script.setAttribute("data-button-image", "https://your-domain.com/chatbot-icon.png");
    script.setAttribute("data-button-text-font-size", "0px");
    script.setAttribute("data-button-border-radius", "50%");
    script.setAttribute("data-button-bg-color", "#38393a");
    script.setAttribute("data-button-border", "#38393a");
    script.setAttribute("data-button-height", "45px");
    script.setAttribute("data-button-width", "45px");
    script.setAttribute("data-button-animation-enabled", "false");
    script.setAttribute("data-button-image-height", "100%");
    script.setAttribute("data-button-image-width", "100%");
    script.setAttribute("data-button-padding", "0");
    script.setAttribute("data-button-hover-animation-enabled", "false");

    // Button position
    script.setAttribute("data-button-position-top", "50px");
    script.setAttribute("data-button-position-left", "305px");
    script.setAttribute("data-button-box-shadow", "0px 6px 12px 1px rgba(0,0,0,0.16)");

    // Modal settings
    script.setAttribute("data-modal-override-open-class", "test-ai");
    script.setAttribute("data-user-analytics-fingerprint-enabled", "true");

    // Language setting
    script.setAttribute("data-language", "zh");

    // Example questions title
    script.setAttribute("data-modal-example-questions-title", "问题示例");

    // Disclaimer message
    script.setAttribute("data-modal-disclaimer",
        "欢迎使用文档智能问答助手！本助手基于项目文档，旨在为您提供技术支持和解答。" +
        "如有任何意见或建议，欢迎留下反馈！\n\n" +
        "**注意**：本回答由 AI 生成，可能存在不准确之处，请核实重要信息。");

    // Example questions
    script.setAttribute("data-modal-example-questions",
        "这个项目是什么？，我该如何开始？，API 文档在哪里？");

    script.async = true;
    document.head.appendChild(script);
});

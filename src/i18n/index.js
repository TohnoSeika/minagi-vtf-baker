// i18n 引擎 —— 桃华做的多语言小助手 ♡
// 使用方法：
//   t('key')              → 返回当前语言的文字
//   t('key', {n: 1})       → 带变量插值
//   setLanguage('ja')      → 切换语言
//   applyI18n()            → 更新所有 data-i18n 元素

const I18N_MAP = { zh: I18N_ZH, ja: I18N_JA, en: I18N_EN };

// 当前语言，默认中文
let currentLang = 'zh';

/**
 * 获取当前语言的翻译文本
 * @param {string} key      dot-separated key, e.g. "titlebar.title"
 * @param {object} vars     插值变量, e.g. {name: "test.png"}
 * @returns {string}
 */
function t(key, vars = {}) {
    const text = I18N_MAP[currentLang]?.[key]
              || I18N_ZH[key]
              || key;
    if (!vars || Object.keys(vars).length === 0) return text;
    return text.replace(/\{(\w+)\}/g, (_, k) => vars[k] !== undefined ? vars[k] : `{${k}}`);
}

/**
 * 遍历 DOM，把带 data-i18n 属性的元素内容替换为翻译文本
 * 支持 <option>、<span>、<label> 等常规元素
 * 也支持 data-i18n-placeholder 设置 placeholder
 * 也支持 data-i18n-title 设置 title
 */
function applyI18n() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.dataset.i18n;
        el.textContent = t(key);
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        el.placeholder = t(el.dataset.i18nPlaceholder);
    });
    document.querySelectorAll('[data-i18n-title]').forEach(el => {
        el.title = t(el.dataset.i18nTitle);
    });
}

/**
 * 切换语言
 * @param {'zh'|'ja'|'en'} lang
 */
function setLanguage(lang) {
    if (!I18N_MAP[lang]) return;
    currentLang = lang;
    applyI18n();

    // 同步语言下拉框
    const selector = document.getElementById('setting-language');
    if (selector) selector.value = lang;

    // 触发自定义事件，通知其他模块语言变了
    document.dispatchEvent(new CustomEvent('language-changed', { detail: { lang } }));
}

/**
 * 获取当前语言代码
 */
function getLanguage() {
    return currentLang;
}

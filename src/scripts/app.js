// 这个 JS 是桃华写给 Minagi 的哦 ♡
// —— 把每一张图变成独一无二的喷漆

const TAURI = window.__TAURI__;

// ─── DOM 引用 ──────────────────────────────

const $ = (s) => document.querySelector(s);
const $$ = (s) => document.querySelectorAll(s);

const el = {
    dropZone: $('#drop-zone'),
    fileInput: $('#file-input'),
    previewArea: $('#preview-area'),
    previewImage: $('#preview-image'),
    infoFilename: $('#info-filename'),
    infoDimensions: $('#info-dimensions'),
    infoAlpha: $('#info-alpha'),
    btnRemove: $('#btn-remove'),
    btnConvert: $('#btn-convert'),
    btnMinimize: $('#btn-minimize'),
    btnClose: $('#btn-close'),
    progressBar: $('#progress-bar'),
    progressFill: $('#progress-fill'),
    progressText: $('#progress-text'),
    toast: $('#toast'),
    // params
    paramFormat: $('#param-format'),
    paramSize: $('#param-size'),
    paramSampling: $('#param-sampling'),
    paramSharpen: $('#param-sharpen'),
    paramAlpha: $('#param-alpha'),
    paramBgColor: $('#param-bgcolor'),
    paramBgColorText: $('#param-bgcolor-text'),
    // settings
    settingThemeColor: $('#setting-theme-color'),
    settingThemeColorText: $('#setting-theme-color-text'),
    settingThemeReset: $('#setting-theme-reset'),
    settingBgSelect: $('#setting-bg-select'),
    settingBgInput: $('#setting-bg-input'),
    settingBgClear: $('#setting-bg-clear'),
    settingBgDefault: $('#setting-bg-default'),
    settingOpacityReset: $('#setting-opacity-reset'),
    settingWindowOpacity: $('#setting-window-opacity'),
    settingWindowOpacityVal: $('#setting-window-opacity-val'),
    settingBgOpacity: $('#setting-bg-opacity'),
    settingBgOpacityVal: $('#setting-bg-opacity-val'),
    // 动态喷漆
    animFrameList: $('#anim-frame-list'),
    animEmptyHint: $('#anim-empty-hint'),
    animCount: $('#anim-count'),
    animAddBtn: $('#anim-add-btn'),
    animFileInput: $('#anim-file-input'),
    btnConvertAnim: $('#btn-convert-anim'),
    // 远近喷漆
    distFrameList: $('#dist-frame-list'),
    distCount: $('#dist-count'),
    distAddBtn: $('#dist-add-btn'),
    distFileInput: $('#dist-file-input'),
    btnConvertDist: $('#btn-convert-dist'),
};

// ─── 状态 ──────────────────────────────────

let state = {
    currentPath: null,
    currentInfo: null,
    isConverting: false,
    config: {
        opacity: 0.95,
        bgOpacity: 0.25,
    },
};

// ─── 设置持久化 ──────────────────────────

// 当前已存储的背景图数据（undefined=未设 null=已清除 string=dataURL）
let persistedBgImage = undefined;

async function loadSavedSettings() {
    try {
        const d = await TAURI.core.invoke('load_settings');
        const hasData = d.themeColor !== null || d.windowOpacity !== null || d.bgOpacity !== null || d.bgImage !== undefined || d.language !== null;
        return {
            themeColor: d.themeColor || null,
            windowOpacity: typeof d.windowOpacity === 'number' ? d.windowOpacity : null,
            bgOpacity: typeof d.bgOpacity === 'number' ? d.bgOpacity : null,
            bgImage: d.bgImage !== undefined ? d.bgImage : undefined,
            language: d.language && ['zh', 'ja', 'en'].includes(d.language) ? d.language : null,
            _hasSaved: hasData,
        };
    } catch (e) {
        console.warn('设置加载失败……', e);
        return { themeColor: null, windowOpacity: null, bgOpacity: null, bgImage: undefined, language: null, _hasSaved: false };
    }
}

function saveSettings() {
    TAURI.core.invoke('save_settings', {
        settings: {
            themeColor: el.settingThemeColor.value,
            windowOpacity: state.config.opacity,
            bgOpacity: state.config.bgOpacity,
            bgImage: persistedBgImage,
            language: getLanguage(),
        },
    }).catch(e => {
        console.warn('设置保存失败……', e);
        showToast(t('toast.saveFailed'), 'error', 2000);
    });
}

// ─── 动态喷漆状态 ──────────────────────────

let animState = {
    frames: [],
    sourceDir: null,
};

const MAX_ANIM_FRAMES = 11;

// ─── 远近喷漆状态 ──────────────────────────

let distState = {
    frames: [],
    sourceDir: null,
};

const MAX_DIST_FRAMES = 5;

// ─── 窗口控制 ──────────────────────────────

el.btnMinimize.addEventListener('click', async () => {
    const win = TAURI.window.getCurrentWindow();
    await win.minimize();
});

el.btnClose.addEventListener('click', async () => {
    const win = TAURI.window.getCurrentWindow();
    await win.close();
});

// ─── 选项卡切换 ──────────────────────────

document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        btn.classList.add('active');
        const tab = document.getElementById(`tab-${btn.dataset.tab}`);
        if (tab) tab.classList.add('active');

        // 更新拖拽提示文字 & 设置页展示图
        const dt = document.querySelector('.drop-text');
        const dh = document.querySelector('.drop-hint');
        const df = document.querySelector('.drop-formats');
        const settingsLeft = document.getElementById('settings-left');
        const dropZone = document.getElementById('drop-zone');
        const previewArea = document.getElementById('preview-area');
        if (btn.dataset.tab === 'settings') {
            // 设置页：隐藏拖曳和预览，显示展示图
            dropZone.classList.add('hidden');
            previewArea.classList.add('hidden');
            settingsLeft.classList.remove('hidden');
            updateSettingsBgImage();
        } else {
            // 离开设置页：隐藏展示图，恢复拖曳/预览状态
            settingsLeft.classList.add('hidden');
            if (state.currentInfo) {
                dropZone.classList.add('hidden');
                previewArea.classList.remove('hidden');
            } else {
                dropZone.classList.remove('hidden');
                previewArea.classList.add('hidden');
            }
        }
        // 更新提示文字
        if (btn.dataset.tab === 'settings') {
            dt.textContent = t('dropzone.settingsText');
            dh.textContent = t('dropzone.settingsHint');
            df.textContent = t('dropzone.settingsFormats');
        } else if (btn.dataset.tab === 'animated') {
            dt.textContent = t('dropzone.animText');
            dh.textContent = t('dropzone.animHint');
            df.textContent = t('dropzone.animFormats');
        } else if (btn.dataset.tab === 'distant') {
            dt.textContent = t('dropzone.distText');
            dh.textContent = t('dropzone.distHint');
            df.textContent = t('dropzone.distFormats');
        } else {
            dt.textContent = t('dropzone.text');
            dh.textContent = t('dropzone.hint');
            df.textContent = t('dropzone.formats');
        }
    });
});

// ─── Toast 通知 ────────────────────────────

let toastTimer = null;

function showToast(msg, type = 'info', duration = 3000) {
    if (toastTimer) clearTimeout(toastTimer);
    el.toast.textContent = msg;
    el.toast.className = 'hidden';
    // force reflow
    void el.toast.offsetWidth;
    el.toast.className = type;
    el.toast.classList.add('show');
    toastTimer = setTimeout(() => {
        el.toast.classList.remove('show');
    }, duration);
}

// ─── 文件选择 & 拖拽上传 ──────────────────

// 用 FileReader 读取文件数据，绕过 Tauri 的 path 限制
function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
            const data = reader.result.split(',')[1];
            resolve(data);
        };
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(file);
    });
}

async function loadImageFromFile(file) {
    try {
        const b64 = await readFileAsBase64(file);
        const info = await TAURI.core.invoke('load_image_data', {
            base64: b64,
            name: file.name,
        });
        state.currentPath = file.name;
        state.currentInfo = info;

        el.previewImage.src = info.preview;
        el.infoFilename.textContent = info.file_name + '  ';
        el.infoDimensions.textContent = `${info.width} × ${info.height}`;
        el.infoAlpha.textContent = info.has_alpha ? t('preview.alphaYes') : t('preview.alphaNo');
        el.infoAlpha.className = 'alpha-badge' + (info.has_alpha ? '' : ' no-alpha');

        el.dropZone.classList.add('hidden');
        el.previewArea.classList.remove('hidden');
        el.btnConvert.disabled = false;

        showToast(t('toast.loaded', { name: info.file_name }), 'info', 2000);
    } catch (err) {
        showToast(t('toast.loadFailed', { err }), 'error');
        console.error(err);
    }
}

el.dropZone.addEventListener('click', () => {
    const animTab = document.getElementById('tab-animated');
    const distTab = document.getElementById('tab-distant');
    if (animTab && animTab.classList.contains('active')) {
        el.animFileInput.click();
    } else if (distTab && distTab.classList.contains('active')) {
        el.distFileInput.click();
    } else {
        el.fileInput.click();
    }
});

el.fileInput.addEventListener('change', () => {
    if (el.fileInput.files.length > 0) {
        loadImageFromFile(el.fileInput.files[0]);
    }
    el.fileInput.value = '';
});

// ─── 全局拖拽上传 ────────────────────────

// HTML5 拖拽视觉反馈
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('dragenter', () => {
    el.dropZone.classList.add('drag-over');
});
document.addEventListener('dragleave', (e) => {
    if (!e.relatedTarget || e.relatedTarget === document.body) {
        el.dropZone.classList.remove('drag-over');
    }
});

// Tauri v2 原生文件拖放事件
TAURI.event.listen('tauri://drag-drop', (event) => {
    el.dropZone.classList.remove('drag-over');
    const paths = event.payload?.paths;
    if (paths && paths.length > 0) {
        const animTab = document.getElementById('tab-animated');
        const distTab = document.getElementById('tab-distant');
        if (animTab && animTab.classList.contains('active')) {
            for (const path of paths) {
                if (animState.frames.length >= MAX_ANIM_FRAMES) break;
                TAURI.core.invoke('read_file_base64', { path })
                    .then((dataUrl) => {
                        const b64 = dataUrl.split(',')[1] || dataUrl;
                        const name = path.split('\\').pop() || path.split('/').pop() || 'frame';
                        const dir = path.substring(0, path.lastIndexOf('\\'));
                        if (!animState.sourceDir) animState.sourceDir = dir;
                        animState.frames.push({ b64, name, path });
                        animRender();
                    })
                    .catch(() => {});
            }
        } else if (distTab && distTab.classList.contains('active')) {
            for (const path of paths) {
                if (distState.frames.length >= MAX_DIST_FRAMES) break;
                TAURI.core.invoke('read_file_base64', { path })
                    .then((dataUrl) => {
                        const b64 = dataUrl.split(',')[1] || dataUrl;
                        const name = path.split('\\').pop() || path.split('/').pop() || 'dist';
                        const dir = path.substring(0, path.lastIndexOf('\\'));
                        if (!distState.sourceDir) distState.sourceDir = dir;
                        distState.frames.push({ b64, name, path });
                        distRender();
                    })
                    .catch(() => {});
            }
        } else {
            const path = paths[0];
            TAURI.core.invoke('load_image', { path })
                .then((info) => {
                    state.currentPath = path;
                    state.currentInfo = info;
                    el.previewImage.src = info.preview;
                    el.infoFilename.textContent = info.file_name + '  ';
                    el.infoDimensions.textContent = `${info.width} × ${info.height}`;
                    el.infoAlpha.textContent = info.has_alpha ? t('preview.alphaYes') : t('preview.alphaNo');
                    el.infoAlpha.className = 'alpha-badge' + (info.has_alpha ? '' : ' no-alpha');
                    el.dropZone.classList.add('hidden');
                    el.previewArea.classList.remove('hidden');
                    el.btnConvert.disabled = false;
                    showToast(t('toast.loaded', { name: info.file_name }), 'info', 2000);
                })
                .catch((err) => showToast(t('toast.loadFailed', { err }), 'error'));
        }
    }
});

// ─── 重新选择 ──────────────────────────────

el.btnRemove.addEventListener('click', () => {
    resetUI();
});

// ─── 加载图片 ──────────────────────────────

async function loadImage(path) {
    try {
        const info = await TAURI.core.invoke('load_image', { path });
        state.currentPath = path;
        state.currentInfo = info;

        // 显示预览
        el.previewImage.src = info.preview;
        el.infoFilename.textContent = info.file_name + '  ';
        el.infoDimensions.textContent = `${info.width} × ${info.height}`;
        el.infoAlpha.textContent = info.has_alpha ? t('preview.alphaYes') : t('preview.alphaNo');
        el.infoAlpha.className = 'alpha-badge' + (info.has_alpha ? '' : ' no-alpha');

        el.dropZone.classList.add('hidden');
        el.previewArea.classList.remove('hidden');
        el.btnConvert.disabled = false;

        showToast(t('toast.loaded', { name: info.file_name }), 'info', 2000);
    } catch (err) {
        showToast(t('toast.loadFailed', { err }), 'error');
        console.error(err);
    }
}

// ─── 重置 UI ──────────────────────────────

function resetUI() {
    state.currentPath = null;
    state.currentInfo = null;
    el.previewArea.classList.add('hidden');
    el.btnConvert.disabled = true;
    el.btnConvert.className = 'primary-btn';
    el.btnConvert.querySelector('.btn-text').textContent = t('btn.convert');
    hideProgress();
    // 如果在设置页则不显示拖曳区
    const settingsTab = document.getElementById('tab-settings');
    if (settingsTab && settingsTab.classList.contains('active')) {
        el.dropZone.classList.add('hidden');
    } else {
        el.dropZone.classList.remove('hidden');
    }
}

// ─── 颜色选择器联动 ───────────────────────

el.paramBgColor.addEventListener('input', () => {
    el.paramBgColorText.value = el.paramBgColor.value;
});

el.paramBgColorText.addEventListener('input', () => {
    const val = el.paramBgColorText.value.trim();
    if (/^#[0-9a-fA-F]{6}$/.test(val)) {
        el.paramBgColor.value = val;
    }
});

el.paramBgColorText.addEventListener('blur', () => {
    const val = el.paramBgColorText.value.trim();
    if (!/^#[0-9a-fA-F]{6}$/.test(val)) {
        el.paramBgColorText.value = el.paramBgColor.value;
    }
});

// ─── 格式联动：DXT1 → 自动切透明处理 ─────────

el.paramFormat.addEventListener('change', () => {
    if (el.paramFormat.value === 'dxt1') {
        el.paramAlpha.value = 'remove';
        el.paramAlpha.disabled = true;
    } else {
        el.paramAlpha.disabled = false;
    }
});

// 初始化：默认 DXT1 时锁定
if (el.paramFormat.value === 'dxt1') {
    el.paramAlpha.disabled = true;
}

// ─── 设置：主题配色 ─────────────────────

const DEFAULT_THEME = '#e8a0b4';

function applyTheme(hex) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    const hover = `rgb(${Math.max(0, r - 25)}, ${Math.max(0, g - 25)}, ${Math.max(0, b - 25)})`;
    const light = `rgba(${r}, ${g}, ${b}, 0.15)`;
    const subtle = `rgba(${r}, ${g}, ${b}, 0.08)`;

    const root = document.documentElement;
    root.style.setProperty('--color-primary', hex);
    root.style.setProperty('--color-primary-hover', hover);
    root.style.setProperty('--color-primary-light', light);
    root.style.setProperty('--color-primary-subtle', subtle);

    document.getElementById('titlebar').style.backgroundColor = hex;
    document.getElementById('progress-fill').style.background =
        `linear-gradient(90deg, ${hex}, ${hover})`;
}

el.settingThemeColor.addEventListener('input', () => {
    el.settingThemeColorText.value = el.settingThemeColor.value;
    applyTheme(el.settingThemeColor.value);
    saveSettings();
});

el.settingThemeColorText.addEventListener('input', () => {
    const val = el.settingThemeColorText.value.trim();
    if (/^#[0-9a-fA-F]{6}$/.test(val)) {
        el.settingThemeColor.value = val;
        applyTheme(val);
        saveSettings();
    }
});

el.settingThemeColorText.addEventListener('blur', () => {
    const val = el.settingThemeColorText.value.trim();
    if (!/^#[0-9a-fA-F]{6}$/.test(val)) {
        el.settingThemeColorText.value = el.settingThemeColor.value;
    }
});

el.settingThemeReset.addEventListener('click', () => {
    el.settingThemeColor.value = DEFAULT_THEME;
    el.settingThemeColorText.value = DEFAULT_THEME;
    applyTheme(DEFAULT_THEME);
    saveSettings();
});

// ─── 设置：背景图片 ─────────────────────

el.settingBgSelect.addEventListener('click', () => {
    el.settingBgInput.click();
});

let bgObjectUrl = null;

el.settingBgInput.addEventListener('change', async () => {
    if (el.settingBgInput.files.length > 0) {
        if (bgObjectUrl) URL.revokeObjectURL(bgObjectUrl);

        const file = el.settingBgInput.files[0];

        // 读取完整 dataURL 以便持久化
        try {
            persistedBgImage = await new Promise((resolve, reject) => {
                const r = new FileReader();
                r.onload = () => resolve(r.result);
                r.onerror = reject;
                r.readAsDataURL(file);
            });
        } catch (_) {
            persistedBgImage = undefined;
        }

        bgObjectUrl = URL.createObjectURL(file);
        const overlay = document.getElementById('app-bg-overlay');
        overlay.style.backgroundImage = `url(${bgObjectUrl})`;
        overlay.classList.add('has-bg');
        document.body.classList.add('has-bg');
        saveSettings();
        showToast(t('toast.bgSet'), 'info', 2000);
    }
    el.settingBgInput.value = '';
});

el.settingBgClear.addEventListener('click', () => {
    const overlay = document.getElementById('app-bg-overlay');
    overlay.style.backgroundImage = '';
    overlay.classList.remove('has-bg');
    document.body.classList.remove('has-bg');
    if (bgObjectUrl) {
        URL.revokeObjectURL(bgObjectUrl);
        bgObjectUrl = null;
    }
    persistedBgImage = null;
    saveSettings();
});

// 恢复默认背景图
el.settingBgDefault.addEventListener('click', () => {
    if (typeof DEFAULT_BG === 'undefined') {
        showToast(t('toast.bgLost'), 'info', 2000);
        return;
    }
    const overlay = document.getElementById('app-bg-overlay');
    overlay.style.backgroundImage = `url(${DEFAULT_BG})`;
    overlay.classList.add('has-bg');
    document.body.classList.add('has-bg');
    if (bgObjectUrl) {
        URL.revokeObjectURL(bgObjectUrl);
        bgObjectUrl = null;
    }
    persistedBgImage = DEFAULT_BG;
    saveSettings();
    showToast(t('toast.bgRestored'), 'info', 2000);
});

// ─── 设置：透明度 ───────────────────────

el.settingWindowOpacity.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value) / 100;
    state.config.opacity = val;
    document.documentElement.style.setProperty('--glass-opacity', val);
    TAURI.core.invoke('set_opacity', { opacity: val });
    el.settingWindowOpacityVal.textContent = Math.round(val * 100) + '%';
    saveSettings();
});

el.settingBgOpacity.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value) / 100;
    state.config.bgOpacity = val;
    document.documentElement.style.setProperty('--bg-opacity', val);
    el.settingBgOpacityVal.textContent = Math.round(val * 100) + '%';
    saveSettings();
});

// 透明度恢复默认
el.settingOpacityReset.addEventListener('click', () => {
    state.config.opacity = 0.95;
    state.config.bgOpacity = 0.25;
    document.documentElement.style.setProperty('--glass-opacity', 0.95);
    document.documentElement.style.setProperty('--bg-opacity', 0.25);
    TAURI.core.invoke('set_opacity', { opacity: 0.95 });
    el.settingWindowOpacity.value = 95;
    el.settingWindowOpacityVal.textContent = '95%';
    el.settingBgOpacity.value = 25;
    el.settingBgOpacityVal.textContent = '25%';
    saveSettings();
    showToast(t('toast.opacityReset'), 'info', 1500);
});

// ─── 软件信息 ──────────────────────────

document.getElementById('about-author').addEventListener('click', () => {
    TAURI.core.invoke('open_external', { url: 'https://space.bilibili.com/14816' });
});

document.querySelector('.github-icon').addEventListener('click', (e) => {
    e.preventDefault();
    TAURI.core.invoke('open_external', { url: 'https://github.com/TohnoSeika' });
});

// ─── 转换 ──────────────────────────────────

el.btnConvert.addEventListener('click', async () => {
    if (state.isConverting || !state.currentPath) return;

    // 获取输出路径（保存对话框）
    const defaultPath = await TAURI.core.invoke('get_default_output_path', {
        path: state.currentPath,
    });

    // Tauri v2 没有自带 save dialog，让用户通过前端选择……
    // 这里简单处理：用同目录下的文件名 + 后缀改
    // 实际应该用 dialog plugin，但为了不引入额外依赖，直接用默认路径
    // 如果有 dialog plugin 可以用 window.__TAURI__.dialog.save()

    const params = {
        sourcePath: state.currentPath,
        outputPath: defaultPath,
        outputSize: parseInt(el.paramSize.value) || 1024,
        format: el.paramFormat.value,
        sampling: el.paramSampling.value,
        alphaHandling: el.paramAlpha.value,
        backgroundColor: hexToRgb(el.paramBgColor.value),
        sharpen: el.paramSharpen.checked,
    };

    // 开始转换
    state.isConverting = true;
    el.btnConvert.disabled = true;
    el.btnConvert.className = 'primary-btn converting';
    el.btnConvert.querySelector('.btn-text').textContent = t('btn.converting');
    showProgress('准备中……', 0);

    try {
        await TAURI.core.invoke('convert_image', params);
        // 转换成功
        el.btnConvert.className = 'primary-btn done';
        el.btnConvert.querySelector('.btn-text').textContent = t('btn.done');
        showToast(t('toast.convertDone'), 'success', 4000);
        hideProgress();
    } catch (err) {
        showToast(t('toast.convertFail', { err }), 'error');
        el.btnConvert.className = 'primary-btn';
        el.btnConvert.querySelector('.btn-text').textContent = t('btn.convert');
        hideProgress();
        console.error(err);
    } finally {
        state.isConverting = false;
        // 允许下次再转换（不 disabled，因为图片还在）
        el.btnConvert.disabled = false;
        // 3秒后恢复按钮文字
        setTimeout(() => {
            if (!state.isConverting) {
                el.btnConvert.className = 'primary-btn';
                el.btnConvert.querySelector('.btn-text').textContent = t('btn.convert');
            }
        }, 3000);
    }
});

// ─── 进度事件监听 ─────────────────────────

TAURI.event.listen('convert-progress', (event) => {
    const { stage, progress } = event.payload;
    // Rust 发来的中文进度消息 → 映射到 i18n 键
    const stageMap = {
        "正在读取图片……": "progress.loading",
        "正在调整尺寸……": "progress.resizing",
        "正在锐化……": "progress.sharpen",
        "正在处理通道……": "progress.alpha",
        "正在缩放到 L4D2 兼容尺寸……": "progress.resizeL4D",
        "正在编码 VTF……": "progress.encoding",
        "正在保存文件……": "progress.saving",
        "完成！": "progress.done",
    };
    showProgress(t(stageMap[stage] || stage), progress);
});

function showProgress(text, pct) {
    el.progressBar.classList.remove('hidden');
    el.progressFill.style.width = `${pct}%`;
    el.progressText.textContent = text;
}

function hideProgress() {
    el.progressBar.classList.add('hidden');
    el.progressFill.style.width = '0%';
}

// ─── 动态喷漆：帧管理 ───────────────────

function readFileAsB64(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result.split(',')[1]);
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(file);
    });
}

function animRender() {
    const list = el.animFrameList;
    list.innerHTML = '';
    if (animState.frames.length === 0) {
        list.innerHTML = '<p class="tab-placeholder">' + t('anim.empty') + '</p>';
        el.btnConvertAnim.disabled = true;
    } else {
        animState.frames.forEach((f, i) => {
            const item = document.createElement('div');
            item.className = 'anim-frame-item';
            item.innerHTML = `
                <img class="anim-frame-thumb" src="data:image/jpeg;base64,${f.b64}" alt="帧${i+1}">
                <div class="anim-frame-info">
                    <span>${t('frame.label', { index: i + 1, name: f.name })}</span>
                </div>
                <button class="anim-frame-del" data-idx="${i}" title="${t('frame.delete')}">✕</button>
            `;
            list.appendChild(item);
        });
        el.btnConvertAnim.disabled = false;
    }
    el.animCount.textContent = t('anim.count', { current: animState.frames.length, max: MAX_ANIM_FRAMES });
}

el.animAddBtn.addEventListener('click', () => el.animFileInput.click());

el.animFileInput.addEventListener('change', async () => {
    const files = Array.from(el.animFileInput.files);
    for (const file of files) {
        if (animState.frames.length >= MAX_ANIM_FRAMES) {
            showToast(t('toast.animFull', { max: MAX_ANIM_FRAMES }), 'info', 2000);
            break;
        }
        const fp = file.path || '';
        const dir = fp ? fp.substring(0, fp.lastIndexOf('\\')) : '';
        if (fp && !animState.sourceDir) animState.sourceDir = dir;
        const b64 = await readFileAsB64(file);
        animState.frames.push({ b64, name: file.name, path: fp });
    }
    animRender();
    el.animFileInput.value = '';
});

el.animFrameList.addEventListener('click', (e) => {
    const btn = e.target.closest('.anim-frame-del');
    if (btn) {
        const idx = parseInt(btn.dataset.idx);
        animState.frames.splice(idx, 1);
        if (animState.frames.length === 0) animState.sourceDir = null;
        animRender();
    }
});

// ─── 动态喷漆：转换 ─────────────────────

el.btnConvertAnim.addEventListener('click', async () => {
    if (animState.frames.length === 0) return;

    const dir = animState.sourceDir;
    if (!dir || !animState.frames.every(f => f.path && f.path.startsWith(dir))) {
        showToast(t('toast.animSameDir'), 'error');
        return;
    }

    const defaultPath = `${dir}\\animated_minagi.vtf`;
    const b64List = animState.frames.map(f => f.b64);

    el.btnConvertAnim.disabled = true;
    el.btnConvertAnim.querySelector('.btn-text').textContent = t('btn.converting');
    showProgress(t('progress.encodingAnim'), 50);

    try {
        await TAURI.core.invoke('convert_animated', {
            frames: b64List,
            outputPath: defaultPath,
        });
        el.btnConvertAnim.querySelector('.btn-text').textContent = t('btn.done');
        showToast(t('toast.animDone'), 'success', 4000);
        hideProgress();
    } catch (err) {
        showToast(t('toast.convertFail', { err }), 'error');
        el.btnConvertAnim.querySelector('.btn-text').textContent = t('btn.convert');
        hideProgress();
    } finally {
        el.btnConvertAnim.disabled = false;
        setTimeout(() => {
            el.btnConvertAnim.querySelector('.btn-text').textContent = t('btn.convert');
        }, 3000);
    }
});

// ─── 远近喷漆：图片管理 ───────────────────

function distRender() {
    const list = el.distFrameList;
    list.innerHTML = '';
    if (distState.frames.length === 0) {
        list.innerHTML = '<p class="tab-placeholder">' + t('dist.empty') + '</p>';
        el.btnConvertDist.disabled = true;
    } else {
        distState.frames.forEach((f, i) => {
            const mipLevels = [512, 256, 128, 64, 32];
            const mipLabel = mipLevels[i] || '—';
            const item = document.createElement('div');
            item.className = 'anim-frame-item';
            item.innerHTML = `
                <img class="anim-frame-thumb" src="data:image/jpeg;base64,${f.b64}" alt="图${i+1}">
                <div class="anim-frame-info">
                    <span>${t('frame.distLabel', { index: i + 1, size: mipLabel, name: f.name })}</span>
                </div>
                <button class="anim-frame-del" data-idx="${i}" title="${t('frame.delete')}">✕</button>
            `;
            list.appendChild(item);
        });
        el.btnConvertDist.disabled = false;
    }
    el.distCount.textContent = t('dist.count', { current: distState.frames.length, max: MAX_DIST_FRAMES });
}

el.distAddBtn.addEventListener('click', () => el.distFileInput.click());

el.distFileInput.addEventListener('change', async () => {
    const files = Array.from(el.distFileInput.files);
    for (const file of files) {
        if (distState.frames.length >= MAX_DIST_FRAMES) {
            showToast(t('toast.distFull', { max: MAX_DIST_FRAMES }), 'info', 2000);
            break;
        }
        const fp = file.path || '';
        const dir = fp ? fp.substring(0, fp.lastIndexOf('\\')) : '';
        if (fp && !distState.sourceDir) distState.sourceDir = dir;
        const b64 = await readFileAsB64(file);
        distState.frames.push({ b64, name: file.name, path: fp });
    }
    distRender();
    el.distFileInput.value = '';
});

el.distFrameList.addEventListener('click', (e) => {
    const btn = e.target.closest('.anim-frame-del');
    if (btn) {
        const idx = parseInt(btn.dataset.idx);
        distState.frames.splice(idx, 1);
        if (distState.frames.length === 0) distState.sourceDir = null;
        distRender();
    }
});

// ─── 远近喷漆：转换 ─────────────────────

el.btnConvertDist.addEventListener('click', async () => {
    if (distState.frames.length < 2) {
        showToast(t('toast.distMin'), 'error');
        return;
    }
    const dir = distState.sourceDir;
    if (!dir || !distState.frames.every(f => f.path && f.path.startsWith(dir))) {
        showToast(t('toast.animSameDir'), 'error');
        return;
    }
    const defaultPath = `${dir}\\distance_minagi.vtf`;
    const b64List = distState.frames.map(f => f.b64);
    el.btnConvertDist.disabled = true;
    el.btnConvertDist.querySelector('.btn-text').textContent = t('btn.converting');
    showProgress(t('progress.encodingDist'), 50);
    try {
        await TAURI.core.invoke('convert_distance', { frames: b64List, outputPath: defaultPath });
        el.btnConvertDist.querySelector('.btn-text').textContent = t('btn.done');
        showToast(t('toast.distDone'), 'success', 4000);
        hideProgress();
    } catch (err) {
        showToast(t('toast.convertFail', { err }), 'error');
        el.btnConvertDist.querySelector('.btn-text').textContent = t('btn.convert');
        hideProgress();
    } finally {
        el.btnConvertDist.disabled = false;
        setTimeout(() => {
            el.btnConvertDist.querySelector('.btn-text').textContent = t('btn.convert');
        }, 3000);
    }
});

// ─── 语言选择器 ──────────────────────────

document.getElementById('setting-language')?.addEventListener('change', (e) => {
    setLanguage(e.target.value);
    // 保存语言设置
    TAURI.core.invoke('save_settings', {
        settings: {
            themeColor: el.settingThemeColor.value,
            windowOpacity: state.config.opacity,
            bgOpacity: state.config.bgOpacity,
            bgImage: persistedBgImage,
            language: e.target.value,
        },
    }).catch(() => {});
});

// 语言切换后更新动态内容（帧列表、toast 等需要重新渲染的）
document.addEventListener('language-changed', () => {
    animRender();
    distRender();
    updateSettingsBgImage();
    // 刷新当前选项卡的拖拽提示
    const activeBtn = document.querySelector('.tab-btn.active');
    if (activeBtn) activeBtn.click();
});

// 更新设置页展示图（从 Rust 获取编译进二进制的图片）
function updateSettingsBgImage() {
    const img = document.getElementById('settings-bg-img');
    if (!img) return;
    const lang = getLanguage();
    TAURI.core.invoke('get_settings_bg_image', { lang })
        .then(dataUrl => { img.src = dataUrl; })
        .catch(() => { img.src = ''; });
}

// 设置页展示图鼠标跟随效果
const settingsDisplay = document.getElementById('settings-bg-display');
if (settingsDisplay) {
    let followTimer = null;

    settingsDisplay.addEventListener('mousemove', (e) => {
        const img = document.getElementById('settings-bg-img');
        if (!img || !img.src) return;

        const rect = settingsDisplay.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        const mouseX = e.clientX - centerX;
        const mouseY = e.clientY - centerY;

        const moveX = (mouseX / rect.width) * 10;
        const moveY = (mouseY / rect.height) * 6;

        img.style.transform = `translate(${moveX}px, ${moveY}px)`;

        // 光晕跟随鼠标
        const pctX = ((e.clientX - rect.left) / rect.width) * 100;
        const pctY = ((e.clientY - rect.top) / rect.height) * 100;
        settingsDisplay.style.background = `
            radial-gradient(
                circle at ${pctX}% ${pctY}%,
                rgba(232, 160, 180, 0.1) 0%,
                transparent 60%
            )
        `;
    });

    settingsDisplay.addEventListener('mouseleave', () => {
        const img = document.getElementById('settings-bg-img');
        if (!img) return;
        img.style.transform = 'translate(0, 0)';
        settingsDisplay.style.background = '';
    });
}

// ─── 工具函数 ──────────────────────────────

function hexToRgb(hex) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return [r, g, b];
}

// ─── 初始化 ──────────────────────────────

// 先确保默认壁纸同步加载（在 async 之前执行，不受异步影响）
(function initDefaultBg() {
    if (typeof DEFAULT_BG !== 'undefined' && DEFAULT_BG) {
        const overlay = document.getElementById('app-bg-overlay');
        // 只在完全没有背景图时才应用默认
        if (!overlay.style.backgroundImage || overlay.style.backgroundImage === 'none') {
            overlay.style.backgroundImage = `url(${DEFAULT_BG})`;
            overlay.classList.add('has-bg');
            document.body.classList.add('has-bg');
        }
    }
})();

(async function init() {
    const saved = await loadSavedSettings();

    // 应用已保存的主题色
    if (saved.themeColor) {
        el.settingThemeColor.value = saved.themeColor;
        el.settingThemeColorText.value = saved.themeColor;
        applyTheme(saved.themeColor);
    }

    // 应用语言设置：已保存的优先，否则根据系统语言自动检测
    if (saved.language) {
        setLanguage(saved.language);
    } else {
        // 首次启动：检测 Windows 系统语言
        const sysLang = (navigator.language || 'en').slice(0, 2);
        const detected = { zh: 'zh', ja: 'ja', en: 'en' }[sysLang] || 'en';
        setLanguage(detected);
        // 保存检测结果，后续就不走自动检测了
        saveSettings();
    }

    // 应用已保存的透明度
    if (saved.windowOpacity !== null) {
        state.config.opacity = saved.windowOpacity;
        el.settingWindowOpacity.value = Math.round(saved.windowOpacity * 100);
        el.settingWindowOpacityVal.textContent = Math.round(saved.windowOpacity * 100) + '%';
    }
    if (saved.bgOpacity !== null) {
        state.config.bgOpacity = saved.bgOpacity;
        el.settingBgOpacity.value = Math.round(saved.bgOpacity * 100);
        el.settingBgOpacityVal.textContent = Math.round(saved.bgOpacity * 100) + '%';
    }
    document.documentElement.style.setProperty('--glass-opacity', state.config.opacity);
    document.documentElement.style.setProperty('--bg-opacity', state.config.bgOpacity);
    TAURI.core.invoke('set_opacity', { opacity: state.config.opacity });

    // 应用已保存的背景图（覆盖同步加载的默认壁纸）
    persistedBgImage = saved.bgImage;
    const overlay = document.getElementById('app-bg-overlay');

    if (persistedBgImage) {
        // 有保存的背景图
        overlay.style.backgroundImage = `url(${persistedBgImage})`;
        overlay.classList.add('has-bg');
        document.body.classList.add('has-bg');
    } else if (saved._hasSaved && persistedBgImage === null) {
        // 用户之前主动清除了背景图 → 去掉同步设的默认壁纸
        overlay.style.backgroundImage = '';
        overlay.classList.remove('has-bg');
        document.body.classList.remove('has-bg');
    }
    // else: 首次启动 → 同步 initDefaultBg 已经设好了默认壁纸

    console.log('♡ VTF Baker —— 桃华帮你做喷漆 ♡');
    console.log('如果 Minagi 读到这一行，会不会笑一下呢？');
})();
console.log('如果 Minagi 读到这一行，会不会笑一下呢？');

// English —— Written by Touka for Minagi ♡

const I18N_EN = {
    // ─── Titlebar ──────────────────────
    "titlebar.title": "Minagi VTF Baker",
    "titlebar.tooltip": "A custom spray maker for Left 4 Dead",
    "titlebar.minimize": "Minimize",
    "titlebar.close": "Close",

    // ─── Tabs ──────────────────────────
    "tab.static": "Static",
    "tab.animated": "Animated",
    "tab.distant": "Distance",
    "tab.settings": "Settings",

    // ─── Drop Zone ─────────────────────
    "dropzone.text": "Drop your image here",
    "dropzone.hint": "or click to select a file",
    "dropzone.formats": "PNG · JPG · BMP · WEBP",
    "dropzone.animText": "Drop JPG files here as frames",
    "dropzone.animHint": "or click to select files (multi-select)",
    "dropzone.animFormats": "Each frame auto-resized to 256×256",
    "dropzone.distText": "Drop JPG files here",
    "dropzone.distHint": "Add in order, 1st = closest",
    "dropzone.distFormats": "Auto-resized to matching mip level",
    "dropzone.settingsText": "Customize your spray tool",
    "dropzone.settingsHint": "",
    "dropzone.settingsFormats": "",

    // ─── Preview ───────────────────────
    "preview.rechoose": "✕ Re-select",
    "preview.alphaYes": "Has alpha",
    "preview.alphaNo": "No alpha",

    // ─── Parameters ────────────────────
    "section.output": "Output Format",
    "section.sample": "Sampling & Enhance",
    "section.alpha": "Alpha Channel",

    "param.format": "Compression Format",
    "param.size": "Output Size",
    "param.sizeUnit": "px",
    "param.sizeHint": "Use 1024 for L4D, other values must be powers of 2",
    "param.sampling": "Sampling Method",
    "param.sharpen": "Sharpen",
    "param.alpha": "Alpha Handling",
    "param.bgColor": "Background Color",

    "option.format.dxt1": "DXT1 (for L4D, no alpha, smallest file)",
    "option.format.dxt5": "DXT5 (supports alpha)",
    "option.format.rgba8888": "RGBA8888 (lossless, largest file)",
    "option.sampling.anisotropic": "Anisotropic (for L4D)",
    "option.sampling.bilinear": "Bilinear",
    "option.sampling.nearest": "Nearest Neighbor",
    "option.alpha.keep": "Keep alpha",
    "option.alpha.remove": "Remove alpha, fill background",
    "option.alpha.fill": "Background fill (for DXT5)",

    // ─── Animated VTF ──────────────────
    "anim.frames": "Frame List",
    "anim.count": "{current} / {max} frames",
    "anim.empty": "No frames yet",
    "anim.addBtn": "＋ Add Frame (JPG)",
    "anim.output": "Output",
    "anim.sizeNote": "256 × 256 (fixed)",
    "anim.formatNote": "DXT1 (fixed)",
    "anim.samplingNote": "Anisotropic (fixed)",

    // ─── Distance VTF ──────────────────
    "dist.title": "Image List (by distance)",
    "dist.count": "{current} / {max} images",
    "dist.empty": "No images yet",
    "dist.addBtn": "＋ Add Image (JPG)",
    "dist.output": "Output",
    "dist.sizeNote": "512 × 512",
    "dist.formatNote": "DXT1 (fixed)",
    "dist.samplingNote": "Anisotropic (fixed)",
    "dist.baseSize": "Base Size",
    "dist.layerLabel": "Mip Level Mapping",
    "dist.layerNote": "1st = closest → 5th = farthest",

    // ─── Settings ──────────────────────
    "settings.theme": "Theme Color",
    "settings.themeColor": "Theme",
    "settings.themeReset": "Reset",
    "settings.bg": "Background Image",
    "settings.bgSelect": "Choose Background Image",
    "settings.bgClear": "Clear Background",
    "settings.bgDefault": "Restore Default",
    "settings.opacity": "Opacity",
    "settings.windowOpacity": "Window Opacity",
    "settings.bgOpacity": "Background Opacity",
    "settings.opacityReset": "Reset",
    "settings.language": "Language",
    "settings.langTitle": "Language 表示言語 界面语言",
    "settings.lang.zh": "简体中文",
    "settings.lang.ja": "日本語",
    "settings.lang.en": "English",

    // ─── About ─────────────────────────
    "about.name": "Minagi VTF Baker",
    "about.version": "v1.0",
    "about.developer": "Developed by",
    "about.line2": "im minagi, im everywhere",
    "about.note": "This software is free and will never charge any fees",

    // ─── Buttons ───────────────────────
    "btn.convert": "Convert",
    "btn.converting": "Converting…",
    "btn.done": "✓ Done",

    // ─── Progress Bar ──────────────────
    "progress.init": "Initializing…",
    "progress.loading": "Loading image…",
    "progress.resizing": "Resizing…",
    "progress.sharpen": "Sharpening…",
    "progress.alpha": "Processing alpha channel…",
    "progress.resizeL4D": "Resizing to L4D2 compatible size…",
    "progress.encoding": "Encoding VTF…",
    "progress.saving": "Saving file…",
    "progress.done": "Done!",
    "progress.encodingAnim": "Encoding animated spray…",
    "progress.encodingDist": "Encoding distance spray…",

    // ─── Toast Messages ────────────────
    "toast.loaded": "Loaded: {name}",
    "toast.loadFailed": "Failed to load… {err}",
    "toast.bgSet": "Background set ✨",
    "toast.bgLost": "Default background is missing…",
    "toast.bgRestored": "Default background restored ✨",
    "toast.opacityReset": "Opacity reset to defaults",
    "toast.saveFailed": "Failed to save settings…",
    "toast.convertDone": "Conversion complete! Saved to source directory ✨",
    "toast.convertFail": "Conversion failed… {err}",
    "toast.animFull": "Maximum {max} frames reached",
    "toast.animSameDir": "All frames must be in the same directory",
    "toast.animDone": "Animated spray saved ✨",
    "toast.distMin": "At least 2 images required",
    "toast.distFull": "Maximum {max} images reached",
    "toast.distDone": "Distance spray saved ✨",

    // ─── Frame List ────────────────────
    "frame.label": "Frame {index} · {name}",
    "frame.distLabel": "Layer {index} ({size}px) · {name}",
    "frame.delete": "Delete",
};

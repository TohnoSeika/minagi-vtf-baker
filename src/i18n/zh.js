// 简体中文 —— 桃华帮 Minagi 写的哦 ♡

const I18N_ZH = {
    // ─── 标题栏 ───────────────────────
    "titlebar.title": "Minagi VTF Baker",
    "titlebar.tooltip": "一个 Left 4 Dead 自定义喷漆制作工具",
    "titlebar.minimize": "最小化",
    "titlebar.close": "关闭",

    // ─── 选项卡 ───────────────────────
    "tab.static": "静态VTF",
    "tab.animated": "动态VTF",
    "tab.distant": "渐变VTF",
    "tab.settings": "设置",

    // ─── 拖拽区 ───────────────────────
    "dropzone.text": "把图片拖到这里",
    "dropzone.hint": "或者点击选择文件",
    "dropzone.formats": "PNG · JPG · BMP · WEBP",
    "dropzone.animText": "把 JPG 拖到这里作为帧",
    "dropzone.animHint": "或者点击选择文件（支持多选）",
    "dropzone.animFormats": "每帧自动缩放到 256×256",
    "dropzone.distText": "把 JPG 拖到这里",
    "dropzone.distHint": "按顺序添加，第1张=最近",
    "dropzone.distFormats": "自动缩放到对应层级",
    "dropzone.settingsText": "自定义你的专属喷漆工具",
    "dropzone.settingsHint": "",
    "dropzone.settingsFormats": "",

    // ─── 预览区 ───────────────────────
    "preview.rechoose": "✕ 重新选择",
    "preview.alphaYes": "含透明通道",
    "preview.alphaNo": "无透明",

    // ─── 参数面板 ─────────────────────
    "section.output": "输出格式",
    "section.sample": "采样 & 增强",
    "section.alpha": "透明通道",

    "param.format": "压缩格式",
    "param.size": "输出尺寸",
    "param.sizeUnit": "px",
    "param.sizeHint": "L4D请使用1024，其他数值必须为2的幂",
    "param.sampling": "采样方式",
    "param.sharpen": "锐化",
    "param.alpha": "处理方式",
    "param.bgColor": "背景色",

    "option.format.dxt1": "DXT1（L4D用这个·无透明·文件最小）",
    "option.format.dxt5": "DXT5（支持透明）",
    "option.format.rgba8888": "RGBA8888（无损·文件最大）",
    "option.sampling.anisotropic": "各向异性（L4D用这个 Anisotropic）",
    "option.sampling.bilinear": "双线性（Bilinear）",
    "option.sampling.nearest": "最近邻（Nearest）",
    "option.alpha.keep": "保留透明",
    "option.alpha.remove": "移除透明·填充背景色",
    "option.alpha.fill": "背景色垫底（用于DXT5）",

    // ─── 动态喷漆 ─────────────────────
    "anim.frames": "帧列表",
    "anim.count": "{current} / {max} 帧",
    "anim.empty": "还没有添加帧",
    "anim.addBtn": "＋ 添加帧（JPG）",
    "anim.output": "输出",
    "anim.sizeNote": "256 × 256（固定）",
    "anim.formatNote": "DXT1（固定）",
    "anim.samplingNote": "Anisotropic（固定）",

    // ─── 渐变喷漆 ─────────────────────
    "dist.title": "图片列表（按距离远近）",
    "dist.count": "{current} / {max} 张",
    "dist.empty": "还没有添加图片",
    "dist.addBtn": "＋ 添加图片（JPG）",
    "dist.output": "输出",
    "dist.sizeNote": "512 × 512",
    "dist.formatNote": "DXT1（固定）",
    "dist.samplingNote": "Anisotropic（固定）",
    "dist.baseSize": "基础尺寸",
    "dist.layerLabel": "层级对应",
    "dist.layerNote": "第1张=最近 → 第5张=最远",

    // ─── 设置页 ───────────────────────
    "settings.theme": "主题配色",
    "settings.themeColor": "主题色",
    "settings.themeReset": "恢复默认",
    "settings.bg": "背景图片",
    "settings.bgSelect": "选择背景图片",
    "settings.bgClear": "清除背景图",
    "settings.bgDefault": "恢复默认",
    "settings.opacity": "透明度",
    "settings.windowOpacity": "界面透明度",
    "settings.bgOpacity": "背景图透明度",
    "settings.opacityReset": "恢复默认",
    "settings.language": "界面语言",
    "settings.langTitle": "界面语言 Language 表示言語",
    "settings.lang.zh": "简体中文",
    "settings.lang.ja": "日本語",
    "settings.lang.en": "English",

    // ─── 软件信息 ─────────────────────
    "about.name": "Minagi VTF Baker",
    "about.version": "v1.0",
    "about.developer": "Developed by",
    "about.line2": "im minagi, im everywhere",
    "about.note": "本软件为免费软件，不会收取任何费用",

    // ─── 按钮 ─────────────────────────
    "btn.convert": "开始转换",
    "btn.converting": "转换中……",
    "btn.done": "✓ 转换完成",

    // ─── 进度条 ───────────────────────
    "progress.init": "初始化……",
    "progress.loading": "正在读取图片……",
    "progress.resizing": "正在调整尺寸……",
    "progress.sharpen": "正在锐化……",
    "progress.alpha": "正在处理通道……",
    "progress.resizeL4D": "正在缩放到 L4D2 兼容尺寸……",
    "progress.encoding": "正在编码 VTF……",
    "progress.saving": "正在保存文件……",
    "progress.done": "完成！",
    "progress.encodingAnim": "正在编码动态喷漆……",
    "progress.encodingDist": "正在编码远近喷漆……",

    // ─── Toast 消息 ────────────────────
    "toast.loaded": "已加载: {name}",
    "toast.loadFailed": "读取失败…… {err}",
    "toast.bgSet": "背景图已设置 ✨",
    "toast.bgLost": "默认背景图丢了……桃华回头补上～",
    "toast.bgRestored": "已恢复默认背景 ✨",
    "toast.opacityReset": "已恢复默认透明度",
    "toast.saveFailed": "设置保存失败了……",
    "toast.convertDone": "转换完成！已保存到原目录 ✨",
    "toast.convertFail": "转换失败…… {err}",
    "toast.animFull": "最多 {max} 帧，已满",
    "toast.animSameDir": "所有帧必须在同一文件夹",
    "toast.animDone": "动态喷漆已保存 ✨",
    "toast.distMin": "至少需要 2 张图",
    "toast.distFull": "最多 {max} 张图，已满",
    "toast.distDone": "远近喷漆已保存 ✨",

    // ─── 帧列表 ───────────────────────
    "frame.label": "第 {index} 帧 · {name}",
    "frame.distLabel": "第 {index} 层（{size}px）· {name}",
    "frame.delete": "删除",
};

// 日本語 —— 桃華が Minagi のために書きました ♡

const I18N_JA = {
    // ─── タイトルバー ─────────────────
    "titlebar.title": "Minagi VTF Baker",
    "titlebar.tooltip": "Left 4 Dead 用カスタムスプレー作成ツール",
    "titlebar.minimize": "最小化",
    "titlebar.close": "閉じる",

    // ─── タブ ─────────────────────────
    "tab.static": "静止VTF",
    "tab.animated": "動画VTF",
    "tab.distant": "距離VTF",
    "tab.settings": "設定",

    // ─── ドロップゾーン ───────────────
    "dropzone.text": "画像をここにドロップ",
    "dropzone.hint": "またはクリックしてファイルを選択",
    "dropzone.formats": "PNG · JPG · BMP · WEBP",
    "dropzone.animText": "JPG をここにドロップ（フレーム）",
    "dropzone.animHint": "またはクリックして選択（複数可）",
    "dropzone.animFormats": "各フレーム自動で 256×256 にリサイズ",
    "dropzone.distText": "JPG をここにドロップ",
    "dropzone.distHint": "順番に追加、1枚目=最も近い",
    "dropzone.distFormats": "自動で各レベルにリサイズ",
    "dropzone.settingsText": "自分好みのスプレーツールに",
    "dropzone.settingsHint": "",
    "dropzone.settingsFormats": "",

    // ─── プレビュー ───────────────────
    "preview.rechoose": "✕ 選択し直す",
    "preview.alphaYes": "透明度あり",
    "preview.alphaNo": "透明度なし",

    // ─── パラメータ ───────────────────
    "section.output": "出力形式",
    "section.sample": "サンプリング & 補強",
    "section.alpha": "アルファチャンネル",

    "param.format": "圧縮形式",
    "param.size": "出力サイズ",
    "param.sizeUnit": "px",
    "param.sizeHint": "L4D では 1024 を使用、他は2の累乗である必要があります",
    "param.sampling": "サンプリング方式",
    "param.sharpen": "シャープ化",
    "param.alpha": "処理方法",
    "param.bgColor": "背景色",

    "option.format.dxt1": "DXT1（L4D用・透明なし・最小）",
    "option.format.dxt5": "DXT5（透明対応）",
    "option.format.rgba8888": "RGBA8888（無劣化・最大）",
    "option.sampling.anisotropic": "異方性フィルタ（L4D用 Anisotropic）",
    "option.sampling.bilinear": "バイリニア（Bilinear）",
    "option.sampling.nearest": "最近傍（Nearest）",
    "option.alpha.keep": "透明を維持",
    "option.alpha.remove": "透明を除去・背景色で塗りつぶし",
    "option.alpha.fill": "背景色を下地に（DXT5用）",

    // ─── 動画VTF ──────────────────────
    "anim.frames": "フレーム一覧",
    "anim.count": "{current} / {max} フレーム",
    "anim.empty": "まだフレームがありません",
    "anim.addBtn": "＋ フレーム追加（JPG）",
    "anim.output": "出力",
    "anim.sizeNote": "256 × 256（固定）",
    "anim.formatNote": "DXT1（固定）",
    "anim.samplingNote": "Anisotropic（固定）",

    // ─── 距離VTF ──────────────────────
    "dist.title": "画像一覧（距離順）",
    "dist.count": "{current} / {max} 枚",
    "dist.empty": "まだ画像がありません",
    "dist.addBtn": "＋ 画像追加（JPG）",
    "dist.output": "出力",
    "dist.sizeNote": "512 × 512",
    "dist.formatNote": "DXT1（固定）",
    "dist.samplingNote": "Anisotropic（固定）",
    "dist.baseSize": "ベースサイズ",
    "dist.layerLabel": "Mipレベル対応",
    "dist.layerNote": "1枚目=最も近い → 5枚目=最も遠い",

    // ─── 設定 ─────────────────────────
    "settings.theme": "テーマカラー",
    "settings.themeColor": "テーマ色",
    "settings.themeReset": "デフォルトに戻す",
    "settings.bg": "背景画像",
    "settings.bgSelect": "背景画像を選択",
    "settings.bgClear": "背景をクリア",
    "settings.bgDefault": "デフォルトに戻す",
    "settings.opacity": "透明度",
    "settings.windowOpacity": "ウィンドウ透明度",
    "settings.bgOpacity": "背景画像の透明度",
    "settings.opacityReset": "デフォルトに戻す",
    "settings.language": "表示言語",
    "settings.langTitle": "表示言語 界面语言 Language",
    "settings.lang.zh": "简体中文",
    "settings.lang.ja": "日本語",
    "settings.lang.en": "English",

    // ─── ソフトウェア情報 ─────────────
    "about.name": "Minagi VTF Baker",
    "about.version": "v1.0",
    "about.developer": "Developed by",
    "about.line2": "im minagi, im everywhere",
    "about.note": "このソフトウェアは無料です。料金を請求することは一切ありません",

    // ─── ボタン ───────────────────────
    "btn.convert": "変換開始",
    "btn.converting": "変換中……",
    "btn.done": "✓ 変換完了",

    // ─── プログレスバー ───────────────
    "progress.init": "初期化中……",
    "progress.loading": "画像を読み込み中……",
    "progress.resizing": "サイズ調整中……",
    "progress.sharpen": "シャープ化中……",
    "progress.alpha": "アルファチャンネル処理中……",
    "progress.resizeL4D": "L4D2 互換サイズにリサイズ中……",
    "progress.encoding": "VTF エンコード中……",
    "progress.saving": "ファイルを保存中……",
    "progress.done": "完了！",
    "progress.encodingAnim": "動画スプレーをエンコード中……",
    "progress.encodingDist": "距離スプレーをエンコード中……",

    // ─── Toast メッセージ ──────────────
    "toast.loaded": "読み込み完了: {name}",
    "toast.loadFailed": "読み込み失敗…… {err}",
    "toast.bgSet": "背景画像を設定しました ✨",
    "toast.bgLost": "デフォルト背景がありません……",
    "toast.bgRestored": "デフォルト背景に戻しました ✨",
    "toast.opacityReset": "透明度をデフォルトに戻しました",
    "toast.saveFailed": "設定の保存に失敗しました……",
    "toast.convertDone": "変換完了！元のディレクトリに保存しました ✨",
    "toast.convertFail": "変換失敗…… {err}",
    "toast.animFull": "最大 {max} フレームまでです",
    "toast.animSameDir": "全フレームが同じフォルダにある必要があります",
    "toast.animDone": "動画スプレーを保存しました ✨",
    "toast.distMin": "少なくとも 2 枚の画像が必要です",
    "toast.distFull": "最大 {max} 枚までです",
    "toast.distDone": "距離スプレーを保存しました ✨",

    // ─── フレーム一覧 ─────────────────
    "frame.label": "第 {index} フレーム · {name}",
    "frame.distLabel": "第 {index} 層（{size}px）· {name}",
    "frame.delete": "削除",
};

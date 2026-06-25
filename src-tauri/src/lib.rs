// Minagi 的委托品 —— 桃华制作 ♡
// VTF Baker —— 把普通图片变成求生之路的喷漆

use base64::Engine;
use image::{
    imageops::FilterType,
    DynamicImage, GenericImageView, ImageBuffer, ImageFormat as ImgFmt,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use byteorder::WriteBytesExt;
use texpresso::{Format as TexpFormat, Params};

// ─── 配置持久化 ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme_color: Option<String>,
    pub window_opacity: Option<f64>,
    pub bg_opacity: Option<f64>,
    pub bg_image: Option<String>,
    pub language: Option<String>,
}

/// 获取设置文件路径：优先 exe 所在目录（绿色模式），不可写时回退到 AppData
fn get_settings_path() -> PathBuf {
    // 优先 exe 同级目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = dir.join("settings.json");
            if path.exists() {
                return path;
            }
            // 检测是否可写
            match fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
            {
                Ok(f) => { drop(f); fs::remove_file(&path).ok(); return path; }
                Err(_) => {}
            }
        }
    }
    // 回退到 %APPDATA%
    let app_data = PathBuf::from(
        std::env::var("APPDATA").unwrap_or_else(|_| ".".into()),
    )
    .join("minagi-vtf-baker");
    fs::create_dir_all(&app_data).ok();
    app_data.join("settings.json")
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    let path = get_settings_path();
    fs::write(&path, &json).map_err(|e| format!("保存设置失败…… {}", e))?;
    Ok(())
}

#[tauri::command]
fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&json).map_err(|e| e.to_string());
    }
    Ok(AppSettings {
        theme_color: None,
        window_opacity: None,
        bg_opacity: None,
        bg_image: None,
        language: None,
    })
}

// ─── Win32 FFI（窗口透明度） ────────────────────────
#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;
    pub type HWND = *mut c_void;
    pub type COLORREF = u32;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const LWA_ALPHA: u32 = 0x00000002;
    extern "system" {
        pub fn GetWindowLongPtrW(hwnd: HWND, nIndex: i32) -> isize;
        pub fn SetWindowLongPtrW(hwnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn SetLayeredWindowAttributes(
            hwnd: HWND,
            crKey: COLORREF,
            bAlpha: u8,
            dwFlags: u32,
        ) -> i32;
    }
}

// ─── 数据结构 ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub preview: String,
    pub file_name: String,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertProgress {
    pub stage: String,
    pub progress: u32,
}

// ─── 应用状态 ────────────────────────────────────────────────

pub struct AppState {
    pub current_image: Mutex<Option<DynamicImage>>,
}

// ─── 工具函数 ────────────────────────────────────────────────

/// 根据 alpha_handling 模式处理透明通道
fn process_alpha(
    img: &DynamicImage,
    mode: &str,
    bg: [u8; 3],
) -> DynamicImage {
    let has_alpha = matches!(
        img.color(),
        image::ColorType::Rgba8
            | image::ColorType::Rgba16
            | image::ColorType::La8
            | image::ColorType::La16
    );

    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8();

    match mode {
        "remove" => {
            // 去透明，用背景色填充
            let mut out = ImageBuffer::from_pixel(w, h, image::Rgba([bg[0], bg[1], bg[2], 255]));
            for (x, y, pixel) in rgba.enumerate_pixels() {
                let a = pixel[3];
                if a == 255 {
                    out.put_pixel(x, y, *pixel);
                } else if a > 0 {
                    let blend = |b: u8, f: u8, a: u8| {
                        ((b as u16 * (255 - a as u16) + f as u16 * a as u16) / 255) as u8
                    };
                    out.put_pixel(x, y, image::Rgba([
                        blend(bg[0], pixel[0], a),
                        blend(bg[1], pixel[1], a),
                        blend(bg[2], pixel[2], a),
                        255,
                    ]));
                }
            }
            DynamicImage::from(out)
        }
        "fill" => {
            // 填充背景色 + 保留透明度 (适合 DXT5)
            let mut out = ImageBuffer::from_pixel(w, h, image::Rgba([bg[0], bg[1], bg[2], 0]));
            for (x, y, pixel) in rgba.enumerate_pixels() {
                let a = pixel[3];
                if a > 0 {
                    let blend = |b: u8, f: u8, a: u8| {
                        ((b as u16 * (255 - a as u16) + f as u16 * a as u16) / 255) as u8
                    };
                    out.put_pixel(x, y, image::Rgba([
                        blend(bg[0], pixel[0], a),
                        blend(bg[1], pixel[1], a),
                        blend(bg[2], pixel[2], a),
                        a,
                    ]));
                }
            }
            DynamicImage::from(out)
        }
        _ => {
            // "keep" 或默认：保留透明通道
            if has_alpha {
                img.clone()
            } else {
                DynamicImage::from(rgba)
            }
        }
    }
}

/// 选择一个采样滤镜
fn select_filter(sampling: &str) -> FilterType {
    match sampling {
        "nearest" => FilterType::Nearest,
        "bilinear" => FilterType::Triangle,
        "anisotropic" => FilterType::Lanczos3,
        _ => FilterType::Lanczos3,
    }
}

/// 将格式字符串转为 DXT 格式代码 (11=DXT1, 13=DXT5)
fn select_dxt_format(fmt: &str) -> i16 {
    match fmt {
        "dxt5" => 15,
        "dxt1" => 13,
        _ => 13,
    }
}

/// ─── 自定义 VTF 编码器 ────────────────────────────

const VTF_SIGNATURE: u32 = 0x00465456;
const VTF_FLAGS: u32 = 0x0000231C;
const FORMAT_DXT1: i16 = 13;
const FORMAT_DXT5: i16 = 15;
const DIST_VTF_FLAGS: u32 = 0x0000220C;

/// 将 RGBA 图像数据编码为 VTF 文件 (DXT1 或 DXT5)
fn encode_vtf(
    width: u16,
    height: u16,
    rgba_data: &[u8],
    dxt_format: i16,
) -> Vec<u8> {
    let header_size: u32 = 64;
    let data_size = match dxt_format {
        FORMAT_DXT5 => TexpFormat::Bc3.compressed_size(width as usize, height as usize),
        _ => TexpFormat::Bc1.compressed_size(width as usize, height as usize),
    };
    let total_size = header_size as usize + data_size;
    let mut buf = Vec::with_capacity(total_size);

    // VTF header (64 bytes, version 7.1)
    buf.write_u32::<byteorder::LittleEndian>(VTF_SIGNATURE).ok();
    buf.write_u32::<byteorder::LittleEndian>(7).ok();
    buf.write_u32::<byteorder::LittleEndian>(1).ok();
    buf.write_u32::<byteorder::LittleEndian>(header_size).ok();
    buf.write_u16::<byteorder::LittleEndian>(width).ok();
    buf.write_u16::<byteorder::LittleEndian>(height).ok();
    buf.write_u32::<byteorder::LittleEndian>(VTF_FLAGS).ok();
    buf.write_u16::<byteorder::LittleEndian>(1).ok();       // frames
    buf.write_u16::<byteorder::LittleEndian>(0).ok();       // first_frame
    buf.write_u32::<byteorder::LittleEndian>(0).ok();       // padding
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();     // reflectivity
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_u32::<byteorder::LittleEndian>(0).ok();       // padding
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();     // bumpmap_scale
    buf.write_u32::<byteorder::LittleEndian>(dxt_format as i16 as u32).ok();
    buf.write_u8(1).ok();                                    // mipmap_count
    buf.write_u32::<byteorder::LittleEndian>(FORMAT_DXT1 as i16 as u32).ok();
    buf.write_u8(0).ok();                                    // lowres_width
    buf.write_u8(0).ok();                                    // lowres_height
    buf.write_u8(1).ok();                                    // depth/padding

    // DXT 压缩
    let mut compressed = vec![0u8; data_size];
    match dxt_format {
        FORMAT_DXT5 => TexpFormat::Bc3.compress(
            rgba_data, width as usize, height as usize, Params::default(), &mut compressed),
        _ => TexpFormat::Bc1.compress(
            rgba_data, width as usize, height as usize, Params::default(), &mut compressed),
    }
    buf.extend_from_slice(&compressed);
    buf
}

// ─── Tauri Commands ──────────────────────────────────────────

#[tauri::command]
fn load_image(path: String) -> Result<ImageInfo, String> {
    let img = image::open(&path).map_err(|e| format!("图片打不开呢…… {}", e))?;

    let (width, height) = img.dimensions();
    let has_alpha = matches!(
        img.color(),
        image::ColorType::Rgba8
            | image::ColorType::Rgba16
            | image::ColorType::La8
            | image::ColorType::La16
    );

    // 生成缩略图预览 (最长边 400px, base64)
    let preview_size = 400u32;
    let preview_img = if width > preview_size || height > preview_size {
        let ratio = (preview_size as f64 / width.max(height) as f64).min(1.0);
        img.resize(
            (width as f64 * ratio) as u32,
            (height as f64 * ratio) as u32,
            FilterType::Lanczos3,
        )
    } else {
        img.clone()
    };

    let mut buf = std::io::Cursor::new(Vec::new());
    preview_img
        .write_to(&mut buf, ImgFmt::Png)
        .map_err(|e| format!("预览生成失败…… {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());

    let file_name = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(ImageInfo {
        width,
        height,
        has_alpha,
        preview: format!("data:image/png;base64,{}", b64),
        file_name,
        file_size,
    })
}

#[tauri::command]
fn load_image_data(base64: String, name: String) -> Result<ImageInfo, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64)
        .map_err(|e| format!("图片数据解码失败…… {}", e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("图片读取失败…… {}", e))?;

    let (width, height) = img.dimensions();
    let has_alpha = matches!(
        img.color(),
        image::ColorType::Rgba8
            | image::ColorType::Rgba16
            | image::ColorType::La8
            | image::ColorType::La16
    );

    let preview_size = 400u32;
    let preview_img = if width > preview_size || height > preview_size {
        let ratio = (preview_size as f64 / width.max(height) as f64).min(1.0);
        img.resize(
            (width as f64 * ratio) as u32,
            (height as f64 * ratio) as u32,
            FilterType::Lanczos3,
        )
    } else {
        img.clone()
    };

    let mut buf = std::io::Cursor::new(Vec::new());
    preview_img
        .write_to(&mut buf, ImgFmt::Png)
        .map_err(|e| format!("预览生成失败…… {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());

    Ok(ImageInfo {
        width,
        height,
        has_alpha,
        preview: format!("data:image/png;base64,{}", b64),
        file_name: name,
        file_size: bytes.len() as u64,
    })
}

#[tauri::command]
async fn convert_image(
    source_path: String,
    output_path: String,
    output_size: u32,
    format: String,
    sampling: String,
    alpha_handling: String,
    background_color: [u8; 3],
    sharpen: bool,
    app: AppHandle,
) -> Result<(), String> {
    // 在后台线程执行重负载工作，不阻塞界面
    let result = tauri::async_runtime::spawn_blocking(move || {
        do_convert(
            &source_path, &output_path, output_size,
            &format, &sampling, &alpha_handling,
            background_color, sharpen, &app,
        )
    })
    .await
    .map_err(|e| format!("转换线程异常退出: {}", e))?;

    result
}

/// 实际的转换逻辑（在后台线程运行）
fn do_convert(
    source_path: &str,
    output_path: &str,
    output_size: u32,
    format: &str,
    sampling: &str,
    alpha_handling: &str,
    background_color: [u8; 3],
    sharpen: bool,
    app: &AppHandle,
) -> Result<(), String> {
    // 桃华觉得这个看起来简单，但花了不少心思呢！

    // 1. 读取图片
    app.emit("convert-progress", ConvertProgress {
        stage: "正在读取图片……".into(),
        progress: 10,
    })
    .ok();

    let mut img =
        image::open(source_path).map_err(|e| format!("图片打不开呢…… {}", e))?;
    let (orig_w, orig_h) = img.dimensions();

    // 2. 调整尺寸 —— 按比例缩放 + 补边到目标尺寸
    app.emit("convert-progress", ConvertProgress {
        stage: "正在调整尺寸……".into(),
        progress: 25,
    })
    .ok();

    let target = output_size.max(16).min(4096).next_power_of_two();

    // 只有任意一边大于目标尺寸时才缩小，否则保持原尺寸直接居中贴到画布
    if orig_w > target || orig_h > target {
        let ratio = target as f64 / orig_w.max(orig_h) as f64;
        let new_w = (orig_w as f64 * ratio) as u32;
        let new_h = (orig_h as f64 * ratio) as u32;
        img = img.resize(new_w, new_h, select_filter(sampling));
    }

    let (cur_w, cur_h) = img.dimensions();
    if cur_w != target || cur_h != target {
        let offset_x = (target - cur_w) / 2;
        let offset_y = (target - cur_h) / 2;
        let bg_color = if alpha_handling == "keep" {
            image::Rgba([0, 0, 0, 0])
        } else {
            image::Rgba([background_color[0], background_color[1], background_color[2], 255])
        };
        let mut canvas = ImageBuffer::from_pixel(target, target, bg_color);
        image::imageops::overlay(&mut canvas, &img, offset_x as i64, offset_y as i64);
        img = DynamicImage::from(canvas);
    }

    // 3. 锐化（可选）
    if sharpen {
        app.emit("convert-progress", ConvertProgress {
            stage: "正在锐化……".into(),
            progress: 40,
        })
        .ok();
        let rgba = img.to_rgba8();
        let sharpened = image::imageops::unsharpen(&rgba, 1.0, 0);
        img = DynamicImage::from(sharpened);
    }

    // 4. 透明通道处理
    app.emit("convert-progress", ConvertProgress {
        stage: "正在处理通道……".into(),
        progress: 55,
    })
    .ok();

    let rgba_img = process_alpha(&img, alpha_handling, background_color);

    // 强制缩放到 L4D2 兼容尺寸 (1020×1024)
    app.emit("convert-progress", ConvertProgress {
        stage: "正在缩放到 L4D2 兼容尺寸……".into(),
        progress: 62,
    })
    .ok();

    let (l4d2_w, l4d2_h) = (1020u32, 1024u32);
    let rgba_resized = rgba_img.resize_exact(
        l4d2_w, l4d2_h,
        image::imageops::FilterType::Lanczos3,
    );
    let rgba_raw = rgba_resized.to_rgba8();

    // 5. VTF 编码（桃华自研编码器 ♡）
    app.emit("convert-progress", ConvertProgress {
        stage: "正在编码 VTF……".into(),
        progress: 70,
    })
    .ok();

    let dxt_fmt = select_dxt_format(format);
    let vtf_bytes = encode_vtf(
        l4d2_w as u16, l4d2_h as u16,
        rgba_raw.as_raw(), dxt_fmt,
    );

    // 6. 写入文件
    app.emit("convert-progress", ConvertProgress {
        stage: "正在保存文件……".into(),
        progress: 90,
    })
    .ok();

    fs::write(output_path, &vtf_bytes)
        .map_err(|e| format!("文件保存失败…… {}", e))?;

    app.emit("convert-progress", ConvertProgress {
        stage: "完成！".into(),
        progress: 100,
    })
    .ok();

    Ok(())
}

/// ─── 动态喷漆编码器 ──────────────────────────

const ANIM_FRAME_W: u16 = 256;
const ANIM_FRAME_H: u16 = 256;

fn encode_animated_vtf(frames: &[Vec<u8>]) -> Vec<u8> {
    let header_size: u32 = 64;
    let frame_size = TexpFormat::Bc1.compressed_size(ANIM_FRAME_W as usize, ANIM_FRAME_H as usize);
    let num_frames = frames.len() as u16;
    let total_size = header_size as usize + frame_size * frames.len();
    let mut buf = Vec::with_capacity(total_size);

    buf.write_u32::<byteorder::LittleEndian>(VTF_SIGNATURE).ok();
    buf.write_u32::<byteorder::LittleEndian>(7).ok();
    buf.write_u32::<byteorder::LittleEndian>(1).ok();
    buf.write_u32::<byteorder::LittleEndian>(header_size).ok();
    buf.write_u16::<byteorder::LittleEndian>(ANIM_FRAME_W).ok();
    buf.write_u16::<byteorder::LittleEndian>(ANIM_FRAME_H).ok();
    buf.write_u32::<byteorder::LittleEndian>(VTF_FLAGS).ok();
    buf.write_u16::<byteorder::LittleEndian>(num_frames).ok();
    buf.write_u16::<byteorder::LittleEndian>(0).ok();
    buf.write_u32::<byteorder::LittleEndian>(0).ok();
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_u32::<byteorder::LittleEndian>(0).ok();
    buf.write_f32::<byteorder::LittleEndian>(0.0).ok();
    buf.write_u32::<byteorder::LittleEndian>(FORMAT_DXT1 as i16 as u32).ok();
    buf.write_u8(1).ok();
    buf.write_u32::<byteorder::LittleEndian>(FORMAT_DXT1 as i16 as u32).ok();
    buf.write_u8(0).ok();
    buf.write_u8(0).ok();
    buf.write_u8(1).ok();

    for rgba_data in frames {
        let mut compressed = vec![0u8; frame_size];
        TexpFormat::Bc1.compress(
            rgba_data, ANIM_FRAME_W as usize, ANIM_FRAME_H as usize,
            Params::default(), &mut compressed,
        );
        buf.extend_from_slice(&compressed);
    }
    buf
}

#[tauri::command]
async fn convert_animated(
    frames: Vec<String>,
    output_path: String,
) -> Result<(), String> {
    use base64::Engine;

    if frames.is_empty() {
        return Err("至少需要一帧图片".into());
    }
    if frames.len() > 11 {
        return Err("最多支持 11 帧".into());
    }

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut rgba_frames: Vec<Vec<u8>> = Vec::with_capacity(frames.len());

        for b64 in &frames {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64).map_err(|e| format!("图片解码失败: {}", e))?;
            let img = image::load_from_memory(&bytes)
                .map_err(|e| format!("图片读取失败: {}", e))?;
            let resized = img.resize_exact(
                ANIM_FRAME_W as u32, ANIM_FRAME_H as u32,
                image::imageops::FilterType::Lanczos3,
            );
            let rgba = resized.to_rgba8();
            rgba_frames.push(rgba.into_raw());
        }

        let vtf_bytes = encode_animated_vtf(&rgba_frames);
        fs::write(&output_path, &vtf_bytes)
            .map_err(|e| format!("文件保存失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("线程异常: {}", e))?;

    result
}

/// ─── 远近喷漆编码器 ─────────────────────────

const DIST_BASE_W: u16 = 512;
const DIST_BASE_H: u16 = 512;
const MAX_DIST_FRAMES: usize = 5;

fn mip_size(base: u16, level: u32) -> u16 {
    (base >> level).max(1)
}

fn encode_distance_vtf(frames: &[Vec<u8>]) -> Vec<u8> {
    // 使用 Little_Angel.vtf 的 header 模板（已验证可工作）
    let header: [u8; 64] = [
        0x56, 0x54, 0x46, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x02, 0x0C, 0x22, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
        0x05, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];
    let num_mips = frames.len();
    let mut buf = Vec::new();
    buf.extend_from_slice(&header);

    // Mishcatt 兼容：mip 从小到大存（mip N 最前，mip 0 最后）
    for i in (0..num_mips).rev() {
        let w = mip_size(DIST_BASE_W, i as u32);
        let h = mip_size(DIST_BASE_H, i as u32);
        let wu = w as usize;
        let hu = h as usize;
        let sz = TexpFormat::Bc1.compressed_size(wu, hu);

        let mut compressed = vec![0u8; sz];
        TexpFormat::Bc1.compress(
            &frames[i], wu, hu,
            Params::default(), &mut compressed,
        );
        buf.extend_from_slice(&compressed);
    }
    buf
}

#[tauri::command]
async fn convert_distance(
    frames: Vec<String>,
    output_path: String,
) -> Result<(), String> {
    use base64::Engine;
    if frames.len() < 2 {
        return Err("至少需要 2 张图才能制作远近喷漆".into());
    }
    if frames.len() > MAX_DIST_FRAMES {
        return Err(format!("最多支持 {} 张图", MAX_DIST_FRAMES));
    }

    let result = tauri::async_runtime::spawn_blocking(move || {
        let user_count = frames.len();
        let mut rgba_frames: Vec<Vec<u8>> = Vec::with_capacity(MAX_DIST_FRAMES);

        for (i, b64) in frames.iter().enumerate() {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64).map_err(|e| format!("图片解码失败: {}", e))?;
            let img = image::load_from_memory(&bytes)
                .map_err(|e| format!("图片读取失败: {}", e))?;
            let mip_w = mip_size(DIST_BASE_W, i as u32);
            let mip_h = mip_size(DIST_BASE_H, i as u32);
            let resized = img.resize_exact(mip_w as u32, mip_h as u32, image::imageops::FilterType::Lanczos3);
            let rgba = resized.to_rgba8();
            rgba_frames.push(rgba.into_raw());
        }

        // 不足 5 级时从最后一张缩小填充
        if user_count < MAX_DIST_FRAMES {
            let last_rgba = &rgba_frames[user_count - 1];
            let last_w = mip_size(DIST_BASE_W, (user_count - 1) as u32);
            let last_img = image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(last_w as u32, last_w as u32, last_rgba.clone()).unwrap()
            );
            for i in user_count..MAX_DIST_FRAMES {
                let mw = mip_size(DIST_BASE_W, i as u32);
                let mh = mip_size(DIST_BASE_H, i as u32);
                let r = last_img.resize_exact(mw as u32, mh as u32, image::imageops::FilterType::Lanczos3);
                rgba_frames.push(r.to_rgba8().into_raw());
            }
        }

        let vtf_bytes = encode_distance_vtf(&rgba_frames);
        fs::write(&output_path, &vtf_bytes).map_err(|e| format!("文件保存失败: {}", e))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("线程异常: {}", e))?;
    result
}

#[tauri::command]
fn get_default_output_path(path: String) -> Result<String, String> {
    let source = Path::new(&path);
    let parent = source
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    Ok(parent
        .join("static_minagi.vtf".to_string())
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn read_file_base64(path: String) -> Result<String, String> {
    let data = fs::read(&path).map_err(|e| format!("文件读取失败…… {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        _ => "image/png",
    };

    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
fn get_desktop_path() -> Result<String, String> {
    if let Ok(path) = std::env::var("USERPROFILE") {
        Ok(format!("{}\\Desktop", path))
    } else {
        Ok("C:\\Users\\Minagi\\Desktop".into())
    }
}

// ─── 打开外部链接 ────────────────────────────────

#[tauri::command]
async fn open_external(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("无法打开链接: {}", e))
}

// ─── 窗口透明度 ────────────────────────────────────

#[tauri::command]
async fn set_opacity(window: tauri::WebviewWindow, opacity: f64) -> Result<(), String> {
    let clamped = opacity.max(0.1).min(1.0);

    #[cfg(target_os = "windows")]
    {
        use win32::*;
        let raw = window.hwnd().map_err(|e| e.to_string())?;
        let hwnd: *mut std::ffi::c_void = raw.0;
        unsafe {
            let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            ex_style |= WS_EX_LAYERED as isize;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
            let alpha = (clamped * 255.0) as u8;
            SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
        }
    }

    // 同步 CSS 变量让面板跟随
    let js = format!(
        "document.documentElement.style.setProperty('--glass-opacity', '{}');",
        clamped
    );
    window.eval(&js).map_err(|e| e.to_string())?;

    Ok(())
}

// ─── 设置页展示图（编译进二进制，不暴露外部文件） ──────────

#[tauri::command]
fn get_settings_bg_image(lang: String) -> Result<String, String> {
    let data: &[u8] = match lang.as_str() {
        "zh" => &include_bytes!("../../Setting_BGP/CN.jpg")[..],
        "ja" => &include_bytes!("../../Setting_BGP/JP.jpg")[..],
        _   => &include_bytes!("../../Setting_BGP/EN.jpg")[..],
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    Ok(format!("data:image/jpeg;base64,{}", b64))
}

// ─── 应用入口 ────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            current_image: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            load_image,
            load_image_data,
            convert_image,
            convert_animated,
            convert_distance,
            get_desktop_path,
            get_default_output_path,
            read_file_base64,
            set_opacity,
            open_external,
            save_settings,
            load_settings,
            get_settings_bg_image,
        ])
        .run(tauri::generate_context!())
        .expect("启动失败……桃华有点难过呢");
}

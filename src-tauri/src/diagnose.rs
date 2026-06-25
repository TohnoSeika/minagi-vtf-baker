// 桃华的 VTF 诊断工具 v3 —— 从原图到 VTF 全流程验证
use image::GenericImageView;
use std::fs;

fn mip_size(base: u16, level: u32) -> u16 {
    (base >> level).max(1)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("analyze");

    match command {
        "test" => generate_test_vtf(),
        "roundtrip" => roundtrip_test(),
        "hybrid" => hybrid_test(),
        "double" => double_compress_test(),
        "noisy" => noisy_test(),
        "range" => range_fit_test(),
        "sameimg" => same_image_test(),
        "clustertrue" => cluster_fit_alpha_test(),
        "realround" => real_roundtrip(),
        "miporder" => check_mip_order(),
        "genpics" => generate_test_pics(),
        "makeicon" => make_titlebar_icon(),
        "makebg" => make_default_bg(),
        "analyze_blocks" => analyze_blocks(),
        "full" => full_pipeline_diagnose(),
        _ => analyze_vtf(args.get(1).map(|s| s.as_str()).unwrap_or("distance_minagi.vtf")),
    }
}

/// ─── 完整管线诊断：从用户的 JPG 原图出发，模拟 convert_distance ───
fn full_pipeline_diagnose() {
    let pic_dir = "../PIC";
    let jpgs = ["距离最近.jpg", "稍微远点.jpg", "更远及更更远.jpg"];
    let mip_names = ["近 (512px)", "中近 (256px)", "中 (128px)", "中远 (64px,填充)", "远 (32px,填充)"];

    println!("=== 完整管线诊断 ===\n");

    // Step 1: 加载并处理用户图片
    let mut rgba_frames: Vec<Vec<u8>> = Vec::new();

    for (i, jpg_name) in jpgs.iter().enumerate() {
        let path = format!("{}/{}", pic_dir, jpg_name);
        let mip_w = mip_size(512, i as u32);
        let mip_h = mip_size(512, i as u32);

        print!("\n--- {} → mip {} ({}×{}) ---", jpg_name, i, mip_w, mip_h);

        let data = fs::read(&path).expect("读不到图");
        print!("\n  原始文件大小: {} bytes", data.len());

        let img = image::load_from_memory(&data).expect("图片解码失败");
        let (orig_w, orig_h) = img.dimensions();
        let color_type = img.color();
        print!("\n  原始尺寸: {}×{}, 颜色类型: {:?}", orig_w, orig_h, color_type);

        // 检查是否有透明通道
        let has_alpha = matches!(color_type,
            image::ColorType::Rgba8 | image::ColorType::Rgba16 |
            image::ColorType::La8 | image::ColorType::La16);
        print!("  [{}]", if has_alpha { "含透明" } else { "无透明" });

        let resized = img.resize_exact(mip_w as u32, mip_h as u32, image::imageops::FilterType::Lanczos3);
        let (rw, rh) = resized.dimensions();
        let rtype = resized.color();
        print!("\n  缩放后: {}×{}, 颜色类型: {:?}", rw, rh, rtype);

        let rgba = resized.to_rgba8();
        let raw = rgba.into_raw();
        print!("\n  RGBA 原始数据: {} bytes (期望 {} bytes) [{}]",
            raw.len(), (mip_w as usize) * (mip_h as usize) * 4,
            if raw.len() == (mip_w as usize) * (mip_h as usize) * 4 { "✓" } else { "✗ 大小不对!" }
        );

        // 检查 alpha 通道
        let mut alpha_zero = 0;
        let mut alpha_varied = false;
        let mut first_alpha = 0u8;
        for (j, chunk) in raw.chunks(4).enumerate() {
            if j == 0 { first_alpha = chunk[3]; }
            if chunk[3] == 0 { alpha_zero += 1; }
            if chunk[3] != first_alpha { alpha_varied = true; }
        }
        print!("\n  Alpha: 首个值={}, 零值像素={}, 有变化={}", first_alpha, alpha_zero, alpha_varied);

        // 保存原始 RGBA 为 PNG（压缩前）
        if let Some(pre_img) = image::RgbaImage::from_raw(mip_w as u32, mip_h as u32, raw.clone()) {
            let pre_path = format!("diagnose_precompress_mip_{}.png", i);
            pre_img.save(&pre_path).ok();
            print!("\n  压缩前预览: {}", pre_path);
        }

        rgba_frames.push(raw);
    }

    // 填充缺失的 mip level
    let user_count = rgba_frames.len();
    if user_count < 5 {
        println!("\n\n--- 填充缺失 mip level (从最后一张图缩小) ---");
        let last_idx = user_count - 1;
        let last_w = mip_size(512, last_idx as u32);
        let last_rgba = &rgba_frames[last_idx];
        let last_img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(last_w as u32, last_w as u32, last_rgba.clone()).unwrap()
        );

        for i in user_count..5 {
            let mw = mip_size(512, i as u32);
            let mh = mip_size(512, i as u32);
            let r = last_img.resize_exact(mw as u32, mh as u32, image::imageops::FilterType::Lanczos3);
            let raw = r.to_rgba8().into_raw();
            print!("\n  Mip {} ({}×{}): RGBA {} bytes ✓", i, mw, mh, raw.len());
            rgba_frames.push(raw);
        }
    }

    // 压缩每个 mip level + 解压验证
    println!("\n\n=== DXT1 压缩 & 解压验证 ===");
    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let rgba = &rgba_frames[i as usize];

        // 压缩
        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(rgba, w, h, texpresso::Params::default(), &mut compressed);

        // 解压
        let mut decompressed = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(&compressed, w, h, &mut decompressed);

        // 逐像素比较
        let mut max_diff = [0u8; 4];
        let mut diff_count = 0u64;
        let mut diff_sum = [0u64; 4];
        for p in 0..(w * h) {
            let orig = &rgba[p*4..p*4+4];
            let dec = &decompressed[p*4..p*4+4];
            let mut different = false;
            for c in 0..4 {
                let d = (orig[c] as i16 - dec[c] as i16).unsigned_abs() as u8;
                if d > max_diff[c] { max_diff[c] = d; }
                diff_sum[c] += d as u64;
                if d > 2 { different = true; }
            }
            if different { diff_count += 1; }
        }
        let n = (w * h) as f64;
        print!("\nMip {} ({}×{}):", i, w, h);
        print!(" 压缩 {} bytes, 解压 {} bytes", sz, w*h*4);
        print!("\n  最大色差 (R,G,B,A): ({},{},{},{})", max_diff[0], max_diff[1], max_diff[2], max_diff[3]);
        if diff_count > 0 {
            print!("\n  {} 像素色差>2 ({:.1}%)", diff_count, diff_count as f64 / n * 100.0);
        } else {
            print!("\n  无显著色差 ✓");
        }

        // 保存解压后的 PNG
        if let Some(img) = image::RgbaImage::from_raw(w as u32, h as u32, decompressed) {
            let out_path = format!("diagnose_mip_{}.png", i);
            img.save(&out_path).ok();
            print!("\n  已保存: {}", out_path);
        }
    }

    println!("\n\n=== 诊断完成 ===");
    println!("请把生成的 diagnose_mip_*.png 和 diagnose_precompress_mip_*.png 给桃华看～");
    println!("这样桃华就能对比压缩前后的差异！");
}

/// 生成合成测试 VTF
fn generate_test_vtf() {
    let colors: [(u8, u8, u8); 5] = [
        (255,  60,  60), // Mip 0: 红
        ( 60, 220,  60), // Mip 1: 绿
        ( 60,  60, 255), // Mip 2: 蓝
        (255, 220,  40), // Mip 3: 黄
        (220,  60, 220), // Mip 4: 紫
    ];
    let labels = ["近 (512px)", "中近 (256px)", "中 (128px)", "中远 (64px)", "远 (32px)"];

    let header: [u8; 64] = [
        0x56, 0x54, 0x46, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x02, 0x1C, 0x22, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
        0x05, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];
    let mut buf = Vec::new();
    buf.extend_from_slice(&header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let (r, g, b) = colors[i as usize];
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let is_cross = (x == w/2 || y == h/2) || (x == w/4 && y == h/4);
                rgba[idx] = if is_cross { 255 } else { r };
                rgba[idx+1] = if is_cross { 255 } else { g };
                rgba[idx+2] = if is_cross { 255 } else { b };
                rgba[idx+3] = 255;
            }
        }
        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, texpresso::Params::default(), &mut compressed);
        buf.extend_from_slice(&compressed);
        println!("Mip {} ({}x{}): #{:02X}{:02X}{:02X}, {} bytes — {}",
            i, w, h, r, g, b, sz, labels[i as usize]);
    }

    let out_path = "test_distance_colors.vtf";
    fs::write(out_path, &buf).expect("保存失败");
    println!("\n✓ 测试 VTF 已保存: {} ({} bytes)", out_path, buf.len());
}

/// ─── 关键实验：解压参考 VTF → 重新压缩 → 对比 ───
fn roundtrip_test() {
    let ref_path = "../Little_Angel.vtf";
    println!("=== 关键实验：参考文件解压→再压缩 ===\n");
    println!("读取: {}", ref_path);

    let ref_data = fs::read(ref_path).expect("读不到参考文件");
    let header = &ref_data[..64];
    let width = u16::from_le_bytes([header[0x10], header[0x11]]);
    let mip_count = header[0x38];
    println!("参考文件: {}x{}, {} mips, {} bytes total", width, width, mip_count, ref_data.len());

    let mut recompressed_mips: Vec<Vec<u8>> = Vec::new();
    let mut original_mips: Vec<&[u8]> = Vec::new();
    let mut offset = 64usize;

    // 解压每个 mip level
    for i in 0..mip_count as u32 {
        let w = mip_size(width, i) as usize;
        let h = mip_size(width, i) as usize;
        let compressed_sz = texpresso::Format::Bc1.compressed_size(w, h);

        let original_compressed = &ref_data[offset..offset + compressed_sz];
        original_mips.push(original_compressed);

        // 解压
        let mut decompressed = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(original_compressed, w, h, &mut decompressed);

        // 重新压缩
        let mut recompressed = vec![0u8; compressed_sz];
        texpresso::Format::Bc1.compress(&decompressed, w, h, texpresso::Params::default(), &mut recompressed);
        recompressed_mips.push(recompressed);

        // 对比
        let identical = original_compressed == recompressed_mips[i as usize].as_slice();
        let diff_bytes = original_compressed.iter().zip(recompressed_mips[i as usize].iter())
            .filter(|(a, b)| a != b)
            .count();
        println!("Mip {} ({}x{}): {} bytes, 解压→再压缩: {} 字节不同 {}",
            i, w, h, compressed_sz, diff_bytes,
            if identical { "✓ 完全一致!" } else { "✗ 有差异!" }
        );

        offset += compressed_sz;
    }

    // 生成再压缩版 VTF
    let mut new_vtf = Vec::new();
    new_vtf.extend_from_slice(header);
    let mut total_identical = true;
    for (i, recomp) in recompressed_mips.iter().enumerate() {
        new_vtf.extend_from_slice(recomp);
        if original_mips[i] != recomp.as_slice() { total_identical = false; }
    }

    let out_path = "roundtrip_minagi.vtf";
    fs::write(out_path, &new_vtf).expect("保存失败");

    // Alpha channel analysis on both reference and recompressed
    println!("\n=== Alpha 通道分析（检查 1-bit alpha 模式）===");
    let mut offset2 = 64usize;
    for i in 0..mip_count as u32 {
        let w = mip_size(width, i) as usize;
        let h = mip_size(width, i) as usize;
        let compressed_sz = texpresso::Format::Bc1.compressed_size(w, h);

        // Count 1-bit alpha blocks in original (color0 <= color1)
        let orig = &ref_data[offset2..offset2 + compressed_sz];
        let mut orig_alpha_blocks = 0u32;
        let mut recomp_alpha_blocks = 0u32;
        for b in 0..(compressed_sz / 8) {
            let c0 = u16::from_le_bytes([orig[b*8], orig[b*8+1]]);
            let c1 = u16::from_le_bytes([orig[b*8+2], orig[b*8+3]]);
            if c0 <= c1 { orig_alpha_blocks += 1; }

            let rec = &recompressed_mips[i as usize];
            let c0r = u16::from_le_bytes([rec[b*8], rec[b*8+1]]);
            let c1r = u16::from_le_bytes([rec[b*8+2], rec[b*8+3]]);
            if c0r <= c1r { recomp_alpha_blocks += 1; }
        }
        let total_blocks = compressed_sz / 8;
        // 进一步细分 1-bit alpha 块: color0 < color1 vs color0 == color1
        let mut orig_eq = 0u32; let mut orig_lt = 0u32;
        for b in 0..(compressed_sz / 8) {
            let c0 = u16::from_le_bytes([orig[b*8], orig[b*8+1]]);
            let c1 = u16::from_le_bytes([orig[b*8+2], orig[b*8+3]]);
            if c0 == c1 { orig_eq += 1; }
            else if c0 < c1 { orig_lt += 1; }
        }
        println!("Mip {}: 原 {}/{} 1-bit alpha (c0<c1:{} c0==c1:{}), 重压缩 {} ({:.1}%)",
            i, orig_alpha_blocks, total_blocks, orig_lt, orig_eq,
            recomp_alpha_blocks, recomp_alpha_blocks as f64 / total_blocks as f64 * 100.0);
        offset2 += compressed_sz;
    }

    // Also analyze test_distance_colors.vtf
    println!("\n=== test_distance_colors.vtf Alpha 分析 ===");
    let test_paths = ["test_distance_colors.vtf", "../test_distance_colors.vtf"];
    let mut test_data_opt = None;
    for p in &test_paths {
        if let Ok(d) = fs::read(p) { test_data_opt = Some(d); println!("(读取自: {})", p); break; }
    }
    if let Some(test_data) = test_data_opt {
        let mut off = 64usize;
        for i in 0..5u32 {
            let w = mip_size(512, i) as usize;
            let h = mip_size(512, i) as usize;
            let sz = texpresso::Format::Bc1.compressed_size(w, h);
            let data = &test_data[off..off+sz];
            let mut alpha_blocks = 0u32; let mut eq_blocks = 0u32; let mut lt_blocks = 0u32;
            for b in 0..(sz/8) {
                let c0 = u16::from_le_bytes([data[b*8], data[b*8+1]]);
                let c1 = u16::from_le_bytes([data[b*8+2], data[b*8+3]]);
                if c0 <= c1 {
                    alpha_blocks += 1;
                    if c0 == c1 { eq_blocks += 1; } else { lt_blocks += 1; }
                }
            }
            println!("Mip {}: {} 1-bit alpha / {} ({:.1}%) — c0<c1:{} c0==c1:{}",
                i, alpha_blocks, sz/8, alpha_blocks as f64 / (sz/8) as f64 * 100.0, lt_blocks, eq_blocks);
            off += sz;
        }
    } else {
        println!("(文件未找到)");
    }

    // Also analyze distance_minagi.vtf
    println!("\n=== distance_minagi.vtf Alpha 分析 ===");
    if let Ok(dist_data) = fs::read("../distance_minagi.vtf") {
        let mut off = 64usize;
        for i in 0..5u32 {
            let w = mip_size(512, i) as usize;
            let h = mip_size(512, i) as usize;
            let sz = texpresso::Format::Bc1.compressed_size(w, h);
            let data = &dist_data[off..off+sz];
            let mut alpha_blocks = 0u32; let mut eq_blocks = 0u32; let mut lt_blocks = 0u32;
            for b in 0..(sz/8) {
                let c0 = u16::from_le_bytes([data[b*8], data[b*8+1]]);
                let c1 = u16::from_le_bytes([data[b*8+2], data[b*8+3]]);
                if c0 <= c1 {
                    alpha_blocks += 1;
                    if c0 == c1 { eq_blocks += 1; } else { lt_blocks += 1; }
                }
            }
            println!("Mip {}: {} 1-bit alpha / {} ({:.1}%) — c0<c1:{} c0==c1:{}",
                i, alpha_blocks, sz/8, alpha_blocks as f64 / (sz/8) as f64 * 100.0, lt_blocks, eq_blocks);
            off += sz;
        }
    }

    println!("\n=== 实验结果 ===");
    if total_identical {
        println!("✓✓✓ 再压缩后与原始文件逐字节完全一致！");
        println!("说明 texpresso 和 VTFLib 的 DXT1 压缩输出完全相同。");
        println!("问题不在压缩库。");
    } else {
        println!("✗✗✗ 再压缩后与原始文件有差异！");
        println!("说明 texpresso 的 DXT1 压缩输出和 VTFLib 不同。");
        println!("这可能就是百叶窗的原因——引擎对 texpresso 的压缩数据解读不同。");
        println!("\n再压缩版已保存: {}", out_path);
        println!("请把这个文件放到游戏里测试，看是否也出现百叶窗！");
    }

    // 字节级对比
    if !total_identical {
        let mut total_diff = 0usize;
        for i in 0..mip_count as usize {
            let diff = original_mips[i].iter().zip(recompressed_mips[i].iter())
                .filter(|(a, b)| a != b)
                .count();
            total_diff += diff;
        }
        println!("总差异字节: {} / {}", total_diff, ref_data.len() - 64);
    }
}

/// ─── 终极实验：用参考 VTF 的 alpha 通道 + 我们的测试色 ───
fn hybrid_test() {
    let ref_path = "../Little_Angel.vtf";
    println!("=== 混合实验：参考 alpha + 测试颜色 ===\n");

    let ref_data = fs::read(ref_path).expect("读不到参考文件");
    let header = &ref_data[..64];
    let width = u16::from_le_bytes([header[0x10], header[0x11]]);
    let mip_count = header[0x38];

    let test_colors: [(u8, u8, u8); 5] = [
        (255, 60, 60), (60, 220, 60), (60, 60, 255),
        (255, 220, 40), (220, 60, 220),
    ];

    let mut new_vtf = Vec::new();
    new_vtf.extend_from_slice(header);

    let mut offset = 64usize;
    for i in 0..mip_count as u32 {
        let w = mip_size(width, i) as usize;
        let h = mip_size(width, i) as usize;
        let sz = texpresso::Format::Bc1.compressed_size(w, h);

        // 解压参考
        let original = &ref_data[offset..offset + sz];
        let mut decompressed = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(original, w, h, &mut decompressed);

        // 替换 RGB，保留 alpha
        let (r, g, b) = test_colors[i as usize];
        let mut alpha_non_255 = 0usize;
        for p in 0..(w * h) {
            let idx = p * 4;
            if decompressed[idx + 3] != 255 { alpha_non_255 += 1; }
            decompressed[idx] = r;
            decompressed[idx + 1] = g;
            decompressed[idx + 2] = b;
        }
        println!("Mip {}: 非255 alpha 像素: {} / {}", i, alpha_non_255, w * h);

        // 重新压缩
        let mut recompressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&decompressed, w, h, texpresso::Params::default(), &mut recompressed);
        new_vtf.extend_from_slice(&recompressed);
        offset += sz;
    }

    let out_path = "hybrid_test.vtf";
    fs::write(out_path, &new_vtf).expect("save fail");
    println!("\n✓ hybrid_test.vtf 已保存 ({} bytes)", new_vtf.len());
    println!("这个文件用参考的 alpha + 测试颜色。");
    println!("如果游戏里正常显示→说明 alpha 通道是关键。");
    println!("如果还是百叶窗→说明问题不在 alpha。");
}

/// ─── 双重压缩实验：先 DXT1→解压→再 DXT1 ───
fn double_compress_test() {
    println!("=== 双重压缩实验 ===\n");

    let colors: [(u8, u8, u8); 5] = [
        (255, 60, 60), (60, 220, 60), (60, 60, 255),
        (255, 220, 40), (220, 60, 220),
    ];
    let labels = ["近 (512px)", "中近 (256px)", "中 (128px)", "中远 (64px)", "远 (32px)"];

    let header: [u8; 64] = [
        0x56, 0x54, 0x46, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x02, 0x1C, 0x22, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
        0x05, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];

    let mut buf = Vec::new();
    buf.extend_from_slice(&header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let (r, g, b) = colors[i as usize];

        // 创建原始纯色 RGBA
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let is_cross = (x == w/2 || y == h/2) || (x == w/4 && y == h/4);
                rgba[idx] = if is_cross { 255 } else { r };
                rgba[idx+1] = if is_cross { 255 } else { g };
                rgba[idx+2] = if is_cross { 255 } else { b };
                rgba[idx+3] = 255;
            }
        }

        // 第一轮压缩
        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut first_pass = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, texpresso::Params::default(), &mut first_pass);

        // 解压
        let mut mid = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(&first_pass, w, h, &mut mid);

        // 第二轮压缩（模拟 roundtrip 流程）
        let mut second_pass = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&mid, w, h, texpresso::Params::default(), &mut second_pass);

        // 比较两轮压缩
        let diff = first_pass.iter().zip(second_pass.iter()).filter(|(a,b)| a!=b).count();
        println!("Mip {} ({}x{}): 第一轮 {} bytes, 第二轮 {} bytes, 不同: {} ({:.1}%)",
            i, w, h, sz, sz, diff, diff as f64 / sz as f64 * 100.0);

        buf.extend_from_slice(&second_pass);
    }

    let out_path = "double_compress.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ double_compress.vtf 已保存 ({} bytes)", buf.len());
    println!("这个文件对纯色做了 压缩→解压→再压缩。");
    println!("如果游戏里正常 → 关键差异在这里！");
    println!("如果还是百叶窗 → 桃华需要新思路了……");
}

/// ─── 算法测试：用 RangeFit 替代 ClusterFit ───
fn range_fit_test() {
    println!("=== RangeFit 算法测试 ===\n");

    let colors: [(u8, u8, u8); 5] = [
        (255, 60, 60), (60, 220, 60), (60, 60, 255),
        (255, 220, 40), (220, 60, 220),
    ];

    let header: [u8; 64] = [
        0x56, 0x54, 0x46, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x02, 0x1C, 0x22, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
        0x05, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];

    let params = texpresso::Params {
        algorithm: texpresso::Algorithm::RangeFit,
        weights: texpresso::COLOUR_WEIGHTS_PERCEPTUAL,
        weigh_colour_by_alpha: true, // match libsquish default
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(&header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let (r, g, b) = colors[i as usize];

        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let is_cross = (x == w/2 || y == h/2) || (x == w/4 && y == h/4);
                rgba[idx] = if is_cross { 255 } else { r };
                rgba[idx+1] = if is_cross { 255 } else { g };
                rgba[idx+2] = if is_cross { 255 } else { b };
                rgba[idx+3] = 255;
            }
        }

        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, params, &mut compressed);

        let mut eq_blocks = 0u32;
        for b in 0..(sz/8) {
            let c0 = u16::from_le_bytes([compressed[b*8], compressed[b*8+1]]);
            let c1 = u16::from_le_bytes([compressed[b*8+2], compressed[b*8+3]]);
            if c0 == c1 { eq_blocks += 1; }
        }
        println!("Mip {} ({}x{}): c0==c1 块: {} / {}", i, w, h, eq_blocks, sz/8);
        buf.extend_from_slice(&compressed);
    }

    let out_path = "range_fit_test.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ range_fit_test.vtf 已保存 ({} bytes)", buf.len());
    println!("用 RangeFit + weigh_colour_by_alpha=true，更接近 libsquish 默认行为。");
}

/// ─── 标准 mipmap 测试：同一张图逐级缩小 ───
fn same_image_test() {
    let ref_path = "../Little_Angel.vtf";
    println!("=== 同图标准 Mipmap 测试 ===\n");

    let ref_data = fs::read(ref_path).expect("读不到参考");
    let header = &ref_data[..64];

    // 解压参考 mip 0
    let mip0_w = 512usize;
    let mip0_h = 512usize;
    let mip0_sz = texpresso::Format::Bc1.compressed_size(mip0_w, mip0_h);
    let mut mip0_rgba = vec![0u8; mip0_w * mip0_h * 4];
    texpresso::Format::Bc1.decompress(&ref_data[64..64+mip0_sz], mip0_w, mip0_h, &mut mip0_rgba);

    // 从 mip 0 创建标准 mip 链
    let base_img = image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(512, 512, mip0_rgba).unwrap()
    );

    let mut buf = Vec::new();
    buf.extend_from_slice(header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;

        let rgba: Vec<u8> = if i == 0 {
            // mip 0 直接用原图
            base_img.to_rgba8().into_raw()
        } else {
            // 从原图缩小
            let r = base_img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3);
            r.to_rgba8().into_raw()
        };

        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, texpresso::Params::default(), &mut compressed);
        println!("Mip {} ({}x{}): {} bytes", i, w, h, sz);
        buf.extend_from_slice(&compressed);
    }

    let out_path = "same_image_test.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ same_image_test.vtf 已保存 ({} bytes)", buf.len());
    println!("全部 5 个 mip 是同一张图逐级缩小（标准 mipmap）。");
    println!("如果没有百叶窗 → 说明引擎只在使用不同图时出问题");
    println!("如果也有百叶窗 → 说明引擎始终把所有 mip 拼成长图");
}

/// ─── 噪音测试：给纯色图加微量噪声 ───
fn noisy_test() {
    println!("=== 加噪测试 ===\n");

    let colors: [(u8, u8, u8); 5] = [
        (255, 60, 60), (60, 220, 60), (60, 60, 255),
        (255, 220, 40), (220, 60, 220),
    ];

    let header: [u8; 64] = [
        0x56, 0x54, 0x46, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x02, 0x1C, 0x22, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
        0x05, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];

    // 使用简单的伪随机种子（避免使用 std::time）
    let mut seed = 12345u32;
    fn lcg(s: &mut u32) -> u8 {
        *s = s.wrapping_mul(1103515245).wrapping_add(12345);
        (*s >> 16) as u8
    }

    let mut buf = Vec::new();
    buf.extend_from_slice(&header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let (r, g, b) = colors[i as usize];

        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let is_cross = (x == w/2 || y == h/2) || (x == w/4 && y == h/4);
                let mut noise = |v: u8| -> u8 {
                    let n = lcg(&mut seed) % 3;
                    if n == 0 && v > 0 { v - 1 }
                    else if n == 1 && v < 254 { v + 1 }
                    else { v }
                };
                rgba[idx] = if is_cross { noise(255) } else { noise(r) };
                rgba[idx+1] = if is_cross { noise(255) } else { noise(g) };
                rgba[idx+2] = if is_cross { noise(255) } else { noise(b) };
                rgba[idx+3] = 255;
            }
        }

        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, texpresso::Params::default(), &mut compressed);

        // 检查 c0==c1 块
        let mut eq_blocks = 0u32;
        for b in 0..(sz/8) {
            let c0 = u16::from_le_bytes([compressed[b*8], compressed[b*8+1]]);
            let c1 = u16::from_le_bytes([compressed[b*8+2], compressed[b*8+3]]);
            if c0 == c1 { eq_blocks += 1; }
        }
        println!("Mip {} ({}x{}): c0==c1 块: {} / {}", i, w, h, eq_blocks, sz/8);

        buf.extend_from_slice(&compressed);
    }

    let out_path = "noisy_test.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ noisy_test.vtf 已保存 ({} bytes)", buf.len());
    println!("加了微量噪声的测试 VTF——如果这个正常，说明问题就是纯色块的 c0==c1！");
}

/// ─── ClusterFit + weigh_colour_by_alpha=true ───
fn cluster_fit_alpha_test() {
    println!("=== ClusterFit + weigh_colour_by_alpha=true 测试 ===\n");

    let colors: [(u8, u8, u8); 5] = [
        (255, 60, 60), (60, 220, 60), (60, 60, 255),
        (255, 220, 40), (220, 60, 220),
    ];

    // 从参考文件拷贝 header
    let ref_data = fs::read("../Little_Angel.vtf").expect("读不到参考");
    let header = &ref_data[..64];

    let params = texpresso::Params {
        algorithm: texpresso::Algorithm::ClusterFit,
        weights: texpresso::COLOUR_WEIGHTS_PERCEPTUAL,
        weigh_colour_by_alpha: true, // 这是 libsquish 的默认行为
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let (r, g, b) = colors[i as usize];

        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let is_cross = (x == w/2 || y == h/2) || (x == w/4 && y == h/4);
                rgba[idx] = if is_cross { 255 } else { r };
                rgba[idx+1] = if is_cross { 255 } else { g };
                rgba[idx+2] = if is_cross { 255 } else { b };
                rgba[idx+3] = 255;
            }
        }

        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut compressed = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&rgba, w, h, params, &mut compressed);

        let mut eq_blocks = 0u32;
        for b in 0..(sz/8) {
            let c0 = u16::from_le_bytes([compressed[b*8], compressed[b*8+1]]);
            let c1 = u16::from_le_bytes([compressed[b*8+2], compressed[b*8+3]]);
            if c0 == c1 { eq_blocks += 1; }
        }
        println!("Mip {} ({}x{}): c0==c1 块: {} / {}", i, w, h, eq_blocks, sz/8);
        buf.extend_from_slice(&compressed);
    }

    let out_path = "cluster_alpha_test.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ cluster_alpha_test.vtf 已保存 ({} bytes)", buf.len());
    println!("用参考 header + ClusterFit + weigh_colour_by_alpha=true");
    println!("这是最接近 Mishcatt/VTFLib/libSquish 的配置");
}

/// ─── 真实 roundtrip：用用户截图 + 双压缩 ───
fn real_roundtrip() {
    println!("=== 真实 roundtrip：用户截图 + 压缩→解压→再压缩 ===\n");

    let pic_dir = "../PIC";
    let jpgs = ["距离最近.jpg", "稍微远点.jpg", "更远及更更远.jpg"];

    let ref_data = fs::read("../Little_Angel.vtf").expect("读不到参考");
    let header = &ref_data[..64];

    let mut rgba_frames: Vec<Vec<u8>> = Vec::new();

    for (i, jpg_name) in jpgs.iter().enumerate() {
        let path = format!("{}/{}", pic_dir, jpg_name);
        let mip_w = mip_size(512, i as u32);
        let mip_h = mip_size(512, i as u32);
        let data = fs::read(&path).expect("读不到图");
        let img = image::load_from_memory(&data).expect("解码失败");
        let resized = img.resize_exact(mip_w as u32, mip_h as u32, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8().into_raw();
        println!("{} → {}×{} RGBA {} bytes", jpg_name, mip_w, mip_h, rgba.len());
        rgba_frames.push(rgba);
    }

    let user_count = rgba_frames.len();
    if user_count < 5 {
        let last_idx = user_count - 1;
        let last_w = mip_size(512, last_idx as u32);
        let last_rgba = &rgba_frames[last_idx];
        let last_img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(last_w as u32, last_w as u32, last_rgba.clone()).unwrap()
        );
        for i in user_count..5 {
            let mw = mip_size(512, i as u32);
            let mh = mip_size(512, i as u32);
            let r = last_img.resize_exact(mw as u32, mh as u32, image::imageops::FilterType::Lanczos3);
            rgba_frames.push(r.to_rgba8().into_raw());
        }
    }

    let mut buf = Vec::new();
    buf.extend_from_slice(header);

    for i in 0..5u32 {
        let w = mip_size(512, i) as usize;
        let h = mip_size(512, i) as usize;
        let rgba = &rgba_frames[i as usize];
        let sz = texpresso::Format::Bc1.compressed_size(w, h);

        // 第一轮压缩
        let mut first = vec![0u8; sz];
        texpresso::Format::Bc1.compress(rgba, w, h, texpresso::Params::default(), &mut first);
        // 解压
        let mut mid = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(&first, w, h, &mut mid);
        // 第二轮压缩
        let mut second = vec![0u8; sz];
        texpresso::Format::Bc1.compress(&mid, w, h, texpresso::Params::default(), &mut second);

        let diff = first.iter().zip(second.iter()).filter(|(a,b)| a!=b).count();
        println!("Mip {} ({}x{}): 双压差异 {} / {} ({:.1}%)",
            i, w, h, diff, sz, diff as f64 / sz as f64 * 100.0);

        buf.extend_from_slice(&second);
    }

    let out_path = "real_roundtrip.vtf";
    fs::write(out_path, &buf).expect("save fail");
    println!("\n✓ real_roundtrip.vtf 已保存 ({} bytes)", buf.len());
    println!("用真实截图+双压缩管线。如果这个能正常 → 方案找到了！");
}

/// ─── 验证参考文件的 mip 存储顺序 ───
fn check_mip_order() {
    let ref_path = "../Little_Angel.vtf";
    let data = fs::read(ref_path).expect("读不到参考");
    println!("=== 参考文件 Mip 顺序验证 ===");

    // 尝试两种解释方式
    let mip_sizes: [(usize, usize); 5] = [(512,512), (256,256), (128,128), (64,64), (32,32)];

    // 假设1: 从大到小 (offset 64 = mip 0, 131072 bytes)
    println!("\n假设1: 从大到小存储");
    let mut off = 64usize;
    for i in 0..5 {
        let (w,h) = mip_sizes[i];
        let sz = texpresso::Format::Bc1.compressed_size(w, h);
        let mut decomp = vec![0u8; w*h*4];
        texpresso::Format::Bc1.decompress(&data[off..off+sz], w, h, &mut decomp);

        // 计算边界连续性：4x4 块边界的颜色差异
        let mut edge_diff_sum = 0u64;
        let mut edge_count = 0u64;
        let b_w = w/4; let b_h = h/4;
        for by in 0..b_h {
            for bx in 1..b_w {
                for py in 0..4 {
                    let x1 = bx*4-1; let x2 = bx*4;
                    let y = by*4+py;
                    let i1 = (y*w + x1)*4;
                    let i2 = (y*w + x2)*4;
                    let dr = (decomp[i1] as i32 - decomp[i2] as i32).abs() as u64;
                    let dg = (decomp[i1+1] as i32 - decomp[i2+1] as i32).abs() as u64;
                    let db = (decomp[i1+2] as i32 - decomp[i2+2] as i32).abs() as u64;
                    edge_diff_sum += dr + dg + db;
                    edge_count += 3;
                }
            }
        }
        let avg_edge = edge_diff_sum as f64 / edge_count as f64;
        println!("  Mip {} ({}x{}): 平均块边界色差 {:.1}", i, w, h, avg_edge);
        off += sz;
    }

    // 假设2: 从小到大 (offset 64 = mip 4, 512 bytes)
    println!("\n假设2: 从小到大存储");
    let rev_order = [(32,32,0), (64,64,1), (128,128,2), (256,256,3), (512,512,4)];
    let mut off2 = 64usize;
    for (w,h,_real_mip) in &rev_order {
        let sz = texpresso::Format::Bc1.compressed_size(*w, *h);
        let mut decomp = vec![0u8; w*h*4];
        texpresso::Format::Bc1.decompress(&data[off2..off2+sz], *w, *h, &mut decomp);

        let mut edge_diff_sum = 0u64;
        let mut edge_count = 0u64;
        let b_w = w/4; let b_h = h/4;
        for by in 0..b_h {
            for bx in 1..b_w {
                for py in 0..4 {
                    let x1 = bx*4-1; let x2 = bx*4;
                    let y = by*4+py;
                    let i1 = (y*w + x1)*4;
                    let i2 = (y*w + x2)*4;
                    let dr = (decomp[i1] as i32 - decomp[i2] as i32).abs() as u64;
                    let dg = (decomp[i1+1] as i32 - decomp[i2+1] as i32).abs() as u64;
                    let db = (decomp[i1+2] as i32 - decomp[i2+2] as i32).abs() as u64;
                    edge_diff_sum += dr + dg + db;
                    edge_count += 3;
                }
            }
        }
        let avg_edge = edge_diff_sum as f64 / edge_count as f64;
        println!("  {}x{} @ offset {}: 平均块边界色差 {:.1}", w, h, off2, avg_edge);
        off2 += sz;
    }

    println!("\n如果假设1的边界色差小 → 参考文件是从大到小存 (和我们一样)");
    println!("如果假设2的边界色差小 → 参考文件是从小到大存 (Mishcatt 的方式)");
    println!("如果两者差不多 → 无法判断 (需要看具体图片内容)");
}

/// ─── 生成测试纯色 PNG ───
fn generate_test_pics() {
    use image::RgbaImage;
    let colors: [(u8, u8, u8, &str); 5] = [
        (255, 60, 60, "test_red"),
        (60, 220, 60, "test_green"),
        (60, 60, 255, "test_blue"),
        (255, 220, 40, "test_yellow"),
        (220, 60, 220, "test_purple"),
    ];
    for (r, g, b, name) in &colors {
        let mut img = RgbaImage::from_pixel(512, 512, image::Rgba([*r, *g, *b, 255]));
        // 加个白色小十字方便辨认
        for i in 0..512 {
            img.put_pixel(256, i, image::Rgba([255, 255, 255, 255]));
            img.put_pixel(i, 256, image::Rgba([255, 255, 255, 255]));
        }
        let path = format!("{}.png", name);
        img.save(&path).expect("save fail");
        println!("✓ {} (纯色 #{:02X}{:02X}{:02X} + 白色十字)", path, r, g, b);
    }
    println!("\n把这 5 张 PNG 上传到 Mishcatt 做测试～");
}

/// ─── 转换 ico 为 titlebar 用的 base64 PNG ───
fn make_titlebar_icon() {
    use base64::Engine;
    let img = image::open("../ico.ico").expect("读不到 ico.ico");
    let resized = img.resize_exact(40, 40, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let mut png_bytes = Vec::new();
    rgba.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).expect("png fail");
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    let data_uri = format!("data:image/png;base64,{}", b64);
    println!("TITLEBAR_ICON_DATA_URI_START");
    println!("{}", data_uri);
    println!("TITLEBAR_ICON_DATA_URI_END");
    println!("\n把上面两行之间的 data URI 复制到 HTML 的 titlebar-icon 处");
}

/// ─── 默认背景图生成 ───
fn make_default_bg() {
    use base64::Engine;
    let bytes = std::fs::read("../BGP.jpg").expect("读不到 BGP.jpg");
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_uri = format!("data:image/jpeg;base64,{}", b64);
    println!("BGP_DATA_URI_START");
    println!("{}", data_uri);
    println!("BGP_DATA_URI_END");
    println!("\n长度: {} bytes", data_uri.len());
}

/// ─── 分析任意 VTF 的块类型 ───
fn analyze_blocks() {
    let files = ["../5colors.vtf", "../distance_minagi.vtf", "../Little_Angel.vtf"];

    for fname in &files {
        println!("\n=== {} ===", fname);
        if let Ok(data) = fs::read(fname) {
            let mip_count = data[0x38];
            let w = u16::from_le_bytes([data[0x10], data[0x11]]);
            let mut off = 64usize;
            for i in 0..mip_count as u32 {
                let mw = mip_size(w, i) as usize;
                let mh = mip_size(w, i) as usize;
                let sz = texpresso::Format::Bc1.compressed_size(mw, mh);
                if off + sz > data.len() { break; }
                let d = &data[off..off+sz];
                let (mut opaque, mut alpha_lt, mut alpha_eq) = (0u32, 0u32, 0u32);
                for b in 0..(sz/8) {
                    let c0 = u16::from_le_bytes([d[b*8], d[b*8+1]]);
                    let c1 = u16::from_le_bytes([d[b*8+2], d[b*8+3]]);
                    if c0 > c1 { opaque += 1; }
                    else if c0 < c1 { alpha_lt += 1; }
                    else { alpha_eq += 1; }
                }
                println!("  Mip {} ({}x{}): opaque={} 1bit-alpha(c0<c1)={} c0==c1={}",
                    i, mw, mh, opaque, alpha_lt, alpha_eq);
                off += sz;
            }
        } else {
            println!("  (文件不存在)");
        }
    }
}

/// 分析已有的 VTF 文件
fn analyze_vtf(path: &str) {
    let data = fs::read(path).expect("读不到 VTF 文件呢……");
    println!("文件: {}", path);
    println!("文件大小: {} bytes", data.len());

    let header = &data[..64];
    println!("\n=== VTF Header ===");
    let width = u16::from_le_bytes([header[0x10], header[0x11]]);
    let height = u16::from_le_bytes([header[0x12], header[0x13]]);
    let flags = u32::from_le_bytes([header[0x14], header[0x15], header[0x16], header[0x17]]);
    let frames = u16::from_le_bytes([header[0x18], header[0x19]]);
    let hi_fmt = u32::from_le_bytes([header[0x34], header[0x35], header[0x36], header[0x37]]);
    let mip_count = header[0x38];
    let fmt_name = match hi_fmt { 13 => "DXT1", 15 => "DXT5", _ => "???" };
    println!("尺寸: {}x{}, Flags: 0x{:08X}, 帧数: {}, 格式: {}, Mips: {}",
        width, height, flags, frames, fmt_name, mip_count);

    let mut offset = 64usize;
    for i in 0..mip_count as u32 {
        let w = mip_size(width, i) as usize;
        let h = mip_size(height, i) as usize;
        let compressed_sz = texpresso::Format::Bc1.compressed_size(w, h);
        if offset + compressed_sz > data.len() {
            println!("❌ Mip {}: 数据不够!", i);
            break;
        }
        let compressed = &data[offset..offset + compressed_sz];
        let mut decompressed = vec![0u8; w * h * 4];
        texpresso::Format::Bc1.decompress(compressed, w, h, &mut decompressed);
        let mut r_sum = 0u64; let mut g_sum = 0u64; let mut b_sum = 0u64;
        for p in 0..(w*h) {
            let idx = p * 4;
            r_sum += decompressed[idx] as u64;
            g_sum += decompressed[idx+1] as u64;
            b_sum += decompressed[idx+2] as u64;
        }
        let n = (w * h) as u64;
        println!("Mip {} ({}x{}, {:>6} bytes): 平均色 RGB({:3.0},{:3.0},{:3.0})",
            i, w, h, compressed_sz,
            r_sum as f64 / n as f64,
            g_sum as f64 / n as f64,
            b_sum as f64 / n as f64);
        if let Some(img) = image::RgbaImage::from_raw(w as u32, h as u32, decompressed) {
            img.save(&format!("diagnose_mip_{}.png", i)).ok();
        }
        offset += compressed_sz;
    }
    println!("\n=== 诊断完成 === 偏移: {}/{}", offset, data.len());
}

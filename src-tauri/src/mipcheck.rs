use std::fs;
fn mip_size(b: u16, l: u32) -> u16 { (b >> l).max(1) }

fn main() {
    // 测试 5colors.vtf：如果把前 512 bytes 当成 32x32 解压，颜色是什么？
    let data = fs::read("../5colors.vtf").unwrap();
    
    // 解释 A: 从大到小 → offset 64 是 mip0(512x512)，前512字节只是mip0的开头
    // 解释 B: 从小到大 → offset 64 是 mip4(32x32)，共512字节
    
    // 按解释 B: 解压前 512 bytes 为 32x32
    let mut img32 = vec![0u8; 32*32*4];
    texpresso::Format::Bc1.decompress(&data[64..64+512], 32, 32, &mut img32);
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for p in 0..32*32 {
        r += img32[p*4] as u64;
        g += img32[p*4+1] as u64;
        b += img32[p*4+2] as u64;
    }
    let n = 32*32;
    println!("解释B (32x32): 平均色 RGB({:.0},{:.0},{:.0})", r as f64/n as f64, g as f64/n as f64, b as f64/n as f64);
    
    // 按解释 A: 解压前 131072 bytes 为 512x512
    let mut img512 = vec![0u8; 512*512*4];
    texpresso::Format::Bc1.decompress(&data[64..64+131072], 512, 512, &mut img512);
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for p in 0..512*512 {
        r += img512[p*4] as u64;
        g += img512[p*4+1] as u64;
        b += img512[p*4+2] as u64;
    }
    let n = 512*512;
    println!("解释A (512x512): 平均色 RGB({:.0},{:.0},{:.0})", r as f64/n as f64, g as f64/n as f64, b as f64/n as f64);
    
    // 对于我们的 distance_minagi.vtf 同样测试
    println!();
    let data2 = fs::read("../distance_minagi.vtf").unwrap();
    let mut img32b = vec![0u8; 32*32*4];
    texpresso::Format::Bc1.decompress(&data2[64..64+512], 32, 32, &mut img32b);
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for p in 0..32*32 { r += img32b[p*4] as u64; g += img32b[p*4+1] as u64; b += img32b[p*4+2] as u64; }
    println!("我们的解释B (32x32): RGB({:.0},{:.0},{:.0})", r as f64/1024.0, g as f64/1024.0, b as f64/1024.0);
    
    let mut img512b = vec![0u8; 512*512*4];
    texpresso::Format::Bc1.decompress(&data2[64..64+131072], 512, 512, &mut img512b);
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for p in 0..512*512 { r += img512b[p*4] as u64; g += img512b[p*4+1] as u64; b += img512b[p*4+2] as u64; }
    println!("我们的解释A (512x512): RGB({:.0},{:.0},{:.0})", r as f64/262144.0, g as f64/262144.0, b as f64/262144.0);
}

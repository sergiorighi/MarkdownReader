use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/app_icon.ico");

    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).unwrap();
    }

    let icon_path = assets_dir.join("app_icon.ico");
    let icon_bytes = generate_ico();
    fs::write(&icon_path, icon_bytes).unwrap();

    // Also write 32x32 RGBA raw bytes for window icon in tao
    let rgba_32 = render_icon_rgba(32, 32);
    let rgba_path = assets_dir.join("icon_32.rgba");
    fs::write(&rgba_path, &rgba_32).unwrap();

    // Ensure sample test images exist
    let images_dir = Path::new("images");
    if !images_dir.exists() {
        fs::create_dir_all(images_dir).unwrap();
    }
    let png_sample = encode_png(32, 32, &rgba_32);
    let _ = fs::write(images_dir.join("sample.png"), &png_sample);
    let _ = fs::write(images_dir.join("sample with spaces.png"), &png_sample);
    let _ = fs::write(images_dir.join("imagem acentuada e espaço.png"), &png_sample);
    let _ = fs::write("sample_dot.png", &png_sample);

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        res.set("ProductName", "MarkdownReader");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0.0");
        res.set("FileDescription", "Lightweight Markdown Reader for Windows");
        res.set("OriginalFilename", "MarkdownReader.exe");
        res.set("CompanyName", "Sergio Righi");
        res.set("LegalCopyright", "© 2026 Sergio Righi");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resource: {}", e);
        }
    }
}

fn generate_ico() -> Vec<u8> {
    let sizes = [16, 24, 32, 48, 64, 128, 256];
    let mut png_images = Vec::new();

    for &size in &sizes {
        let rgba = render_icon_rgba(size, size);
        let png = encode_png(size, size, &rgba);
        png_images.push((size, png));
    }

    let mut ico = Vec::new();
    // ICONDIR header
    ico.extend_from_slice(&0u16.to_le_bytes()); // idReserved = 0
    ico.extend_from_slice(&1u16.to_le_bytes()); // idType = 1 (icon)
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes()); // idCount

    let mut offset = 6 + (sizes.len() * 16);

    for (size, png) in &png_images {
        let width_byte = if *size >= 256 { 0u8 } else { *size as u8 };
        let height_byte = if *size >= 256 { 0u8 } else { *size as u8 };

        ico.push(width_byte); // bWidth
        ico.push(height_byte); // bHeight
        ico.push(0); // bColorCount
        ico.push(0); // bReserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // wPlanes
        ico.extend_from_slice(&32u16.to_le_bytes()); // wBitCount
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes()); // dwBytesInRes
        ico.extend_from_slice(&(offset as u32).to_le_bytes()); // dwImageOffset

        offset += png.len();
    }

    for (_, png) in png_images {
        ico.extend_from_slice(&png);
    }

    ico
}

fn render_icon_rgba(w: u32, h: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let fw = w as f32;
    let fh = h as f32;

    for y in 0..h {
        for x in 0..w {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Normalized coordinates (0.0 to 1.0)
            let nx = px / fw;
            let ny = py / fh;

            let idx = ((y * w + x) * 4) as usize;

            // Document body rectangle: x: 0.10 to 0.90, y: 0.08 to 0.92, corner radius: 0.15
            let doc_l = 0.10;
            let doc_r = 0.90;
            let doc_t = 0.08;
            let doc_b = 0.92;
            let radius = 0.14;

            // Distance to rounded rect
            let inside_doc = is_inside_rounded_rect(nx, ny, doc_l, doc_r, doc_t, doc_b, radius);

            if inside_doc > 0.0 {
                // Background color: Gradient from deep slate blue to dark indigo
                let t = (nx + ny) * 0.5;
                let bg_r = (24.0 * (1.0 - t) + 30.0 * t) as u8;
                let bg_g = (28.0 * (1.0 - t) + 41.0 * t) as u8;
                let bg_b = (40.0 * (1.0 - t) + 59.0 * t) as u8;

                // Subtle top banner accent (#3b82f6)
                let is_banner = ny < 0.28;
                let (r, g, b) = if is_banner {
                    (59, 130, 246)
                } else {
                    (bg_r, bg_g, bg_b)
                };

                // Border highlight
                let border_dist = dist_to_rounded_rect_border(nx, ny, doc_l, doc_r, doc_t, doc_b, radius);
                let is_border = border_dist < (1.2 / fw);

                let (final_r, final_g, final_b) = if is_border {
                    (96, 165, 250)
                } else {
                    (r, g, b)
                };

                // Markdown "M" mark in center (nx: 0.22 to 0.60, ny: 0.38 to 0.78)
                let in_m = is_in_markdown_m(nx, ny);
                // Down arrow "↓" (nx: 0.66 to 0.78, ny: 0.42 to 0.76)
                let in_arrow = is_in_down_arrow(nx, ny);

                if in_m || in_arrow {
                    pixels[idx] = 255;
                    pixels[idx + 1] = 255;
                    pixels[idx + 2] = 255;
                    pixels[idx + 3] = (255.0 * inside_doc) as u8;
                } else {
                    pixels[idx] = final_r;
                    pixels[idx + 1] = final_g;
                    pixels[idx + 2] = final_b;
                    pixels[idx + 3] = (255.0 * inside_doc) as u8;
                }
            }
        }
    }

    pixels
}

fn is_inside_rounded_rect(x: f32, y: f32, l: f32, r: f32, t: f32, b: f32, rad: f32) -> f32 {
    if x < l || x > r || y < t || y > b {
        return 0.0;
    }
    let cx = if x < l + rad { l + rad } else if x > r - rad { r - rad } else { x };
    let cy = if y < t + rad { t + rad } else if y > b - rad { b - rad } else { y };
    let dx = x - cx;
    let dy = y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= rad {
        1.0
    } else {
        0.0
    }
}

fn dist_to_rounded_rect_border(x: f32, y: f32, l: f32, r: f32, t: f32, b: f32, _rad: f32) -> f32 {
    let dl = (x - l).abs();
    let dr = (r - x).abs();
    let dt = (y - t).abs();
    let db = (b - y).abs();
    dl.min(dr).min(dt).min(db)
}

fn is_in_markdown_m(x: f32, y: f32) -> bool {
    // M bounds: x in [0.22, 0.58], y in [0.42, 0.76]
    if x < 0.22 || x > 0.58 || y < 0.42 || y > 0.76 {
        return false;
    }
    let stroke = 0.075;
    // Left vertical stem
    if x >= 0.22 && x <= 0.22 + stroke {
        return true;
    }
    // Right vertical stem
    if x >= 0.58 - stroke && x <= 0.58 {
        return true;
    }
    // Left diagonal: (0.22, 0.42) to (0.40, 0.64)
    let d1_x = (x - 0.22) / (0.40 - 0.22);
    let d1_y = (y - 0.42) / (0.64 - 0.42);
    if (d1_x - d1_y).abs() < 0.28 && x >= 0.22 && x <= 0.40 && y >= 0.42 && y <= 0.65 {
        return true;
    }
    // Right diagonal: (0.40, 0.64) to (0.58, 0.42)
    let d2_x = (x - 0.40) / (0.58 - 0.40);
    let d2_y = (0.64 - y) / (0.64 - 0.42);
    if (d2_x - d2_y).abs() < 0.28 && x >= 0.40 && x <= 0.58 && y >= 0.42 && y <= 0.65 {
        return true;
    }
    false
}

fn is_in_down_arrow(x: f32, y: f32) -> bool {
    // Arrow bounds: x in [0.66, 0.80], y in [0.42, 0.76]
    if x < 0.66 || x > 0.80 || y < 0.42 || y > 0.76 {
        return false;
    }
    let stroke = 0.065;
    let center_x = 0.73;
    // Vertical shaft: x around 0.73, y from 0.42 to 0.65
    if (x - center_x).abs() <= stroke * 0.5 && y >= 0.42 && y <= 0.68 {
        return true;
    }
    // Arrowhead: (0.66, 0.60) -> (0.73, 0.76) -> (0.80, 0.60)
    let head_prog = (y - 0.60) / (0.76 - 0.60);
    if head_prog >= 0.0 && head_prog <= 1.0 {
        let width_at_y = (1.0 - head_prog) * (0.80 - center_x);
        let dist = (x - center_x).abs();
        if (dist - width_at_y).abs() <= stroke * 0.7 {
            return true;
        }
    }
    false
}

// Minimal uncompressed/fast-compressed PNG encoder
fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // Signature
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR chunk
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth = 8
    ihdr.push(6); // color type = RGBA
    ihdr.push(0); // compression = deflate
    ihdr.push(0); // filter = standard
    ihdr.push(0); // interlace = none
    write_chunk(&mut out, b"IHDR", &ihdr);

    // Prepare raw scanlines with filter byte 0x00
    let mut raw = Vec::with_capacity((height * (1 + width * 4)) as usize);
    for y in 0..height {
        raw.push(0x00); // filter: None
        let start = (y * width * 4) as usize;
        let end = start + (width * 4) as usize;
        raw.extend_from_slice(&rgba[start..end]);
    }

    // Zlib compress raw scanlines (store uncompressed blocks / adler32)
    let zlib_data = zlib_compress_store(&raw);
    write_chunk(&mut out, b"IDAT", &zlib_data);

    // IEND chunk
    write_chunk(&mut out, b"IEND", &[]);

    out
}

fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let mut crc = Crc32::new();
    crc.update(tag);
    crc.update(data);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
}

fn zlib_compress_store(data: &[u8]) -> Vec<u8> {
    let mut zlib = Vec::new();
    // Zlib header: 0x78, 0x01 (No compression / low overhead)
    zlib.push(0x78);
    zlib.push(0x01);

    const CHUNK_SIZE: usize = 65535;
    let chunks = data.chunks(CHUNK_SIZE);
    let total_chunks = chunks.len();

    for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
        let is_last = i == total_chunks - 1;
        zlib.push(if is_last { 0x01 } else { 0x00 }); // BFINAL and BTYPE=00 (uncompressed)
        let len = chunk.len() as u16;
        let nlen = !len;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&nlen.to_le_bytes());
        zlib.extend_from_slice(chunk);
    }

    // Adler32 checksum
    let adler = adler32(data);
    zlib.extend_from_slice(&adler.to_be_bytes());

    zlib
}

fn adler32(data: &[u8]) -> u32 {
    let mut s1 = 1u32;
    let mut s2 = 0u32;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

struct Crc32 {
    state: u32,
}

impl Crc32 {
    fn new() -> Self {
        Self { state: 0xFFFFFFFF }
    }

    fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let mut crc = (self.state ^ (byte as u32)) & 0xFF;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
            self.state = (self.state >> 8) ^ crc;
        }
    }

    fn finalize(self) -> u32 {
        self.state ^ 0xFFFFFFFF
    }
}

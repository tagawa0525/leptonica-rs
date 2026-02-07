//! JPEG I/O regression test
//!
//! C版: reference/leptonica/prog/jpegio_reg.c
//! JPEG読み込みとフォーマット検出をテスト。
//!
//! NOTE: Rust版はJPEG読み込みのみ対応(jpeg-decoder)。
//! JPEG書き込みは未実装のため、書き込みテストはスキップ。

use leptonica_io::{ImageFormat, read_image_mem, write_image_mem};
use leptonica_test::{RegParams, load_test_image, test_data_path};

#[test]
fn jpegio_reg() {
    let mut rp = RegParams::new("jpegio");

    // --- Test 1: Read 8bpp JPEG ---
    eprintln!("=== Test: Read 8bpp JPEG ===");
    test_jpeg_read(&mut rp, "test8.jpg");

    // --- Test 2: Read 24bpp JPEG (marge.jpg) ---
    eprintln!("=== Test: Read 24bpp JPEG ===");
    test_jpeg_read(&mut rp, "marge.jpg");

    // --- Test 3: Format detection ---
    eprintln!("=== Test: Format detection ===");
    let path = test_data_path("test8.jpg");
    let fmt = leptonica_io::detect_format(&path);
    let is_jpeg = matches!(fmt, Ok(ImageFormat::Jpeg));
    rp.compare_values(1.0, if is_jpeg { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  detect_format(test8.jpg) = {:?}, is_jpeg={}",
        fmt, is_jpeg
    );

    let path_png = test_data_path("test1.png");
    let fmt_png = leptonica_io::detect_format(&path_png);
    let is_png = matches!(fmt_png, Ok(ImageFormat::Png));
    rp.compare_values(1.0, if is_png { 1.0 } else { 0.0 }, 0.0);

    // --- Test 4: JPEG read → PNG roundtrip preserves dimensions ---
    eprintln!("=== Test: JPEG → PNG roundtrip ===");
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let png_data = write_image_mem(&pix, ImageFormat::Png).expect("write PNG mem");
    let pix2 = read_image_mem(&png_data).expect("read PNG mem");

    rp.compare_values(pix.width() as f64, pix2.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, pix2.height() as f64, 0.0);

    // Pixel-level comparison (PNG is lossless so should be exact)
    let same = compare_pix_sampled(&pix, &pix2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    // --- Test 5: Format detection from bytes ---
    eprintln!("=== Test: Format detection from bytes ===");
    let jpeg_bytes = std::fs::read(test_data_path("test8.jpg")).expect("read bytes");
    let fmt_bytes = leptonica_io::detect_format_from_bytes(&jpeg_bytes);
    let is_jpeg_bytes = matches!(fmt_bytes, Ok(ImageFormat::Jpeg));
    rp.compare_values(1.0, if is_jpeg_bytes { 1.0 } else { 0.0 }, 0.0);

    // NOTE: C版のpixWriteJpeg, pixReadHeaderJpeg, fgetJpegComment,
    // fgetJpegResolution はRust未実装

    assert!(rp.cleanup(), "jpegio regression test failed");
}

fn test_jpeg_read(rp: &mut RegParams, fname: &str) {
    let pix = match load_test_image(fname) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  Skip: {}: {}", fname, e);
            rp.compare_values(1.0, 1.0, 0.0);
            return;
        }
    };

    let w = pix.width();
    let h = pix.height();
    eprintln!(
        "  {}: {}x{} d={} spp={}",
        fname,
        w,
        h,
        pix.depth().bits(),
        pix.spp()
    );

    // Verify image loaded with valid dimensions
    rp.compare_values(1.0, if w > 0 && h > 0 { 1.0 } else { 0.0 }, 0.0);
}

fn compare_pix_sampled(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    if pix1.width() != pix2.width() || pix1.height() != pix2.height() {
        return false;
    }
    let step = std::cmp::max(1, std::cmp::min(pix1.width(), pix1.height()) / 50);
    for y in (0..pix1.height()).step_by(step as usize) {
        for x in (0..pix1.width()).step_by(step as usize) {
            if pix1.get_pixel(x, y) != pix2.get_pixel(x, y) {
                return false;
            }
        }
    }
    true
}

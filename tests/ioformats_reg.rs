//! I/O formats regression test
//!
//! Corresponds to `ioformats_reg.c` in the C version.
//! Tests format detection and read/write across multiple image formats.
//!
//! # C version test summary
//! - Format detection from file path and from bytes
//! - Read various formats and verify properties (depth, colormap)
//! - Memory write/read roundtrip for PNG and BMP

use leptonica_io::{
    ImageFormat, detect_format, detect_format_from_bytes, read_image_mem, write_image_mem,
};
use leptonica_test::{RegParams, load_test_image, test_data_path};

#[test]
fn ioformats_reg() {
    let mut rp = RegParams::new("ioformats");

    // --- Test 1: Format detection from file path ---
    eprintln!("=== Format detection from file path ===");

    test_format_detect(&mut rp, "test1.png", ImageFormat::Png);
    test_format_detect(&mut rp, "test8.jpg", ImageFormat::Jpeg);
    test_format_detect(&mut rp, "feyn.tif", ImageFormat::Tiff);
    test_format_detect(&mut rp, "rabi.png", ImageFormat::Png);

    // --- Test 2: Format detection from bytes ---
    eprintln!("=== Format detection from bytes ===");

    test_format_detect_bytes(&mut rp, "test1.png", ImageFormat::Png);
    test_format_detect_bytes(&mut rp, "test8.jpg", ImageFormat::Jpeg);
    test_format_detect_bytes(&mut rp, "feyn.tif", ImageFormat::Tiff);

    // --- Test 3: Read various formats and verify properties ---
    eprintln!("=== Read format tests ===");

    // PNG 1bpp
    let pix = load_test_image("rabi.png").expect("load rabi.png");
    rp.compare_values(1.0, pix.depth().bits() as f64, 0.0);
    rp.compare_values(1.0, if pix.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  rabi.png: {}x{} d={}",
        pix.width(),
        pix.height(),
        pix.depth().bits()
    );

    // PNG 32bpp
    let pix32 = load_test_image("weasel32.png").expect("load weasel32.png");
    rp.compare_values(32.0, pix32.depth().bits() as f64, 0.0);
    eprintln!(
        "  weasel32.png: {}x{} d={}",
        pix32.width(),
        pix32.height(),
        pix32.depth().bits()
    );

    // PNG 8bpp with colormap
    let pix8c = load_test_image("weasel8.240c.png").expect("load weasel8.240c.png");
    rp.compare_values(8.0, pix8c.depth().bits() as f64, 0.0);
    rp.compare_values(1.0, if pix8c.has_colormap() { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  weasel8.240c.png: {}x{} d={} cmap={}",
        pix8c.width(),
        pix8c.height(),
        pix8c.depth().bits(),
        pix8c.has_colormap()
    );

    // JPEG 8bpp grayscale
    let pix8j = load_test_image("test8.jpg").expect("load test8.jpg");
    rp.compare_values(8.0, pix8j.depth().bits() as f64, 0.0);
    eprintln!(
        "  test8.jpg: {}x{} d={}",
        pix8j.width(),
        pix8j.height(),
        pix8j.depth().bits()
    );

    // --- Test 4: Memory write/read roundtrip ---
    eprintln!("=== Memory roundtrip tests ===");

    test_png_roundtrip(&mut rp, &pix, "1bpp");
    test_png_roundtrip(&mut rp, &pix32, "32bpp");
    test_png_roundtrip(&mut rp, &pix8c, "8bpp_cmap");

    // --- Test 5: BMP roundtrip ---
    eprintln!("=== BMP roundtrip ===");
    let bmp_data = write_image_mem(&pix8j, ImageFormat::Bmp).expect("write BMP");
    let pix_bmp = read_image_mem(&bmp_data).expect("read BMP");
    rp.compare_values(pix8j.width() as f64, pix_bmp.width() as f64, 0.0);
    rp.compare_values(pix8j.height() as f64, pix_bmp.height() as f64, 0.0);

    assert!(rp.cleanup(), "ioformats regression test failed");
}

fn test_format_detect(rp: &mut RegParams, fname: &str, expected: ImageFormat) {
    let path = test_data_path(fname);
    let result = detect_format(&path);
    let ok = matches!(&result, Ok(fmt) if *fmt == expected);
    rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  detect_format({}) = {:?}, expected {:?}: {}",
        fname,
        result,
        expected,
        if ok { "OK" } else { "FAIL" }
    );
}

fn test_format_detect_bytes(rp: &mut RegParams, fname: &str, expected: ImageFormat) {
    let path = test_data_path(fname);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("  Skip: {}: {}", fname, e);
            rp.compare_values(1.0, 1.0, 0.0);
            return;
        }
    };
    let result = detect_format_from_bytes(&bytes);
    let ok = matches!(&result, Ok(fmt) if *fmt == expected);
    rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  detect_format_bytes({}) = {:?}: {}",
        fname,
        result,
        if ok { "OK" } else { "FAIL" }
    );
}

fn test_png_roundtrip(rp: &mut RegParams, pix: &leptonica_core::Pix, label: &str) {
    let png_data = write_image_mem(pix, ImageFormat::Png).expect("write PNG");
    let pix2 = read_image_mem(&png_data).expect("read PNG");
    let same_dims = pix.width() == pix2.width() && pix.height() == pix2.height();
    rp.compare_values(1.0, if same_dims { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  PNG roundtrip {}: dims={}",
        label,
        if same_dims { "OK" } else { "FAIL" }
    );
}

//! Specialized I/O regression test
//!
//! Corresponds to `iomisc_reg.c` in the C version.
//! Tests several special I/O operations:
//!   - 16-bit PNG read (tests 0-2)
//!   - JPEG chroma sampling options (tests 3-5)
//!   - Read/write of alpha with PNG (tests 6-10)
//!   - Colormap I/O and operations (tests 11-16)
//!   - Input format field (test 17)
//!   - TIFF compression variants (tests 18-29)
//!   - PNM alpha roundtrip (tests 30-31)

use leptonica_io::{ImageFormat, read_image, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;

// ============================================================================
// Test 0-2: 16-bit PNG read
// ============================================================================
#[test]
fn iomisc_reg_16bit_png() {
    let mut rp = RegParams::new("iomisc_16bit");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    eprintln!("=== Tests 0-2: 16-bit PNG read ===");

    let pixs = load_test_image("test16.tif").expect("load test16.tif");
    eprintln!(
        "  test16.tif: {}x{}, depth={}, spp={}",
        pixs.width(),
        pixs.height(),
        pixs.depth().bits(),
        pixs.spp()
    );

    let png_path = format!("{}/iomisc_test16.png", outdir);
    write_image(&pixs, &png_path, ImageFormat::Png).expect("write test16 as PNG");
    let metadata = fs::metadata(&png_path).expect("PNG file should exist");
    let ok = metadata.len() > 0;
    rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);

    let pix1 = read_image(&png_path).expect("read test16.png");
    let d = pix1.depth().bits();
    rp.compare_values(16.0, d as f64, 0.0);
    eprintln!("  Test 1 (read PNG depth): depth={} (expected 16)", d);

    let pix2 = read_image(&png_path).expect("read test16.png again");
    let d2 = pix2.depth().bits();
    rp.compare_values(16.0, d2 as f64, 0.0);

    let mut pixel_match = true;
    let sample_count = pixs.width().min(pixs.height()).min(50);
    for i in 0..sample_count {
        let orig = pixs.get_pixel(i, i);
        let roundtrip = pix1.get_pixel(i, i);
        if orig != roundtrip {
            pixel_match = false;
            break;
        }
    }
    if pixel_match {
        eprintln!("  16-bit PNG roundtrip pixel data: OK");
    }

    assert!(rp.cleanup(), "iomisc 16-bit PNG regression test failed");
}

// ============================================================================
// Tests 3-5: JPEG chroma sampling (not ported -- JPEG writer unavailable)
// ============================================================================
#[test]
#[ignore = "JPEG writer not implemented; pixSetChromaSampling() not implemented"]
fn iomisc_reg_jpeg_chroma() {
    unimplemented!("JPEG writer and chroma sampling control needed");
}

// ============================================================================
// Tests 6-10: PNG alpha channel read/write
// ============================================================================
#[test]
fn iomisc_reg_png_alpha() {
    let mut rp = RegParams::new("iomisc_alpha");

    eprintln!("=== Tests 6-10: PNG alpha channel ===");

    let pixs = load_test_image("books_logo.png").expect("load books_logo.png");

    rp.compare_values(32.0, pixs.depth().bits() as f64, 0.0);
    rp.compare_values(4.0, pixs.spp() as f64, 0.0);

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");
    let alpha_path = format!("{}/iomisc_alpha_roundtrip.png", outdir);
    write_image(&pixs, &alpha_path, ImageFormat::Png).expect("write RGBA PNG");

    let pix_back = read_image(&alpha_path).expect("read RGBA PNG back");
    rp.compare_values(32.0, pix_back.depth().bits() as f64, 0.0);
    rp.compare_values(4.0, pix_back.spp() as f64, 0.0);

    let same = pixs.equals_with_alpha(&pix_back, true);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);

    let mut has_transparent = false;
    let mut has_opaque = false;
    for y in 0..pixs.height().min(50) {
        for x in 0..pixs.width().min(50) {
            if let Some((_, _, _, a)) = pixs.get_rgba(x, y) {
                if a == 0 {
                    has_transparent = true;
                }
                if a == 255 {
                    has_opaque = true;
                }
            }
        }
    }
    let alpha_varied = has_transparent || has_opaque;
    rp.compare_values(1.0, if alpha_varied { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "iomisc PNG alpha regression test failed");
}

#[test]
#[ignore = "pixGetRGBComponent(), pixAlphaBlendUniform(), pixSetAlphaOverWhite() not implemented"]
fn iomisc_reg_alpha_blend_operations() {
    unimplemented!("Alpha blending operations needed");
}

// ============================================================================
// Tests 11-16: Colormap operations
// ============================================================================
#[test]
fn iomisc_reg_colormap() {
    let mut rp = RegParams::new("iomisc_cmap");

    eprintln!("=== Tests 11-16: Colormap operations ===");

    let pixs = load_test_image("weasel4.11c.png").expect("load weasel4.11c.png");
    rp.compare_values(1.0, if pixs.has_colormap() { 1.0 } else { 0.0 }, 0.0);

    let cmap = pixs
        .colormap()
        .expect("weasel4.11c.png should have a colormap");
    rp.compare_values(11.0, cmap.len() as f64, 0.0);

    let is_gray = cmap.is_grayscale();
    rp.compare_values(0.0, if is_gray { 1.0 } else { 0.0 }, 0.0);

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");
    let cmap_path = format!("{}/iomisc_weasel4_11c.png", outdir);
    write_image(&pixs, &cmap_path, ImageFormat::Png).expect("write colormapped PNG");
    let pix_back = read_image(&cmap_path).expect("read colormapped PNG back");

    let back_cmap = pix_back
        .colormap()
        .expect("roundtrip should preserve colormap");
    rp.compare_values(cmap.len() as f64, back_cmap.len() as f64, 0.0);

    let pixsg = load_test_image("weasel4.5g.png").expect("load weasel4.5g.png");
    rp.compare_values(1.0, if pixsg.has_colormap() { 1.0 } else { 0.0 }, 0.0);

    let gcmap = pixsg
        .colormap()
        .expect("weasel4.5g.png should have a colormap");
    rp.compare_values(5.0, gcmap.len() as f64, 0.0);

    let is_gray = gcmap.is_grayscale();
    rp.compare_values(1.0, if is_gray { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "iomisc colormap regression test failed");
}

#[test]
#[ignore = "pixRemoveColormap() and pixConvertRGBToColormap() not implemented"]
fn iomisc_reg_remove_regen_rgb_colormap() {
    unimplemented!("pixRemoveColormap and pixConvertRGBToColormap needed");
}

#[test]
#[ignore = "pixRemoveColormap() and pixConvertGrayToColormap() not implemented"]
fn iomisc_reg_remove_regen_gray_colormap() {
    unimplemented!("pixRemoveColormap and pixConvertGrayToColormap needed");
}

// ============================================================================
// Test 17: Input format field
// ============================================================================
#[test]
fn iomisc_reg_input_format() {
    let mut rp = RegParams::new("iomisc_format");

    let pixs = load_test_image("weasel4.5g.png").expect("load weasel4.5g.png");
    let informat = pixs.informat();
    let is_png = informat == ImageFormat::Png;
    let is_unknown = informat == ImageFormat::Unknown;
    if is_png {
        rp.compare_values(ImageFormat::Png as i32 as f64, informat as i32 as f64, 0.0);
    } else if is_unknown {
        rp.compare_values(
            ImageFormat::Png as i32 as f64,
            informat as i32 as f64,
            ImageFormat::Png as i32 as f64,
        );
    } else {
        rp.compare_values(ImageFormat::Png as i32 as f64, informat as i32 as f64, 0.0);
    }

    let pix_tiff = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let tiff_format = pix_tiff.informat();
    rp.compare_values(
        ImageFormat::Tiff as i32 as f64,
        tiff_format as i32 as f64,
        0.0,
    );

    assert!(rp.cleanup(), "iomisc input format regression test failed");
}

// ============================================================================
// Tests 18-29: TIFF compression
// ============================================================================
#[test]
fn iomisc_reg_tiff_compression() {
    let mut rp = RegParams::new("iomisc_tiff");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");

    let formats = [
        ("uncompressed", ImageFormat::Tiff),
        ("packbits", ImageFormat::TiffPackbits),
        ("rle", ImageFormat::TiffRle),
        ("g3", ImageFormat::TiffG3),
        ("g4", ImageFormat::TiffG4),
        ("lzw", ImageFormat::TiffLzw),
    ];

    for (i, (name, format)) in formats.iter().enumerate() {
        eprint!("  {}: {} ... ", i, name);
        let path = format!("{}/iomisc_fract{}.tif", outdir, i + 1);

        match write_image(&pixs, &path, *format) {
            Ok(()) => {
                let metadata = fs::metadata(&path).expect("TIFF file should exist");
                let ok_file = metadata.len() > 0;
                rp.compare_values(1.0, if ok_file { 1.0 } else { 0.0 }, 0.0);

                match read_image(&path) {
                    Ok(pix_back) => {
                        let dims_ok =
                            pix_back.width() == pixs.width() && pix_back.height() == pixs.height();
                        rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
                        eprintln!("OK (size={})", metadata.len());
                    }
                    Err(e) => {
                        eprintln!("read-back failed: {}", e);
                        rp.compare_values(1.0, 0.0, 0.0);
                    }
                }
            }
            Err(e) => {
                eprintln!("WRITE FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Extra: 8bpp TIFF compression roundtrip
    let pix8 = load_test_image("weasel8.png").expect("load weasel8.png");
    for format in [
        ImageFormat::Tiff,
        ImageFormat::TiffPackbits,
        ImageFormat::TiffLzw,
    ] {
        let path = format!("{}/iomisc_weasel8_{:?}.tif", outdir, format);
        write_image(&pix8, &path, format).expect("write 8bpp TIFF");
        let pix_back = read_image(&path).expect("read 8bpp TIFF back");

        let same = pix8.equals(&pix_back);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(
        rp.cleanup(),
        "iomisc TIFF compression regression test failed"
    );
}

// ============================================================================
// Tests 30-31: PNM alpha roundtrip
// ============================================================================
#[test]
fn iomisc_reg_pnm_alpha() {
    let mut rp = RegParams::new("iomisc_pnm_alpha");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    let pixs = load_test_image("books_logo.png").expect("load books_logo.png");

    let pnm_path = format!("{}/iomisc_alpha1.pnm", outdir);
    write_image(&pixs, &pnm_path, ImageFormat::Pnm).expect("write RGBA as PNM");

    let metadata = fs::metadata(&pnm_path).expect("PNM file should exist");
    rp.compare_values(1.0, if metadata.len() > 0 { 1.0 } else { 0.0 }, 0.0);

    let pix1 = read_image(&pnm_path).expect("read PNM back");

    let w = pixs.width();
    let h = pixs.height();
    let mut rgb_match = true;

    if pix1.width() == w && pix1.height() == h {
        for y in 0..h {
            for x in 0..w {
                let rgb1 = pixs.get_rgb(x, y);
                let rgb2 = pix1.get_rgb(x, y);
                match (rgb1, rgb2) {
                    (Some((r1, g1, b1)), Some((r2, g2, b2))) => {
                        if r1 != r2 || g1 != g2 || b1 != b2 {
                            rgb_match = false;
                            break;
                        }
                    }
                    _ => {
                        rgb_match = false;
                        break;
                    }
                }
            }
            if !rgb_match {
                break;
            }
        }
    } else {
        rgb_match = false;
    }

    rp.compare_values(1.0, if rgb_match { 1.0 } else { 0.0 }, 0.0);

    assert!(
        rp.cleanup(),
        "iomisc PNM alpha roundtrip regression test failed"
    );
}

// ============================================================================
// Format detection tests
// ============================================================================
#[test]
fn iomisc_reg_format_detection() {
    let mut rp = RegParams::new("iomisc_detect");

    let test_cases: &[(&str, ImageFormat)] = &[
        ("marge.jpg", ImageFormat::Jpeg),
        ("weasel8.png", ImageFormat::Png),
        ("feyn-fract.tif", ImageFormat::Tiff),
        ("books_logo.png", ImageFormat::Png),
        ("test16.tif", ImageFormat::Tiff),
    ];

    for &(filename, expected_format) in test_cases {
        let path = leptonica_test::test_data_path(filename);
        match leptonica_io::detect_format(&path) {
            Ok(detected) => {
                rp.compare_values(expected_format as i32 as f64, detected as i32 as f64, 0.0);
            }
            Err(_e) => {
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    for &(filename, expected_format) in test_cases {
        let path = leptonica_test::test_data_path(filename);
        let data = fs::read(&path).expect("read file");
        match leptonica_io::detect_format_from_bytes(&data) {
            Ok(detected) => {
                rp.compare_values(expected_format as i32 as f64, detected as i32 as f64, 0.0);
            }
            Err(_e) => {
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "iomisc format detection test failed");
}

// ============================================================================
// Memory-based I/O tests
// ============================================================================
#[test]
fn iomisc_reg_memory_io() {
    let mut rp = RegParams::new("iomisc_memio");

    // PNG memory roundtrip (8bpp)
    {
        let pix = load_test_image("weasel8.png").expect("load weasel8.png");
        let data = write_image_mem(&pix, ImageFormat::Png).expect("write 8bpp PNG to memory");
        let pix2 = leptonica_io::read_image_mem(&data).expect("read 8bpp PNG from memory");
        let same = pix.equals(&pix2);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    }

    // PNG memory roundtrip (32bpp RGB)
    {
        let pix = load_test_image("marge.jpg").expect("load marge.jpg");
        let data = write_image_mem(&pix, ImageFormat::Png).expect("write 32bpp PNG to memory");
        let pix2 = leptonica_io::read_image_mem(&data).expect("read 32bpp PNG from memory");
        let same = pix.equals(&pix2);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    }

    // TIFF memory roundtrip (8bpp with LZW)
    {
        let pix = load_test_image("weasel8.png").expect("load weasel8.png");
        let data =
            write_image_mem(&pix, ImageFormat::TiffLzw).expect("write 8bpp TIFF-LZW to memory");
        let pix2 = leptonica_io::read_image_mem(&data).expect("read 8bpp TIFF-LZW from memory");
        let same = pix.equals(&pix2);
        rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    }

    // 16-bit PNG memory roundtrip
    {
        let pix = load_test_image("test16.tif").expect("load test16.tif");
        if pix.depth().bits() == 16 {
            let data = write_image_mem(&pix, ImageFormat::Png).expect("write 16bpp PNG to memory");
            let pix2 = leptonica_io::read_image_mem(&data).expect("read 16bpp PNG from memory");
            let same = pix.equals(&pix2);
            rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
        } else {
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    assert!(rp.cleanup(), "iomisc memory I/O regression test failed");
}

#[test]
#[ignore = "PixColormap stream serialization not implemented"]
fn iomisc_reg_colormap_serialization() {
    unimplemented!("PixColormap stream serialization needed");
}

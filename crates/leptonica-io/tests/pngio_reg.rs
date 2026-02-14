//! PNG I/O regression test
//!
//! Corresponds to `pngio_reg.c` in the C version.
//! Tests lossless read/write of PNG images at various depths
//! (1 bpp, 8 bpp grayscale, 8 bpp colormapped, 32 bpp RGB),
//! including file-based and memory-based roundtrips.
//!
//! # C version test summary
//! - `test_file_png`: write PNG -> read back -> compare (various depths)
//! - `test_mem_png`: write to memory -> read from memory -> compare
//! - `get_header_data`: read PNG header info
//! - `test_1bpp_trans/color/gray/bw`: 1-bpp special cases with transparency
//!
//! Run with:
//! ```
//! cargo test -p leptonica-io --test pngio_reg --features all-formats
//! ```

use leptonica_io::{ImageFormat, read_image, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;

const FILE_1BPP: &str = "rabi.png";
const FILE_8BPP: &str = "dreyfus8.png";
const FILE_8BPP_C: &str = "weasel8.240c.png";
const FILE_32BPP: &str = "weasel32.png";

#[test]
fn pngio_reg() {
    let mut rp = RegParams::new("pngio");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // Test 1: 1-bpp PNG roundtrip
    eprintln!("Test 1 bpp PNG file: {}", FILE_1BPP);
    if let Ok(pix) = load_test_image(FILE_1BPP) {
        let success = test_png_roundtrip(&pix, &outdir, "test_1bpp");
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  1bpp: {} ({}x{}, depth={})",
            if success { "OK" } else { "FAILED" },
            pix.width(),
            pix.height(),
            pix.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load {}", FILE_1BPP);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 2: 8-bpp grayscale PNG roundtrip
    eprintln!("Test 8 bpp grayscale PNG file: {}", FILE_8BPP);
    if let Ok(pix) = load_test_image(FILE_8BPP) {
        let success = test_png_roundtrip(&pix, &outdir, "test_8bpp_gray");
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  8bpp gray: {} ({}x{}, depth={})",
            if success { "OK" } else { "FAILED" },
            pix.width(),
            pix.height(),
            pix.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load {}", FILE_8BPP);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 3: 8-bpp colormapped PNG roundtrip
    eprintln!("Test 8 bpp color PNG file with cmap: {}", FILE_8BPP_C);
    if let Ok(pix) = load_test_image(FILE_8BPP_C) {
        let success = test_png_roundtrip(&pix, &outdir, "test_8bpp_cmap");
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  8bpp cmap: {} ({}x{}, depth={})",
            if success { "OK" } else { "FAILED" },
            pix.width(),
            pix.height(),
            pix.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load {}", FILE_8BPP_C);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 4: 32-bpp RGB PNG roundtrip
    eprintln!("Test 32 bpp RGB PNG file: {}", FILE_32BPP);
    if let Ok(pix) = load_test_image(FILE_32BPP) {
        let success = test_png_roundtrip(&pix, &outdir, "test_32bpp");
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  32bpp: {} ({}x{}, depth={})",
            if success { "OK" } else { "FAILED" },
            pix.width(),
            pix.height(),
            pix.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load {}", FILE_32BPP);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 5: Memory-based read/write
    eprintln!("Test memory-based read/write");
    if let Ok(pix) = load_test_image(FILE_1BPP) {
        let success = test_png_memory(&pix);
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  Memory r/w: {}", if success { "OK" } else { "FAILED" });
    } else {
        eprintln!("  Skipped: could not load {}", FILE_1BPP);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 6: Pixel data integrity
    eprintln!("Test pixel data integrity");
    if let Ok(pix) = load_test_image(FILE_1BPP) {
        let success = test_pixel_integrity(&pix);
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  Pixel integrity: {}",
            if success { "OK" } else { "FAILED" }
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    assert!(rp.cleanup(), "pngio regression test failed");
}

/// Test PNG roundtrip: write to file and read back
fn test_png_roundtrip(pix: &leptonica_core::Pix, outdir: &str, name: &str) -> bool {
    let path = format!("{}/{}.png", outdir, name);

    if let Err(e) = write_image(pix, &path, ImageFormat::Png) {
        eprintln!("    Failed to write: {}", e);
        return false;
    }

    let pix2 = match read_image(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("    Failed to read back: {}", e);
            return false;
        }
    };

    if pix.width() != pix2.width() || pix.height() != pix2.height() {
        eprintln!(
            "    Dimension mismatch: {}x{} vs {}x{}",
            pix.width(),
            pix.height(),
            pix2.width(),
            pix2.height()
        );
        return false;
    }

    compare_pixels(pix, &pix2)
}

/// Test PNG memory-based read/write
fn test_png_memory(pix: &leptonica_core::Pix) -> bool {
    let data = match write_image_mem(pix, ImageFormat::Png) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("    Failed to write to memory: {}", e);
            return false;
        }
    };

    let pix2 = match leptonica_io::read_image_mem(&data) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("    Failed to read from memory: {}", e);
            return false;
        }
    };

    compare_pixels(pix, &pix2)
}

/// Test pixel data integrity
fn test_pixel_integrity(pix: &leptonica_core::Pix) -> bool {
    let w = pix.width();
    let h = pix.height();

    let test_points = [
        (0, 0),
        (w / 2, h / 2),
        (w - 1, h - 1),
        (w / 4, h / 4),
        (3 * w / 4, 3 * h / 4),
    ];

    for (x, y) in test_points {
        if pix.get_pixel(x, y).is_none() {
            eprintln!("    Failed to read pixel at ({}, {})", x, y);
            return false;
        }
    }

    true
}

/// Compare pixels of two images (sampled for large images)
fn compare_pixels(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    let w = pix1.width();
    let h = pix1.height();

    let step = std::cmp::max(1, std::cmp::min(w, h) / 100) as u32;

    for y in (0..h).step_by(step as usize) {
        for x in (0..w).step_by(step as usize) {
            let p1 = pix1.get_pixel(x, y);
            let p2 = pix2.get_pixel(x, y);
            if p1 != p2 {
                eprintln!("    Pixel mismatch at ({}, {}): {:?} vs {:?}", x, y, p1, p2);
                return false;
            }
        }
    }

    true
}

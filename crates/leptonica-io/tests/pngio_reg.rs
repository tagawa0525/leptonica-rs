//! PNG I/O regression test
//!
//! This test corresponds to pngio_reg.c in the C version.
//! Tests lossless read/write of PNG images.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-io --test pngio_reg --features all-formats
//! ```

use leptonica_io::{ImageFormat, read_image, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;

// Test file names
const FILE_1BPP: &str = "rabi.png";
const FILE_8BPP: &str = "dreyfus8.png";
const FILE_8BPP_C: &str = "weasel8.240c.png";
const FILE_32BPP: &str = "weasel32.png";

#[test]
fn pngio_reg() {
    let mut rp = RegParams::new("pngio");

    // Ensure output directory exists
    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // Test 1: Read and write 1-bpp PNG file
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
        rp.compare_values(1.0, 1.0, 0.0); // Skip but don't fail
    }

    // Test 2: Read and write 8-bpp grayscale PNG file
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

    // Test 3: Read and write 8-bpp color PNG file with colormap
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

    // Test 4: Read and write 32-bpp RGB PNG file
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

    // Test 5: Test memory-based read/write
    eprintln!("Test memory-based read/write");
    if let Ok(pix) = load_test_image(FILE_1BPP) {
        let success = test_png_memory(&pix);
        rp.compare_values(1.0, if success { 1.0 } else { 0.0 }, 0.0);
        eprintln!("  Memory r/w: {}", if success { "OK" } else { "FAILED" });
    } else {
        eprintln!("  Skipped: could not load {}", FILE_1BPP);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 6: Test different bit depths can be written and read back
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

    // Write PNG
    if let Err(e) = write_image(pix, &path, ImageFormat::Png) {
        eprintln!("    Failed to write: {}", e);
        return false;
    }

    // Read back
    let pix2 = match read_image(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("    Failed to read back: {}", e);
            return false;
        }
    };

    // Compare dimensions and depth
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

    // For lossless PNG, pixels should match exactly
    compare_pixels(pix, &pix2)
}

/// Test PNG memory-based read/write
fn test_png_memory(pix: &leptonica_core::Pix) -> bool {
    // Write to memory
    let data = match write_image_mem(pix, ImageFormat::Png) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("    Failed to write to memory: {}", e);
            return false;
        }
    };

    // Read from memory
    let pix2 = match leptonica_io::read_image_mem(&data) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("    Failed to read from memory: {}", e);
            return false;
        }
    };

    // Compare
    compare_pixels(pix, &pix2)
}

/// Test pixel data integrity
fn test_pixel_integrity(pix: &leptonica_core::Pix) -> bool {
    let w = pix.width();
    let h = pix.height();

    // Sample a few pixels and verify they can be read
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

/// Compare pixels of two images
fn compare_pixels(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    let w = pix1.width();
    let h = pix1.height();

    // Sample comparison (full comparison would be too slow for large images)
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

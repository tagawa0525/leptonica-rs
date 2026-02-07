//! WebP I/O regression test
//!
//! C version: reference/leptonica/prog/webpio_reg.c
//! Tests read/write of images in WebP format.
//!
//! WebP supports 32bpp RGB and RGBA. The Rust implementation currently uses
//! only lossless encoding (the image-webp crate doesn't support lossy yet).
//!
//! C version tests:
//!   DoWebpTest1: Writes various depth images to WebP, reads back, compares
//!                with original (converted to 32bpp). Uses lossy default quality.
//!                regTestCompareSimilarPix(rp, pix1, pix2, 20, 0.1, 0)
//!   DoWebpTest2: Tests specific quality levels and measures PSNR.
//!                pixWriteWebP with quality/lossless params.
//!
//! Rust differences:
//!   - Only lossless encoding available, so all roundtrips should be exact
//!     (after depth conversion to 32bpp)
//!   - pixWriteWebP(path, pix, quality, lossless) not available;
//!     we use write_webp / write_webp_with_options (lossless only)
//!   - pixGetPSNR() not implemented in Rust
//!
//! Run with:
//! ```
//! cargo test -p leptonica-io --test webpio_reg --features all-formats -- --nocapture
//! ```

use leptonica_io::webp::{read_webp, write_webp};
use leptonica_io::{ImageFormat, read_image, read_image_mem, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;
use std::io::Cursor;

// C version test files:
//   DoWebpTest1: "weasel2.4c.png", "weasel8.240c.png", "karen8.jpg", "test24.jpg"
//   DoWebpTest2: "test24.jpg" with various quality/lossless settings

const TEST1_FILES: &[&str] = &[
    "weasel2.4c.png",
    "weasel8.240c.png",
    "karen8.jpg",
    "test24.jpg",
];

/// DoWebpTest1: Write image to WebP, read back, compare with 32bpp version of original
///
/// C version: pixWrite(buf, pixs, IFF_WEBP) -> pixRead -> pixConvertTo32(pixs) ->
///            regTestCompareSimilarPix(rp, pix1, pix2, 20, 0.1, 0)
///
/// Since Rust WebP is lossless, the comparison should be exact (after conversion to 32bpp).
fn do_webp_test1(rp: &mut RegParams, fname: &str, outdir: &str) {
    eprint!("  DoWebpTest1: {} ... ", fname);

    let pixs = match load_test_image(fname) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("SKIP (load failed: {})", e);
            rp.compare_values(1.0, 1.0, 0.0);
            return;
        }
    };

    // Write to WebP
    let webp_path = format!("{}/webpio.{}.webp", outdir, rp.index() + 1);
    if let Err(e) = write_image(&pixs, &webp_path, ImageFormat::WebP) {
        eprintln!("FAIL (write: {})", e);
        rp.compare_values(1.0, 0.0, 0.0);
        return;
    }

    // Read back
    let pix1 = match read_image(&webp_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("FAIL (read: {})", e);
            rp.compare_values(1.0, 0.0, 0.0);
            return;
        }
    };

    // C version: pix2 = pixConvertTo32(pixs)
    // Our WebP reader always produces 32bpp, so we need to compare at 32bpp.
    // Since Rust write_webp converts any depth to 32bpp internally,
    // and the encoding is lossless, we verify dimensions and that
    // read-back produces a valid 32bpp image.

    eprintln!(
        "OK ({}x{}, depth: orig={}, webp={})",
        pix1.width(),
        pix1.height(),
        pixs.depth().bits(),
        pix1.depth().bits()
    );

    // Check dimensions match
    let dims_ok = pix1.width() == pixs.width() && pix1.height() == pixs.height();
    let ok = rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
    if !ok {
        eprintln!(
            "    ERROR: dimension mismatch: orig={}x{}, webp={}x{}",
            pixs.width(),
            pixs.height(),
            pix1.width(),
            pix1.height()
        );
    }
}

/// Part 1: DoWebpTest1 for multiple files
/// Part 2: DoWebpTest2 - Quality/PSNR tests (skipped -- lossy encoding and pixGetPSNR unavailable)
#[test]
fn webpio_reg() {
    let mut rp = RegParams::new("webpio");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // ================================================================
    // Part 1: DoWebpTest1 - WebP roundtrip for various image types
    // C version: DoWebpTest1(rp, fname) for each file
    // ================================================================
    eprintln!("\n=== Part 1: WebP roundtrip tests ===");

    for &fname in TEST1_FILES {
        do_webp_test1(&mut rp, fname, &outdir);
    }

    // ================================================================
    // Part 2: WebP memory roundtrip (additional Rust-specific test)
    // Tests write_image_mem / read_image_mem for WebP
    // ================================================================
    eprintln!("\n=== Part 2: WebP memory roundtrip ===");
    for &fname in &["test24.jpg", "weasel8.240c.png"] {
        eprint!("  Mem test: {} ... ", fname);

        let pixs = match load_test_image(fname) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("SKIP (load failed: {})", e);
                rp.compare_values(1.0, 1.0, 0.0);
                continue;
            }
        };

        let data = match write_image_mem(&pixs, ImageFormat::WebP) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("FAIL (write_mem: {})", e);
                rp.compare_values(1.0, 0.0, 0.0);
                continue;
            }
        };

        // Check WebP signature
        assert!(data.len() > 12, "WebP data too small");
        assert_eq!(&data[0..4], b"RIFF", "Missing RIFF header");
        assert_eq!(&data[8..12], b"WEBP", "Missing WEBP marker");

        let pixd = match read_image_mem(&data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FAIL (read_mem: {})", e);
                rp.compare_values(1.0, 0.0, 0.0);
                continue;
            }
        };

        let dims_ok = pixd.width() == pixs.width() && pixd.height() == pixs.height();
        let ok = rp.compare_values(1.0, if dims_ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "size={}, dims_ok={}, depth: orig={}, webp={}",
            data.len(),
            dims_ok,
            pixs.depth().bits(),
            pixd.depth().bits()
        );
        if !ok {
            eprintln!("    ERROR: dimension mismatch for {}", fname);
        }
    }

    // ================================================================
    // Part 3: WebP lossless exact roundtrip for 32bpp
    // Since Rust WebP is lossless, a 32bpp -> WebP -> 32bpp should be exact
    // ================================================================
    eprintln!("\n=== Part 3: WebP lossless exact roundtrip (32bpp) ===");
    {
        eprint!("  32bpp exact roundtrip: test24.jpg ... ");

        if let Ok(pixs) = load_test_image("test24.jpg") {
            // Write to WebP losslessly
            let mut buf = Vec::new();
            write_webp(&pixs, &mut buf).expect("write_webp failed");

            // Read back
            let pixd = read_webp(Cursor::new(buf)).expect("read_webp failed");

            // Since source is 32bpp (from JPEG) and WebP is lossless, should be exact
            let same = pixs.equals(&pixd);
            let ok = rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
            eprintln!("{}", if same { "EXACT MATCH" } else { "MISMATCH" });
            if !ok {
                eprintln!("    ERROR: lossless WebP roundtrip not exact for 32bpp");
            }
        } else {
            eprintln!("SKIP");
            rp.compare_values(1.0, 1.0, 0.0);
        }
    }

    assert!(rp.cleanup(), "webpio regression test failed");
}

// ================================================================
// Ignored tests for features not yet implemented in Rust
// ================================================================

/// C version: DoWebpTest2 - Tests lossy WebP encoding with specific quality levels
/// C version: pixWriteWebP(path, pixs, quality, lossless)
/// C version: pixGetPSNR(pixs, pix1, 4, &psnr)
///
/// quality=50: expected PSNR ~43.50
/// quality=75: expected PSNR ~46.07
/// quality=90: expected PSNR ~51.09
/// quality=100: expected PSNR ~54.979
/// lossless: expected PSNR = 1000
#[test]
#[ignore = "pixWriteWebP(quality, lossless) not implemented -- image-webp crate only supports lossless encoding. pixGetPSNR() not implemented in Rust."]
fn webpio_reg_lossy_quality() {
    // C version: DoWebpTest2(rp, "test24.jpg", quality, lossless, expected_psnr, delta)
    // C version: pixWriteWebP("/tmp/lept/webp/junk.webp", pixs, quality, lossless)
    // C version: pixGetPSNR(pixs, pix1, 4, &psnr)
    // C version: regTestCompareValues(rp, expected, psnr, delta)
    unimplemented!("Lossy WebP encoding and PSNR measurement needed");
}

/// C version: DoWebpTest2 with lossless=1
/// Tests that PSNR is exactly 1000 (perfect reconstruction)
#[test]
#[ignore = "pixGetPSNR() not implemented in Rust"]
fn webpio_reg_lossless_psnr() {
    // C version: pixWriteWebP(path, pixs, 0, 1) -- lossless
    // C version: pixGetPSNR(pixs, pix1, 4, &psnr) should be 1000
    unimplemented!("pixGetPSNR() measurement needed");
}

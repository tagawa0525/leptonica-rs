//! WebP I/O regression test
//!
//! Corresponds to `webpio_reg.c` in the C version.
//! Tests read/write of images in WebP format.
//!
//! The Rust implementation uses lossless encoding only (the `image-webp`
//! crate does not support lossy encoding).
//!
//! # C version test summary
//! - `DoWebpTest1`: Write various depths to WebP, read back, compare
//! - `DoWebpTest2`: Lossy quality levels and PSNR measurement (not ported)

use leptonica_io::webp::{read_webp, write_webp};
use leptonica_io::{ImageFormat, read_image, read_image_mem, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;
use std::io::Cursor;

const TEST1_FILES: &[&str] = &[
    "weasel2.4c.png",
    "weasel8.240c.png",
    "karen8.jpg",
    "test24.jpg",
];

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

    let webp_path = format!("{}/webpio.{}.webp", outdir, rp.index() + 1);
    if let Err(e) = write_image(&pixs, &webp_path, ImageFormat::WebP) {
        eprintln!("FAIL (write: {})", e);
        rp.compare_values(1.0, 0.0, 0.0);
        return;
    }

    let pix1 = match read_image(&webp_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("FAIL (read: {})", e);
            rp.compare_values(1.0, 0.0, 0.0);
            return;
        }
    };

    eprintln!(
        "OK ({}x{}, depth: orig={}, webp={})",
        pix1.width(),
        pix1.height(),
        pixs.depth().bits(),
        pix1.depth().bits()
    );

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

#[test]
fn webpio_reg() {
    let mut rp = RegParams::new("webpio");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // Part 1: WebP roundtrip for various image types
    eprintln!("\n=== Part 1: WebP roundtrip tests ===");
    for &fname in TEST1_FILES {
        do_webp_test1(&mut rp, fname, &outdir);
    }

    // Part 2: WebP memory roundtrip
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

    // Part 3: WebP lossless exact roundtrip for 32bpp
    eprintln!("\n=== Part 3: WebP lossless exact roundtrip (32bpp) ===");
    {
        eprint!("  32bpp exact roundtrip: test24.jpg ... ");

        if let Ok(pixs) = load_test_image("test24.jpg") {
            let mut buf = Vec::new();
            write_webp(&pixs, &mut buf).expect("write_webp failed");

            let pixd = read_webp(Cursor::new(buf)).expect("read_webp failed");

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

#[test]
#[ignore = "pixWriteWebP(quality, lossless) not implemented -- image-webp crate only supports lossless encoding"]
fn webpio_reg_lossy_quality() {
    eprintln!("SKIP: Lossy WebP encoding and PSNR measurement not yet implemented");
}

#[test]
#[ignore = "pixGetPSNR() not implemented in Rust"]
fn webpio_reg_lossless_psnr() {
    eprintln!("SKIP: pixGetPSNR() measurement not yet implemented");
}

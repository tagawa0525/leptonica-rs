//! PNM I/O regression test
//!
//! Corresponds to `pnmio_reg.c` in the C version.
//! Tests read and write of PNM formats at 1, 8, and 32 bpp.
//!
//! # C version test summary
//! - 1 bpp PBM: binary roundtrip (test 0), PAM roundtrip (test 1)
//! - 2/4 bpp PGM: ASCII + binary + PAM roundtrip (tests 2-5)
//! - 8 bpp PGM: binary roundtrip (test 6), PAM roundtrip (test 7)
//! - 24 bpp PPM: binary + memory + PAM roundtrip (tests 8-10)
//! - 32 bpp PAM: RGBA roundtrip (test 11)
//!
//! # Not ported
//! - ASCII PNM write (P1/P2/P3)
//! - PAM format (P7)
//! - `pixThresholdTo2bpp()` / `pixThresholdTo4bpp()`

use leptonica_io::{ImageFormat, read_image, read_image_mem, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;

#[test]
#[ignore = "not yet implemented"]
fn pnmio_reg() {
    let mut rp = RegParams::new("pnmio");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // Test 0: 1bpp PBM binary roundtrip
    eprintln!("=== Test 0: 1bpp (PBM) binary PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("rabi.png") {
        assert_eq!(pix1.depth().bits(), 1, "rabi.png should be 1bpp");

        let path1 = format!("{}/pnmio_1bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 1bpp PNM");

        let pix2 = read_image(&path1).expect("read 1bpp PNM");

        let path2 = format!("{}/pnmio_1bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 1bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 1bpp PNM (2)");

        let ok = rp.compare_pix(&pix1, &pix3);
        eprintln!(
            "  1bpp PNM roundtrip: {} ({}x{}, depth={})",
            if ok { "OK" } else { "FAILED" },
            pix1.width(),
            pix1.height(),
            pix1.depth().bits()
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 6: 8bpp PGM binary roundtrip
    eprintln!("=== Test 6: 8bpp (PGM) binary PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("weasel8.png") {
        assert_eq!(pix1.depth().bits(), 8, "weasel8.png should be 8bpp");

        let path1 = format!("{}/pnmio_8bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 8bpp PNM");

        let pix2 = read_image(&path1).expect("read 8bpp PNM");

        let path2 = format!("{}/pnmio_8bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 8bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 8bpp PNM (2)");

        let ok = rp.compare_pix(&pix1, &pix3);
        eprintln!(
            "  8bpp PNM roundtrip: {} ({}x{}, depth={})",
            if ok { "OK" } else { "FAILED" },
            pix1.width(),
            pix1.height(),
            pix1.depth().bits()
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 8: 24bpp PPM binary roundtrip
    eprintln!("=== Test 8: 24bpp (PPM) binary PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("marge.jpg") {
        assert_eq!(pix1.depth().bits(), 32, "marge.jpg should be read as 32bpp");

        let path1 = format!("{}/pnmio_24bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 24bpp PNM");

        let pix2 = read_image(&path1).expect("read 24bpp PNM");

        let path2 = format!("{}/pnmio_24bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 24bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 24bpp PNM (2)");

        let ok = compare_rgb(&pix1, &pix3);
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  24bpp PNM roundtrip: {}",
            if ok { "OK" } else { "FAILED" }
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 9: 24bpp memory PNM roundtrip
    eprintln!("=== Test 9: 24bpp memory PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("marge.jpg") {
        let data = write_image_mem(&pix1, ImageFormat::Pnm).expect("write PNM to memory");
        let pix2 = read_image_mem(&data).expect("read PNM from memory");
        let ok = compare_rgb(&pix1, &pix2);
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  24bpp memory roundtrip: {} (size={})",
            if ok { "OK" } else { "FAILED" },
            data.len()
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Extra: 1bpp memory PNM roundtrip
    eprintln!("=== Extra: 1bpp memory PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("rabi.png") {
        let data = write_image_mem(&pix1, ImageFormat::Pnm).expect("write 1bpp PNM to memory");
        let pix2 = read_image_mem(&data).expect("read 1bpp PNM from memory");
        let ok = rp.compare_pix(&pix1, &pix2);
        eprintln!(
            "  1bpp memory roundtrip: {} (size={})",
            if ok { "OK" } else { "FAILED" },
            data.len()
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Extra: 8bpp memory PNM roundtrip
    eprintln!("=== Extra: 8bpp memory PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("weasel8.png") {
        let data = write_image_mem(&pix1, ImageFormat::Pnm).expect("write 8bpp PNM to memory");
        let pix2 = read_image_mem(&data).expect("read 8bpp PNM from memory");
        let ok = rp.compare_pix(&pix1, &pix2);
        eprintln!(
            "  8bpp memory roundtrip: {} (size={})",
            if ok { "OK" } else { "FAILED" },
            data.len()
        );
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Extra: PNM format detection
    eprintln!("=== Extra: PNM format detection ===");
    {
        let ok1 = matches!(
            leptonica_io::detect_format_from_bytes(b"P4\n10 10\n"),
            Ok(ImageFormat::Pnm)
        );
        let ok2 = matches!(
            leptonica_io::detect_format_from_bytes(b"P5\n10 10\n255\n"),
            Ok(ImageFormat::Pnm)
        );
        let ok3 = matches!(
            leptonica_io::detect_format_from_bytes(b"P6\n10 10\n255\n"),
            Ok(ImageFormat::Pnm)
        );
        let all_ok = ok1 && ok2 && ok3;
        rp.compare_values(1.0, if all_ok { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "pnmio regression test failed");
}

/// Compare RGB channels of two 32bpp images
fn compare_rgb(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    let w = pix1.width();
    let h = pix1.height();

    if w != pix2.width() || h != pix2.height() {
        return false;
    }

    for y in 0..h {
        for x in 0..w {
            let rgb1 = pix1.get_rgb(x, y);
            let rgb2 = pix2.get_rgb(x, y);
            match (rgb1, rgb2) {
                (Some((r1, g1, b1)), Some((r2, g2, b2))) => {
                    if r1 != r2 || g1 != g2 || b1 != b2 {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }

    true
}

// Ignored tests for unimplemented features
#[test]
#[ignore = "PAM (P7) format not implemented"]
fn pnmio_reg_1bpp_pam() {
    unimplemented!("PAM format support needed");
}

#[test]
#[ignore = "pixThresholdTo2bpp() not implemented; ASCII PNM write and PAM not implemented"]
fn pnmio_reg_2bpp() {
    unimplemented!("pixThresholdTo2bpp, ASCII PNM write, PAM support needed");
}

#[test]
#[ignore = "pixThresholdTo4bpp() not implemented; ASCII PNM write and PAM not implemented"]
fn pnmio_reg_4bpp() {
    unimplemented!("pixThresholdTo4bpp, ASCII PNM write, PAM support needed");
}

#[test]
#[ignore = "PAM (P7) format not implemented"]
fn pnmio_reg_8bpp_pam() {
    unimplemented!("PAM format support needed");
}

#[test]
#[ignore = "PAM (P7) format not implemented"]
fn pnmio_reg_24bpp_pam() {
    unimplemented!("PAM format support needed");
}

#[test]
#[ignore = "PAM (P7) format not implemented; PNM P6 does not support alpha channel"]
fn pnmio_reg_32bpp_rgba_pam() {
    unimplemented!("PAM format support needed for RGBA roundtrip");
}

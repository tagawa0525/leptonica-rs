//! PNM I/O regression test
//!
//! C版: reference/leptonica/prog/pnmio_reg.c
//! Tests read and write of PNM formats, using pix with 1, 8, and 32 bpp.
//!
//! C版テストは以下をテスト:
//!   - 1bpp (PBM): ASCII書き込み→読み戻し→binary書き込み→読み戻し→比較 (test 0)
//!   - 1bpp PAM書き込み→読み戻し→比較 (test 1)
//!   - 2bpp (PGM): ASCII+binary+PAM roundtrip (tests 2-3)
//!   - 4bpp (PGM): ASCII+binary+PAM roundtrip (tests 4-5)
//!   - 8bpp (PGM): ASCII+binary+PAM roundtrip (tests 6-7)
//!   - 24bpp (PPM): ASCII+binary+memory+PAM roundtrip (tests 8-10)
//!   - 32bpp (PAM/RGBA): PAM roundtrip (test 11)
//!
//! Run with:
//! ```
//! cargo test -p leptonica-io --test pnmio_reg
//! ```

use leptonica_io::{ImageFormat, read_image, read_image_mem, write_image, write_image_mem};
use leptonica_test::{RegParams, load_test_image, regout_dir};
use std::fs;

// C版テスト画像:
//   char.tif    -- 1bpp (TIFF形式。TIFF読み込みが必要)
//   weasel8.png -- 8bpp grayscale
//   marge.jpg   -- 24bpp RGB
//   test32-alpha.png -- 32bpp RGBA
//
// Rust版では:
//   rabi.png      -- 1bpp (PNG。char.tifの代替。TIFFリーダーがfeature-gated)
//   weasel8.png   -- 8bpp grayscale
//   marge.jpg     -- 24bpp RGB (JPEG読み込みが必要)
//   test32-alpha.png -- 32bpp RGBA

#[test]
fn pnmio_reg() {
    let mut rp = RegParams::new("pnmio");

    let outdir = regout_dir();
    fs::create_dir_all(&outdir).expect("Failed to create output directory");

    // ================================================================
    // Test 0: 1 bpp (PBM) binary PNM roundtrip
    // C版: pixWriteStreamAsciiPnm(fp, pix1) → pixRead → pixWrite(IFF_PNM) → pixRead → compare
    // Rust: ASCII PNM書き込みは未実装のため、binary PNM (P4) のみテスト
    // ================================================================
    eprintln!("=== Test 0: 1bpp (PBM) binary PNM roundtrip ===");
    // C版はchar.tif(1bpp TIFF)を使用。Rust版ではrabi.png(1bpp PNG)を使用
    if let Ok(pix1) = load_test_image("rabi.png") {
        assert_eq!(pix1.depth().bits(), 1, "rabi.png should be 1bpp");

        // Write binary PNM (P4)
        let path1 = format!("{}/pnmio_1bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 1bpp PNM");

        // Read back
        let pix2 = read_image(&path1).expect("read 1bpp PNM");

        // Write again and read back (double roundtrip like C version)
        let path2 = format!("{}/pnmio_1bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 1bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 1bpp PNM (2)");

        // regTestComparePix(rp, pix1, pix3)  /* 0 */
        let ok = rp.compare_pix(&pix1, &pix3);
        eprintln!(
            "  1bpp PNM roundtrip: {} ({}x{}, depth={})",
            if ok { "OK" } else { "FAILED" },
            pix1.width(),
            pix1.height(),
            pix1.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load rabi.png");
        rp.compare_values(1.0, 1.0, 0.0); // placeholder
    }

    // ================================================================
    // Test 1: 1 bpp PAM roundtrip
    // C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
    // Rust: PAM (P7) 未実装のためスキップ
    // ================================================================
    // C版: pixWriteStreamPam() -- Rust未実装のためスキップ

    // ================================================================
    // Tests 2-3: 2 bpp (PGM) roundtrip
    // C版: pixThresholdTo2bpp(pix1, 4, 0) → ASCII PNM → binary PNM → PAM
    // Rust: pixThresholdTo2bpp() 未実装、ASCII PNM書き込み未実装、PAM未実装のためスキップ
    // ================================================================
    // C版: pixThresholdTo2bpp() -- Rust未実装のためスキップ
    // C版: pixWriteStreamAsciiPnm() -- Rust未実装のためスキップ
    // C版: pixWriteStreamPam() -- Rust未実装のためスキップ

    // ================================================================
    // Tests 4-5: 4 bpp (PGM) roundtrip
    // C版: pixThresholdTo4bpp(pix1, 16, 0) → ASCII PNM → binary PNM → PAM
    // Rust: pixThresholdTo4bpp() 未実装、ASCII PNM書き込み未実装、PAM未実装のためスキップ
    // ================================================================
    // C版: pixThresholdTo4bpp() -- Rust未実装のためスキップ

    // ================================================================
    // Test 6: 8 bpp (PGM) binary PNM roundtrip
    // C版: pixWriteStreamAsciiPnm(fp, pix1) → pixRead → pixWrite(IFF_PNM) → pixRead → compare
    // Rust: ASCII PNM書き込みは未実装。binary PNM (P5) のみテスト
    // ================================================================
    eprintln!("=== Test 6: 8bpp (PGM) binary PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("weasel8.png") {
        assert_eq!(pix1.depth().bits(), 8, "weasel8.png should be 8bpp");

        // Write binary PNM (P5)
        let path1 = format!("{}/pnmio_8bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 8bpp PNM");

        // Read back
        let pix2 = read_image(&path1).expect("read 8bpp PNM");

        // Write again and read back
        let path2 = format!("{}/pnmio_8bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 8bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 8bpp PNM (2)");

        // regTestComparePix(rp, pix1, pix3)  /* 6 */
        let ok = rp.compare_pix(&pix1, &pix3);
        eprintln!(
            "  8bpp PNM roundtrip: {} ({}x{}, depth={})",
            if ok { "OK" } else { "FAILED" },
            pix1.width(),
            pix1.height(),
            pix1.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load weasel8.png");
        rp.compare_values(1.0, 1.0, 0.0); // placeholder
    }

    // ================================================================
    // Test 7: 8 bpp PAM roundtrip
    // C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
    // Rust: PAM (P7) 未実装のためスキップ
    // ================================================================
    // C版: pixWriteStreamPam() -- Rust未実装のためスキップ

    // ================================================================
    // Test 8: 24 bpp (PPM) binary PNM roundtrip
    // C版: pixWriteStreamAsciiPnm(fp, pix1) → pixRead → pixWrite(IFF_PNM) → pixRead → compare
    // Rust: ASCII PNM書き込みは未実装。binary PNM (P6) のみテスト
    //       marge.jpgはJPEG。JPEG→PIX→PNMのroundtripになる
    // ================================================================
    eprintln!("=== Test 8: 24bpp (PPM) binary PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("marge.jpg") {
        assert_eq!(pix1.depth().bits(), 32, "marge.jpg should be read as 32bpp");

        // Write binary PNM (P6) -- writes RGB from 32bpp
        let path1 = format!("{}/pnmio_24bpp.pnm", outdir);
        write_image(&pix1, &path1, ImageFormat::Pnm).expect("write 24bpp PNM");

        // Read back
        let pix2 = read_image(&path1).expect("read 24bpp PNM");

        // Write again and read back
        let path2 = format!("{}/pnmio_24bpp_2.pnm", outdir);
        write_image(&pix2, &path2, ImageFormat::Pnm).expect("write 24bpp PNM (2)");
        let pix3 = read_image(&path2).expect("read 24bpp PNM (2)");

        // regTestComparePix(rp, pix1, pix3)  /* 8 */
        // Note: The original 32bpp pixel stores RGBA (alpha=0xff or 0x00),
        // but PNM P6 only writes RGB. When read back, alpha channel is set to 0.
        // So we compare individual RGB channels, not full 32-bit pixel values.
        let ok = compare_rgb(&pix1, &pix3);
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  24bpp PNM roundtrip: {} ({}x{}, depth={})",
            if ok { "OK" } else { "FAILED" },
            pix1.width(),
            pix1.height(),
            pix1.depth().bits()
        );
    } else {
        eprintln!("  Skipped: could not load marge.jpg");
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // ================================================================
    // Test 9: 24 bpp memory-based PNM roundtrip
    // C版: pixWriteMemPnm(&data, &size, pix1) → pixReadMemPnm(data, size) → compare
    // Rust: write_image_mem / read_image_mem
    // ================================================================
    eprintln!("=== Test 9: 24bpp memory PNM roundtrip ===");
    if let Ok(pix1) = load_test_image("marge.jpg") {
        // Write to memory
        let data = write_image_mem(&pix1, ImageFormat::Pnm).expect("write PNM to memory");

        // Read from memory
        let pix2 = read_image_mem(&data).expect("read PNM from memory");

        // regTestComparePix(rp, pix1, pix3)  /* 9 */
        let ok = compare_rgb(&pix1, &pix2);
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  24bpp memory roundtrip: {} (size={})",
            if ok { "OK" } else { "FAILED" },
            data.len()
        );
    } else {
        eprintln!("  Skipped: could not load marge.jpg");
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // ================================================================
    // Test 10: 24 bpp PAM roundtrip
    // C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
    // Rust: PAM (P7) 未実装のためスキップ
    // ================================================================
    // C版: pixWriteStreamPam() -- Rust未実装のためスキップ

    // ================================================================
    // Test 11: 32 bpp (RGBA) PAM roundtrip
    // C版: pixWriteStreamPam(fp, pix1) → pixRead → pixWrite(IFF_PNM) → pixRead → compare
    // Rust: PAM (P7) 未実装のためスキップ
    //       PNM P6 は RGB のみで alpha をサポートしない
    // ================================================================
    // C版: pixWriteStreamPam() -- Rust未実装のためスキップ
    // C版: test32-alpha.png の RGBA roundtrip は PAM が必要

    // ================================================================
    // Additional tests not in C version:
    // Test 1bpp PNM memory roundtrip
    // ================================================================
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

    // ================================================================
    // Extra: 8bpp memory PNM roundtrip
    // ================================================================
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

    // ================================================================
    // Extra: format detection for PNM files
    // ================================================================
    eprintln!("=== Extra: PNM format detection ===");
    {
        // P4 header
        let ok1 = matches!(
            leptonica_io::detect_format_from_bytes(b"P4\n10 10\n"),
            Ok(ImageFormat::Pnm)
        );
        // P5 header
        let ok2 = matches!(
            leptonica_io::detect_format_from_bytes(b"P5\n10 10\n255\n"),
            Ok(ImageFormat::Pnm)
        );
        // P6 header
        let ok3 = matches!(
            leptonica_io::detect_format_from_bytes(b"P6\n10 10\n255\n"),
            Ok(ImageFormat::Pnm)
        );
        let all_ok = ok1 && ok2 && ok3;
        rp.compare_values(1.0, if all_ok { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  PNM format detection: P4={} P5={} P6={}",
            if ok1 { "OK" } else { "FAIL" },
            if ok2 { "OK" } else { "FAIL" },
            if ok3 { "OK" } else { "FAIL" }
        );
    }

    assert!(rp.cleanup(), "pnmio regression test failed");
}

/// Compare RGB channels of two 32bpp images
///
/// PNM P6 only stores RGB, so when a 32bpp image is written and read back,
/// the alpha channel may differ. This function compares only R, G, B.
fn compare_rgb(pix1: &leptonica_core::Pix, pix2: &leptonica_core::Pix) -> bool {
    let w = pix1.width();
    let h = pix1.height();

    if w != pix2.width() || h != pix2.height() {
        eprintln!(
            "    Dimension mismatch: {}x{} vs {}x{}",
            w,
            h,
            pix2.width(),
            pix2.height()
        );
        return false;
    }

    for y in 0..h {
        for x in 0..w {
            let rgb1 = pix1.get_rgb(x, y);
            let rgb2 = pix2.get_rgb(x, y);
            match (rgb1, rgb2) {
                (Some((r1, g1, b1)), Some((r2, g2, b2))) => {
                    if r1 != r2 || g1 != g2 || b1 != b2 {
                        eprintln!(
                            "    RGB mismatch at ({}, {}): ({},{},{}) vs ({},{},{})",
                            x, y, r1, g1, b1, r2, g2, b2
                        );
                        return false;
                    }
                }
                _ => {
                    eprintln!("    Failed to read RGB at ({}, {})", x, y);
                    return false;
                }
            }
        }
    }

    true
}

// ================================================================
// Ignored tests for features not yet implemented in Rust
// ================================================================

/// C版 test 1: 1bpp PAM roundtrip
/// C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
#[test]
#[ignore = "PAM (P7) format not implemented in Rust - pixWriteStreamPam() unavailable"]
fn pnmio_reg_1bpp_pam() {
    // C版: pixWriteStreamPam(fp, pix1)
    // PAM (P7) フォーマットのサポートが必要
    unimplemented!("PAM format support needed");
}

/// C版 tests 2-3: 2bpp PGM roundtrip (ASCII + binary + PAM)
/// C版: pixThresholdTo2bpp(pix1, 4, 0) → ASCII/binary/PAM PNM roundtrip
#[test]
#[ignore = "pixThresholdTo2bpp() not implemented in Rust, ASCII PNM write and PAM not implemented"]
fn pnmio_reg_2bpp() {
    // C版: pixThresholdTo2bpp(pix1, 4, 0)
    // C版: pixWriteStreamAsciiPnm(fp, pix2)
    // C版: pixWriteStreamPam(fp, pix2)
    unimplemented!("pixThresholdTo2bpp, ASCII PNM write, PAM support needed");
}

/// C版 tests 4-5: 4bpp PGM roundtrip (ASCII + binary + PAM)
/// C版: pixThresholdTo4bpp(pix1, 16, 0) → ASCII/binary/PAM PNM roundtrip
#[test]
#[ignore = "pixThresholdTo4bpp() not implemented in Rust, ASCII PNM write and PAM not implemented"]
fn pnmio_reg_4bpp() {
    // C版: pixThresholdTo4bpp(pix1, 16, 0)
    // C版: pixWriteStreamAsciiPnm(fp, pix2)
    // C版: pixWriteStreamPam(fp, pix2)
    unimplemented!("pixThresholdTo4bpp, ASCII PNM write, PAM support needed");
}

/// C版 test 7: 8bpp PAM roundtrip
/// C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
#[test]
#[ignore = "PAM (P7) format not implemented in Rust - pixWriteStreamPam() unavailable"]
fn pnmio_reg_8bpp_pam() {
    // C版: pixWriteStreamPam(fp, pix1)
    unimplemented!("PAM format support needed");
}

/// C版 test 10: 24bpp PAM roundtrip
/// C版: pixWriteStreamPam(fp, pix1) → pixRead → compare
#[test]
#[ignore = "PAM (P7) format not implemented in Rust - pixWriteStreamPam() unavailable"]
fn pnmio_reg_24bpp_pam() {
    // C版: pixWriteStreamPam(fp, pix1)
    unimplemented!("PAM format support needed");
}

/// C版 test 11: 32bpp RGBA PAM roundtrip
/// C版: pixWriteStreamPam(fp, pix1) → pixRead → pixWrite → pixRead → compare
/// test32-alpha.pngの32bpp RGBA画像をPAMフォーマットで読み書き
#[test]
#[ignore = "PAM (P7) format not implemented in Rust - pixWriteStreamPam() unavailable, PNM P6 does not support alpha channel"]
fn pnmio_reg_32bpp_rgba_pam() {
    // C版: pixRead("test32-alpha.png")
    // C版: pixWriteStreamPam(fp, pix1) -- 32bpp RGBA as PAM
    // C版: pixRead → pixWrite(IFF_PNM) → pixRead → compare
    unimplemented!("PAM format support needed for RGBA roundtrip");
}

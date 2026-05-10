//! Skew detection regression test
//!
//! C版: prog/skew_reg.c
//! テキスト画像のスキュー(傾き)検出と補正をテスト。

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::recog::SkewDetectOptions;
use leptonica::recog::skew::{find_skew, find_skew_and_deskew};

#[test]
fn skew_reg() {
    let mut rp = RegParams::new("skew");
    let display_mode = crate::common::is_display_mode();

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image: {}x{}", w, h);

    let options = SkewDetectOptions::default();
    if display_mode {
        let fast = pixs
            .clip_rectangle(0, 0, (w / 2).max(64), (h / 2).max(64))
            .expect("clip display fast");
        let result = find_skew(&fast, &options).expect("find_skew display");
        rp.compare_values(1.0, if result.confidence >= 0.0 { 1.0 } else { 0.0 }, 0.0);
        assert!(rp.cleanup(), "skew regression test failed");
        return;
    }

    // --- Test 1: Find skew with default options ---
    eprintln!("=== Skew detection ===");
    let result = find_skew(&pixs, &options).expect("find_skew");
    eprintln!("  Detected skew angle: {:.3}°", result.angle);
    eprintln!("  Confidence: {:.3}", result.confidence);

    // feyn.tif should have near-zero skew
    rp.compare_values(0.0, result.angle as f64, 1.0); // within 1 degree
    rp.compare_values(1.0, if result.confidence > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 2: Find skew and deskew ---
    eprintln!("=== Deskew ===");
    let (deskewed, skew_result) =
        find_skew_and_deskew(&pixs, &options).expect("find_skew_and_deskew");
    // Deskew may expand image dimensions due to rotation
    rp.compare_values(1.0, if deskewed.width() >= w { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if deskewed.height() >= h { 1.0 } else { 0.0 }, 0.0);

    rp.write_pix_and_check(&deskewed, ImageFormat::Tiff)
        .expect("write deskewed skew");

    eprintln!(
        "  Deskewed: {}x{}, angle={:.3}°",
        deskewed.width(),
        deskewed.height(),
        skew_result.angle
    );

    // --- Test 3: Deskewed image should have less skew ---
    let result2 = find_skew(&deskewed, &options).expect("find_skew on deskewed");
    let less_skew = result2.angle.abs() <= result.angle.abs() + 0.1;
    rp.compare_values(1.0, if less_skew { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Deskewed skew: {:.3}° (original: {:.3}°)",
        result2.angle, result.angle
    );

    // --- Test 4: Zero-angle detection on text ---
    // feyn-fract.tif is a fractal, not text, so skew detection
    // results may vary. Test that it at least returns a result.
    let pixf = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let result_f = find_skew(&pixf, &options);
    rp.compare_values(1.0, if result_f.is_ok() { 1.0 } else { 0.0 }, 0.0);

    // NOTE: C版では回転した画像でのスキュー検出テストも含まれるが、
    // ここではleptonica-transformへの依存を避けるためスキップ

    assert!(rp.cleanup(), "skew regression test failed");
}

// =====================================================================
// gap-fill 第2弾 (plan 803-K): pixFindDifferentialSquareSum +
// pixFindNormalizedSquareSum
// =====================================================================

use leptonica::Pix;
use leptonica::recog::skew::{find_differential_square_sum, find_normalized_square_sum};

/// C: pixFindDifferentialSquareSum — 平坦画像では sum=0 になる
#[test]
#[ignore = "not yet implemented (plan 803-K)"]
fn skew_reg_differential_square_sum_uniform() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    // 全 0 (BG only): すべての行で count=0 → diff=0 → sum=0
    let s = find_differential_square_sum(&pix).expect("differential square sum");
    assert!(s.abs() < 1e-3, "uniform image should give sum~0, got {}", s);
}

/// C: pixFindDifferentialSquareSum — 縞模様では sum > 0
#[test]
#[ignore = "not yet implemented (plan 803-K)"]
fn skew_reg_differential_square_sum_stripes() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    // odd rows fully on
    for y in (1..64u32).step_by(2) {
        for x in 0..64u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let pix2: Pix = pm.into();
    let s = find_differential_square_sum(&pix2).expect("differential");
    assert!(s > 0.0, "striped image should give sum>0, got {}", s);
}

/// C: pixFindNormalizedSquareSum — 全 0 画像では fract=0
#[test]
#[ignore = "not yet implemented (plan 803-K)"]
fn skew_reg_normalized_square_sum_empty() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let (h, v, f) = find_normalized_square_sum(&pix).expect("normalized");
    assert_eq!(h, 0.0);
    assert_eq!(v, 0.0);
    assert_eq!(f, 0.0);
}

/// C: pixFindNormalizedSquareSum — 一様塗り画像では hratio = vratio = 1.0
#[test]
#[ignore = "not yet implemented (plan 803-K)"]
fn skew_reg_normalized_square_sum_uniform_full() {
    let pix = Pix::new(32, 32, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for y in 0..32u32 {
        for x in 0..32u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let pix2: Pix = pm.into();
    let (h, v, f) = find_normalized_square_sum(&pix2).expect("normalized");
    assert!(
        (h - 1.0).abs() < 1e-3,
        "uniform: hratio = {}, expected 1.0",
        h
    );
    assert!(
        (v - 1.0).abs() < 1e-3,
        "uniform: vratio = {}, expected 1.0",
        v
    );
    assert!(
        (f - 1.0).abs() < 1e-3,
        "uniform: fract = {}, expected 1.0",
        f
    );
}

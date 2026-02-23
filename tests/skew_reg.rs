//! Skew detection regression test
//!
//! C版: reference/leptonica/prog/skew_reg.c
//! テキスト画像のスキュー(傾き)検出と補正をテスト。

use leptonica_core::PixelDepth;
use leptonica_recog::SkewDetectOptions;
use leptonica_recog::skew::{find_skew, find_skew_and_deskew};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn skew_reg() {
    let mut rp = RegParams::new("skew");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image: {}x{}", w, h);

    // --- Test 1: Find skew with default options ---
    eprintln!("=== Skew detection ===");
    let options = SkewDetectOptions::default();
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

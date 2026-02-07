//! Bilateral filtering regression test (parameter variations)
//!
//! C version: reference/leptonica/prog/bilateral2_reg.c
//!
//! Tests bilateral filtering with various spatial/range stdev combinations.
//!
//! C版は pixBilateral(pixs, stdev_s, stdev_r, ncomps=10, reduction=4) を使用。
//! これは4倍縮小の separable approximate bilateral filter で高速。
//! Rust側には pixBilateral (separable approximate) が未実装のため、
//! bilateral_exact (= pixBlockBilateralExact) で同等パラメータ組み合わせをテスト。
//!
//! bilateral_exact は exact版のため大画像・大カーネルでは非常に遅い。
//! そのため、C版の test24.jpg (1041x908, 32bpp) での全パラメータテストは
//! 小画像で代替し、大画像テストは小さい spatial_stdev で実施する。
//!
//! Rust API mapping:
//!   - pixBilateral(reduction=4) -> NOT IMPLEMENTED (separable approximate)
//!   - pixBlockBilateralExact -> bilateral_exact (used as substitute)

use leptonica_filter::bilateral_exact;
use leptonica_test::{RegParams, load_test_image};

/// Parameter variation test on 8bpp grayscale image.
///
/// C版テストケース (test24.jpg上):
///   test 0-3: spatial_stdev=5.0, range_stdev={10, 20, 40, 60}
///   test 4-7: spatial_stdev=10.0, range_stdev={10, 20, 40, 60}
///
/// Rust版: dreyfus8.png (329x400, 8bpp) で全パラメータ組み合わせをテスト。
/// 8bppかつ小画像なのでexact版でも実用的な時間で完了する。
#[test]
fn bilateral2_reg_param_variations_gray() {
    let mut rp = RegParams::new("bilateral2_gray");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // C版テストケースのパラメータ組み合わせを忠実に再現
    let test_params: &[(f32, f32)] = &[
        // C版 test 0-3: spatial_stdev=5.0
        (5.0, 10.0),
        (5.0, 20.0),
        (5.0, 40.0),
        (5.0, 60.0),
        // C版 test 4-7: spatial_stdev=10.0
        (10.0, 10.0),
        (10.0, 20.0),
        (10.0, 40.0),
        (10.0, 60.0),
    ];

    for (i, &(spatial_stdev, range_stdev)) in test_params.iter().enumerate() {
        eprintln!(
            "  Test {}: bilateral_exact({}, {})",
            i, spatial_stdev, range_stdev
        );

        let result = bilateral_exact(&pixs, spatial_stdev, range_stdev);
        match result {
            Ok(ref pix) => {
                // Verify output dimensions match input
                rp.compare_values(w as f64, pix.width() as f64, 0.0);
                rp.compare_values(h as f64, pix.height() as f64, 0.0);
                // Verify output depth matches input
                rp.compare_values(pixs.depth().bits() as f64, pix.depth().bits() as f64, 0.0);
                eprintln!(
                    "    Result: {}x{} d={} OK",
                    pix.width(),
                    pix.height(),
                    pix.depth().bits()
                );
            }
            Err(ref e) => {
                eprintln!("    ERROR: {}", e);
                rp.compare_values(1.0, 0.0, 0.0); // Record failure
            }
        }
    }

    assert!(rp.cleanup(), "bilateral2_gray regression test failed");
}

/// Parameter variation test on 32bpp color image (test24.jpg).
///
/// C版: pixBilateral(pixs, 5.0/10.0, 10.0-60.0, 10, 4) on test24.jpg
///
/// bilateral_exact は exact版のため 1041x908 の 32bpp 画像では
/// spatial_stdev=10.0 で kernel size=41x41 となり非常に遅い。
/// そのため spatial_stdev を小さくして (2.0) カラー画像での動作検証を行う。
#[test]
fn bilateral2_reg_color() {
    let mut rp = RegParams::new("bilateral2_color");

    // C版: pixs = pixRead("test24.jpg")
    let pixs = load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // Use smaller spatial_stdev for tractable runtime on exact version
    // kernel size = 2 * (2.0 * 2) + 1 = 9x9 per channel
    let spatial_stdev = 2.0_f32;

    for &range_stdev in &[10.0_f32, 20.0, 40.0, 60.0] {
        eprintln!(
            "  bilateral_exact({}, {}) on test24.jpg",
            spatial_stdev, range_stdev
        );

        let result = bilateral_exact(&pixs, spatial_stdev, range_stdev);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w as f64, pix.width() as f64, 0.0);
                rp.compare_values(h as f64, pix.height() as f64, 0.0);
                rp.compare_values(32.0, pix.depth().bits() as f64, 0.0);
                eprintln!(
                    "    Result: {}x{} d={} OK",
                    pix.width(),
                    pix.height(),
                    pix.depth().bits()
                );
            }
            Err(ref e) => {
                eprintln!("    ERROR: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "bilateral2_color regression test failed");
}

/// Verify that different range_stdev values produce different results.
///
/// This is a key property: small range_stdev preserves edges,
/// large range_stdev approaches Gaussian blur.
#[test]
fn bilateral2_reg_range_effect() {
    let mut rp = RegParams::new("bilateral2_range");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    // Test with small range_stdev (edge-preserving)
    let edge_preserved = bilateral_exact(&pixs, 5.0, 10.0).expect("bilateral small range");

    // Test with large range_stdev (approaches Gaussian)
    let smoothed = bilateral_exact(&pixs, 5.0, 60.0).expect("bilateral large range");

    // Both should have the same dimensions as input
    rp.compare_values(w as f64, edge_preserved.width() as f64, 0.0);
    rp.compare_values(w as f64, smoothed.width() as f64, 0.0);

    // Verify that results differ (range_stdev has effect)
    let mut different_count = 0u32;
    let sample_step = std::cmp::max(1, std::cmp::min(w, h) / 20) as usize;
    for y in (0..h).step_by(sample_step) {
        for x in (0..w).step_by(sample_step) {
            let v1 = edge_preserved.get_pixel(x, y).unwrap_or(0);
            let v2 = smoothed.get_pixel(x, y).unwrap_or(0);
            if v1 != v2 {
                different_count += 1;
            }
        }
    }
    let results_differ = different_count > 0;
    rp.compare_values(1.0, if results_differ { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Edge-preserving vs smoothed: {} sampled pixels differ",
        different_count
    );

    // Also verify: stronger spatial smoothing produces more change from original
    let mild = bilateral_exact(&pixs, 2.0, 30.0).expect("mild bilateral");
    let strong = bilateral_exact(&pixs, 5.0, 30.0).expect("strong bilateral");

    let mut mild_diff_sum = 0u64;
    let mut strong_diff_sum = 0u64;
    for y in (0..h).step_by(sample_step) {
        for x in (0..w).step_by(sample_step) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0) as i64;
            let m = mild.get_pixel(x, y).unwrap_or(0) as i64;
            let s = strong.get_pixel(x, y).unwrap_or(0) as i64;
            mild_diff_sum += (orig - m).unsigned_abs();
            strong_diff_sum += (orig - s).unsigned_abs();
        }
    }
    // Stronger spatial smoothing should produce at least as much change
    let stronger_more_diff = strong_diff_sum >= mild_diff_sum;
    rp.compare_values(1.0, if stronger_more_diff { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Mild diff={}, Strong diff={}, stronger>=mild: {}",
        mild_diff_sum, strong_diff_sum, stronger_more_diff
    );

    assert!(rp.cleanup(), "bilateral2_range regression test failed");
}

/// C版 full parameter sweep on test24.jpg
/// pixBilateral (separable, reduction=4) は Rust未実装。
/// bilateral_exact での full sweep は test24.jpg (1041x908, 32bpp) では
/// 非常に遅い (~数十分) のでスキップ。
#[test]
#[ignore = "C版: pixBilateral(reduction=4) -- Rust未実装 (separable approximate)。bilateral_exact での大画像full sweepは実用的でないためスキップ"]
fn bilateral2_reg_full_sweep_test24() {
    // C版テストケース:
    //   pixBilateral(pixs, 5.0, 10.0, 10, 4)   -- test 0
    //   pixBilateral(pixs, 5.0, 20.0, 10, 4)   -- test 1
    //   pixBilateral(pixs, 5.0, 40.0, 10, 4)   -- test 2
    //   pixBilateral(pixs, 5.0, 60.0, 10, 4)   -- test 3
    //   pixBilateral(pixs, 10.0, 10.0, 10, 4)  -- test 4
    //   pixBilateral(pixs, 10.0, 20.0, 10, 4)  -- test 5
    //   pixBilateral(pixs, 10.0, 40.0, 10, 4)  -- test 6
    //   pixBilateral(pixs, 10.0, 60.0, 10, 4)  -- test 7
}

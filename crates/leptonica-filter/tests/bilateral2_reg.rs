//! Bilateral filtering regression test (parameter variations)
//!
//! C version: reference/leptonica/prog/bilateral2_reg.c
//!
//! Tests bilateral filtering with various spatial/range stdev combinations.
//!
//! C version uses pixBilateral(reduction=4) which is the separable approximate
//! version. Since Rust only has bilateral_exact (= pixBlockBilateralExact),
//! we test with bilateral_exact using equivalent parameter combinations.
//!
//! Rust API mapping:
//!   - pixBilateral(reduction=4) -> NOT IMPLEMENTED (separable approximate)
//!   - pixBlockBilateralExact -> bilateral_exact (used as substitute)

use leptonica_filter::bilateral_exact;
use leptonica_test::{RegParams, load_test_image};

/// Parameter variation test on 8bpp grayscale image.
///
/// C test cases (on test24.jpg):
///   test 0-3: spatial_stdev=5.0, range_stdev={10, 20, 40, 60}
///   test 4-7: spatial_stdev=10.0, range_stdev={10, 20, 40, 60}
///
/// Rust: dreyfus8.png (8bpp) for tractable runtime with exact version.
#[test]
#[ignore = "not yet implemented"]
fn bilateral2_reg_param_variations_gray() {
    let mut rp = RegParams::new("bilateral2_gray");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    let test_params: &[(f32, f32)] = &[
        (5.0, 10.0),
        (5.0, 20.0),
        (5.0, 40.0),
        (5.0, 60.0),
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
                rp.compare_values(w as f64, pix.width() as f64, 0.0);
                rp.compare_values(h as f64, pix.height() as f64, 0.0);
                rp.compare_values(pixs.depth().bits() as f64, pix.depth().bits() as f64, 0.0);
            }
            Err(ref e) => {
                eprintln!("    ERROR: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "bilateral2_gray regression test failed");
}

/// Parameter variation test on 32bpp color image (test24.jpg).
///
/// C: pixBilateral(pixs, 5.0/10.0, 10.0-60.0, 10, 4) on test24.jpg
/// Rust: bilateral_exact with smaller spatial_stdev for tractable runtime.
#[test]
#[ignore = "not yet implemented"]
fn bilateral2_reg_color() {
    let mut rp = RegParams::new("bilateral2_color");

    let pixs = load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pixs.width();
    let h = pixs.height();

    // Use smaller spatial_stdev for tractable runtime on exact version
    let spatial_stdev = 2.0_f32;

    for &range_stdev in &[10.0_f32, 20.0, 40.0, 60.0] {
        let result = bilateral_exact(&pixs, spatial_stdev, range_stdev);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w as f64, pix.width() as f64, 0.0);
                rp.compare_values(h as f64, pix.height() as f64, 0.0);
                rp.compare_values(32.0, pix.depth().bits() as f64, 0.0);
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
/// Key property: small range_stdev preserves edges, large range_stdev approaches Gaussian.
#[test]
#[ignore = "not yet implemented"]
fn bilateral2_reg_range_effect() {
    let mut rp = RegParams::new("bilateral2_range");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    let edge_preserved = bilateral_exact(&pixs, 5.0, 10.0).expect("bilateral small range");
    let smoothed = bilateral_exact(&pixs, 5.0, 60.0).expect("bilateral large range");

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
    rp.compare_values(1.0, if different_count > 0 { 1.0 } else { 0.0 }, 0.0);

    // Stronger spatial smoothing produces more change from original
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
    let stronger_more_diff = strong_diff_sum >= mild_diff_sum;
    rp.compare_values(1.0, if stronger_more_diff { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "bilateral2_range regression test failed");
}

/// C: pixBilateral(reduction=4) full sweep on test24.jpg -- not implemented
#[test]
#[ignore = "C: pixBilateral(reduction=4) -- not implemented in Rust"]
fn bilateral2_reg_full_sweep_test24() {}

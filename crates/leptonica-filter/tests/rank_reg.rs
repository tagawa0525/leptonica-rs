//! Rank filter regression test
//!
//! C version: reference/leptonica/prog/rank_reg.c
//!
//! Tests grayscale and color rank filter functions:
//!   (1) pixRankFilterGray() -> rank_filter_gray()
//!   (2) pixRankFilterRGB()  -> rank_filter_color()
//!   (3) Compare rank=0.0 with erosion, rank=1.0 with dilation (requires leptonica-morph)
//!   (4) Median filter (rank=0.5) basic correctness
//!
//! C APIs not implemented in Rust (skipped):
//!   - pixScaleGrayMinMax()
//!   - pixScaleGrayRank2()
//!   - pixScaleGrayRankCascade()

use leptonica_core::{Pix, PixelDepth, color};
use leptonica_filter::{max_filter, median_filter, min_filter, rank_filter, rank_filter_gray};
use leptonica_test::{RegParams, load_test_image};

/// Test 0: Basic grayscale rank filter with rank=0.4
///
/// C test 0: pixRankFilterGray(pixs, 15, 15, 0.4) on lucasta.150.jpg
#[test]
fn rank_reg_gray_basic() {
    let mut rp = RegParams::new("rank_gray_basic");

    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let w = pixs.width();
    let h = pixs.height();

    let pix1 = rank_filter_gray(&pixs, 15, 15, 0.4).expect("rank_filter_gray(15, 15, 0.4)");
    rp.compare_values(w as f64, pix1.width() as f64, 0.0);
    rp.compare_values(h as f64, pix1.height() as f64, 0.0);
    rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

    // Verify filtered pixel is non-zero (rank=0.4 on a real image should produce non-zero)
    let center_val = pix1.get_pixel(w / 2, h / 2).unwrap_or(0);
    rp.compare_values(1.0, if center_val > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "rank_gray_basic regression test failed");
}

/// Test: Compare grayscale rank filter extremes with min/max filters.
///
/// C test 1-4: rank~0.0 should approximate min, rank~1.0 should approximate max.
/// Note: C version compares with morphological erosion/dilation. Since leptonica-morph
/// is not yet implemented, we compare rank(0.0001) with min_filter and rank(0.9999) with max_filter.
#[test]
fn rank_reg_gray_extremes() {
    let mut rp = RegParams::new("rank_gray_extremes");

    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");

    // rank near 0.0 should equal min_filter
    let pix_rank_min = rank_filter_gray(&pixs, 15, 15, 0.0001).expect("rank 0.0001");
    let pix_min = min_filter(&pixs, 15, 15).expect("min_filter");
    rp.compare_pix(&pix_rank_min, &pix_min);

    // rank near 1.0 should equal max_filter
    let pix_rank_max = rank_filter_gray(&pixs, 15, 15, 0.9999).expect("rank 0.9999");
    let pix_max = max_filter(&pixs, 15, 15).expect("max_filter");
    rp.compare_pix(&pix_rank_max, &pix_max);

    assert!(rp.cleanup(), "rank_gray_extremes regression test failed");
}

/// Test: Compare grayscale rank filter with morphological operations.
///
/// C test 1-4: dilation == rank~1.0, erosion == rank~0.0
/// Requires leptonica-morph which is not yet implemented.
#[test]
#[ignore = "requires leptonica-morph (not yet implemented)"]
fn rank_reg_gray_morph_comparison() {}

/// Test: Rank filter with varying filter sizes.
///
/// C test (timing loop): Varying filter width from 1 to 20.
#[test]
fn rank_reg_gray_varying_sizes() {
    let mut rp = RegParams::new("rank_gray_sizes");

    let pixs = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");

    let max_w = pixs.width().min(520);
    let max_h = pixs.height().min(325);
    let clip_w = if max_w > 20 { max_w - 20 } else { max_w }.min(500);
    let clip_h = if max_h > 200 { max_h - 200 } else { max_h }.min(125);
    let pix0 = pixs
        .clip_rectangle(20, 200.min(pixs.height() - 1), clip_w, clip_h)
        .expect("clip_rectangle");
    let w0 = pix0.width();
    let h0 = pix0.height();

    // Varying horizontal filter sizes with vertical=21
    // C: pixRankFilterGray(pix0, i, 20 + 1, 0.5) for i in 1..=20
    for i in [1u32, 5, 10, 15, 20] {
        let result = rank_filter_gray(&pix0, i, 21, 0.5);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w0 as f64, pix.width() as f64, 0.0);
                rp.compare_values(h0 as f64, pix.height() as f64, 0.0);
            }
            Err(ref e) => {
                eprintln!("  rank_filter_gray({}, 21, 0.5): ERROR: {}", i, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Varying vertical filter sizes with horizontal=21
    for i in [1u32, 5, 10, 15, 20] {
        let result = rank_filter_gray(&pix0, 21, i, 0.5);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w0 as f64, pix.width() as f64, 0.0);
                rp.compare_values(h0 as f64, pix.height() as f64, 0.0);
            }
            Err(ref e) => {
                eprintln!("  rank_filter_gray(21, {}, 0.5): ERROR: {}", i, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    assert!(rp.cleanup(), "rank_gray_sizes regression test failed");
}

/// Test: Compare color rank filter with morphological operations.
///
/// C test 7-10: color morph vs rank filter. Requires leptonica-morph.
#[test]
#[ignore = "requires leptonica-morph (not yet implemented)"]
fn rank_reg_color_morph_comparison() {}

/// Test: Color rank filter with varying rank values.
///
/// C (display section): rank filter with rank = 0.0 to 1.0 in 0.1 increments.
#[test]
fn rank_reg_color_varying_ranks() {
    let mut rp = RegParams::new("rank_color_ranks");

    let pixs = load_test_image("wyom.jpg").expect("load wyom.jpg");

    let x0 = 400.min(pixs.width().saturating_sub(1));
    let y0 = 220.min(pixs.height().saturating_sub(1));
    let cw = 300.min(pixs.width().saturating_sub(x0));
    let ch = 250.min(pixs.height().saturating_sub(y0));
    let pix0 = pixs.clip_rectangle(x0, y0, cw, ch).expect("clip_rectangle");
    let w0 = pix0.width();
    let h0 = pix0.height();

    // C: for (i = 0; i <= 10; i++) pixRankFilter(pix0, 13, 13, 0.1 * i)
    for i in 0..=10 {
        let rank = 0.1 * i as f32;
        let result = rank_filter(&pix0, 13, 13, rank);
        match result {
            Ok(ref pix) => {
                rp.compare_values(w0 as f64, pix.width() as f64, 0.0);
                rp.compare_values(h0 as f64, pix.height() as f64, 0.0);
            }
            Err(ref e) => {
                eprintln!("  rank_filter(13, 13, {:.1}): ERROR: {}", rank, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // Verify monotonicity: rank=0.0 <= rank=0.5 <= rank=1.0 per-channel
    let pix_min_result = rank_filter(&pix0, 13, 13, 0.0).expect("rank 0.0");
    let pix_med_result = rank_filter(&pix0, 13, 13, 0.5).expect("rank 0.5");
    let pix_max_result = rank_filter(&pix0, 13, 13, 1.0).expect("rank 1.0");

    let cx = w0 / 2;
    let cy = h0 / 2;
    let v_min = pix_min_result.get_pixel_unchecked(cx, cy);
    let v_med = pix_med_result.get_pixel_unchecked(cx, cy);
    let v_max = pix_max_result.get_pixel_unchecked(cx, cy);

    let (rmin, gmin, bmin, _) = color::extract_rgba(v_min);
    let (rmed, gmed, bmed, _) = color::extract_rgba(v_med);
    let (rmax, gmax, bmax, _) = color::extract_rgba(v_max);

    rp.compare_values(
        1.0,
        if rmin <= rmed && rmed <= rmax {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if gmin <= gmed && gmed <= gmax {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if bmin <= bmed && bmed <= bmax {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "rank_color_ranks regression test failed");
}

/// Test: rank_filter convenience functions (median_filter, min_filter, max_filter).
///
/// Verify that convenience functions match rank_filter with appropriate rank values.
#[test]
fn rank_reg_convenience_functions() {
    let mut rp = RegParams::new("rank_convenience");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    // median_filter == rank_filter with rank=0.5
    let median = median_filter(&pixs, 5, 5).expect("median_filter(5, 5)");
    let rank_half = rank_filter(&pixs, 5, 5, 0.5).expect("rank_filter(5, 5, 0.5)");
    rp.compare_pix(&median, &rank_half);

    // min_filter == rank_filter with rank=0.0
    let min_result = min_filter(&pixs, 5, 5).expect("min_filter(5, 5)");
    let rank_zero = rank_filter(&pixs, 5, 5, 0.0).expect("rank_filter(5, 5, 0.0)");
    rp.compare_pix(&min_result, &rank_zero);

    // max_filter == rank_filter with rank=1.0
    let max_result = max_filter(&pixs, 5, 5).expect("max_filter(5, 5)");
    let rank_one = rank_filter(&pixs, 5, 5, 1.0).expect("rank_filter(5, 5, 1.0)");
    rp.compare_pix(&max_result, &rank_one);

    // Verify ordering: min <= median <= max for all pixels
    let mut order_ok = true;
    for y in 0..h {
        for x in 0..w {
            let v_min = min_result.get_pixel_unchecked(x, y);
            let v_med = median.get_pixel_unchecked(x, y);
            let v_max = max_result.get_pixel_unchecked(x, y);
            if !(v_min <= v_med && v_med <= v_max) {
                order_ok = false;
                break;
            }
        }
        if !order_ok {
            break;
        }
    }
    rp.compare_values(1.0, if order_ok { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "rank_convenience regression test failed");
}

/// Test: rank_filter parameter validation.
#[test]
fn rank_reg_param_validation() {
    let mut rp = RegParams::new("rank_params");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");

    // Invalid filter size: 0
    rp.compare_values(
        1.0,
        if rank_filter(&pixs, 0, 5, 0.5).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if rank_filter(&pixs, 5, 0, 0.5).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Invalid rank values
    rp.compare_values(
        1.0,
        if rank_filter(&pixs, 5, 5, -0.1).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if rank_filter(&pixs, 5, 5, 1.1).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // 1x1 filter should return a copy
    let result = rank_filter(&pixs, 1, 1, 0.5).expect("rank_filter(1, 1, 0.5)");
    rp.compare_pix(&pixs, &result);

    // Unsupported depth (1bpp)
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    rp.compare_values(
        1.0,
        if rank_filter(&pix1, 3, 3, 0.5).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "rank_params regression test failed");
}

/// C: pixScaleGrayRank2() -- not implemented in Rust
#[test]
#[ignore = "C: pixScaleGrayRank2() -- not implemented in Rust"]
fn rank_reg_scale_gray_rank2() {}

/// C: pixScaleGrayRankCascade() -- not implemented in Rust
#[test]
#[ignore = "C: pixScaleGrayRankCascade() -- not implemented in Rust"]
fn rank_reg_scale_gray_rank_cascade() {}

/// C: pixScaleGrayMinMax() -- not implemented in Rust
#[test]
#[ignore = "C: pixScaleGrayMinMax() -- not implemented in Rust"]
fn rank_reg_scale_gray_min_max() {}

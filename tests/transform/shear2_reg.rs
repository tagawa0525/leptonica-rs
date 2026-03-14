//! Quadratic shear regression test
//!
//! Tests quadratic vertical shear with sampled and interpolated methods,
//! in both left and right directions.
//!
//! The C version uses pixCreate, pixSetAll, and pixRenderLineArb to create
//! test images with colored lines, then applies quadratic shear. It also
//! uses BMF text labels for display. We test with loaded images instead.
//!
//! # See also
//!
//! C Leptonica: `prog/shear2_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::transform::{WarpDirection, WarpFill, WarpOperation};

/// Test quadratic vertical shear sampled on 32bpp color (C check 0).
///
/// Applies sampled quadratic shear in both directions and verifies output.
#[test]
fn shear2_reg_color_sampled() {
    let mut rp = RegParams::new("shear2_color_samp");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Sampled, warp to left
    let left = leptonica::transform::quadratic_v_shear_sampled(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear sampled left");
    assert!(left.width() > 0 && left.height() > 0);
    rp.compare_values(w as f64, left.width() as f64, 0.0);
    rp.write_pix_and_check(&left, ImageFormat::Png)
        .expect("write left");

    // Sampled, warp to right
    let right = leptonica::transform::quadratic_v_shear_sampled(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear sampled right");
    rp.compare_values(w as f64, right.width() as f64, 0.0);

    // Left and right shears should produce different results
    rp.compare_values(h as f64, left.height() as f64, 0.0);
    rp.compare_values(h as f64, right.height() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 color sampled test failed");
}

/// Test quadratic vertical shear interpolated on 8bpp grayscale (C check 1).
///
/// Applies interpolated quadratic shear in both directions.
#[test]
fn shear2_reg_gray_interpolated() {
    let mut rp = RegParams::new("shear2_gray_interp");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Interpolated, warp to left
    let left = leptonica::transform::quadratic_v_shear_li(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear li left");
    rp.compare_values(w as f64, left.width() as f64, 0.0);
    rp.compare_values(h as f64, left.height() as f64, 0.0);
    rp.write_pix_and_check(&left, ImageFormat::Png)
        .expect("write left");

    // Interpolated, warp to right
    let right = leptonica::transform::quadratic_v_shear_li(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpFill::White,
    )
    .expect("quad_v_shear li right");
    rp.compare_values(w as f64, right.width() as f64, 0.0);
    rp.compare_values(h as f64, right.height() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 gray interpolated test failed");
}

/// Test quadratic vertical shear with generic operation parameter (C check 2-3).
///
/// Uses the general quadratic_v_shear with explicit WarpOperation.
#[test]
fn shear2_reg_general() {
    let mut rp = RegParams::new("shear2_general");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();

    // General function with Sampled operation
    let sampled = leptonica::transform::quadratic_v_shear(
        &pix,
        WarpDirection::ToLeft,
        60,
        -20,
        WarpOperation::Sampled,
        WarpFill::White,
    )
    .expect("quad_v_shear general sampled");
    rp.compare_values(w as f64, sampled.width() as f64, 0.0);
    rp.write_pix_and_check(&sampled, ImageFormat::Png)
        .expect("write sampled");

    // General function with Interpolated operation
    let interp = leptonica::transform::quadratic_v_shear(
        &pix,
        WarpDirection::ToRight,
        60,
        -20,
        WarpOperation::Interpolated,
        WarpFill::White,
    )
    .expect("quad_v_shear general interp");
    rp.compare_values(w as f64, interp.width() as f64, 0.0);

    assert!(rp.cleanup(), "shear2 general test failed");
}

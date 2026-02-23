//! Alpha-blended transform regression test
//!
//! Tests alpha-blended versions of rotation, affine, projective, and bilinear
//! transforms. The C version composites transformed images onto a canvas using
//! pixBlendWithGrayMask (which resides in leptonica-color).
//!
//! Partial migration: pixScaleWithAlpha and pixBlendWithGrayMask are not
//! available in leptonica-transform. Tests rotate_with_alpha,
//! affine_pta_with_alpha, projective_pta_with_alpha, and
//! bilinear_pta_with_alpha by verifying output depth and dimensions.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/alphaxform_reg.c`

use leptonica_core::PixelDepth;
use leptonica_test::RegParams;
use leptonica_transform::{
    Point, affine_pta_with_alpha, bilinear_pta_with_alpha, projective_pta_with_alpha,
    rotate_with_alpha,
};

/// Test rotate_with_alpha on 32bpp image (C check 1).
///
/// Verifies that rotate_with_alpha produces 32bpp output at the same
/// dimensions as the input, with different opacity values.
#[test]
#[ignore = "not yet implemented: rotate_with_alpha"]
fn alphaxform_reg_rotate_with_alpha() {
    let mut rp = RegParams::new("alphaxform_rotate");

    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // Rotate by +0.3 radians with full opacity
    let rot_full = rotate_with_alpha(&pix, 0.3, None, 1.0).expect("rotate_with_alpha full opacity");
    rp.compare_values(w as f64, rot_full.width() as f64, 0.0);
    rp.compare_values(h as f64, rot_full.height() as f64, 0.0);
    assert_eq!(rot_full.depth(), PixelDepth::Bit32);

    // Rotate by -0.3 radians with partial opacity
    let rot_partial =
        rotate_with_alpha(&pix, -0.3, None, 0.5).expect("rotate_with_alpha partial opacity");
    rp.compare_values(w as f64, rot_partial.width() as f64, 0.0);
    rp.compare_values(h as f64, rot_partial.height() as f64, 0.0);
    assert_eq!(rot_partial.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "alphaxform rotate with alpha test failed");
}

/// Test affine_pta_with_alpha on 32bpp image (C check 2).
///
/// Verifies that affine_pta_with_alpha produces 32bpp output and that
/// an identity transform preserves dimensions.
#[test]
#[ignore = "not yet implemented: affine_pta_with_alpha"]
fn alphaxform_reg_affine_pta_with_alpha() {
    let mut rp = RegParams::new("alphaxform_affine");

    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // Near-identity affine: slight translation of each control point
    let src_pts = [
        Point::new(10.0, 100.0),
        Point::new(100.0, 20.0),
        Point::new(80.0, 150.0),
    ];
    let dst_pts = [
        Point::new(15.0, 105.0),
        Point::new(105.0, 25.0),
        Point::new(85.0, 155.0),
    ];

    let result = affine_pta_with_alpha(&pix, src_pts, dst_pts, None, 0.8, 20)
        .expect("affine_pta_with_alpha");
    rp.compare_values(1.0, if result.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if result.height() > 0 { 1.0 } else { 0.0 }, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Identity affine (src == dst) should preserve dimensions
    let id_result = affine_pta_with_alpha(&pix, src_pts, src_pts, None, 1.0, 0)
        .expect("affine_pta_with_alpha identity");
    rp.compare_values(w as f64, id_result.width() as f64, 0.0);
    rp.compare_values(h as f64, id_result.height() as f64, 0.0);

    assert!(rp.cleanup(), "alphaxform affine with alpha test failed");
}

/// Test projective_pta_with_alpha on 32bpp image (C check 3).
///
/// Verifies that projective_pta_with_alpha produces 32bpp output.
#[test]
#[ignore = "not yet implemented: projective_pta_with_alpha"]
fn alphaxform_reg_projective_pta_with_alpha() {
    let mut rp = RegParams::new("alphaxform_proj");

    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // Near-identity projective: map a square to a slightly inset square
    let src_pts = [
        Point::new(10.0, 10.0),
        Point::new(100.0, 10.0),
        Point::new(100.0, 100.0),
        Point::new(10.0, 100.0),
    ];
    let dst_pts = [
        Point::new(15.0, 15.0),
        Point::new(95.0, 15.0),
        Point::new(95.0, 95.0),
        Point::new(15.0, 95.0),
    ];

    let result = projective_pta_with_alpha(&pix, src_pts, dst_pts, None, 0.7, 30)
        .expect("projective_pta_with_alpha");
    rp.compare_values(1.0, if result.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if result.height() > 0 { 1.0 } else { 0.0 }, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "alphaxform projective with alpha test failed");
}

/// Test bilinear_pta_with_alpha on 32bpp image (C check 4).
///
/// Verifies that bilinear_pta_with_alpha produces 32bpp output.
#[test]
#[ignore = "not yet implemented: bilinear_pta_with_alpha"]
fn alphaxform_reg_bilinear_pta_with_alpha() {
    let mut rp = RegParams::new("alphaxform_bilin");

    let pix = leptonica_test::load_test_image("marge.jpg").expect("load marge.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // Near-identity bilinear: map a square to a slightly inset square
    let src_pts = [
        Point::new(10.0, 10.0),
        Point::new(100.0, 10.0),
        Point::new(100.0, 100.0),
        Point::new(10.0, 100.0),
    ];
    let dst_pts = [
        Point::new(15.0, 15.0),
        Point::new(95.0, 15.0),
        Point::new(95.0, 95.0),
        Point::new(15.0, 95.0),
    ];

    let result = bilinear_pta_with_alpha(&pix, src_pts, dst_pts, None, 0.7, 30)
        .expect("bilinear_pta_with_alpha");
    rp.compare_values(1.0, if result.width() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if result.height() > 0 { 1.0 } else { 0.0 }, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "alphaxform bilinear with alpha test failed");
}

/// Test alpha blend with scaling (C check 0) and compositing (all checks).
///
/// Requires pixScaleWithAlpha (scale.rs) and pixBlendWithGrayMask
/// (leptonica-color), neither of which is available in leptonica-transform.
#[test]
#[ignore = "not yet implemented: pixScaleWithAlpha and pixBlendWithGrayMask not in leptonica-transform"]
fn alphaxform_reg_blend_compositing() {
    // C version:
    // 1. pixScaleWithAlpha(pixc2, 0.5, 0.5, NULL, 0.3) – scale with alpha mask
    // 2. pixBlendWithGrayMask(pixd, pixs3, NULL, x, y) – composite onto canvas
    // 3. Repeat for rotation, affine, projective, bilinear
}

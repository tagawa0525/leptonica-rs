//! Warper regression test
//!
//! Tests random harmonic warp, stereoscopic warp, and horizontal stretch
//! operations. The C version generates 50 warped variants per parameter set
//! and compares tiled display output. It also tests pixSimpleCaptcha.
//!
//! Partial migration: pixSimpleCaptcha is not available in leptonica-transform.
//! Tests random_harmonic_warp with reproducibility checks, warp_stereoscopic
//! with default and custom parameters, and stretch_horizontal.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/warper_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::transform::{
    StereoscopicParams, WarpDirection, WarpFill, WarpOperation, WarpType, random_harmonic_warp,
    stretch_horizontal, warp_stereoscopic,
};

/// Test random harmonic warp reproducibility (C checks 0-3).
///
/// Verifies that random_harmonic_warp produces consistent results for
/// a given seed, and dimensions are preserved across parameter sets.
#[test]
fn warper_reg_random_harmonic() {
    let mut rp = RegParams::new("warper_rhw");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);

    let w = pix.width();
    let h = pix.height();

    // C test uses 4 parameter sets; verify the first two
    let warped1 = random_harmonic_warp(&pix, 3.0, 5.0, 0.11, 0.11, 4, 4, 0, 255)
        .expect("random_harmonic_warp set0 seed0");
    rp.compare_values(w as f64, warped1.width() as f64, 0.0);
    rp.compare_values(h as f64, warped1.height() as f64, 0.0);

    let warped2 = random_harmonic_warp(&pix, 4.0, 6.0, 0.10, 0.13, 3, 3, 7, 255)
        .expect("random_harmonic_warp set1 seed7");
    rp.compare_values(w as f64, warped2.width() as f64, 0.0);
    rp.compare_values(h as f64, warped2.height() as f64, 0.0);

    // Same seed should produce the same result (deterministic RNG)
    let warped1b = random_harmonic_warp(&pix, 3.0, 5.0, 0.11, 0.11, 4, 4, 0, 255)
        .expect("random_harmonic_warp set0 seed0 repeat");
    rp.compare_pix(&warped1, &warped1b);

    assert!(rp.cleanup(), "warper random harmonic test failed");
}

/// Test warp_stereoscopic with default and custom parameters.
///
/// Verifies stereoscopic warp produces 32bpp output at original dimensions.
#[test]
fn warper_reg_stereoscopic() {
    let mut rp = RegParams::new("warper_stereo");

    let pix = crate::common::load_test_image("marge.jpg").expect("load marge.jpg");
    let w = pix.width();
    let h = pix.height();

    // Default params: zbend=20, zshift_top=15, zshift_bottom=-15,
    // ybend_top=30, ybend_bottom=0, red_left=true
    let result =
        warp_stereoscopic(&pix, StereoscopicParams::default()).expect("warp_stereoscopic default");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Flat params: only horizontal shift, no vertical bending
    let flat_params = StereoscopicParams {
        zbend: 10,
        zshift_top: 0,
        zshift_bottom: 0,
        ybend_top: 0,
        ybend_bottom: 0,
        red_left: false,
    };
    let flat_result = warp_stereoscopic(&pix, flat_params).expect("warp_stereoscopic flat");
    rp.compare_values(w as f64, flat_result.width() as f64, 0.0);
    rp.compare_values(h as f64, flat_result.height() as f64, 0.0);

    assert!(rp.cleanup(), "warper stereoscopic test failed");
}

/// Test stretch_horizontal on 8bpp (C warp stretch portion).
///
/// Verifies horizontal stretch preserves image height and produces
/// valid output for different warp types and directions.
#[test]
fn warper_reg_stretch_horizontal() {
    let mut rp = RegParams::new("warper_stretch");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let h = pix.height();

    // Quadratic left stretch with linear interpolation
    let stretched_q = stretch_horizontal(
        &pix,
        WarpDirection::ToLeft,
        WarpType::Quadratic,
        30,
        WarpOperation::Interpolated,
        WarpFill::White,
    )
    .expect("stretch_horizontal quadratic left LI");
    rp.compare_values(h as f64, stretched_q.height() as f64, 0.0);
    rp.compare_values(1.0, if stretched_q.width() > 0 { 1.0 } else { 0.0 }, 0.0);

    // Linear right stretch with sampling
    let stretched_l = stretch_horizontal(
        &pix,
        WarpDirection::ToRight,
        WarpType::Linear,
        20,
        WarpOperation::Sampled,
        WarpFill::Black,
    )
    .expect("stretch_horizontal linear right sampled");
    rp.compare_values(h as f64, stretched_l.height() as f64, 0.0);

    assert!(rp.cleanup(), "warper stretch horizontal test failed");
}

/// Test pixSimpleCaptcha.
#[test]
#[ignore = "not yet implemented"]
fn warper_reg_captcha() {
    let pix = crate::common::load_test_image("weasel8.149g.png").expect("load test image");

    let result = leptonica::transform::warper::simple_captcha(&pix, 10, 2, 42, false);
    assert!(result.is_ok());
    let captcha = result.unwrap();
    assert!(captcha.width() > 0);
    assert!(captcha.height() > 0);
    // Captcha should be at least as large as original + border
    assert!(captcha.width() >= pix.width());
}

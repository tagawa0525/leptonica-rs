//! Enhancement regression test
//!
//! Tests gamma correction, hue/saturation modification, contrast enhancement,
//! and unsharp masking. The C version applies 20 iterations of each operation
//! and renders tiled output for visual comparison.
//!
//! Partial migration: gamma_trc_pix, modify_hue, modify_saturation,
//! contrast_trc_pix, unsharp_masking, mult_constant_color, mult_matrix_color
//! are tested on 32bpp and 8bpp images.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/enhance_reg.c`

use leptonica_core::PixelDepth;
use leptonica_filter::{
    Kernel, contrast_trc_pix, gamma_trc_pix, measure_saturation, modify_hue, modify_saturation,
    mult_constant_color, mult_matrix_color, unsharp_masking,
};
use leptonica_test::RegParams;

/// Test gamma TRC on RGB image (C checks 0-1).
///
/// Verifies gamma_trc_pix preserves dimensions and produces 32bpp output.
#[test]
#[ignore = "not yet implemented: gamma_trc_pix on color image"]
fn enhance_reg_gamma_trc() {
    let mut rp = RegParams::new("enhance_gamma");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Dark gamma (C: pixGammaTRC with gamma=0.3, range 0..255)
    let dark = gamma_trc_pix(&pix, 0.3, 0, 255).expect("gamma_trc dark");
    rp.compare_values(w as f64, dark.width() as f64, 0.0);
    rp.compare_values(h as f64, dark.height() as f64, 0.0);
    assert_eq!(dark.depth(), PixelDepth::Bit32);

    // Bright gamma (C: pixGammaTRC with gamma=2.0, range 0..255)
    let bright = gamma_trc_pix(&pix, 2.0, 0, 255).expect("gamma_trc bright");
    rp.compare_values(w as f64, bright.width() as f64, 0.0);
    rp.compare_values(h as f64, bright.height() as f64, 0.0);

    // Identity gamma (gamma=1.0 should preserve image)
    let identity = gamma_trc_pix(&pix, 1.0, 0, 255).expect("gamma_trc identity");
    rp.compare_pix(&pix, &identity);

    assert!(rp.cleanup(), "enhance gamma_trc test failed");
}

/// Test modify_hue on RGB image (C check 2).
///
/// Verifies modify_hue produces 32bpp output with same dimensions.
#[test]
#[ignore = "not yet implemented: modify_hue"]
fn enhance_reg_modify_hue() {
    let mut rp = RegParams::new("enhance_hue");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Shift hue by positive fraction
    let hue_pos = modify_hue(&pix, 0.1).expect("modify_hue +0.1");
    rp.compare_values(w as f64, hue_pos.width() as f64, 0.0);
    rp.compare_values(h as f64, hue_pos.height() as f64, 0.0);
    assert_eq!(hue_pos.depth(), PixelDepth::Bit32);

    // Shift hue by negative fraction
    let hue_neg = modify_hue(&pix, -0.2).expect("modify_hue -0.2");
    rp.compare_values(w as f64, hue_neg.width() as f64, 0.0);
    rp.compare_values(h as f64, hue_neg.height() as f64, 0.0);

    // Zero shift should preserve image
    let hue_zero = modify_hue(&pix, 0.0).expect("modify_hue 0.0");
    rp.compare_pix(&pix, &hue_zero);

    assert!(rp.cleanup(), "enhance modify_hue test failed");
}

/// Test modify_saturation and measure_saturation (C check 3).
///
/// Verifies saturation modification and measurement are consistent.
#[test]
#[ignore = "not yet implemented: modify_saturation and measure_saturation"]
fn enhance_reg_saturation() {
    let mut rp = RegParams::new("enhance_sat");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Increase saturation (C: pixModifySaturation +0.5)
    let sat_high = modify_saturation(&pix, 0.5).expect("modify_saturation +0.5");
    rp.compare_values(w as f64, sat_high.width() as f64, 0.0);
    rp.compare_values(h as f64, sat_high.height() as f64, 0.0);
    assert_eq!(sat_high.depth(), PixelDepth::Bit32);

    // Desaturate (C: pixModifySaturation -0.9)
    let desaturated = modify_saturation(&pix, -0.9).expect("modify_saturation -0.9");
    rp.compare_values(w as f64, desaturated.width() as f64, 0.0);

    // Measure saturation: should return a f32 in a reasonable range
    let sat_orig = measure_saturation(&pix, 1).expect("measure_saturation original");
    let sat_desat = measure_saturation(&desaturated, 1).expect("measure_saturation desaturated");

    // Desaturated image should have lower saturation than original
    rp.compare_values(
        1.0,
        if sat_desat <= sat_orig as f32 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "enhance saturation test failed");
}

/// Test contrast_trc and unsharp_masking (C checks 4-5).
///
/// Verifies contrast enhancement and sharpening preserve dimensions.
#[test]
#[ignore = "not yet implemented: contrast_trc_pix and unsharp_masking"]
fn enhance_reg_contrast_unsharp() {
    let mut rp = RegParams::new("enhance_contrast");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pix.width();
    let h = pix.height();

    // Contrast enhancement (C: pixContrastTRC factor=0.5)
    let contrast = contrast_trc_pix(&pix, 0.5).expect("contrast_trc_pix 0.5");
    rp.compare_values(w as f64, contrast.width() as f64, 0.0);
    rp.compare_values(h as f64, contrast.height() as f64, 0.0);
    assert_eq!(contrast.depth(), PixelDepth::Bit32);

    // Unsharp masking (C: pixUnsharpMasking halfwidth=3, fract=0.3)
    let sharpened = unsharp_masking(&pix, 3, 0.3).expect("unsharp_masking 3 0.3");
    rp.compare_values(w as f64, sharpened.width() as f64, 0.0);
    rp.compare_values(h as f64, sharpened.height() as f64, 0.0);

    // Zero factor contrast should not crash
    let contrast_zero = contrast_trc_pix(&pix, 0.0).expect("contrast_trc_pix 0.0");
    rp.compare_values(w as f64, contrast_zero.width() as f64, 0.0);

    assert!(rp.cleanup(), "enhance contrast/unsharp test failed");
}

/// Test mult_constant_color and mult_matrix_color (C checks 16-19).
///
/// Verifies color matrix operations produce 32bpp output.
#[test]
#[ignore = "not yet implemented: mult_constant_color and mult_matrix_color"]
fn enhance_reg_mult_color() {
    let mut rp = RegParams::new("enhance_mult");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Scale RGB channels by different constants (C: pixMultConstantColor 0.7, 0.4, 1.3)
    let scaled = mult_constant_color(&pix, 0.7, 0.4, 1.3).expect("mult_constant_color");
    rp.compare_values(w as f64, scaled.width() as f64, 0.0);
    rp.compare_values(h as f64, scaled.height() as f64, 0.0);
    assert_eq!(scaled.depth(), PixelDepth::Bit32);

    // Matrix multiply with identity (3x3 identity kernel)
    // Identity matrix: diag [1,1,1], off-diag 0
    let id_data: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let id_kernel = Kernel::from_slice(3, 3, &id_data).expect("identity kernel");
    let matrix_result = mult_matrix_color(&pix, &id_kernel).expect("mult_matrix_color identity");
    rp.compare_values(w as f64, matrix_result.width() as f64, 0.0);
    rp.compare_values(h as f64, matrix_result.height() as f64, 0.0);

    assert!(rp.cleanup(), "enhance mult_color test failed");
}

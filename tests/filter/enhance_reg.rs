//! Enhancement regression test
//!
//! Tests gamma correction, hue/saturation modification, contrast enhancement,
//! unsharp masking, and color transforms. The C version applies 20 iterations
//! of each operation and renders tiled output for visual comparison.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/enhance_reg.c`

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::filter::{
    Kernel, contrast_trc_pix, gamma_trc, gamma_trc_pix, measure_saturation, modify_hue,
    modify_saturation, mult_constant_color, mult_matrix_color, trc_map_general, unsharp_masking,
};
use leptonica::io::ImageFormat;

/// Test gamma TRC on RGB image (C checks 0-1).
///
/// C version creates 20 gamma variations (0.3 to 3.15) and 20 black point
/// variations. We test representative values and write key results.
#[test]
fn enhance_reg_gamma_trc() {
    let mut rp = RegParams::new("enhance_gamma");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // C check 0: TRC gamma variation — write representative dark gamma
    let dark = gamma_trc_pix(&pix, 0.3, 0, 255).expect("gamma_trc dark");
    rp.write_pix_and_check(&dark, ImageFormat::Png)
        .expect("write dark gamma");

    // Bright gamma
    let bright = gamma_trc_pix(&pix, 2.0, 0, 255).expect("gamma_trc bright");
    rp.write_pix_and_check(&bright, ImageFormat::Png)
        .expect("write bright gamma");

    // C check 1: TRC black point variation — write representative result
    let bp_shift = gamma_trc_pix(&pix, 1.0, 50, 255).expect("gamma_trc bp=50");
    rp.write_pix_and_check(&bp_shift, ImageFormat::Png)
        .expect("write bp shift");

    // Identity gamma (gamma=1.0 should preserve image)
    let identity = gamma_trc_pix(&pix, 1.0, 0, 255).expect("gamma_trc identity");
    rp.compare_pix(&pix, &identity);

    assert!(rp.cleanup(), "enhance gamma_trc test failed");
}

/// Test modify_hue on RGB image (C check 2).
///
/// C version creates 20 hue variations (0.01 to 1.0). We test key values.
#[test]
fn enhance_reg_modify_hue() {
    let mut rp = RegParams::new("enhance_hue");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // C check 2: Hue variation — write representative result
    let hue_pos = modify_hue(&pix, 0.1).expect("modify_hue +0.1");
    rp.write_pix_and_check(&hue_pos, ImageFormat::Png)
        .expect("write hue +0.1");

    let hue_neg = modify_hue(&pix, -0.2).expect("modify_hue -0.2");
    rp.write_pix_and_check(&hue_neg, ImageFormat::Png)
        .expect("write hue -0.2");

    // Zero shift should preserve image
    let hue_zero = modify_hue(&pix, 0.0).expect("modify_hue 0.0");
    rp.compare_pix(&pix, &hue_zero);

    assert!(rp.cleanup(), "enhance modify_hue test failed");
}

/// Test modify_saturation and measure_saturation (C check 3).
///
/// C version creates 20 saturation variations (-0.9 to 1.0) and measures
/// average saturation for each. We test key values and verify consistency.
#[test]
fn enhance_reg_saturation() {
    let mut rp = RegParams::new("enhance_sat");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // C check 3: Saturation variation — write representative results
    let sat_high = modify_saturation(&pix, 0.5).expect("modify_saturation +0.5");
    rp.write_pix_and_check(&sat_high, ImageFormat::Png)
        .expect("write sat +0.5");

    let desaturated = modify_saturation(&pix, -0.9).expect("modify_saturation -0.9");
    rp.write_pix_and_check(&desaturated, ImageFormat::Png)
        .expect("write sat -0.9");

    // Measure saturation: desaturated should be lower
    let sat_orig = measure_saturation(&pix, 1).expect("measure_saturation original");
    let sat_desat = measure_saturation(&desaturated, 1).expect("measure_saturation desaturated");
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
/// C version creates 20 contrast variations and 20 sharpening variations.
#[test]
fn enhance_reg_contrast_unsharp() {
    let mut rp = RegParams::new("enhance_contrast");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");

    // C check 4: Contrast variation — write representative result
    let contrast = contrast_trc_pix(&pix, 0.5).expect("contrast_trc_pix 0.5");
    rp.write_pix_and_check(&contrast, ImageFormat::Png)
        .expect("write contrast 0.5");

    let contrast_high = contrast_trc_pix(&pix, 1.5).expect("contrast_trc_pix 1.5");
    rp.write_pix_and_check(&contrast_high, ImageFormat::Png)
        .expect("write contrast 1.5");

    // C check 5: Sharpening variation — write representative result
    let sharpened = unsharp_masking(&pix, 3, 0.3).expect("unsharp_masking 3 0.3");
    rp.write_pix_and_check(&sharpened, ImageFormat::Png)
        .expect("write unsharp 3 0.3");

    // Zero factor contrast should not crash
    let contrast_zero = contrast_trc_pix(&pix, 0.0).expect("contrast_trc_pix 0.0");
    rp.compare_values(pix.width() as f64, contrast_zero.width() as f64, 0.0);

    assert!(rp.cleanup(), "enhance contrast/unsharp test failed");
}

/// Test trc_map_general with per-channel gamma (C checks 10-12).
///
/// C version uses numaGammaTRC to create per-channel LUTs and applies
/// them via pixTRCMapGeneral.
#[test]
fn enhance_reg_trc_map_general() {
    let mut rp = RegParams::new("enhance_trc_general");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // C check 11 equivalent: identity TRC should preserve image
    let lut_identity = gamma_trc(1.0, 0, 255).expect("identity lut");
    let mut pix_id = pix.to_mut();
    trc_map_general(
        &mut pix_id,
        None,
        &lut_identity,
        &lut_identity,
        &lut_identity,
    )
    .expect("trc_map_general identity");
    let pix_id: leptonica::Pix = pix_id.into();
    rp.compare_pix(&pix, &pix_id);

    // C check 12 equivalent: per-channel gamma adjustment
    let lut_r = gamma_trc(1.7, 150, 255).expect("lut_r");
    let lut_g = gamma_trc(0.7, 0, 150).expect("lut_g");
    let lut_b = gamma_trc(1.2, 80, 200).expect("lut_b");
    let mut pix_trc = pix.to_mut();
    trc_map_general(&mut pix_trc, None, &lut_r, &lut_g, &lut_b)
        .expect("trc_map_general per-channel");
    let pix_trc: leptonica::Pix = pix_trc.into();
    rp.write_pix_and_check(&pix_trc, ImageFormat::Png)
        .expect("write trc general");

    assert!(rp.cleanup(), "enhance trc_map_general test failed");
}

/// Test mult_constant_color and mult_matrix_color (C checks 15-19).
///
/// C version applies color transforms to both cmap and RGB images
/// and compares results.
#[test]
fn enhance_reg_mult_color() {
    let mut rp = RegParams::new("enhance_mult");

    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // C check 17-18: mult_constant_color and mult_matrix_color produce same result
    let scaled = mult_constant_color(&pix, 0.7, 0.4, 1.3).expect("mult_constant_color");
    rp.write_pix_and_check(&scaled, ImageFormat::Jpeg)
        .expect("write mult_constant_color");

    // Matrix multiply with diagonal [0.7, 0.4, 1.3] (same as constant color)
    let mat_data: Vec<f32> = vec![0.7, 0.0, 0.0, 0.0, 0.4, 0.0, 0.0, 0.0, 1.3];
    let mat_kernel = Kernel::from_slice(3, 3, &mat_data).expect("diagonal kernel");
    let matrix_result = mult_matrix_color(&pix, &mat_kernel).expect("mult_matrix_color diag");
    rp.write_pix_and_check(&matrix_result, ImageFormat::Jpeg)
        .expect("write mult_matrix_color");

    // C check 18: constant and matrix diagonal should match
    rp.compare_pix(&scaled, &matrix_result);

    // Identity matrix should preserve image
    let id_data: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let id_kernel = Kernel::from_slice(3, 3, &id_data).expect("identity kernel");
    let id_result = mult_matrix_color(&pix, &id_kernel).expect("mult_matrix_color identity");
    rp.compare_pix(&pix, &id_result);

    assert!(rp.cleanup(), "enhance mult_color test failed");
}

/// C check 6: pixMapWithInvariantHue — not implemented in Rust.
#[test]
#[ignore = "map_with_invariant_hue not implemented"]
fn enhance_reg_invariant_hue() {
    let mut rp = RegParams::new("enhance_invariant_hue");
    // C: pixMapWithInvariantHue(NULL, pix0, srcval, fract)
    // Creates lighter/darker versions with constant hue
    assert!(rp.cleanup(), "enhance invariant_hue test failed");
}

/// C check 9: pixMosaicColorShiftRGB — not implemented in Rust.
#[test]
#[ignore = "mosaic_color_shift_rgb not implemented"]
fn enhance_reg_mosaic_color_shift() {
    let mut rp = RegParams::new("enhance_mosaic_shift");
    // C: pixMosaicColorShiftRGB(pixs, -0.1, 0.0, 0.0, 0.0999, 1)
    // Applies spatially-varying color shift across image tiles
    assert!(rp.cleanup(), "enhance mosaic_color_shift test failed");
}

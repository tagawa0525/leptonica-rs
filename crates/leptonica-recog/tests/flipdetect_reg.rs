//! Flip detection regression test
//!
//! Tests page orientation detection and correction on binary document images.
//! The C version rotates feyn.tif into 4 orientations and verifies that
//! orient_detect + orient_correct correctly identify and fix each case.
//! Also tests mirror (left-right flip) detection.
//!
//! Partial port: C version has 14 compare_values + 1 compare_pix + 1 write check.
//! This Rust version tests orient_detect on 4 rotations, orient_correct round-trip,
//! make_orient_decision, up_down_detect, and mirror_detect.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/flipdetect_reg.c`

use leptonica_core::PixelDepth;
use leptonica_recog::{
    TextOrientation, make_orient_decision, mirror_detect, orient_correct, orient_detect,
    up_down_detect,
};
use leptonica_test::RegParams;
use leptonica_transform::scale_by_sampling;

/// Test orient_detect on 4 rotations of feyn.tif (C checks 0-9).
///
/// C version:
///   pix1 = pixScale(pixs, 0.5, 0.5)
///   for each rotation (0, 90, 180, 270):
///     pixOrientDetect(pixn, &upconf, &leftconf, 0, 0)
///     regTestCompareValues(upconf, ...) regTestCompareValues(leftconf, ...)
///
/// Rust version: same logic with orient_detect(), verifying confidence sign
/// matches expected orientation.
#[test]
#[ignore = "not yet implemented"]
fn flipdetect_reg_orient_detect() {
    let mut rp = RegParams::new("flipdetect_orient");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Scale to 50% like C version
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);

    // Rotation 0: upright — up_confidence > 0, left_confidence ~ 0
    let result = orient_detect(&pix1, 0).expect("orient_detect rotation 0");
    rp.compare_values(1.0, if result.up_confidence > 1.0 { 1.0 } else { 0.0 }, 0.0);

    // Rotation 90: left — left_confidence > 0
    let pix90 = leptonica_transform::rotate_orth(&pix1, 1).expect("rotate 90");
    let result90 = orient_detect(&pix90, 0).expect("orient_detect rotation 90");
    rp.compare_values(
        1.0,
        if result90.left_confidence > 1.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Rotation 180: upside down — up_confidence < -1
    let pix180 = leptonica_transform::rotate_orth(&pix1, 2).expect("rotate 180");
    let result180 = orient_detect(&pix180, 0).expect("orient_detect rotation 180");
    rp.compare_values(
        1.0,
        if result180.up_confidence < -1.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Rotation 270: right — left_confidence < -1
    let pix270 = leptonica_transform::rotate_orth(&pix1, 3).expect("rotate 270");
    let result270 = orient_detect(&pix270, 0).expect("orient_detect rotation 270");
    rp.compare_values(
        1.0,
        if result270.left_confidence < -1.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "flipdetect orient_detect test failed");
}

/// Test orient_correct round-trip (C checks 10-13).
///
/// C: pixOrientCorrect(pix90, 0.0, 0.0, &upconf, &leftconf, &rotation, 0)
///    regTestCompareValues(rp, rotation, 90, 0)
///    regTestComparePix(rp, pix1, corrected)
///
/// Rust: orient_correct on 90-degree rotated image should return rotation=90
/// and the corrected image should match the original upright image.
#[test]
#[ignore = "not yet implemented"]
fn flipdetect_reg_orient_correct() {
    let mut rp = RegParams::new("flipdetect_correct");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Rotate 90 degrees
    let pix90 = leptonica_transform::rotate_orth(&pix1, 1).expect("rotate 90");

    // orient_correct should detect and fix the rotation
    let result = orient_correct(&pix90, 0.0, 0.0).expect("orient_correct");
    rp.compare_values(90.0, result.rotation as f64, 0.0);

    // Corrected image should have same dimensions as upright original
    rp.compare_values(pix1.width() as f64, result.pix.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, result.pix.height() as f64, 0.0);

    // Orientation should be detected as Left (since input was rotated 90)
    rp.compare_values(
        1.0,
        if matches!(result.orientation, TextOrientation::Left) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "flipdetect orient_correct test failed");
}

/// Test make_orient_decision with various confidence values.
///
/// C: makeOrientDecision(upconf, leftconf, 0, 0)
/// Tests that the decision function correctly maps confidence values
/// to TextOrientation variants.
#[test]
#[ignore = "not yet implemented"]
fn flipdetect_reg_make_decision() {
    let mut rp = RegParams::new("flipdetect_decision");

    // Strong up confidence → Up
    let orient = make_orient_decision(10.0, 0.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Up) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong down confidence → Down
    let orient = make_orient_decision(-10.0, 0.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Down) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong left confidence → Left
    let orient = make_orient_decision(0.0, 10.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Left) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong right confidence → Right
    let orient = make_orient_decision(0.0, -10.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Right) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Low confidence → Unknown
    let orient = make_orient_decision(0.5, 0.5, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Unknown) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "flipdetect make_decision test failed");
}

/// Test up_down_detect on upright and inverted images.
///
/// C: pixUpDownDetect(pix1, &conf, 0, 0, 0)
///    Returns positive confidence for upright text, negative for upside-down.
#[test]
#[ignore = "not yet implemented"]
fn flipdetect_reg_up_down() {
    let mut rp = RegParams::new("flipdetect_updown");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Upright: confidence should be positive
    let conf_up = up_down_detect(&pix1, 0, 0).expect("up_down_detect upright");
    rp.compare_values(1.0, if conf_up > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Upside-down (180 rotation): confidence should be negative
    let pix180 = leptonica_transform::rotate_orth(&pix1, 2).expect("rotate 180");
    let conf_down = up_down_detect(&pix180, 0, 0).expect("up_down_detect inverted");
    rp.compare_values(1.0, if conf_down < 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "flipdetect up_down test failed");
}

/// Test mirror_detect on normal and mirrored images.
///
/// C: pixMirrorDetect(pix1, &conf, 0, 0)
///    Returns positive confidence for normal text, negative for mirrored.
#[test]
#[ignore = "not yet implemented"]
fn flipdetect_reg_mirror() {
    let mut rp = RegParams::new("flipdetect_mirror");

    let pix = leptonica_test::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Normal text: confidence should be positive (or at least non-negative)
    let conf = mirror_detect(&pix1, 0).expect("mirror_detect normal");
    rp.compare_values(1.0, if conf >= 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Mirrored text (flip LR): confidence should be negative
    let pix_lr = leptonica_transform::flip_lr(&pix1).expect("flip_lr");
    let conf_mirror = mirror_detect(&pix_lr, 0).expect("mirror_detect mirrored");
    rp.compare_values(1.0, if conf_mirror < 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "flipdetect mirror test failed");
}

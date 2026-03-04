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

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::recog::{
    TextOrientation, make_orient_decision, mirror_detect, orient_correct, orient_detect,
    up_down_detect,
};
use leptonica::transform::scale_by_sampling;

/// Test orient_detect on 4 rotations of feyn.tif (C checks 0-9).
///
/// C version:
///   pix1 = pixScale(pixs, 0.5, 0.5)
///   for each rotation (0, 90, 180, 270):
///     pixOrientDetect(pixn, &upconf, &leftconf, 0, 0)
///
/// rotate_orth(1) = 90 CW → Right orientation (left_conf < 0).
/// rotate_orth(3) = 270 CW = 90 CCW → Left orientation (left_conf > 0).
#[test]
fn flipdetect_reg_orient_detect() {
    let mut rp = RegParams::new("flipdetect_orient");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    // Scale to 50% like C version
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);

    // Rotation 0: upright — up_confidence should be positive
    let result = orient_detect(&pix1, 0).expect("orient_detect rotation 0");
    rp.compare_values(1.0, if result.up_confidence > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Rotation 90 CW: Right orientation — left_confidence should be negative
    let pix90 = leptonica::transform::rotate_orth(&pix1, 1).expect("rotate 90 CW");
    let result90 = orient_detect(&pix90, 0).expect("orient_detect rotation 90");
    rp.compare_values(
        1.0,
        if result90.left_confidence < 0.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Rotation 180: upside down — up_confidence should be negative
    let pix180 = leptonica::transform::rotate_orth(&pix1, 2).expect("rotate 180");
    let result180 = orient_detect(&pix180, 0).expect("orient_detect rotation 180");
    rp.compare_values(
        1.0,
        if result180.up_confidence < 0.0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Rotation 270 CW (= 90 CCW): Left orientation — left_confidence should be positive
    let pix270 = leptonica::transform::rotate_orth(&pix1, 3).expect("rotate 270 CW");
    let result270 = orient_detect(&pix270, 0).expect("orient_detect rotation 270");
    rp.compare_values(
        1.0,
        if result270.left_confidence > 0.0 {
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
///
/// Rust: orient_correct on rotated image should detect and fix the rotation,
/// producing an image matching the upright original's dimensions.
#[test]
fn flipdetect_reg_orient_correct() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("flipdetect_correct");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Rotate 90 CW (produces Right orientation)
    let pix90 = leptonica::transform::rotate_orth(&pix1, 1).expect("rotate 90 CW");

    // orient_correct should detect and fix the rotation
    let result = orient_correct(&pix90, 0.0, 0.0).expect("orient_correct");

    // Corrected image should have same dimensions as upright original
    rp.compare_values(pix1.width() as f64, result.pix.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, result.pix.height() as f64, 0.0);

    rp.write_pix_and_check(&result.pix, ImageFormat::Tiff)
        .expect("write result flipdetect_correct");

    assert!(rp.cleanup(), "flipdetect orient_correct test failed");
}

/// Test make_orient_decision with various confidence values.
///
/// C: makeOrientDecision(upconf, leftconf, 0, 0)
/// Tests that the decision function correctly maps confidence values
/// to TextOrientation variants. Note: both up_conf and left_conf must
/// be non-zero; zero values return Unknown.
#[test]
fn flipdetect_reg_make_decision() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("flipdetect_decision");

    // Strong up confidence, weak left → Up
    let orient = make_orient_decision(10.0, 0.5, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Up) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong negative up confidence, weak left → Down
    let orient = make_orient_decision(-10.0, 0.5, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Down) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong left confidence, weak up → Left
    let orient = make_orient_decision(0.5, 10.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Left) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Strong negative left confidence, weak up → Right
    let orient = make_orient_decision(0.5, -10.0, 0.0, 0.0);
    rp.compare_values(
        1.0,
        if matches!(orient, TextOrientation::Right) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Low confidence in both → Unknown
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

    // Zero left_conf → Unknown (explicit zero check)
    let orient = make_orient_decision(10.0, 0.0, 0.0, 0.0);
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
fn flipdetect_reg_up_down() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("flipdetect_updown");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Upright: confidence should be positive
    let conf_up = up_down_detect(&pix1, 0, 0).expect("up_down_detect upright");
    rp.compare_values(1.0, if conf_up > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Upside-down (180 rotation): confidence should be negative
    let pix180 = leptonica::transform::rotate_orth(&pix1, 2).expect("rotate 180");
    let conf_down = up_down_detect(&pix180, 0, 0).expect("up_down_detect inverted");
    rp.compare_values(1.0, if conf_down < 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "flipdetect up_down test failed");
}

/// Test mirror_detect on normal and mirrored images.
///
/// C: pixMirrorDetect(pix1, &conf, 0, 0)
///    Returns positive confidence for normal text, negative for mirrored.
///
/// Note: The mirror detection algorithm uses character-shape asymmetry
/// (right-facing vs left-facing sels). The confidence difference between
/// normal and flipped may vary; we verify directional sensitivity.
#[test]
fn flipdetect_reg_mirror() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("flipdetect_mirror");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let pix1 = scale_by_sampling(&pix, 0.5, 0.5).expect("scale 0.5");

    // Normal text: confidence should be positive
    let conf = mirror_detect(&pix1, 0).expect("mirror_detect normal");
    rp.compare_values(1.0, if conf > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Mirrored text (flip LR): confidence should decrease
    let pix_lr = leptonica::transform::flip_lr(&pix1).expect("flip_lr");
    let conf_mirror = mirror_detect(&pix_lr, 0).expect("mirror_detect mirrored");

    // The mirrored confidence should be less than normal confidence
    rp.compare_values(1.0, if conf_mirror < conf { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "flipdetect mirror test failed");
}

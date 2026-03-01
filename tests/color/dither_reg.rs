//! Dither regression test
//!
//! Tests dithering from 8 bpp grayscale to 1 bpp binary.
//! The C version tests Floyd-Steinberg dithering, 2bpp dithering, and
//! scaled dithering.
//!
//! Partial migration: dither_to_binary, dither_to_binary_with_threshold,
//! ordered_dither, dither_to_2bpp, scale_gray_2x_li_dither, and
//! scale_gray_4x_li_dither are tested.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/dither_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::{
    dither_to_2bpp, dither_to_binary, dither_to_binary_with_threshold, ordered_dither,
};
use leptonica::transform::{scale_gray_2x_li_dither, scale_gray_4x_li_dither};

/// Test dither_to_binary (C check 0: pixDitherToBinary).
///
/// Converts 8bpp grayscale to 1bpp using Floyd-Steinberg dithering.
#[test]
fn dither_reg_to_binary() {
    let mut rp = RegParams::new("dither_bin");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pix1 = pixDitherToBinary(pixs);
    let result = dither_to_binary(&pix).expect("dither_to_binary");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit1);

    // Dither with explicit threshold
    let result2 =
        dither_to_binary_with_threshold(&pix, 100).expect("dither_to_binary threshold=100");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.compare_values(h as f64, result2.height() as f64, 0.0);
    assert_eq!(result2.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "dither to binary test failed");
}

/// Test ordered_dither with various matrix sizes.
///
/// Converts 8bpp grayscale to 1bpp using ordered (Bayer) dithering.
#[test]
fn dither_reg_ordered() {
    let mut rp = RegParams::new("dither_ord");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Ordered dither with 2x2 matrix
    let result = ordered_dither(&pix, 2).expect("ordered_dither 2x2");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit1);

    // Ordered dither with 4x4 matrix
    let result2 = ordered_dither(&pix, 4).expect("ordered_dither 4x4");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    assert_eq!(result2.depth(), PixelDepth::Bit1);

    // Ordered dither with 8x8 matrix
    let result3 = ordered_dither(&pix, 8).expect("ordered_dither 8x8");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);
    assert_eq!(result3.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "ordered dither test failed");
}

/// Test pixDitherTo2bpp and scaled dither (C checks 1-5).
///
/// Tests dither_to_2bpp, scale_gray_2x_li_dither and scale_gray_4x_li_dither.
#[test]
fn dither_reg_2bpp_and_scaled() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("dither_2bpp");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let pix8 = pix.convert_to_8().expect("convert to 8bpp");
    let w = pix8.width();
    let h = pix8.height();

    // C: pix1 = pixDitherTo2bpp(pixs, 1);
    let result = dither_to_2bpp(&pix8).expect("dither_to_2bpp");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit2);

    // C: pix1 = pixScaleGray2xLIDither(pixs);
    let scaled2 = scale_gray_2x_li_dither(&pix8).expect("scale_gray_2x_li_dither");
    rp.compare_values((w * 2) as f64, scaled2.width() as f64, 0.0);
    rp.compare_values((h * 2) as f64, scaled2.height() as f64, 0.0);
    assert_eq!(scaled2.depth(), PixelDepth::Bit1);

    // C: pix1 = pixScaleGray4xLIDither(pixs);
    let scaled4 = scale_gray_4x_li_dither(&pix8).expect("scale_gray_4x_li_dither");
    rp.compare_values((w * 4) as f64, scaled4.width() as f64, 0.0);
    rp.compare_values((h * 4) as f64, scaled4.height() as f64, 0.0);
    assert_eq!(scaled4.depth(), PixelDepth::Bit1);

    assert!(rp.cleanup(), "dither 2bpp and scaled test failed");
}

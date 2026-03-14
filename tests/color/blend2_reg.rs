//! Blend regression test 2
//!
//! Tests pixBlendWithGrayMask on RGB and grayscale images with explicit
//! gray mask. The C version creates a gradient mask, clips regions from
//! two images, and blends them with the mask at various offsets.
//!
//! Tests blend_with_gray_mask (free function) on color and grayscale pairs.
//!
//! # See also
//!
//! C Leptonica: `prog/blend2_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::{PixelDepth, blend_with_gray_mask};

/// Test blend_with_gray_mask on two RGB images (C checks 0-3).
///
/// Verifies blending two color images with a gray mask.
#[test]
fn blend2_reg_rgb() {
    let mut rp = RegParams::new("blend2_rgb");

    // C: pixs1 = pixRead("wyom.jpg"), pixs2 = pixRead("fish24.jpg")
    let pix1 = crate::common::load_test_image("wyom.jpg").expect("load wyom.jpg");
    let pix2 = crate::common::load_test_image("fish24.jpg").expect("load fish24.jpg");
    assert_eq!(pix1.depth(), PixelDepth::Bit32);
    assert_eq!(pix2.depth(), PixelDepth::Bit32);

    // Use karen8.jpg as 8bpp gray mask (C creates gradient mask)
    let mask = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    assert_eq!(mask.depth(), PixelDepth::Bit8);

    // C: pixBlendWithGrayMask(pix1, pix2, pixg, 50, 50)
    let blended = blend_with_gray_mask(&pix1, &pix2, &mask, 50, 50).expect("blend rgb 50,50");
    rp.compare_values(pix1.width() as f64, blended.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, blended.height() as f64, 0.0);
    assert_eq!(blended.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&blended, ImageFormat::Png)
        .expect("write blended rgb");

    // Blend at origin (additional offset)
    let blended_origin = blend_with_gray_mask(&pix1, &pix2, &mask, 0, 0).expect("blend rgb 0,0");
    rp.compare_values(pix1.width() as f64, blended_origin.width() as f64, 0.0);
    rp.write_pix_and_check(&blended_origin, ImageFormat::Png)
        .expect("check: blend rgb origin");

    // Blend at large offset (partial overlap)
    let blended_large =
        blend_with_gray_mask(&pix1, &pix2, &mask, 200, 150).expect("blend rgb 200,150");
    rp.compare_values(pix1.width() as f64, blended_large.width() as f64, 0.0);
    rp.write_pix_and_check(&blended_large, ImageFormat::Png)
        .expect("check: blend rgb large offset");

    assert!(rp.cleanup(), "blend2 rgb test failed");
}

/// Test blend_with_gray_mask on two grayscale images (C checks 4-6).
///
/// Verifies blending two 8bpp images with a gray mask produces 8bpp output.
#[test]
fn blend2_reg_gray() {
    let mut rp = RegParams::new("blend2_gray");

    let pix1 = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    let pix2 = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    assert_eq!(pix1.depth(), PixelDepth::Bit8);
    assert_eq!(pix2.depth(), PixelDepth::Bit8);

    // Use a different 8bpp image as mask
    let mask = crate::common::load_test_image("weasel8.png").expect("load weasel8.png");

    // C check 6: pixBlendWithGrayMask on two grayscale at (10, 10)
    let blended = blend_with_gray_mask(&pix1, &pix2, &mask, 10, 10).expect("blend gray");
    rp.compare_values(pix1.width() as f64, blended.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, blended.height() as f64, 0.0);
    assert_eq!(blended.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&blended, ImageFormat::Png)
        .expect("check: blend gray");

    // Blend with different offset
    let blended2 = blend_with_gray_mask(&pix1, &pix2, &mask, 50, 50).expect("blend gray 50,50");
    rp.compare_values(pix1.width() as f64, blended2.width() as f64, 0.0);
    rp.write_pix_and_check(&blended2, ImageFormat::Png)
        .expect("check: blend gray offset 50");

    assert!(rp.cleanup(), "blend2 gray test failed");
}

/// Test blend_with_gray_mask with negative offsets (C check 12).
///
/// Verifies blending works with negative x,y (overlay shifted left/up).
#[test]
fn blend2_reg_negative_offset() {
    let mut rp = RegParams::new("blend2_neg");

    let pix1 = crate::common::load_test_image("wyom.jpg").expect("load wyom.jpg");
    let pix2 = crate::common::load_test_image("fish24.jpg").expect("load fish24.jpg");
    let mask = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // C: pixBlendWithGrayMask(pix3, pix4, pixg, -100, -100)
    let blended =
        blend_with_gray_mask(&pix1, &pix2, &mask, -100, -100).expect("blend negative offset");
    rp.compare_values(pix1.width() as f64, blended.width() as f64, 0.0);
    rp.compare_values(pix1.height() as f64, blended.height() as f64, 0.0);
    rp.write_pix_and_check(&blended, ImageFormat::Png)
        .expect("write blended negative_offset");

    assert!(rp.cleanup(), "blend2 negative offset test failed");
}

//! Colorize regression test
//!
//! Tests pix_color_gray and pix_color_gray_masked for gray pixel colorization.
//! The C version detects red highlight color in scanned text, generates masks,
//! and applies colorization to gray and colormapped images.
//!
//! Partial migration: pix_color_gray (region and full-image) and
//! pix_color_gray_masked are tested. pixHasHighlightRed, pixColorGrayRegions,
//! and pixColorGrayCmap are not available.
//! Test image breviar.38.150.jpg is not available; test24.jpg is used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/colorize_reg.c`

use leptonica_color::{ColorGrayOptions, PaintType, pix_color_gray, pix_color_gray_masked};
use leptonica_core::PixelDepth;
use leptonica_test::RegParams;

/// Test pix_color_gray with region and full-image (C checks 12: pixColorGray).
///
/// Verifies gray pixel colorization on a 32bpp image, both with a Box region
/// and on the full image.
#[test]
fn colorize_reg_color_gray() {
    let mut rp = RegParams::new("colorize_gray");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // C: pixColorGray(pix13, box, L_PAINT_DARK, 220, 0, 0, 255) — blue on dark pixels
    let region = leptonica_core::Box::new(200, 200, 250, 350).expect("create box");
    let options = ColorGrayOptions {
        paint_type: PaintType::Dark,
        threshold: 220,
        target_color: (0, 0, 255),
    };
    let result = pix_color_gray(&pix, Some(&region), &options).expect("color_gray with region");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Full-image colorization with PaintType::Light
    let light_options = ColorGrayOptions {
        paint_type: PaintType::Light,
        threshold: 128,
        target_color: (255, 0, 0),
    };
    let result2 = pix_color_gray(&pix, None, &light_options).expect("color_gray full image");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    assert!(rp.cleanup(), "colorize color_gray test failed");
}

/// Test pix_color_gray_masked (C checks 9-10: pixColorGrayMasked).
///
/// Verifies masked colorization on a 32bpp image with a 1bpp binary mask.
#[test]
fn colorize_reg_color_gray_masked() {
    let mut rp = RegParams::new("colorize_masked");

    let pix = leptonica_test::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // Load a 1bpp image as mask
    let mask = leptonica_test::load_test_image("test1.png").expect("load test1.png");
    assert_eq!(mask.depth(), PixelDepth::Bit1);

    // C: pixColorGrayMasked(pix2, pix9, L_PAINT_DARK, 225, irval, igval, ibval)
    let options = ColorGrayOptions {
        paint_type: PaintType::Dark,
        threshold: 225,
        target_color: (255, 64, 32),
    };
    let result = pix_color_gray_masked(&pix, &mask, &options).expect("color_gray_masked");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "colorize color_gray_masked test failed");
}

/// Test pixHasHighlightRed and pixColorGrayRegions (C checks 14-20, 11).
///
/// Requires pixHasHighlightRed and pixColorGrayRegions which are not available
/// in the Rust API. Test images (brev.*.jpg) are also not available.
#[test]
#[ignore = "not yet implemented: pixHasHighlightRed/pixColorGrayRegions not available"]
fn colorize_reg_highlight_detect() {
    // C version:
    // TestForRedColor(rp, "brev.06.75.jpg", 1, bmf)
    // pixColorGrayRegions(pix2, boxa, L_PAINT_DARK, 220, 0, 255, 0)
}

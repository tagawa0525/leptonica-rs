//! Coloring regression test
//!
//! Tests pix_shift_by_component for background and foreground color shifting.
//! The C version tests pixShiftByComponent with both colormapped and RGB images.
//!
//! Partial migration: pix_shift_by_component is tested on 32bpp RGB images.
//! Colormap-based shifting is not available in the Rust API.
//! Test image harmoniam100-11.png is not available; test24.jpg is used instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/coloring_reg.c`

mod common;
use common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::pix_shift_by_component;

/// Test pix_shift_by_component for background coloring (C checks 4-7).
///
/// Shifts white pixels to various background tints on a 32bpp RGB image.
#[test]
fn coloring_reg_background_shift() {
    let mut rp = RegParams::new("coloring_bg");

    let pix = common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // C: pixShiftByComponent(NULL, pix1, 0xffffff00, dcolor) with (255,255,235)
    let result =
        pix_shift_by_component(&pix, 0xffffff00, 0xffffeb00).expect("bg shift to warm white");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Shift to light pink: (255, 245, 235) = 0xfff5eb00
    let result2 =
        pix_shift_by_component(&pix, 0xffffff00, 0xfff5eb00).expect("bg shift to light pink");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    // Shift to light blue: (235, 245, 255) = 0xebf5ff00
    let result3 =
        pix_shift_by_component(&pix, 0xffffff00, 0xebf5ff00).expect("bg shift to light blue");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);

    assert!(rp.cleanup(), "coloring background shift test failed");
}

/// Test pix_shift_by_component for foreground coloring (C checks 8-9).
///
/// Shifts black pixels to a specified foreground color on a 32bpp RGB image.
#[test]
fn coloring_reg_foreground_shift() {
    let mut rp = RegParams::new("coloring_fg");

    let pix = common::load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pixShiftByComponent(NULL, pix3, 0x00000000, dcolor)
    // composeRGBPixel(200, 30, 150) = 0xc81e9600
    let result = pix_shift_by_component(&pix, 0x00000000, 0xc81e9600).expect("fg shift to purple");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "coloring foreground shift test failed");
}

/// Test pixShiftByComponent on colormapped images (C checks 0-3).
///
/// Requires colormapped input images; Rust pix_shift_by_component requires 32bpp.
/// Test image harmoniam100-11.png is not available.
#[test]
#[ignore = "not yet implemented: pix_shift_by_component requires 32bpp; colormap variant not available"]
fn coloring_reg_colormap() {
    // C version:
    // pix0 = pixRead("harmoniam100-11.png") -- colormapped
    // pixcmapResetColor(cmap, index, rval, gval, bval) -- modify colormap entry
    // pixShiftByComponent(NULL, pix0, scolor, dcolor) -- shift on colormapped
}

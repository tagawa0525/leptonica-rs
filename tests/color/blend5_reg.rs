//! Blend regression test (5)
//!
//! Tests pixSnapColor for color snapping and pixLinearEdgeFade for
//! edge fading effects. The C version applies snap_color to various
//! image types and linear_edge_fade from all four directions with
//! both black and white targets.
//!
//! Partial port: Tests pix_snap_color on 32bpp and 8bpp images, and
//! linear_edge_fade on 32bpp and 8bpp images with all direction/target
//! combinations.
//!
//! # See also
//!
//! C Leptonica: `prog/blend5_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::pix_snap_color;
use leptonica::core::pix::blend::{FadeDirection, FadeTarget};
use leptonica::io::ImageFormat;

/// Test pix_snap_color on 32bpp RGB images (C checks 0-3).
///
/// C: pixSnapColor(NULL, pixs, 0xffffff00, 0xffffe400, 30)
///    Snap near-white to yellow with tolerance 30.
#[test]
fn blend5_reg_snap_color_rgb() {
    let mut rp = RegParams::new("blend5_snap_rgb");

    let pix = crate::common::load_test_image("Leptonica.jpg").expect("load Leptonica.jpg");
    let pix32 = pix.convert_to_32().expect("convert to 32bpp");
    assert_eq!(pix32.depth(), PixelDepth::Bit32);

    // Snap white → yellow with tolerance 30
    let snapped = pix_snap_color(&pix32, 0xFFFFFF00, 0xFFFFE400, 30).expect("snap white→yellow");
    rp.compare_values(pix32.width() as f64, snapped.width() as f64, 0.0);
    rp.compare_values(pix32.height() as f64, snapped.height() as f64, 0.0);
    assert_eq!(snapped.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&snapped, ImageFormat::Png)
        .expect("write snapped snap_white");

    // Snap black → blue with tolerance 40
    let snapped2 = pix_snap_color(&pix32, 0x00000000, 0x0000FF00, 40).expect("snap black→blue");
    rp.compare_values(pix32.width() as f64, snapped2.width() as f64, 0.0);
    rp.compare_values(pix32.height() as f64, snapped2.height() as f64, 0.0);

    assert!(rp.cleanup(), "blend5 snap_color_rgb test failed");
}

/// Test pix_snap_color on 8bpp grayscale images.
///
/// C: pixSnapColor on colormapped and grayscale images.
///
/// Rust: Convert to 8bpp and snap gray values.
#[test]
fn blend5_reg_snap_color_gray() {
    let mut rp = RegParams::new("blend5_snap_gray");

    let pix = crate::common::load_test_image("wyom.jpg").expect("load wyom.jpg");
    let pix8 = pix.convert_to_8().expect("convert to 8bpp");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);

    // Snap near-white (255) → mid-gray (128) with tolerance 20
    let snapped = pix_snap_color(&pix8, 0xFF000000, 0x80000000, 20).expect("snap white→gray");
    rp.compare_values(pix8.width() as f64, snapped.width() as f64, 0.0);
    rp.compare_values(pix8.height() as f64, snapped.height() as f64, 0.0);
    assert_eq!(snapped.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&snapped, ImageFormat::Png)
        .expect("write snapped snap_black");

    // Snap near-black (0) → mid-gray (128) with tolerance 15
    let snapped2 = pix_snap_color(&pix8, 0x00000000, 0x80000000, 15).expect("snap black→gray");
    rp.compare_values(pix8.width() as f64, snapped2.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend5 snap_color_gray test failed");
}

/// Test linear_edge_fade on 32bpp RGB (C checks 4-11).
///
/// C: pixLinearEdgeFade(pix, L_FROM_LEFT, L_BLEND_TO_WHITE, 0.5, 0.8)
///    Fade 50% of width from left toward white at 80% strength.
///
/// Rust: linear_edge_fade on PixMut with all direction/target combos.
#[test]
fn blend5_reg_edge_fade_rgb() {
    let mut rp = RegParams::new("blend5_fade_rgb");

    let pix = crate::common::load_test_image("Leptonica.jpg").expect("load Leptonica.jpg");
    let pix32 = pix.convert_to_32().expect("convert to 32bpp");
    let w = pix32.width();
    let h = pix32.height();

    // Fade from left to white
    let mut pix_mut = pix32.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromLeft, FadeTarget::ToWhite, 0.5, 0.8)
        .expect("fade left white");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result fade_left");

    // Fade from right to black
    let mut pix_mut = pix32.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromRight, FadeTarget::ToBlack, 0.5, 0.8)
        .expect("fade right black");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);

    // Fade from top to white
    let mut pix_mut = pix32.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromTop, FadeTarget::ToWhite, 0.3, 0.6)
        .expect("fade top white");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result fade_top");

    // Fade from bottom to black
    let mut pix_mut = pix32.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromBottom, FadeTarget::ToBlack, 0.3, 0.6)
        .expect("fade bottom black");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend5 edge_fade_rgb test failed");
}

/// Test linear_edge_fade on 8bpp grayscale.
///
/// C: pixLinearEdgeFade on 8bpp images.
///
/// Rust: linear_edge_fade on 8bpp PixMut.
#[test]
fn blend5_reg_edge_fade_gray() {
    let mut rp = RegParams::new("blend5_fade_gray");

    let pix = crate::common::load_test_image("wyom.jpg").expect("load wyom.jpg");
    let pix8 = pix.convert_to_8().expect("convert to 8bpp");
    let w = pix8.width();
    let h = pix8.height();

    // Fade from left to black
    let mut pix_mut = pix8.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromLeft, FadeTarget::ToBlack, 0.5, 0.8)
        .expect("fade left black 8bpp");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit8);

    // Fade from right to white
    let mut pix_mut = pix8.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromRight, FadeTarget::ToWhite, 0.5, 0.8)
        .expect("fade right white 8bpp");
    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);

    assert!(rp.cleanup(), "blend5 edge_fade_gray test failed");
}

/// Test combined edge fades from all four directions (C check 11).
///
/// C: Applies all four fades sequentially on the same image.
///
/// Rust: Apply all four fades to the same PixMut.
#[test]
fn blend5_reg_edge_fade_combined() {
    let mut rp = RegParams::new("blend5_fade_combined");

    let pix = crate::common::load_test_image("Leptonica.jpg").expect("load Leptonica.jpg");
    let pix32 = pix.convert_to_32().expect("convert to 32bpp");
    let w = pix32.width();
    let h = pix32.height();

    let mut pix_mut = pix32.to_mut();
    pix_mut
        .linear_edge_fade(FadeDirection::FromLeft, FadeTarget::ToWhite, 0.3, 0.8)
        .expect("fade left");
    pix_mut
        .linear_edge_fade(FadeDirection::FromRight, FadeTarget::ToWhite, 0.3, 0.8)
        .expect("fade right");
    pix_mut
        .linear_edge_fade(FadeDirection::FromTop, FadeTarget::ToWhite, 0.3, 0.8)
        .expect("fade top");
    pix_mut
        .linear_edge_fade(FadeDirection::FromBottom, FadeTarget::ToWhite, 0.3, 0.8)
        .expect("fade bottom");

    let result: leptonica::Pix = pix_mut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "blend5 edge_fade_combined test failed");
}

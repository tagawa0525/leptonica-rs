//! Paint regression test
//!
//! Tests painting on images of various types: colorizing gray pixels,
//! painting through masks, and rendering lines and boxes.
//! The C version tests pixColorGray, pixPaintThroughMask, and rendering
//! functions on both RGB and colormapped images.
//!
//! Partial migration: pix_color_gray on 32bpp, paint_through_mask,
//! render_line_color, render_box_color, render_line_blend, and
//! render_box_blend are tested. Colormap-based operations (pixColorGrayCmap,
//! pixColorGrayRegions, ReconstructByValue) are not available.
//! Test image lucasta-frag.jpg is not available; lucasta.150.jpg is used.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/paint_reg.c`

use leptonica_color::{ColorGrayOptions, PaintType, pix_color_gray, threshold_to_binary};
use leptonica_core::{Color, PixelDepth};
use leptonica_test::RegParams;

/// Test pix_color_gray on 32bpp RGB (C checks 0-1, 4-5).
///
/// Colorizes dark and light gray pixels in a 32bpp image.
#[test]
fn paint_reg_color_gray() {
    let mut rp = RegParams::new("paint_cgray");

    // C: pixs = pixRead("lucasta-frag.jpg"); pixt = pixConvert8To32(pixs);
    let pix8 = leptonica_test::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix = pix8.convert_8_to_32().expect("convert_8_to_32");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // C: pixColorGray(pixt, box, L_PAINT_DARK, 220, 0, 0, 255) — blue on dark
    let region = leptonica_core::Box::new(120, 30, 200, 200).expect("create box");
    let dark_options = ColorGrayOptions {
        paint_type: PaintType::Dark,
        threshold: 220,
        target_color: (0, 0, 255),
    };
    let result = pix_color_gray(&pix, Some(&region), &dark_options).expect("color_gray dark box");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // C: pixColorGray(pixt, NULL, L_PAINT_DARK, 220, 255, 100, 100) — red on dark
    let dark_full = ColorGrayOptions {
        paint_type: PaintType::Dark,
        threshold: 220,
        target_color: (255, 100, 100),
    };
    let result2 = pix_color_gray(&result, None, &dark_full).expect("color_gray dark full");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);

    // C: pixColorGray(pixt, box, L_PAINT_LIGHT, 20, 0, 0, 255) — blue on light
    let light_options = ColorGrayOptions {
        paint_type: PaintType::Light,
        threshold: 20,
        target_color: (0, 0, 255),
    };
    let result3 = pix_color_gray(&pix, Some(&region), &light_options).expect("color_gray light");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);

    assert!(rp.cleanup(), "paint color_gray test failed");
}

/// Test paint_through_mask on 32bpp (C check 8).
///
/// Creates a binary mask from thresholding, then paints a color through it.
#[test]
fn paint_reg_through_mask() {
    let mut rp = RegParams::new("paint_mask");

    // C: pixs = pixRead("lucasta-frag.jpg");
    let pix8 = leptonica_test::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix32 = pix8.convert_8_to_32().expect("convert_8_to_32");
    let w = pix32.width();
    let h = pix32.height();

    // Create a mask by thresholding and inverting
    // C: pixb = pixThresholdToBinary(pixg, 180); pixInvert(pixb, pixb);
    let mask = threshold_to_binary(&pix8, 180).expect("threshold for mask");
    let mask = mask.invert();

    // C: composeRGBPixel(50, 0, 250, &val32); pixPaintThroughMask(pixt, pixb, x, y, val32);
    let val = leptonica_core::color::compose_rgb(50, 0, 250);
    let mut pixmut = pix32.try_into_mut().expect("try_into_mut");
    pixmut
        .paint_through_mask(&mask, 0, 0, val)
        .expect("paint_through_mask");
    let result: leptonica_core::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "paint through mask test failed");
}

/// Test render_line_color and render_box_color on 32bpp (C check 10).
///
/// Renders colored lines and box outlines on a 32bpp image.
#[test]
fn paint_reg_render_color() {
    let mut rp = RegParams::new("paint_render");

    let pix8 = leptonica_test::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix32 = pix8.convert_8_to_32().expect("convert_8_to_32");
    let w = pix32.width();
    let h = pix32.height();

    let mut pixmut = pix32.try_into_mut().expect("try_into_mut");

    // C: pixRenderLineArb(pixt, 450, 20, 850, 320, 5, 200, 50, 125);
    let color1 = Color::new(200, 50, 125);
    pixmut
        .render_line_color(50, 20, 350, 200, 5, color1)
        .expect("render_line_color");

    // C: pixRenderLineArb(pixt, 30, 40, 440, 40, 5, 100, 200, 25);
    let color2 = Color::new(100, 200, 25);
    pixmut
        .render_line_color(30, 40, 300, 40, 5, color2)
        .expect("render_line_color 2");

    // C: box = boxCreate(70, 80, 300, 245); pixRenderBoxArb(pixt, box, 3, 200, 200, 25);
    let region = leptonica_core::Box::new(70, 80, 200, 150).expect("create box");
    let color3 = Color::new(200, 200, 25);
    pixmut
        .render_box_color(&region, 3, color3)
        .expect("render_box_color");

    let result: leptonica_core::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "paint render color test failed");
}

/// Test render_line_blend and render_box_blend (C check 12).
///
/// Renders blended lines and box outlines on a 32bpp image.
#[test]
fn paint_reg_render_blend() {
    let mut rp = RegParams::new("paint_blend");

    let pix8 = leptonica_test::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix32 = pix8.convert_8_to_32().expect("convert_8_to_32");
    let w = pix32.width();
    let h = pix32.height();

    let mut pixmut = pix32.try_into_mut().expect("try_into_mut");

    // C: pixRenderLineBlend(pixt, 450, 20, 850, 320, 5, 200, 50, 125, 0.35);
    let color1 = Color::new(200, 50, 125);
    pixmut
        .render_line_blend(50, 20, 350, 200, 5, color1, 0.35)
        .expect("render_line_blend");

    // C: box = boxCreate(70, 80, 300, 245); pixRenderBoxBlend(pixt, box, 3, 200, 200, 25, 0.6);
    let region = leptonica_core::Box::new(70, 80, 200, 150).expect("create box");
    let color2 = Color::new(200, 200, 25);
    pixmut
        .render_box_blend(&region, 3, color2, 0.6)
        .expect("render_box_blend");

    let result: leptonica_core::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    assert!(rp.cleanup(), "paint render blend test failed");
}

/// Test colormap-based painting (C checks 2-3, 6-7, 9, 13-22).
///
/// Requires pixColorGrayCmap, pixColorGrayRegions, pixThresholdTo4bpp,
/// pixSetSelectCmap, ReconstructByValue, and FakeReconstructByBand
/// which are not available or require colormap input.
#[test]
#[ignore = "not yet implemented: pixColorGrayCmap/pixColorGrayRegions/ReconstructByValue not available"]
fn paint_reg_colormap() {
    // C version:
    // pixt = pixThresholdTo4bpp(pixs, 6, 1);
    // pixColorGray(pixt, box, L_PAINT_DARK, 220, 0, 0, 255); -- on cmapped
    // pixColorGrayCmap(pix2, box1, L_PAINT_LIGHT, 130, 207, 43);
    // pix4 = pixColorGrayRegions(pix2, boxa, L_PAINT_DARK, 230, 255, 0, 0);
    // pixd = ReconstructByValue(rp, "weasel2.4c.png");
}

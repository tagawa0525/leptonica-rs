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
//! C Leptonica: `prog/paint_reg.c`

use crate::common::RegParams;
use leptonica::color::{PaintType, pix_color_gray, threshold_to_binary};
use leptonica::io::ImageFormat;
use leptonica::{Color, PixelDepth};

/// Test pix_color_gray on 32bpp RGB (C checks 0-1, 4-5).
///
/// Colorizes dark and light gray pixels in a 32bpp image.
#[test]
fn paint_reg_color_gray() {
    let mut rp = RegParams::new("paint_cgray");

    // C: pixs = pixRead("lucasta-frag.jpg"); pixt = pixConvert8To32(pixs);
    let pix8 = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix = pix8.convert_8_to_32().expect("convert_8_to_32");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // C: pixColorGray(pixt, box, L_PAINT_DARK, 220, 0, 0, 255) — blue on dark
    let region = leptonica::Box::new(120, 30, 200, 200).expect("create box");
    let result = pix_color_gray(&pix, Some(&region), PaintType::Dark, 220, (0, 0, 255))
        .expect("color_gray dark box");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result paint_color_gray");

    // C check 1: pixColorGray(pixt, NULL, L_PAINT_DARK, 220, 255, 100, 100) — red on dark
    let result2 = pix_color_gray(&result, None, PaintType::Dark, 220, (255, 100, 100))
        .expect("color_gray dark full");
    rp.compare_values(w as f64, result2.width() as f64, 0.0);
    rp.write_pix_and_check(&result2, ImageFormat::Png)
        .expect("check: color_gray dark full");

    // C check 4: pixColorGray(pixt, box, L_PAINT_LIGHT, 20, 0, 0, 255) — blue on light
    let result3 = pix_color_gray(&pix, Some(&region), PaintType::Light, 20, (0, 0, 255))
        .expect("color_gray light");
    rp.compare_values(w as f64, result3.width() as f64, 0.0);
    rp.write_pix_and_check(&result3, ImageFormat::Png)
        .expect("check: color_gray light box");

    // C check 5: pixColorGray(pixt, NULL, L_PAINT_LIGHT, 20, 255, 100, 100)
    let result4 = pix_color_gray(&result3, None, PaintType::Light, 20, (255, 100, 100))
        .expect("color_gray light full");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);
    rp.write_pix_and_check(&result4, ImageFormat::Png)
        .expect("check: color_gray light full");

    assert!(rp.cleanup(), "paint color_gray test failed");
}

/// Test paint_through_mask on 32bpp (C check 8).
///
/// Creates a binary mask from thresholding, then paints a color through it.
#[test]
fn paint_reg_through_mask() {
    let mut rp = RegParams::new("paint_mask");

    // C: pixs = pixRead("lucasta-frag.jpg");
    let pix8 = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let pix32 = pix8.convert_8_to_32().expect("convert_8_to_32");
    let w = pix32.width();
    let h = pix32.height();

    // Create a mask by thresholding and inverting
    // C: pixb = pixThresholdToBinary(pixg, 180); pixInvert(pixb, pixb);
    let mask = threshold_to_binary(&pix8, 180).expect("threshold for mask");
    let mask = mask.invert();

    // C: composeRGBPixel(50, 0, 250, &val32); pixPaintThroughMask(pixt, pixb, x, y, val32);
    let val = leptonica::core::pixel::compose_rgb(50, 0, 250);
    let mut pixmut = pix32.try_into_mut().expect("try_into_mut");
    pixmut
        .paint_through_mask(&mask, 0, 0, val)
        .expect("paint_through_mask");
    let result: leptonica::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result through_mask");

    assert!(rp.cleanup(), "paint through mask test failed");
}

/// Test render_line_color and render_box_color on 32bpp (C check 10).
///
/// Renders colored lines and box outlines on a 32bpp image.
#[test]
fn paint_reg_render_color() {
    let mut rp = RegParams::new("paint_render");

    let pix8 = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
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
    let region = leptonica::Box::new(70, 80, 200, 150).expect("create box");
    let color3 = Color::new(200, 200, 25);
    pixmut
        .render_box_color(&region, 3, color3)
        .expect("render_box_color");

    let result: leptonica::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result render_color");

    assert!(rp.cleanup(), "paint render color test failed");
}

/// Test render_line_blend and render_box_blend (C check 12).
///
/// Renders blended lines and box outlines on a 32bpp image.
#[test]
fn paint_reg_render_blend() {
    let mut rp = RegParams::new("paint_blend");

    let pix8 = crate::common::load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
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
    let region = leptonica::Box::new(70, 80, 200, 150).expect("create box");
    let color2 = Color::new(200, 200, 25);
    pixmut
        .render_box_blend(&region, 3, color2, 0.6)
        .expect("render_box_blend");

    let result: leptonica::Pix = pixmut.into();
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result render_blend");

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

/// C-compat: `prog/paint_reg.c` checks 23-28.
///
/// The colormap reconstruction block. Its inputs (`weasel2.4c.png`,
/// `weasel4.11c.png`, `weasel8.240c.png`) are lossless colormapped PNGs, so
/// unlike the JPEG-driven checks earlier in that program these are
/// bit-exactly comparable against C.
#[test]
fn paint_c_compat_reconstruct() {
    use leptonica::Pix;
    use leptonica::color::paintcmap::pix_set_masked_cmap;
    use leptonica::color::{generate_mask_by_band, generate_mask_by_value};
    use leptonica::core::PixColormap;

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("paint_c");

    const NAMES: [&str; 3] = ["weasel2.4c.png", "weasel4.11c.png", "weasel8.240c.png"];

    // C 23-25: rebuild each cmapped image one colormap index at a time.
    // pixd starts as a template (same cmap, zeroed data), so every colour is
    // already present and pixSetMaskedCmap reuses its index.
    for name in NAMES {
        let pixs = crate::common::load_test_image(name).expect("load weasel");
        let n = pixs.colormap().expect("cmapped input").len();
        let pixd = pixs.create_template();
        let mut dm = pixd.try_into_mut().unwrap();
        for i in 0..n {
            let mask = generate_mask_by_value(&pixs, i as u32).expect("mask by value");
            let (r, g, b) = pixs.colormap().unwrap().get_rgb(i).expect("cmap entry");
            pix_set_masked_cmap(&mut dm, &mask, 0, 0, (r, g, b)).expect("set masked cmap");
        }
        let pixd: Pix = dm.into();
        // C: regTestComparePix(rp, pixs, pixd) — the reconstruction is exact.
        rp.compare_pix(&pixs, &pixd);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: reconstruct by value");
    }

    // C 26-28: rebuild with pairs of colormap entries collapsed into their
    // average colour, into a fresh (initially empty) colormap.
    for name in NAMES {
        let pixs = crate::common::load_test_image(name).expect("load weasel");
        let cmaps = pixs.colormap().expect("cmapped input").clone();
        let n = cmaps.len();
        let nbands = n.div_ceil(2);
        let pixd = pixs.create_template();
        let mut dm = pixd.try_into_mut().unwrap();
        dm.set_colormap(Some(
            PixColormap::new(pixs.depth().bits()).expect("empty cmap"),
        ))
        .expect("set empty cmap");
        for i in 0..nbands {
            let jlow = 2 * i;
            let jup = (jlow + 1).min(n - 1);
            let mask =
                generate_mask_by_band(&pixs, jlow as u32, jup as u32, true).expect("mask by band");
            let (r1, g1, b1) = cmaps.get_rgb(jlow).expect("cmap low");
            let (r2, g2, b2) = cmaps.get_rgb(jup).expect("cmap up");
            let avg = (
                ((r1 as u32 + r2 as u32) / 2) as u8,
                ((g1 as u32 + g2 as u32) / 2) as u8,
                ((b1 as u32 + b2 as u32) / 2) as u8,
            );
            pix_set_masked_cmap(&mut dm, &mask, 0, 0, avg).expect("set masked cmap band");
        }
        let pixd: Pix = dm.into();
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: fake reconstruct by band");
    }

    assert!(rp.cleanup(), "paint C-compat reconstruction test failed");
}

/// C-compat: `prog/paint_reg.c` checks 18-22.
///
/// The `feyn-fract.tif` block: a lossless 1 bpp input is blurred, thresholded
/// and connected-component analysed, then the gray regions are colorized both
/// as 32 bpp RGB and through a colormap.
#[test]
fn paint_c_compat_color_gray() {
    use leptonica::Pix;
    use leptonica::color::{color_gray_regions, pix_color_gray, threshold_on_8bpp};
    use leptonica::filter::{Kernel, convolve};
    use leptonica::region::{ConnectivityType, conncomp_pixa};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("paint_cg");

    let pixs = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pix1 = pixs.convert_to_8().expect("convert to 8");
    let kel = Kernel::make_gaussian(2, 2, 1.5, 1.0).expect("gaussian kernel");
    let pix2 = convolve(&pix1, &kel).expect("convolve");
    let pix3 = threshold_to_binary(&pix2, 230).expect("threshold to binary");
    let (boxa, _) = conncomp_pixa(&pix3, ConnectivityType::EightWay).expect("conn comp");

    // C 18: colorize each component in the gray image (result is 32 bpp).
    let pix4 = color_gray_regions(&pix2, &boxa, PaintType::Dark, 230, (255, 0, 0))
        .expect("color_gray_regions gray");
    rp.write_pix_and_check(&pix4, ImageFormat::Png)
        .expect("check: color_gray_regions on 8bpp");

    // C 19: threshold to 10 levels of gray, keeping a colormap.
    let pix3c = threshold_on_8bpp(&pix2, 10, true).expect("threshold on 8bpp");
    rp.write_pix_and_check(&pix3c, ImageFormat::Png)
        .expect("check: threshold_on_8bpp 10 levels");

    // C 20: colorize each component in the cmapped image (stays cmapped).
    let pix5 = color_gray_regions(&pix3c, &boxa, PaintType::Dark, 230, (255, 0, 0))
        .expect("color_gray_regions cmapped");
    rp.write_pix_and_check(&pix5, ImageFormat::Png)
        .expect("check: color_gray_regions on cmapped");

    // C 21: colorize the entire gray image, not component-wise.
    let pix6: Pix =
        pix_color_gray(&pix2, None, PaintType::Dark, 230, (255, 0, 0)).expect("color_gray gray");
    rp.write_pix_and_check(&pix6, ImageFormat::Png)
        .expect("check: color_gray on 8bpp");

    // C 22: colorize the entire cmapped image.
    let pix7 =
        pix_color_gray(&pix3c, None, PaintType::Dark, 230, (255, 0, 0)).expect("color_gray cmap");
    rp.write_pix_and_check(&pix7, ImageFormat::Png)
        .expect("check: color_gray on cmapped");

    assert!(rp.cleanup(), "paint C-compat color gray test failed");
}

//! Coloring regression test
//!
//! Tests pix_shift_by_component for background and foreground color
//! shifting, on both colormapped and 32bpp RGB images, plus the
//! C-comparable series over harmoniam100-11.png (plan 902 PR 13).
//!
//! # See also
//!
//! C Leptonica: `prog/coloring_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::pix_shift_by_component;
use leptonica::io::ImageFormat;

/// Test pix_shift_by_component for background coloring (C checks 4-7).
///
/// Shifts white pixels to various background tints on a 32bpp RGB image.
#[test]
fn coloring_reg_background_shift() {
    let mut rp = RegParams::new("coloring_bg");

    let pix = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit32);
    let w = pix.width();
    let h = pix.height();

    // C: pixShiftByComponent(NULL, pix1, 0xffffff00, dcolor) with (255,255,235)
    let result =
        pix_shift_by_component(&pix, 0xffffff00, 0xffffeb00).expect("bg shift to warm white");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result background_shift");

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

    let pix = crate::common::load_test_image("test24.jpg").expect("load test24.jpg");
    let w = pix.width();
    let h = pix.height();

    // C: pixShiftByComponent(NULL, pix3, 0x00000000, dcolor)
    // composeRGBPixel(200, 30, 150) = 0xc81e9600
    let result = pix_shift_by_component(&pix, 0x00000000, 0xc81e9600).expect("fg shift to purple");
    rp.compare_values(w as f64, result.width() as f64, 0.0);
    rp.compare_values(h as f64, result.height() as f64, 0.0);
    assert_eq!(result.depth(), PixelDepth::Bit32);
    rp.write_pix_and_check(&result, ImageFormat::Png)
        .expect("write result foreground_shift");

    assert!(rp.cleanup(), "coloring foreground shift test failed");
}

/// C-comparable coloring series (plan 902 PR 13).
///
/// Mirrors C coloring_reg checks 2-15 exactly: harmoniam100-11.png
/// (8bpp cmapped), bmfCreate(NULL, 8) — identical to the C "fonts" pixa
/// per genfonts (PR 12) — and pixAddSingleTextblock at L_ADD_AT_BOT with
/// color 0xff000000 on every output. The colormap resets accumulate on
/// pix0 across sections, as in C.
#[test]
fn coloring_c_compat() {
    use leptonica::Bmf;
    use leptonica::core::bmf::TextblockLocation;
    use leptonica::core::pixel;

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("coloring_c");

    let mut pix0 = crate::common::load_test_image("harmoniam100-11.png").expect("load harmoniam");
    assert!(pix0.has_colormap());
    let bmf = Bmf::new(8).expect("bmf size 8");
    let bgcolors: [(u8, u8, u8); 4] = [
        (255, 255, 235),
        (255, 245, 235),
        (255, 235, 245),
        (235, 245, 255),
    ];
    let white_index = pix0
        .colormap()
        .unwrap()
        .get_index(255, 255, 255)
        .expect("white in cmap");

    let mut pixa: Vec<leptonica::Pix> = Vec::new();

    // C checks 2-5: cmapped coloring of the white pixels only, by
    // resetting the white colormap entry in place (accumulates on pix0).
    for (r, g, b) in bgcolors {
        let mut pm = pix0.try_into_mut().unwrap_or_else(|p| p.to_mut());
        pm.colormap_mut()
            .unwrap()
            .reset_color(white_index, r, g, b)
            .expect("reset white entry");
        pix0 = pm.into();
        let buf = format!("(rval, bval, gval) = ({r}, {g}, {b})");
        let (pix1, _) = bmf
            .add_single_textblock(&pix0, &buf, 0xff000000, TextblockLocation::AtBot)
            .expect("textblock on cmap-reset");
        pixa.push(pix1);
    }

    // C checks 6-9: cmapped background coloring on all pixels. Note pix0
    // still carries the last reset (white -> (235, 245, 255)).
    for (r, g, b) in bgcolors {
        let dcolor = pixel::compose_rgba(r, g, b, 0);
        let pix1 = pix_shift_by_component(&pix0, 0xffffff00, dcolor).expect("cmapped bg shift");
        let buf = format!("(rval, bval, gval) = ({r}, {g}, {b})");
        let (pix2, _) = bmf
            .add_single_textblock(&pix1, &buf, 0xff000000, TextblockLocation::AtBot)
            .expect("textblock on cmapped shift");
        pixa.push(pix2);
    }

    // C checks 10-13: background coloring on rgb.
    let pix1_32 = pix0.convert_to_32().expect("convert to 32");
    for (r, g, b) in bgcolors {
        let dcolor = pixel::compose_rgba(r, g, b, 0);
        let pix2 = pix_shift_by_component(&pix1_32, 0xffffff00, dcolor).expect("rgb bg shift");
        let buf = format!("(rval, bval, gval) = ({r}, {g}, {b})");
        let (pix3, _) = bmf
            .add_single_textblock(&pix2, &buf, 0xff000000, TextblockLocation::AtBot)
            .expect("textblock on rgb shift");
        pixa.push(pix3);
    }

    // C checks 14-15 (and compares 0-1): fg coloring, cmapped vs rgb.
    let dcolor = pixel::compose_rgba(200, 30, 150, 0);
    let fg_cmapped = pix_shift_by_component(&pix0, 0x0000_0000, dcolor).expect("fg cmapped");
    let buf = "(rval, bval, gval) = (200, 100, 50)";
    let (fg_cmapped_txt, _) = bmf
        .add_single_textblock(&fg_cmapped, buf, 0xff000000, TextblockLocation::AtBot)
        .expect("textblock fg cmapped");
    pixa.push(fg_cmapped_txt.clone());
    let fg_rgb =
        pix_shift_by_component(&pix0.convert_to_32().expect("to 32"), 0, dcolor).expect("fg rgb");
    let (fg_rgb_txt, _) = bmf
        .add_single_textblock(&fg_rgb, buf, 0xff000000, TextblockLocation::AtBot)
        .expect("textblock fg rgb");
    pixa.push(fg_rgb_txt.clone());

    // C regTestComparePix 0-1: the cmapped and rgb paths must agree.
    // (C pixEqual expands the colormap; here we convert to 32bpp first.
    // These two calls consume indices 1-2, keeping the write indices
    // aligned with C at rust_index = c_index + 1.)
    let a = fg_cmapped.convert_to_32().expect("cmapped to 32");
    rp.compare_pix(&a, &fg_rgb);
    let b = fg_cmapped_txt.convert_to_32().expect("cmapped txt to 32");
    rp.compare_pix(&b, &fg_rgb_txt);

    // C checks 2-15: write out all 14 images.
    for pix in &pixa {
        rp.write_pix_and_check(pix, ImageFormat::Png)
            .expect("check: coloring output");
    }

    assert!(rp.cleanup(), "coloring c-compat test failed");
}

/// Test pixShiftByComponent on colormapped images (plan 902 PR 13).
///
/// C pixShiftByComponent on a cmapped pix leaves the index raster
/// untouched and shifts each colormap entry with
/// pixcmapShiftByComponent. Expected entry values are hand-computed
/// from the C formula (dst < src: val*dst/src; dst > src:
/// 255 - (255-dst)*(255-val)/(255-src), truncating).
#[test]
fn coloring_reg_colormap_shift() {
    use leptonica::core::{PixColormap, RgbaQuad};

    // 2x1 2bpp cmapped pix: index 0 = white, index 1 = mid gray.
    let pix = {
        let p = leptonica::Pix::new(2, 1, PixelDepth::Bit2).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_color(RgbaQuad::rgb(255, 255, 255)).unwrap();
        cmap.add_color(RgbaQuad::rgb(128, 64, 200)).unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        pm.set_pixel(0, 0, 0).unwrap();
        pm.set_pixel(1, 0, 1).unwrap();
        let p: leptonica::Pix = pm.into();
        p
    };

    // Shift white -> (255, 255, 235): only blue moves (dst < src).
    let out = pix_shift_by_component(&pix, 0xffffff00, 0xffffeb00).expect("cmapped shift");
    assert_eq!(out.depth(), PixelDepth::Bit2);
    let cmap = out.colormap().expect("colormap preserved");
    // white: b = 255*235/255 = 235
    assert_eq!(cmap.get_rgb(0).unwrap(), (255, 255, 235));
    // mid gray: r,g unchanged; b = 200*235/255 = 184 (truncated)
    assert_eq!(cmap.get_rgb(1).unwrap(), (128, 64, 184));
    // index raster is untouched
    assert_eq!(out.get_pixel(0, 0).unwrap(), 0);
    assert_eq!(out.get_pixel(1, 0).unwrap(), 1);

    // Foreground shift black -> (200, 30, 150): dst > src pushes toward 255.
    // white stays white: 255 - (255-d)*(255-255)/255 = 255 for each channel.
    let out = pix_shift_by_component(&pix, 0x00000000, 0xc81e9600).expect("fg cmapped shift");
    let cmap = out.colormap().expect("colormap preserved");
    assert_eq!(cmap.get_rgb(0).unwrap(), (255, 255, 255));
    // mid gray (128, 64, 200):
    //   r = 255 - (255-200)*(255-128)/255 = 255 - 55*127/255 = 255 - 27 = 228
    //   g = 255 - (255-30)*(255-64)/255 = 255 - 225*191/255 = 255 - 168 = 87
    //   b = 255 - (255-150)*(255-200)/255 = 255 - 105*55/255 = 255 - 22 = 233
    assert_eq!(cmap.get_rgb(1).unwrap(), (228, 87, 233));
}

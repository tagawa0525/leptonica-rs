//! Colormap painting regression test
//!
//! Tests colormap painting functions for modifying colormapped images.
//!
//! # See also
//!
//! C Leptonica: `prog/paintcmap_reg.c`

use leptonica::core::{Pix, PixColormap, PixelDepth, RgbaQuad};

/// Test pix_set_select_cmap: repaint a colormap entry.
#[test]
fn paintcmap_set_select() {
    use leptonica::color::paintcmap::pix_set_select_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap(); // index 0: black
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap(); // index 1: gray

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    pix.set_pixel(5, 5, 1).unwrap(); // Set to gray

    pix_set_select_cmap(&mut pix, None, 1, (255, 0, 0)).unwrap();

    // Now index 1 should be red
    let cmap = pix.colormap().unwrap();
    let (r, g, b) = cmap.get_rgb(1).unwrap();
    assert_eq!((r, g, b), (255, 0, 0));
}

/// Test pix_color_gray_cmap: colorize gray entries.
#[test]
fn paintcmap_color_gray() {
    use leptonica::color::paintcmap::pix_color_gray_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 255, 255)).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    for x in 0..10 {
        pix.set_pixel(x, 0, 1).unwrap(); // gray row
    }

    pix_color_gray_cmap(&mut pix, None, (255, 0, 0), 1, 254).unwrap();
}

/// Test pix_color_gray_regions_cmap with bounding boxes.
#[test]
fn paintcmap_color_gray_regions() {
    use leptonica::color::paintcmap::pix_color_gray_regions_cmap;
    use leptonica::core::Boxa;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(100, 100, 100)).unwrap();

    let mut pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();

    let mut boxa = Boxa::new();
    boxa.push(leptonica::Box::new(0, 0, 10, 10).unwrap());

    pix_color_gray_regions_cmap(&mut pix, &boxa, (0, 255, 0), 1, 254).unwrap();
}

/// Test pix_color_gray_masked_cmap.
#[test]
fn paintcmap_color_gray_masked() {
    use leptonica::color::paintcmap::pix_color_gray_masked_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    pix.set_pixel(5, 5, 1).unwrap();

    // Create mask
    let mut mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap().to_mut();
    mask.set_pixel(5, 5, 1).unwrap();
    let mask: Pix = mask.into();

    pix_color_gray_masked_cmap(&mut pix, &mask, (0, 0, 255), 1, 254).unwrap();
}

/// Test add_colorized_gray_to_cmap.
#[test]
fn paintcmap_add_colorized_gray() {
    use leptonica::color::paintcmap::add_colorized_gray_to_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 255, 255)).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 0, 0)).unwrap(); // non-gray

    let n_before = cmap.len();
    let mapping = add_colorized_gray_to_cmap(&mut cmap, (255, 0, 0)).unwrap();

    // Should have added new entries for gray entries (indices 0, 1, 2)
    assert!(!mapping.is_empty());
    assert!(cmap.len() > n_before);
}

/// Test pix_set_select_masked_cmap.
#[test]
fn paintcmap_set_select_masked() {
    use leptonica::color::paintcmap::pix_set_select_masked_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    pix.set_pixel(5, 5, 1).unwrap(); // gray index

    let mut mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap().to_mut();
    mask.set_pixel(5, 5, 1).unwrap();
    let mask: Pix = mask.into();

    pix_set_select_masked_cmap(&mut pix, &mask, 0, 0, 1, (0, 255, 0)).unwrap();
}

/// Test pix_set_masked_cmap.
#[test]
fn paintcmap_set_masked() {
    use leptonica::color::paintcmap::pix_set_masked_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();

    let mut mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap().to_mut();
    mask.set_pixel(3, 3, 1).unwrap();
    mask.set_pixel(7, 7, 1).unwrap();
    let mask: Pix = mask.into();

    pix_set_masked_cmap(&mut pix, &mask, 0, 0, (255, 128, 0)).unwrap();
}

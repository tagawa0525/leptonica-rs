//! Colormap painting regression test
//!
//! Tests colormap painting functions for modifying colormapped images.
//!
//! # See also
//!
//! C Leptonica: `prog/paintcmap_reg.c`

use leptonica::core::{Pix, PixColormap, PixelDepth, RgbaQuad};

/// Test pix_set_select_cmap: repaint the pixels of one index, not the entry.
///
/// C `pixSetSelectCmap` appends the new colour (index 2 here) and rewrites
/// the *pixels* that held `old_index`; the original entry is left alone, so
/// other pixels using index 1 keep their old colour.
#[test]
fn paintcmap_set_select() {
    use leptonica::color::paintcmap::pix_set_select_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap(); // index 0: black
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap(); // index 1: gray

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    pix.set_pixel(5, 5, 1).unwrap(); // Set to gray
    pix.set_pixel(9, 9, 1).unwrap(); // Also gray, but outside the region

    let region = leptonica::core::Box::new(0, 0, 8, 8).unwrap();
    pix_set_select_cmap(&mut pix, Some(&region), 1, (255, 0, 0)).unwrap();

    // A new entry was appended; index 1 still holds the original gray.
    let cmap = pix.colormap().unwrap();
    assert_eq!(cmap.len(), 3);
    assert_eq!(cmap.get_rgb(1).unwrap(), (128, 128, 128));
    assert_eq!(cmap.get_rgb(2).unwrap(), (255, 0, 0));

    // Only the pixel inside the region was repainted.
    assert_eq!(pix.get_pixel(5, 5).unwrap(), 2);
    assert_eq!(pix.get_pixel(9, 9).unwrap(), 1);
}

/// An already-present colour is reused rather than appended again.
#[test]
fn paintcmap_set_select_reuses_existing_color() {
    use leptonica::color::paintcmap::pix_set_select_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 0, 0)).unwrap();

    let mut pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();
    pix.set_pixel(1, 1, 1).unwrap();

    pix_set_select_cmap(&mut pix, None, 1, (255, 0, 0)).unwrap();

    assert_eq!(pix.colormap().unwrap().len(), 3);
    assert_eq!(pix.get_pixel(1, 1).unwrap(), 2);
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

    pix_color_gray_cmap(
        &mut pix,
        None,
        leptonica::color::PaintType::Light,
        (255, 0, 0),
    )
    .unwrap();
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

    pix_color_gray_regions_cmap(
        &mut pix,
        &boxa,
        leptonica::color::PaintType::Light,
        (0, 255, 0),
    )
    .unwrap();
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
    let map =
        add_colorized_gray_to_cmap(&mut cmap, leptonica::color::PaintType::Light, (255, 0, 0))
            .unwrap();

    // One map entry per original colormap entry.
    assert_eq!(map.len(), n_before);
    // Index 0 is black, which PaintType::Light leaves alone; 1 and 2 are gray.
    assert_eq!(map[0], leptonica::color::paintcmap::CMAP_NO_REMAP);
    assert_ne!(map[1], leptonica::color::paintcmap::CMAP_NO_REMAP);
    assert_ne!(map[2], leptonica::color::paintcmap::CMAP_NO_REMAP);
    // Index 3 is red, not gray.
    assert_eq!(map[3], leptonica::color::paintcmap::CMAP_NO_REMAP);
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

/// Test pix_set_masked_cmap: a colour not yet present is appended.
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

    assert_eq!(pix.colormap().unwrap().len(), 3);
    assert_eq!(pix.get_pixel(3, 3).unwrap(), 2);
    assert_eq!(pix.get_pixel(7, 7).unwrap(), 2);
    assert_eq!(pix.get_pixel(0, 0).unwrap(), 0);
}

/// C `pixSetMaskedCmap` looks the colour up first (`pixcmapGetIndex`) and
/// only appends when it is absent, so repainting with a colour already in
/// the colormap must reuse its entry rather than duplicate it.
#[test]
fn paintcmap_set_masked_reuses_existing_color() {
    use leptonica::color::paintcmap::pix_set_masked_cmap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(0, 0, 0)).unwrap();
    cmap.add_color(RgbaQuad::rgb(128, 128, 128)).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 128, 0)).unwrap();

    let mut pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();

    let mut mask = Pix::new(4, 4, PixelDepth::Bit1).unwrap().to_mut();
    mask.set_pixel(1, 1, 1).unwrap();
    let mask: Pix = mask.into();

    pix_set_masked_cmap(&mut pix, &mask, 0, 0, (255, 128, 0)).unwrap();

    assert_eq!(pix.colormap().unwrap().len(), 3);
    assert_eq!(pix.get_pixel(1, 1).unwrap(), 2);
}

/// When the colormap is full and the colour is absent, C returns an error
/// ("no room in cmap"); the nearest-colour fallback lives one level up.
#[test]
fn paintcmap_set_masked_full_cmap_is_an_error() {
    use leptonica::color::paintcmap::pix_set_masked_cmap;

    let mut cmap = PixColormap::new(2).unwrap();
    for v in [0u8, 85, 170, 255] {
        cmap.add_color(RgbaQuad::rgb(v, v, v)).unwrap();
    }

    let mut pix = Pix::new(4, 4, PixelDepth::Bit2).unwrap().to_mut();
    pix.set_colormap(Some(cmap)).unwrap();

    let mut mask = Pix::new(4, 4, PixelDepth::Bit1).unwrap().to_mut();
    mask.set_pixel(1, 1, 1).unwrap();
    let mask: Pix = mask.into();

    assert!(pix_set_masked_cmap(&mut pix, &mask, 0, 0, (10, 20, 30)).is_err());
}

//! Regression tests for plan 108 (Pixa property/inspection 8 関数).

use leptonica::core::Box;
use leptonica::{Pix, PixColormap, Pixa, PixelDepth};

fn make_pix(w: u32, h: u32, d: PixelDepth) -> Pix {
    Pix::new(w, h, d).unwrap()
}

fn make_pix_with_cmap(w: u32, h: u32, color_entry: bool) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(0, 0, 0).unwrap();
    if color_entry {
        cmap.add_rgb(255, 0, 0).unwrap();
    } else {
        cmap.add_rgb(128, 128, 128).unwrap();
    }
    m.set_colormap(Some(cmap)).unwrap();
    m.into()
}

// -- any_colormaps -------------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn any_colormaps_none() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix(4, 4, PixelDepth::Bit32));
    assert!(!pa.any_colormaps());
}

#[test]
#[ignore = "not yet implemented"]
fn any_colormaps_one() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix_with_cmap(4, 4, false));
    assert!(pa.any_colormaps());
}

// -- has_color ----------------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn has_color_gray_only_is_false() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix_with_cmap(4, 4, false));
    assert!(!pa.has_color());
}

#[test]
#[ignore = "not yet implemented"]
fn has_color_via_cmap() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix_with_cmap(4, 4, true));
    assert!(pa.has_color());
}

#[test]
#[ignore = "not yet implemented"]
fn has_color_via_32bpp() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit32));
    assert!(pa.has_color());
}

// -- get_depth_info -----------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn get_depth_info_uniform() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    let (max, same) = pa.get_depth_info().unwrap();
    assert_eq!(max, 8);
    assert!(same);
}

#[test]
#[ignore = "not yet implemented"]
fn get_depth_info_mixed() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit1));
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix(4, 4, PixelDepth::Bit32));
    let (max, same) = pa.get_depth_info().unwrap();
    assert_eq!(max, 32);
    assert!(!same);
}

#[test]
#[ignore = "not yet implemented"]
fn get_depth_info_empty_errors() {
    let pa = Pixa::new();
    assert!(pa.get_depth_info().is_err());
}

// -- get_rendering_depth -------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn rendering_depth_all_1bpp() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit1));
    pa.push(make_pix(4, 4, PixelDepth::Bit1));
    assert_eq!(pa.get_rendering_depth().unwrap(), 1);
}

#[test]
#[ignore = "not yet implemented"]
fn rendering_depth_gray_8bpp() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    assert_eq!(pa.get_rendering_depth().unwrap(), 8);
}

#[test]
#[ignore = "not yet implemented"]
fn rendering_depth_color_returns_32() {
    let mut pa = Pixa::new();
    pa.push(make_pix(4, 4, PixelDepth::Bit8));
    pa.push(make_pix_with_cmap(4, 4, true));
    assert_eq!(pa.get_rendering_depth().unwrap(), 32);
}

// -- size_range ---------------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn size_range_single() {
    let mut pa = Pixa::new();
    pa.push(make_pix(7, 5, PixelDepth::Bit8));
    assert_eq!(pa.size_range(), Some((7, 5, 7, 5)));
}

#[test]
#[ignore = "not yet implemented"]
fn size_range_varied() {
    let mut pa = Pixa::new();
    pa.push(make_pix(7, 5, PixelDepth::Bit8));
    pa.push(make_pix(3, 9, PixelDepth::Bit8));
    pa.push(make_pix(11, 2, PixelDepth::Bit8));
    assert_eq!(pa.size_range(), Some((3, 2, 11, 9)));
}

#[test]
#[ignore = "not yet implemented"]
fn size_range_empty() {
    let pa = Pixa::new();
    assert_eq!(pa.size_range(), None);
}

// -- set_full_size_boxa --------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn set_full_size_boxa_creates_matching_boxes() {
    let mut pa = Pixa::new();
    pa.push(make_pix(7, 5, PixelDepth::Bit8));
    pa.push(make_pix(11, 9, PixelDepth::Bit8));
    pa.set_full_size_boxa();
    let b0 = pa.boxa().get(0).unwrap();
    let b1 = pa.boxa().get(1).unwrap();
    assert_eq!((b0.x, b0.y, b0.w, b0.h), (0, 0, 7, 5));
    assert_eq!((b1.x, b1.y, b1.w, b1.h), (0, 0, 11, 9));
}

#[test]
#[ignore = "not yet implemented"]
fn set_full_size_boxa_empty_noop() {
    let mut pa = Pixa::new();
    pa.set_full_size_boxa();
    assert_eq!(pa.boxa().len(), 0);
}

// -- equal_to_ordered ---------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn equal_to_ordered_identical() {
    let mut a = Pixa::new();
    let mut b = Pixa::new();
    for _ in 0..3 {
        a.push(make_pix(4, 4, PixelDepth::Bit8));
        b.push(make_pix(4, 4, PixelDepth::Bit8));
    }
    assert!(a.equal_to_ordered(&b, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn equal_to_ordered_length_diff() {
    let mut a = Pixa::new();
    a.push(make_pix(4, 4, PixelDepth::Bit8));
    let mut b = Pixa::new();
    b.push(make_pix(4, 4, PixelDepth::Bit8));
    b.push(make_pix(4, 4, PixelDepth::Bit8));
    assert!(!a.equal_to_ordered(&b, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn equal_to_ordered_pix_diff() {
    let mut a = Pixa::new();
    a.push(make_pix(4, 4, PixelDepth::Bit8));
    let mut b = Pixa::new();
    b.push(make_pix(5, 5, PixelDepth::Bit8));
    assert!(!a.equal_to_ordered(&b, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn equal_to_ordered_box_mismatch() {
    let mut a = Pixa::new();
    a.push_with_box(
        make_pix(4, 4, PixelDepth::Bit8),
        Box::new(0, 0, 4, 4).unwrap(),
    );
    let mut b = Pixa::new();
    b.push_with_box(
        make_pix(4, 4, PixelDepth::Bit8),
        Box::new(10, 10, 4, 4).unwrap(),
    );
    assert!(!a.equal_to_ordered(&b, 0));
}

// -- Pix::get_tile_count -----------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn get_tile_count_no_text() {
    let p = make_pix(4, 4, PixelDepth::Bit8);
    assert_eq!(p.get_tile_count(), 0);
}

#[test]
#[ignore = "not yet implemented"]
fn get_tile_count_valid_format() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_text(Some("n = 42".into()));
    let pix: Pix = m.into();
    assert_eq!(pix.get_tile_count(), 42);
}

#[test]
#[ignore = "not yet implemented"]
fn get_tile_count_bad_format() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_text(Some("not a count string".into()));
    let pix: Pix = m.into();
    assert_eq!(pix.get_tile_count(), 0);
}

#[test]
#[ignore = "not yet implemented"]
fn get_tile_count_short_text() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_text(Some("hi".into()));
    let pix: Pix = m.into();
    assert_eq!(pix.get_tile_count(), 0);
}

//! Regression tests for plan 112 (compare.c の補助系 5 関数).
//!
//! Covers `Colormap::equal_to` / `Pix::uses_cmap_color` /
//! `Pix::centroid8` / `Pix::pad_to_center_centroid` /
//! `pix_crop_aligned_to_centroid`.

use leptonica::core::PixColormap;
use leptonica::core::pix::compare::pix_crop_aligned_to_centroid;
use leptonica::{Pix, PixelDepth};

fn make_cmap_rgb(entries: &[(u8, u8, u8)]) -> PixColormap {
    let mut c = PixColormap::new(8).unwrap();
    for &(r, g, b) in entries {
        c.add_rgb(r, g, b).unwrap();
    }
    c
}

fn make_cmapped_pix(w: u32, h: u32, cmap: PixColormap) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    m.set_colormap(Some(cmap)).unwrap();
    m.into()
}

// -- Colormap::equal_to ---------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn cmap_equal_same_rgb() {
    let a = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0), (0, 255, 0)]);
    let b = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0), (0, 255, 0)]);
    assert!(a.equal_to(&b, false));
}

#[test]
#[ignore = "not yet implemented"]
fn cmap_equal_size_diff() {
    let a = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0)]);
    let b = make_cmap_rgb(&[(0, 0, 0)]);
    assert!(!a.equal_to(&b, false));
}

#[test]
#[ignore = "not yet implemented"]
fn cmap_equal_rgb_diff() {
    let a = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0)]);
    let b = make_cmap_rgb(&[(0, 0, 0), (254, 0, 0)]);
    assert!(!a.equal_to(&b, false));
}

#[test]
#[ignore = "not yet implemented"]
fn cmap_equal_alpha_only_ignored_when_3comp() {
    let mut a = PixColormap::new(8).unwrap();
    a.add_rgba(10, 20, 30, 200).unwrap();
    let mut b = PixColormap::new(8).unwrap();
    b.add_rgba(10, 20, 30, 100).unwrap();
    assert!(a.equal_to(&b, false));
    assert!(!a.equal_to(&b, true));
}

// -- Pix::uses_cmap_color -------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn uses_cmap_color_no_cmap_is_false() {
    let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
    assert!(!pix.uses_cmap_color().unwrap());
}

#[test]
#[ignore = "not yet implemented"]
fn uses_cmap_color_gray_cmap_only_is_false() {
    let cmap = make_cmap_rgb(&[(0, 0, 0), (128, 128, 128), (255, 255, 255)]);
    let pix = make_cmapped_pix(8, 8, cmap);
    assert!(!pix.uses_cmap_color().unwrap());
}

#[test]
#[ignore = "not yet implemented"]
fn uses_cmap_color_color_cmap_but_unused_is_false() {
    let cmap = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0)]);
    // All zeros so only entry 0 (black) is referenced.
    let pix = make_cmapped_pix(8, 8, cmap);
    assert!(!pix.uses_cmap_color().unwrap());
}

#[test]
#[ignore = "not yet implemented"]
fn uses_cmap_color_color_cmap_used_is_true() {
    let cmap = make_cmap_rgb(&[(0, 0, 0), (255, 0, 0)]);
    let pix = make_cmapped_pix(8, 8, cmap);
    let mut m = pix.try_into_mut().unwrap();
    m.set_pixel(0, 0, 1).unwrap();
    let pix: Pix = m.into();
    assert!(pix.uses_cmap_color().unwrap());
}

// -- Pix::centroid8 -------------------------------------------------------

#[test]
#[ignore = "not yet implemented"]
fn centroid8_rejects_non_8bpp() {
    let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
    assert!(pix.centroid8(1).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn centroid8_rejects_factor_zero() {
    let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
    assert!(pix.centroid8(0).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn centroid8_all_white_returns_center() {
    let pix = Pix::new(16, 10, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 0..10 {
        for x in 0..16 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    let pix: Pix = m.into();
    let (cx, cy) = pix.centroid8(1).unwrap();
    assert!((cx - 8.0).abs() < 1e-3);
    assert!((cy - 5.0).abs() < 1e-3);
}

#[test]
#[ignore = "not yet implemented"]
fn centroid8_single_dark_pixel() {
    // 8 bpp, all 255 except one black pixel at (3,2). After invert, the
    // single non-zero pixel is at (3,2), so centroid = (3,2).
    let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 0..8 {
        for x in 0..10 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    m.set_pixel(3, 2, 0).unwrap();
    let pix: Pix = m.into();
    let (cx, cy) = pix.centroid8(1).unwrap();
    assert!((cx - 3.0).abs() < 1e-3);
    assert!((cy - 2.0).abs() < 1e-3);
}

// -- pad_to_center_centroid + pix_crop_aligned_to_centroid ----------------

#[test]
#[ignore = "not yet implemented"]
fn pad_to_center_centroid_rejects_factor_zero() {
    let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
    assert!(pix.pad_to_center_centroid(0).is_err());
}

#[test]
#[ignore = "not yet implemented"]
fn pad_to_center_centroid_centered_input_unchanged_size() {
    // Centroid already at canvas center -> wd=ws, hd=hs.
    let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    // make a symmetric pattern (single pixel at center)
    for y in 0..8 {
        for x in 0..10 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    m.set_pixel(5, 4, 0).unwrap();
    let pix: Pix = m.into();
    let out = pix.pad_to_center_centroid(1).unwrap();
    // Centroid -> (5, 4), so wd = 2*max(5, 5) = 10, hd = 2*max(4, 4) = 8.
    assert_eq!(out.width(), 10);
    assert_eq!(out.height(), 8);
}

#[test]
#[ignore = "not yet implemented"]
fn pad_to_center_centroid_off_center_expands() {
    let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 0..8 {
        for x in 0..10 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    // Place a single dark pixel at (1,1) -> centroid near (1,1).
    m.set_pixel(1, 1, 0).unwrap();
    let pix: Pix = m.into();
    let out = pix.pad_to_center_centroid(1).unwrap();
    // wd = 2*max(1, 9) = 18, hd = 2*max(1, 7) = 14.
    assert_eq!(out.width(), 18);
    assert_eq!(out.height(), 14);
}

#[test]
#[ignore = "not yet implemented"]
fn pix_crop_aligned_returns_boxes() {
    let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for y in 0..8 {
        for x in 0..10 {
            m.set_pixel(x, y, 255).unwrap();
        }
    }
    m.set_pixel(3, 2, 0).unwrap();
    let pix1: Pix = m.into();
    let pix2 = pix1.deep_clone();
    let (b1, b2) = pix_crop_aligned_to_centroid(&pix1, &pix2, 1).unwrap();
    assert_eq!(b1.w, b2.w);
    assert_eq!(b1.h, b2.h);
}

#[test]
#[ignore = "not yet implemented"]
fn pix_crop_aligned_rejects_factor_zero() {
    let p1 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let p2 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    assert!(pix_crop_aligned_to_centroid(&p1, &p2, 0).is_err());
}

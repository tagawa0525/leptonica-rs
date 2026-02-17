//! Test Sel extensions and morphological application functions
//!
//! # See also
//!
//! C Leptonica: `sel1.c`, `morphapp.c`
//! - selFindMaxTranslations, selCreateFromPix
//! - pixExtractBoundary

use leptonica_core::{Pix, PixelDepth};
use leptonica_morph::binary::{BoundaryType, extract_boundary};
use leptonica_morph::sel::{Sel, SelElement};

/// Create a small binary image with a filled rectangle in the center
fn make_binary_rect(w: u32, h: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in y0..y1 {
        for x in x0..x1 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

// ============================================================================
// Sel::find_max_translations
// ============================================================================

#[test]
fn test_find_max_translations_brick() {
    // 5x3 brick with origin at center (2, 1)
    let sel = Sel::create_brick(5, 3).unwrap();
    let (xp, yp, xn, yn) = sel.find_max_translations();
    // Hits at columns 0..5, origin at cx=2
    // xp = max(cx - x) for x < cx = 2-0 = 2
    // xn = max(x - cx) for x > cx = 4-2 = 2
    // yp = max(cy - y) for y < cy = 1-0 = 1
    // yn = max(y - cy) for y > cy = 2-1 = 1
    assert_eq!((xp, yp, xn, yn), (2, 1, 2, 1));
}

#[test]
fn test_find_max_translations_asymmetric_origin() {
    // 5x1 horizontal with origin at (0, 0) — all hits to the right
    let mut sel = Sel::new(5, 1).unwrap();
    let _ = sel.set_origin(0, 0);
    for x in 0..5 {
        sel.set_element(x, 0, SelElement::Hit);
    }
    let (xp, yp, xn, yn) = sel.find_max_translations();
    assert_eq!(xp, 0); // no hits left of origin
    assert_eq!(yp, 0);
    assert_eq!(xn, 4); // furthest hit right of origin
    assert_eq!(yn, 0);
}

#[test]
fn test_find_max_translations_no_hits() {
    // Sel with only DontCare elements
    let sel = Sel::new(3, 3).unwrap();
    let (xp, yp, xn, yn) = sel.find_max_translations();
    assert_eq!((xp, yp, xn, yn), (0, 0, 0, 0));
}

// ============================================================================
// Sel::from_pix
// ============================================================================

#[test]
fn test_from_pix_basic() {
    // Create a 3x3 binary image with cross pattern
    let pix = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(1, 0, 1); // top center
    pm.set_pixel_unchecked(0, 1, 1); // middle left
    pm.set_pixel_unchecked(1, 1, 1); // center
    pm.set_pixel_unchecked(2, 1, 1); // middle right
    pm.set_pixel_unchecked(1, 2, 1); // bottom center
    let pix: Pix = pm.into();

    let sel = Sel::from_pix(&pix, 1, 1).unwrap();
    assert_eq!(sel.width(), 3);
    assert_eq!(sel.height(), 3);
    assert_eq!(sel.origin_x(), 1);
    assert_eq!(sel.origin_y(), 1);
    // Hit elements at cross positions
    assert_eq!(sel.get_element(1, 0), Some(SelElement::Hit));
    assert_eq!(sel.get_element(0, 1), Some(SelElement::Hit));
    assert_eq!(sel.get_element(1, 1), Some(SelElement::Hit));
    assert_eq!(sel.get_element(2, 1), Some(SelElement::Hit));
    assert_eq!(sel.get_element(1, 2), Some(SelElement::Hit));
    // Non-hit positions are DontCare
    assert_eq!(sel.get_element(0, 0), Some(SelElement::DontCare));
    assert_eq!(sel.get_element(2, 2), Some(SelElement::DontCare));
}

#[test]
fn test_from_pix_single_pixel() {
    let pix = Pix::new(1, 1, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 0, 1);
    let pix: Pix = pm.into();

    let sel = Sel::from_pix(&pix, 0, 0).unwrap();
    assert_eq!(sel.width(), 1);
    assert_eq!(sel.height(), 1);
    assert_eq!(sel.get_element(0, 0), Some(SelElement::Hit));
}

#[test]
fn test_from_pix_invalid_depth() {
    let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
    assert!(Sel::from_pix(&pix, 1, 1).is_err());
}

// ============================================================================
// extract_boundary
// ============================================================================

#[test]
fn test_extract_boundary_inner() {
    // 20x20 image with filled 10x10 rectangle at (5,5)-(15,15)
    let pix = make_binary_rect(20, 20, 5, 5, 15, 15);
    let boundary = extract_boundary(&pix, BoundaryType::Inner).unwrap();
    assert_eq!(boundary.depth(), PixelDepth::Bit1);
    // Interior pixel (10, 10) should NOT be on boundary
    assert_eq!(boundary.get_pixel_unchecked(10, 10), 0);
    // Edge pixel (5, 5) should be on boundary
    assert_eq!(boundary.get_pixel_unchecked(5, 5), 1);
    // Pixel outside rectangle should be 0
    assert_eq!(boundary.get_pixel_unchecked(0, 0), 0);
}

#[test]
fn test_extract_boundary_outer() {
    // 20x20 image with filled 10x10 rectangle at (5,5)-(15,15)
    let pix = make_binary_rect(20, 20, 5, 5, 15, 15);
    let boundary = extract_boundary(&pix, BoundaryType::Outer).unwrap();
    assert_eq!(boundary.depth(), PixelDepth::Bit1);
    // Pixel just outside rectangle should be on boundary
    assert_eq!(boundary.get_pixel_unchecked(4, 5), 1);
    // Interior pixel should NOT be on boundary
    assert_eq!(boundary.get_pixel_unchecked(10, 10), 0);
    // Far exterior should be 0
    assert_eq!(boundary.get_pixel_unchecked(0, 0), 0);
}

#[test]
fn test_extract_boundary_empty_image() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let boundary = extract_boundary(&pix, BoundaryType::Inner).unwrap();
    // No foreground → no boundary
    let total: u32 = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x, y)))
        .map(|(x, y)| boundary.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(total, 0);
}

#[test]
fn test_extract_boundary_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(extract_boundary(&pix, BoundaryType::Inner).is_err());
}

//! Regression tests for plan 111 (Pta + graphics 7 関数).

use leptonica::core::Box;
use leptonica::core::pta::{PatternSource, pix_generate_from_pta, pta_get_pixels_from_pix};
use leptonica::{Pix, PixelDepth, Pta};

fn make_1bpp_with_pixels(w: u32, h: u32, points: &[(u32, u32)]) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    for &(x, y) in points {
        m.set_pixel(x, y, 1).unwrap();
    }
    m.into()
}

// -- Pta::bounding_region -----------------------------------------------

#[test]
fn bounding_region_typical() {
    let mut pta = Pta::new();
    pta.push(1.0, 2.0);
    pta.push(5.0, 8.0);
    pta.push(3.0, 4.0);
    let b = pta.bounding_region().unwrap().unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (1, 2, 5, 7)); // (5-1+1=5, 8-2+1=7)
}

#[test]
fn bounding_region_empty_is_none() {
    let pta = Pta::new();
    assert!(pta.bounding_region().unwrap().is_none());
}

#[test]
fn bounding_region_single_point() {
    let mut pta = Pta::new();
    pta.push(3.0, 5.0);
    let b = pta.bounding_region().unwrap().unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (3, 5, 1, 1));
}

#[test]
fn bounding_region_symmetric_rounding_for_negative() {
    // Verify the doc-promised symmetric rounding for negative coords:
    // (-1.5, -2.5).round() = (-2, -3), so bounds must include (-2, -3).
    let mut pta = Pta::new();
    pta.push(-1.5, -2.5);
    pta.push(1.5, 2.5);
    let b = pta.bounding_region().unwrap().unwrap();
    assert_eq!(b.x, -2);
    assert_eq!(b.y, -3);
    assert_eq!(b.w, 2 - (-2) + 1);
    assert_eq!(b.h, 3 - (-3) + 1);
}

// -- Pta::to_numa_pair --------------------------------------------------

#[test]
fn to_numa_pair_round_trip() {
    let mut pta = Pta::new();
    pta.push(1.0, 2.5);
    pta.push(3.0, 4.5);
    let (nax, nay) = pta.to_numa_pair();
    assert_eq!(nax.len(), 2);
    assert_eq!(nay.len(), 2);
    assert!((nax.get(0).unwrap() - 1.0).abs() < 1e-6);
    assert!((nay.get(1).unwrap() - 4.5).abs() < 1e-6);
}

// -- pix_generate_from_pta ----------------------------------------------

#[test]
fn pix_generate_from_pta_renders_points() {
    let mut pta = Pta::new();
    pta.push(2.0, 3.0);
    pta.push(4.0, 1.0);
    pta.push(10.0, 10.0); // outside 5x5 canvas: dropped
    let pix = pix_generate_from_pta(&pta, 5, 5).unwrap();
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    assert_eq!(pix.count_pixels(), 2);
    assert_eq!(pix.get_pixel(2, 3), Some(1));
    assert_eq!(pix.get_pixel(4, 1), Some(1));
}

#[test]
fn pix_generate_from_pta_empty_pta() {
    let pta = Pta::new();
    let pix = pix_generate_from_pta(&pta, 4, 4).unwrap();
    assert_eq!(pix.count_pixels(), 0);
}

// -- pta_get_pixels_from_pix --------------------------------------------

#[test]
fn pta_get_pixels_from_pix_full_image() {
    let pix = make_1bpp_with_pixels(5, 5, &[(0, 0), (2, 1), (4, 4)]);
    let pta = pta_get_pixels_from_pix(&pix, None).unwrap();
    assert_eq!(pta.len(), 3);
    let pts: Vec<(i32, i32)> = (0..pta.len()).map(|i| pta.get_i_pt(i).unwrap()).collect();
    assert!(pts.contains(&(0, 0)));
    assert!(pts.contains(&(2, 1)));
    assert!(pts.contains(&(4, 4)));
}

#[test]
fn pta_get_pixels_from_pix_with_region() {
    let pix = make_1bpp_with_pixels(8, 8, &[(0, 0), (3, 3), (7, 7)]);
    let region = Box::new(2, 2, 4, 4).unwrap();
    let pta = pta_get_pixels_from_pix(&pix, Some(&region)).unwrap();
    // Only (3,3) lies within [2..=5]x[2..=5].
    assert_eq!(pta.len(), 1);
    assert_eq!(pta.get_i_pt(0), Some((3, 3)));
}

#[test]
fn pta_get_pixels_from_pix_rejects_non_1bpp() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    assert!(pta_get_pixels_from_pix(&pix, None).is_err());
}

// -- Pix::find_corner_pixels --------------------------------------------

#[test]
fn find_corner_pixels_rejects_non_1bpp() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    assert!(pix.find_corner_pixels().is_err());
}

#[test]
fn find_corner_pixels_all_zero() {
    let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
    let pta = pix.find_corner_pixels().unwrap();
    assert_eq!(pta.len(), 0);
}

#[test]
fn find_corner_pixels_all_four_corners_present() {
    // Foreground pixels at each corner.
    let pix = make_1bpp_with_pixels(8, 8, &[(0, 0), (7, 0), (0, 7), (7, 7)]);
    let pta = pix.find_corner_pixels().unwrap();
    assert_eq!(pta.len(), 4);
    let pts: Vec<(i32, i32)> = (0..pta.len()).map(|i| pta.get_i_pt(i).unwrap()).collect();
    assert!(pts.contains(&(0, 0)));
    assert!(pts.contains(&(7, 0)));
    assert!(pts.contains(&(0, 7)));
    assert!(pts.contains(&(7, 7)));
}

// -- Pta::replicate_pattern ---------------------------------------------

#[test]
fn replicate_pattern_simple() {
    // Anchor points (3, 3) and (5, 5); pattern = single point at (1, 1)
    // with centre (1, 1). Each anchor maps the pattern point to (x - cx + 1,
    // y - cy + 1) = (x, y).
    let mut ptas = Pta::new();
    ptas.push(3.0, 3.0);
    ptas.push(5.0, 5.0);
    let mut patt = Pta::new();
    patt.push(1.0, 1.0);
    let out = ptas
        .replicate_pattern(PatternSource::Pta(&patt), 1, 1, 10, 10)
        .unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out.get_i_pt(0), Some((3, 3)));
    assert_eq!(out.get_i_pt(1), Some((5, 5)));
}

#[test]
fn replicate_pattern_clips_off_canvas() {
    // Anchor at (8, 8); pattern point at (3, 3) with centre (0, 0).
    // Result lands at (11, 11) which is outside 10x10 canvas → dropped.
    let mut ptas = Pta::new();
    ptas.push(8.0, 8.0);
    let mut patt = Pta::new();
    patt.push(3.0, 3.0);
    let out = ptas
        .replicate_pattern(PatternSource::Pta(&patt), 0, 0, 10, 10)
        .unwrap();
    assert_eq!(out.len(), 0);
}

#[test]
fn replicate_pattern_from_pix() {
    // 3x3 pattern with two FG points; anchor at (5, 5) and centre at (1, 1).
    let patt_pix = make_1bpp_with_pixels(3, 3, &[(0, 0), (2, 2)]);
    let mut ptas = Pta::new();
    ptas.push(5.0, 5.0);
    let out = ptas
        .replicate_pattern(PatternSource::Pix(&patt_pix), 1, 1, 10, 10)
        .unwrap();
    // Pattern points (0,0) -> (4,4) and (2,2) -> (6,6).
    assert_eq!(out.len(), 2);
    let pts: Vec<(i32, i32)> = (0..out.len()).map(|i| out.get_i_pt(i).unwrap()).collect();
    assert!(pts.contains(&(4, 4)));
    assert!(pts.contains(&(6, 6)));
}

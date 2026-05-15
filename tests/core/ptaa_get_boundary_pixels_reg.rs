//! Regression tests for ptaa_get_boundary_pixels (plan 139).

use leptonica::core::pta::{BoundaryType, ptaa_get_boundary_pixels};
use leptonica::{Pix, PixMut, PixelDepth};

fn two_blocks() -> Pix {
    // 30x20 canvas with two 5x5 fg blocks: one at (2, 2), one at (20, 10).
    let mut pm = PixMut::new(30, 20, PixelDepth::Bit1).unwrap();
    for j in 2..7 {
        for i in 2..7 {
            pm.set_pixel(i, j, 1).unwrap();
        }
    }
    for j in 10..15 {
        for i in 20..25 {
            pm.set_pixel(i, j, 1).unwrap();
        }
    }
    pm.into()
}

#[test]
fn ptaa_boundary_fg_two_components_each_has_outer_ring() {
    let pix = two_blocks();
    let (ptaa, _, _) =
        ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 8, false, false).unwrap();
    assert_eq!(ptaa.len(), 2, "expected 2 components");
    // Each 5x5 block has 16 outer-ring pixels.
    assert_eq!(ptaa.get(0).unwrap().len(), 16);
    assert_eq!(ptaa.get(1).unwrap().len(), 16);
}

#[test]
fn ptaa_boundary_optional_outputs() {
    let pix = two_blocks();
    let (_ptaa, boxa, pixa) =
        ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 8, true, true).unwrap();
    let boxa = boxa.expect("want_boxa=true should populate boxa");
    let pixa = pixa.expect("want_pixa=true should populate pixa");
    assert_eq!(boxa.len(), 2);
    assert_eq!(pixa.pix_slice().len(), 2);
}

#[test]
fn ptaa_boundary_coordinates_are_in_parent_frame() {
    // Single block at (10, 5). The fg-boundary outer ring should sit
    // entirely within (10..=14, 5..=9) — i.e. parent-frame coordinates,
    // not local 0..5 coordinates.
    let mut pm = PixMut::new(30, 20, PixelDepth::Bit1).unwrap();
    for j in 5..10 {
        for i in 10..15 {
            pm.set_pixel(i, j, 1).unwrap();
        }
    }
    let pix: Pix = pm.into();
    let (ptaa, _, _) =
        ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 8, false, false).unwrap();
    let pta = ptaa.get(0).unwrap();
    let xs = pta.x_coords();
    let ys = pta.y_coords();
    for (x, y) in xs.iter().zip(ys.iter()) {
        assert!((10.0..=14.0).contains(x), "x {x} not in parent block frame");
        assert!((5.0..=9.0).contains(y), "y {y} not in parent block frame");
    }
}

#[test]
fn ptaa_boundary_rejects_non_1bpp() {
    let pix: Pix = PixMut::new(20, 20, PixelDepth::Bit8).unwrap().into();
    assert!(ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 8, false, false).is_err());
}

#[test]
fn ptaa_boundary_rejects_invalid_connectivity() {
    let pix = two_blocks();
    assert!(ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 6, false, false).is_err());
    assert!(ptaa_get_boundary_pixels(&pix, BoundaryType::Foreground, 0, false, false).is_err());
}

#[test]
fn ptaa_boundary_bg_includes_outside_ring() {
    // Single 5x5 block at (10, 5). bg-boundary ring is the 7x7 frame
    // minus the 5x5 interior = 24 pixels.
    let mut pm = PixMut::new(30, 20, PixelDepth::Bit1).unwrap();
    for j in 5..10 {
        for i in 10..15 {
            pm.set_pixel(i, j, 1).unwrap();
        }
    }
    let pix: Pix = pm.into();
    let (ptaa, _, _) =
        ptaa_get_boundary_pixels(&pix, BoundaryType::Background, 8, false, false).unwrap();
    assert_eq!(ptaa.get(0).unwrap().len(), 24);
}

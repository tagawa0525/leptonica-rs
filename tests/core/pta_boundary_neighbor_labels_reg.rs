//! Regression tests for plan 137 (Pta boundary / neighbor / labeled-pixels).

use leptonica::core::pta::{
    BoundaryType, pta_get_boundary_pixels, pta_get_neighbor_pix_locs, ptaa_index_labeled_pixels,
};
use leptonica::{Pix, PixMut, PixelDepth};

fn make_block(x: u32, y: u32, w: u32, h: u32, canvas: u32) -> Pix {
    let mut pm = PixMut::new(canvas, canvas, PixelDepth::Bit1).unwrap();
    for j in y..y + h {
        for i in x..x + w {
            pm.set_pixel(i, j, 1).unwrap();
        }
    }
    pm.into()
}

// -- pta_get_boundary_pixels ----------------------------------------------

#[test]
fn boundary_fg_extracts_outer_ring() {
    // 5x5 solid block at (5, 5) in a 20x20 canvas. The fg-boundary is the
    // outer ring (16 pixels, perimeter of a 5x5 square = 4 * 4 = 16).
    let pix = make_block(5, 5, 5, 5, 20);
    let pta = pta_get_boundary_pixels(&pix, BoundaryType::Foreground).unwrap();
    assert_eq!(
        pta.len(),
        16,
        "expected 16 boundary pixels, got {}",
        pta.len()
    );
}

#[test]
fn boundary_bg_returns_some_pixels() {
    let pix = make_block(5, 5, 5, 5, 20);
    let pta = pta_get_boundary_pixels(&pix, BoundaryType::Background).unwrap();
    // Background boundary is the 1-pixel ring just outside the block.
    // For a 5x5 block, that's a 7x7 frame minus the 5x5 interior, but the
    // dilation produces a 7x7 block, XOR with 5x5 block yields the 24
    // bg-boundary pixels.
    assert_eq!(pta.len(), 24);
}

#[test]
fn boundary_rejects_non_1bpp() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit8).unwrap().into();
    assert!(pta_get_boundary_pixels(&pix, BoundaryType::Foreground).is_err());
}

// -- pta_get_neighbor_pix_locs -------------------------------------------

#[test]
fn neighbors_interior_pixel_4conn() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    let pta = pta_get_neighbor_pix_locs(&pix, 5, 5, 4).unwrap();
    assert_eq!(pta.len(), 4);
}

#[test]
fn neighbors_interior_pixel_8conn() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    let pta = pta_get_neighbor_pix_locs(&pix, 5, 5, 8).unwrap();
    assert_eq!(pta.len(), 8);
}

#[test]
fn neighbors_corner_4conn() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    let pta = pta_get_neighbor_pix_locs(&pix, 0, 0, 4).unwrap();
    // Only (1, 0) and (0, 1) are valid neighbors of (0, 0) in 4-conn.
    assert_eq!(pta.len(), 2);
}

#[test]
fn neighbors_corner_8conn() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    let pta = pta_get_neighbor_pix_locs(&pix, 0, 0, 8).unwrap();
    // 8-conn neighbors of (0, 0): (1, 0), (0, 1), (1, 1).
    assert_eq!(pta.len(), 3);
}

#[test]
fn neighbors_rejects_invalid_pos() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    assert!(pta_get_neighbor_pix_locs(&pix, -1, 5, 4).is_err());
    assert!(pta_get_neighbor_pix_locs(&pix, 10, 5, 4).is_err());
}

#[test]
fn neighbors_rejects_invalid_conn() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit1).unwrap().into();
    assert!(pta_get_neighbor_pix_locs(&pix, 5, 5, 6).is_err());
}

// -- ptaa_index_labeled_pixels -------------------------------------------

#[test]
fn ptaa_index_labels_buckets_correctly() {
    // 32 bpp image with labels: top-left 2x2 has label 1, bottom-right 3x3
    // has label 2, rest is 0.
    let mut pm = PixMut::new(10, 10, PixelDepth::Bit32).unwrap();
    for y in 0..2 {
        for x in 0..2 {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    for y in 5..8 {
        for x in 5..8 {
            pm.set_pixel(x, y, 2).unwrap();
        }
    }
    let pix: Pix = pm.into();
    let (ptaa, ncc) = ptaa_index_labeled_pixels(&pix).unwrap();
    assert_eq!(ncc, 2, "expected max label 2");
    // Ptaa has maxval+1 = 3 entries (index 0..=2). Label 0 is empty,
    // label 1 has 4 pixels, label 2 has 9 pixels.
    assert_eq!(ptaa.len(), 3);
    assert_eq!(ptaa.get(0).unwrap().len(), 0);
    assert_eq!(ptaa.get(1).unwrap().len(), 4);
    assert_eq!(ptaa.get(2).unwrap().len(), 9);
}

#[test]
fn ptaa_index_labels_rejects_non_32bpp() {
    let pix: Pix = PixMut::new(10, 10, PixelDepth::Bit8).unwrap().into();
    assert!(ptaa_index_labeled_pixels(&pix).is_err());
}

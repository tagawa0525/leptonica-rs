//! Test extended connected component functions
//!
//! # See also
//!
//! C Leptonica: `conncomp.c`, `pixlabel.c`
//! - pixConnCompPixa, pixGetSortedNeighborValues

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::ConnectivityType;
use leptonica_region::conncomp::{conncomp_pixa, get_sorted_neighbor_values};

/// Create a binary image with specific pixels set to foreground
fn make_binary_image(w: u32, h: u32, pixels: &[(u32, u32)]) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for &(x, y) in pixels {
        pm.set_pixel_unchecked(x, y, 1);
    }
    pm.into()
}

/// Create a binary rect: foreground pixels in [x0,x1) × [y0,y1)
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
// conncomp_pixa
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_conncomp_pixa_two_components() {
    // Two separate rectangles
    let pix = make_binary_image(
        20,
        10,
        &[
            // Left rect: (1,1)-(3,3)
            (1, 1),
            (2, 1),
            (3, 1),
            (1, 2),
            (2, 2),
            (3, 2),
            (1, 3),
            (2, 3),
            (3, 3),
            // Right rect: (10,1)-(11,2)
            (10, 1),
            (11, 1),
            (10, 2),
            (11, 2),
        ],
    );

    let (boxa, pixa) = conncomp_pixa(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(pixa.len(), 2);
    assert_eq!(boxa.len(), 2);

    // Components should be sorted by label (top-left first in raster order)
    // First component: 3x3 rect
    let b0 = boxa.get(0).unwrap();
    assert_eq!(b0.w, 3);
    assert_eq!(b0.h, 3);
    let p0 = pixa.get(0).unwrap();
    assert_eq!(p0.width(), 3);
    assert_eq!(p0.height(), 3);
    assert_eq!(p0.depth(), PixelDepth::Bit1);

    // Second component: 2x2 rect
    let b1 = boxa.get(1).unwrap();
    assert_eq!(b1.w, 2);
    assert_eq!(b1.h, 2);
    let p1 = pixa.get(1).unwrap();
    assert_eq!(p1.width(), 2);
    assert_eq!(p1.height(), 2);
}

#[test]
#[ignore = "not yet implemented"]
fn test_conncomp_pixa_single_component() {
    let pix = make_binary_rect(10, 10, 2, 3, 7, 8);
    let (boxa, pixa) = conncomp_pixa(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(pixa.len(), 1);
    assert_eq!(boxa.len(), 1);

    let b = boxa.get(0).unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (2, 3, 5, 5));

    let p = pixa.get(0).unwrap();
    assert_eq!((p.width(), p.height()), (5, 5));
    // All pixels in clipped image should be foreground
    for y in 0..5 {
        for x in 0..5 {
            assert_eq!(p.get_pixel_unchecked(x, y), 1);
        }
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_conncomp_pixa_empty_image() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let (boxa, pixa) = conncomp_pixa(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(pixa.len(), 0);
    assert_eq!(boxa.len(), 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_conncomp_pixa_diagonal_8way() {
    // Two diagonal pixels: 4-way = 2 components, 8-way = 1 component
    let pix = make_binary_image(5, 5, &[(1, 1), (2, 2)]);

    let (_, pixa_4) = conncomp_pixa(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(pixa_4.len(), 2);

    let (_, pixa_8) = conncomp_pixa(&pix, ConnectivityType::EightWay).unwrap();
    assert_eq!(pixa_8.len(), 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_conncomp_pixa_invalid_depth() {
    let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
    assert!(conncomp_pixa(&pix, ConnectivityType::FourWay).is_err());
}

// ============================================================================
// get_sorted_neighbor_values
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_center() {
    // Create a 5x5 labeled image (32bpp) with different labels
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    // Center pixel: label 1
    pm.set_pixel_unchecked(2, 2, 1);
    // Neighbors: different labels
    pm.set_pixel_unchecked(1, 2, 2); // left
    pm.set_pixel_unchecked(3, 2, 3); // right
    pm.set_pixel_unchecked(2, 1, 2); // top (same as left)
    pm.set_pixel_unchecked(2, 3, 4); // bottom
    let pix: Pix = pm.into();

    let vals = get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::FourWay).unwrap();
    // Should return unique sorted non-zero values: [2, 3, 4]
    assert_eq!(vals, vec![2, 3, 4]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_8way() {
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(2, 2, 1); // center
    pm.set_pixel_unchecked(1, 1, 5); // top-left diagonal
    pm.set_pixel_unchecked(3, 3, 7); // bottom-right diagonal
    let pix: Pix = pm.into();

    // 4-way: no neighbors with non-zero values
    let vals_4 = get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::FourWay).unwrap();
    assert!(vals_4.is_empty());

    // 8-way: should find diagonal neighbors
    let vals_8 = get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::EightWay).unwrap();
    assert_eq!(vals_8, vec![5, 7]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_corner() {
    // Test at corner (0,0) - only 2 neighbors in 4-way
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 0, 1);
    pm.set_pixel_unchecked(1, 0, 3); // right
    pm.set_pixel_unchecked(0, 1, 3); // below (same value)
    let pix: Pix = pm.into();

    let vals = get_sorted_neighbor_values(&pix, 0, 0, ConnectivityType::FourWay).unwrap();
    // Only one unique value: [3]
    assert_eq!(vals, vec![3]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_no_neighbors() {
    // Single labeled pixel, all neighbors are 0
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(2, 2, 1);
    let pix: Pix = pm.into();

    let vals = get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::FourWay).unwrap();
    assert!(vals.is_empty());
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_excludes_zero() {
    // Some neighbors are 0 (background), should be excluded
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(2, 2, 1);
    pm.set_pixel_unchecked(1, 2, 0); // background
    pm.set_pixel_unchecked(3, 2, 5); // labeled
    let pix: Pix = pm.into();

    let vals = get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::FourWay).unwrap();
    assert_eq!(vals, vec![5]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_8bpp() {
    // Should also work with 8bpp images
    let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(1, 1, 10);
    pm.set_pixel_unchecked(0, 1, 20);
    pm.set_pixel_unchecked(2, 1, 30);
    let pix: Pix = pm.into();

    let vals = get_sorted_neighbor_values(&pix, 1, 1, ConnectivityType::FourWay).unwrap();
    assert_eq!(vals, vec![20, 30]);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_sorted_neighbor_values_invalid_depth() {
    let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
    assert!(get_sorted_neighbor_values(&pix, 2, 2, ConnectivityType::FourWay).is_err());
}

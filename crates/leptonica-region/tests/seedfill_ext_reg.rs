//! Test extended seedfill functions
//!
//! # See also
//!
//! C Leptonica: `seedfill.c`
//! - pixDistanceFunction, pixFindEqualValues, pixFillClosedBorders
//! - pixRemoveSeededComponents, pixSeedfillGrayInv, pixSeedfillBinaryRestricted

use leptonica_core::{Pix, PixelDepth};
use leptonica_region::ConnectivityType;
use leptonica_region::seedfill::{
    BoundaryCondition, distance_function, fill_closed_borders, find_equal_values,
    remove_seeded_components, seedfill_binary_restricted, seedfill_gray_inv,
};

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

/// Create a uniform 8bpp image
fn make_uniform_8bpp(w: u32, h: u32, val: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

// ============================================================================
// distance_function
// ============================================================================

#[test]
fn test_distance_function_single_pixel() {
    // 5x5 image with single foreground pixel at center
    let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    pm.set_pixel_unchecked(2, 2, 1);
    let pix: Pix = pm.into();

    let dist = distance_function(
        &pix,
        ConnectivityType::FourWay,
        PixelDepth::Bit8,
        BoundaryCondition::Background,
    )
    .unwrap();
    assert_eq!(dist.depth(), PixelDepth::Bit8);
    // Center pixel is the only foreground pixel, distance = 1
    assert_eq!(dist.get_pixel_unchecked(2, 2), 1);
    // Background pixels should be 0
    assert_eq!(dist.get_pixel_unchecked(0, 0), 0);
}

#[test]
fn test_distance_function_filled_rect() {
    // 11x11 image completely filled with foreground
    let pix = make_binary_rect(11, 11, 0, 0, 11, 11);
    let dist = distance_function(
        &pix,
        ConnectivityType::FourWay,
        PixelDepth::Bit8,
        BoundaryCondition::Background,
    )
    .unwrap();
    // Center (5,5) should have distance ~6 (distance to nearest edge)
    let center_val = dist.get_pixel_unchecked(5, 5);
    assert!(
        center_val >= 5 && center_val <= 6,
        "center_val = {center_val}"
    );
    // Edge pixel should have distance 1
    assert_eq!(dist.get_pixel_unchecked(0, 5), 1);
}

#[test]
fn test_distance_function_16bit() {
    let pix = make_binary_rect(5, 5, 0, 0, 5, 5);
    let dist = distance_function(
        &pix,
        ConnectivityType::FourWay,
        PixelDepth::Bit16,
        BoundaryCondition::Background,
    )
    .unwrap();
    assert_eq!(dist.depth(), PixelDepth::Bit16);
}

#[test]
fn test_distance_function_invalid_depth() {
    let pix = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
    assert!(
        distance_function(
            &pix,
            ConnectivityType::FourWay,
            PixelDepth::Bit8,
            BoundaryCondition::Background,
        )
        .is_err()
    );
}

// ============================================================================
// find_equal_values
// ============================================================================

#[test]
fn test_find_equal_values_all_equal() {
    let pix1 = make_uniform_8bpp(10, 10, 128);
    let pix2 = make_uniform_8bpp(10, 10, 128);
    let mask = find_equal_values(&pix1, &pix2).unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    // All pixels equal → all ON
    let on_count: u32 = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 100);
}

#[test]
fn test_find_equal_values_none_equal() {
    let pix1 = make_uniform_8bpp(10, 10, 100);
    let pix2 = make_uniform_8bpp(10, 10, 200);
    let mask = find_equal_values(&pix1, &pix2).unwrap();
    let on_count: u32 = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

#[test]
fn test_find_equal_values_gradient() {
    // Create two gradient images that cross at x=4 (value 120)
    let pix1 = Pix::new(9, 1, PixelDepth::Bit8).unwrap();
    let mut pm1 = pix1.try_into_mut().unwrap();
    let pix2 = Pix::new(9, 1, PixelDepth::Bit8).unwrap();
    let mut pm2 = pix2.try_into_mut().unwrap();
    for x in 0..9u32 {
        pm1.set_pixel_unchecked(x, 0, x * 30); // 0,30,60,90,120,150,180,210,240
        pm2.set_pixel_unchecked(x, 0, (8 - x) * 30); // 240,210,180,150,120,90,60,30,0
    }
    let pix1: Pix = pm1.into();
    let pix2: Pix = pm2.into();
    let mask = find_equal_values(&pix1, &pix2).unwrap();
    // Exactly one crossing point at x=4 (both=120)
    let on_count: u32 = (0..9).map(|x| mask.get_pixel_unchecked(x, 0)).sum();
    assert_eq!(on_count, 1);
    assert_eq!(mask.get_pixel_unchecked(4, 0), 1);
}

// ============================================================================
// fill_closed_borders
// ============================================================================

#[test]
fn test_fill_closed_borders_box() {
    // Create a hollow rectangle (border only)
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for x in 2..8 {
        pm.set_pixel_unchecked(x, 2, 1);
        pm.set_pixel_unchecked(x, 7, 1);
    }
    for y in 2..8 {
        pm.set_pixel_unchecked(2, y, 1);
        pm.set_pixel_unchecked(7, y, 1);
    }
    let pix: Pix = pm.into();

    let filled = fill_closed_borders(&pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(filled.depth(), PixelDepth::Bit1);
    // Interior should now be filled
    assert_eq!(filled.get_pixel_unchecked(4, 4), 1);
    // Exterior should remain 0
    assert_eq!(filled.get_pixel_unchecked(0, 0), 0);
}

#[test]
fn test_fill_closed_borders_open() {
    // Create a U-shape (open at top) - should NOT fill interior
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 2..8 {
        pm.set_pixel_unchecked(2, y, 1);
        pm.set_pixel_unchecked(7, y, 1);
    }
    for x in 2..8 {
        pm.set_pixel_unchecked(x, 7, 1); // bottom only
    }
    let pix: Pix = pm.into();

    let filled = fill_closed_borders(&pix, ConnectivityType::FourWay).unwrap();
    // Interior connected to exterior → should NOT be filled
    assert_eq!(filled.get_pixel_unchecked(4, 4), 0);
}

// ============================================================================
// remove_seeded_components
// ============================================================================

#[test]
fn test_remove_seeded_components_basic() {
    // Mask: two separate rectangles
    let mask = Pix::new(20, 10, PixelDepth::Bit1).unwrap();
    let mut pm = mask.try_into_mut().unwrap();
    // Left rect: (1,1)-(8,8)
    for y in 1..8 {
        for x in 1..8 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    // Right rect: (12,1)-(19,8)
    for y in 1..8 {
        for x in 12..19 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    let mask: Pix = pm.into();

    // Seed touches only the left rectangle
    let seed = Pix::new(20, 10, PixelDepth::Bit1).unwrap();
    let mut pm = seed.try_into_mut().unwrap();
    pm.set_pixel_unchecked(3, 3, 1);
    let seed: Pix = pm.into();

    let result = remove_seeded_components(&seed, &mask, ConnectivityType::FourWay).unwrap();
    // Left rectangle should be removed
    assert_eq!(result.get_pixel_unchecked(3, 3), 0);
    // Right rectangle should remain
    assert_eq!(result.get_pixel_unchecked(15, 4), 1);
}

#[test]
fn test_remove_seeded_components_no_seeds() {
    let mask = make_binary_rect(10, 10, 2, 2, 8, 8);
    let seed = Pix::new(10, 10, PixelDepth::Bit1).unwrap(); // empty
    let result = remove_seeded_components(&seed, &mask, ConnectivityType::FourWay).unwrap();
    // Nothing should be removed
    assert_eq!(result.get_pixel_unchecked(5, 5), 1);
}

// ============================================================================
// seedfill_gray_inv
// ============================================================================

#[test]
fn test_seedfill_gray_inv_basic() {
    // Seed: all 200, Mask: has valley at center (100)
    let seed = make_uniform_8bpp(5, 5, 200);
    let mask = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
    let mut pm = mask.try_into_mut().unwrap();
    for y in 0..5u32 {
        for x in 0..5u32 {
            // Valley at center
            let val = if x == 2 && y == 2 { 100 } else { 200 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let mask: Pix = pm.into();

    let result = seedfill_gray_inv(&seed, &mask, ConnectivityType::FourWay).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit8);
    // Seed should not go below mask, so center stays at mask level
    let center = result.get_pixel_unchecked(2, 2);
    assert!(center >= 100, "center = {center}");
    // Other positions should remain at seed level (clamped by mask)
    assert_eq!(result.get_pixel_unchecked(0, 0), 200);
}

#[test]
fn test_seedfill_gray_inv_equal() {
    // Seed == Mask → output should equal both
    let pix = make_uniform_8bpp(5, 5, 128);
    let result = seedfill_gray_inv(&pix, &pix, ConnectivityType::FourWay).unwrap();
    assert_eq!(result.get_pixel_unchecked(2, 2), 128);
}

// ============================================================================
// seedfill_binary_restricted
// ============================================================================

#[test]
fn test_seedfill_binary_restricted_basic() {
    // Mask: long horizontal stripe
    let mask = make_binary_rect(30, 5, 0, 2, 30, 3);
    // Seed: single pixel at left end
    let seed = Pix::new(30, 5, PixelDepth::Bit1).unwrap();
    let mut pm = seed.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 2, 1);
    let seed: Pix = pm.into();

    // Restrict to xmax=10
    let result =
        seedfill_binary_restricted(&seed, &mask, ConnectivityType::FourWay, 10, 0).unwrap();
    // Near seed should be filled
    assert_eq!(result.get_pixel_unchecked(5, 2), 1);
    // Far from seed should NOT be filled
    assert_eq!(result.get_pixel_unchecked(25, 2), 0);
}

#[test]
fn test_seedfill_binary_restricted_no_limit() {
    // xmax=0, ymax=0 → no restriction, same as full seedfill
    let mask = make_binary_rect(10, 10, 0, 0, 10, 10);
    let seed = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut pm = seed.try_into_mut().unwrap();
    pm.set_pixel_unchecked(0, 0, 1);
    let seed: Pix = pm.into();

    let result = seedfill_binary_restricted(&seed, &mask, ConnectivityType::FourWay, 0, 0).unwrap();
    // Everything should be filled
    assert_eq!(result.get_pixel_unchecked(9, 9), 1);
}

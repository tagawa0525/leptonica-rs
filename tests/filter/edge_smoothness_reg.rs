//! Edge smoothness and two-sided edge regression tests
//!
//! C version: reference/leptonica/prog/edge_reg.c (extended)
//!
//! Tests two-sided edge filter, edge smoothness measurement,
//! and edge profile extraction.
//!
//! C API mapping:
//! - pixTwoSidedEdgeFilter -> two_sided_edge_filter
//! - pixMeasureEdgeSmoothness -> measure_edge_smoothness
//! - pixGetEdgeProfile -> get_edge_profile

use crate::common::{RegParams, load_test_image};
use leptonica::filter::edge::EdgeSide;
use leptonica::filter::{
    EdgeOrientation, get_edge_profile, measure_edge_smoothness, two_sided_edge_filter,
};
use leptonica::{Pix, PixelDepth};

/// Test two-sided edge filter on 8bpp grayscale image.
#[test]
#[ignore = "not yet implemented"]
fn edge_smoothness_reg_two_sided() {
    let mut rp = RegParams::new("edge_smoothness");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();

    // Test vertical edges
    let pixv = two_sided_edge_filter(&pixs, EdgeOrientation::Vertical).expect("vertical edges");
    rp.compare_values(w as f64, pixv.width() as f64, 0.0);
    rp.compare_values(h as f64, pixv.height() as f64, 0.0);
    assert_eq!(pixv.depth(), PixelDepth::Bit8);

    // Test horizontal edges
    let pixh = two_sided_edge_filter(&pixs, EdgeOrientation::Horizontal).expect("horizontal edges");
    rp.compare_values(w as f64, pixh.width() as f64, 0.0);
    rp.compare_values(h as f64, pixh.height() as f64, 0.0);

    // Edge pixels should be non-zero somewhere
    let fg_v = pixv.count_pixels();
    let fg_h = pixh.count_pixels();
    rp.compare_values(1.0, if fg_v > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if fg_h > 0 { 1.0 } else { 0.0 }, 0.0);

    // Error: reject non-8bpp
    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    assert!(two_sided_edge_filter(&pix32, EdgeOrientation::Vertical).is_err());

    // Error: reject EdgeOrientation::All
    assert!(two_sided_edge_filter(&pixs, EdgeOrientation::All).is_err());

    assert!(rp.cleanup(), "edge_smoothness_reg_two_sided failed");
}

/// Test edge profile extraction from binary image.
#[test]
#[ignore = "not yet implemented"]
fn edge_smoothness_reg_profile() {
    let mut rp = RegParams::new("edge_profile");

    // Create a simple binary image with a known shape
    let pix = Pix::new(40, 30, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Draw a filled rectangle: columns 10..30, rows 5..25
    for y in 5..25 {
        for x in 10..30 {
            pix_mut.set_pixel_unchecked(x, y, 1);
        }
    }
    let pix = Pix::from(pix_mut);

    // Profile from left should find the left edge of the rectangle
    let na_left = get_edge_profile(&pix, EdgeSide::FromLeft).expect("from_left");
    rp.compare_values(30.0, na_left.len() as f64, 0.0); // h entries

    // Profile from right should find the right edge
    let na_right = get_edge_profile(&pix, EdgeSide::FromRight).expect("from_right");
    rp.compare_values(30.0, na_right.len() as f64, 0.0);

    // Profile from top
    let na_top = get_edge_profile(&pix, EdgeSide::FromTop).expect("from_top");
    rp.compare_values(40.0, na_top.len() as f64, 0.0); // w entries

    // Profile from bottom
    let na_bot = get_edge_profile(&pix, EdgeSide::FromBottom).expect("from_bottom");
    rp.compare_values(40.0, na_bot.len() as f64, 0.0);

    // Error: reject non-1bpp
    let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(get_edge_profile(&pix8, EdgeSide::FromLeft).is_err());

    assert!(rp.cleanup(), "edge_smoothness_reg_profile failed");
}

/// Test edge smoothness measurement.
#[test]
#[ignore = "not yet implemented"]
fn edge_smoothness_reg_measure() {
    let mut rp = RegParams::new("edge_measure");

    // Create a binary image with a jagged left edge
    let pix = Pix::new(50, 40, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0u32..40 {
        // Jagged edge: alternating start column
        let start_x = if y % 2 == 0 { 10 } else { 15 };
        for x in start_x..40 {
            pix_mut.set_pixel_unchecked(x, y, 1);
        }
    }
    let pix = Pix::from(pix_mut);

    let (jpl, jspl, _rpl) =
        measure_edge_smoothness(&pix, EdgeSide::FromLeft, 1, 1).expect("measure from_left");

    // Should detect some jumps (jagged edge)
    rp.compare_values(1.0, if jpl > 0.0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if jspl > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // Smooth edge should have no jumps with large minjump
    let pix2 = Pix::new(50, 40, PixelDepth::Bit1).unwrap();
    let mut pix2_mut = pix2.try_into_mut().unwrap();
    for y in 0u32..40 {
        for x in 10u32..40 {
            pix2_mut.set_pixel_unchecked(x, y, 1);
        }
    }
    let pix2 = Pix::from(pix2_mut);

    let (jpl2, jspl2, _rpl2) =
        measure_edge_smoothness(&pix2, EdgeSide::FromLeft, 1, 1).expect("measure smooth");
    rp.compare_values(0.0, jpl2 as f64, 0.0);
    rp.compare_values(0.0, jspl2 as f64, 0.0);

    // Error: reject non-1bpp
    let pix8 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(measure_edge_smoothness(&pix8, EdgeSide::FromLeft, 1, 1).is_err());

    // Error: minjump < 1
    assert!(measure_edge_smoothness(&pix, EdgeSide::FromLeft, 0, 1).is_err());

    assert!(rp.cleanup(), "edge_smoothness_reg_measure failed");
}

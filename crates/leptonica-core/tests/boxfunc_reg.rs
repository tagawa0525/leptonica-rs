//! Test Box/Boxa extension functions
//!
//! # See also
//!
//! C Leptonica: `boxfunc1.c`, `boxfunc4.c`

use leptonica_core::{Box, Boxa, SizeRelation};

// ============================================================================
// Box::overlap_area / overlap_fraction
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_overlap_area_partial() {
    let b1 = Box::new(0, 0, 100, 100).unwrap();
    let b2 = Box::new(50, 50, 100, 100).unwrap();
    assert_eq!(b1.overlap_area(&b2), 2500); // 50x50 overlap
}

#[test]
#[ignore = "not yet implemented"]
fn test_overlap_area_none() {
    let b1 = Box::new(0, 0, 10, 10).unwrap();
    let b2 = Box::new(20, 20, 10, 10).unwrap();
    assert_eq!(b1.overlap_area(&b2), 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_overlap_fraction_half() {
    let b1 = Box::new(0, 0, 100, 100).unwrap();
    let b2 = Box::new(50, 0, 100, 100).unwrap();
    let frac = b1.overlap_fraction(&b2);
    assert!((frac - 0.5).abs() < 1e-6, "frac = {frac}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_overlap_fraction_zero_area() {
    let b1 = Box::new(0, 0, 0, 0).unwrap();
    let b2 = Box::new(0, 0, 10, 10).unwrap();
    assert_eq!(b1.overlap_fraction(&b2), 0.0);
}

// ============================================================================
// Boxa::contained_in_box
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_contained_in_box() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(10, 10, 20, 20).unwrap()); // inside
    boxa.push(Box::new(0, 0, 200, 200).unwrap()); // too big
    boxa.push(Box::new(50, 50, 10, 10).unwrap()); // inside

    let container = Box::new(0, 0, 100, 100).unwrap();
    let result = boxa.contained_in_box(&container);
    assert_eq!(result.len(), 2);
}

// ============================================================================
// Boxa::intersects_box
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_intersects_box() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 20, 20).unwrap()); // intersects
    boxa.push(Box::new(200, 200, 10, 10).unwrap()); // no overlap
    boxa.push(Box::new(90, 90, 20, 20).unwrap()); // intersects

    let target = Box::new(10, 10, 90, 90).unwrap();
    let result = boxa.intersects_box(&target);
    assert_eq!(result.len(), 2);
}

// ============================================================================
// Boxa::clip_to_box
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_clip_to_box() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 200, 200).unwrap()); // clipped to 100x100
    boxa.push(Box::new(50, 50, 30, 30).unwrap()); // fully inside, unchanged
    boxa.push(Box::new(200, 200, 10, 10).unwrap()); // outside, removed

    let clip = Box::new(0, 0, 100, 100).unwrap();
    let result = boxa.clip_to_box(&clip);
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap().w, 100);
    assert_eq!(result.get(0).unwrap().h, 100);
    assert_eq!(*result.get(1).unwrap(), Box::new(50, 50, 30, 30).unwrap());
}

// ============================================================================
// Boxa::combine_overlaps
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_combine_overlaps() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 60, 60).unwrap());
    boxa.push(Box::new(50, 50, 60, 60).unwrap()); // overlaps first
    boxa.push(Box::new(200, 200, 10, 10).unwrap()); // separate

    let result = boxa.combine_overlaps();
    assert_eq!(result.len(), 2);
    // The merged box should be the union of first two
    let merged = result.get(0).unwrap();
    assert_eq!(merged.x, 0);
    assert_eq!(merged.y, 0);
    assert_eq!(merged.w, 110);
    assert_eq!(merged.h, 110);
}

#[test]
#[ignore = "not yet implemented"]
fn test_combine_overlaps_chain() {
    // Three boxes in a chain: A overlaps B, B overlaps C, but A doesn't overlap C
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 30, 30).unwrap());
    boxa.push(Box::new(20, 20, 30, 30).unwrap()); // overlaps first
    boxa.push(Box::new(40, 40, 30, 30).unwrap()); // overlaps second

    let result = boxa.combine_overlaps();
    // All three should merge into one
    assert_eq!(result.len(), 1);
    assert_eq!(result.get(0).unwrap().x, 0);
    assert_eq!(result.get(0).unwrap().w, 70);
}

// ============================================================================
// Boxa::select_by_size / select_by_area / select_by_wh_ratio
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_size() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 10, 10).unwrap());
    boxa.push(Box::new(0, 0, 50, 50).unwrap());
    boxa.push(Box::new(0, 0, 100, 100).unwrap());

    let result = boxa.select_by_size(30, 30, SizeRelation::GreaterThan);
    assert_eq!(result.len(), 2); // 50x50 and 100x100
}

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_area() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 10, 10).unwrap()); // area 100
    boxa.push(Box::new(0, 0, 20, 20).unwrap()); // area 400
    boxa.push(Box::new(0, 0, 5, 5).unwrap()); // area 25

    let result = boxa.select_by_area(100, SizeRelation::GreaterThanOrEqual);
    assert_eq!(result.len(), 2); // 100 and 400
}

#[test]
#[ignore = "not yet implemented"]
fn test_select_by_wh_ratio() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 100, 50).unwrap()); // ratio 2.0
    boxa.push(Box::new(0, 0, 50, 50).unwrap()); // ratio 1.0
    boxa.push(Box::new(0, 0, 50, 100).unwrap()); // ratio 0.5

    let result = boxa.select_by_wh_ratio(1.5, SizeRelation::GreaterThan);
    assert_eq!(result.len(), 1); // only 2.0
}

// ============================================================================
// Boxa::get_extent / get_coverage / size_range
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_get_extent() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(10, 20, 30, 40).unwrap());
    boxa.push(Box::new(50, 60, 20, 10).unwrap());

    let (w, h, bb) = boxa.get_extent().unwrap();
    assert_eq!(w, 70); // max right edge
    assert_eq!(h, 70); // max bottom edge
    assert_eq!(bb.x, 10);
    assert_eq!(bb.y, 20);
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_extent_empty() {
    let boxa = Boxa::new();
    assert!(boxa.get_extent().is_none());
}

#[test]
#[ignore = "not yet implemented"]
fn test_get_coverage_no_overlap() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 10, 10).unwrap()); // 100 pixels
    boxa.push(Box::new(10, 0, 10, 10).unwrap()); // 100 pixels, no overlap

    let coverage = boxa.get_coverage(100, 100, true);
    assert!((coverage - 0.02).abs() < 1e-6, "coverage = {coverage}"); // 200/10000
}

#[test]
#[ignore = "not yet implemented"]
fn test_size_range() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 10, 20).unwrap());
    boxa.push(Box::new(0, 0, 50, 5).unwrap());
    boxa.push(Box::new(0, 0, 30, 30).unwrap());

    let (min_w, min_h, max_w, max_h) = boxa.size_range().unwrap();
    assert_eq!((min_w, min_h, max_w, max_h), (10, 5, 50, 30));
}

// ============================================================================
// Boxa::similar / join
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_similar_exact() {
    let mut b1 = Boxa::new();
    b1.push(Box::new(0, 0, 10, 10).unwrap());
    b1.push(Box::new(20, 20, 30, 30).unwrap());

    let mut b2 = Boxa::new();
    b2.push(Box::new(0, 0, 10, 10).unwrap());
    b2.push(Box::new(20, 20, 30, 30).unwrap());

    assert!(b1.similar(&b2, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn test_similar_with_tolerance() {
    let mut b1 = Boxa::new();
    b1.push(Box::new(0, 0, 10, 10).unwrap());

    let mut b2 = Boxa::new();
    b2.push(Box::new(1, 1, 11, 11).unwrap());

    assert!(b1.similar(&b2, 1));
    assert!(!b1.similar(&b2, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn test_similar_different_count() {
    let mut b1 = Boxa::new();
    b1.push(Box::new(0, 0, 10, 10).unwrap());

    let b2 = Boxa::new();
    assert!(!b1.similar(&b2, 0));
}

#[test]
#[ignore = "not yet implemented"]
fn test_join_full() {
    let mut b1 = Boxa::new();
    b1.push(Box::new(0, 0, 10, 10).unwrap());

    let mut b2 = Boxa::new();
    b2.push(Box::new(20, 20, 10, 10).unwrap());
    b2.push(Box::new(30, 30, 10, 10).unwrap());

    b1.join(&b2, 0, 0); // end=0 means all
    assert_eq!(b1.len(), 3);
}

#[test]
#[ignore = "not yet implemented"]
fn test_join_range() {
    let mut b1 = Boxa::new();
    b1.push(Box::new(0, 0, 10, 10).unwrap());

    let mut b2 = Boxa::new();
    b2.push(Box::new(10, 10, 10, 10).unwrap());
    b2.push(Box::new(20, 20, 10, 10).unwrap());
    b2.push(Box::new(30, 30, 10, 10).unwrap());

    b1.join(&b2, 1, 3); // only index 1 and 2
    assert_eq!(b1.len(), 3);
    assert_eq!(b1.get(1).unwrap().x, 20);
    assert_eq!(b1.get(2).unwrap().x, 30);
}

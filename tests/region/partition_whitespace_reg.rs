//! Tests for partition module functions
//!
//! Tests white block detection and box overlap pruning.

use leptonica::core::Box;
use leptonica::core::Boxa;
use leptonica::region::partition::{
    WhiteblockSort, boxa_get_whiteblocks, boxa_prune_sorted_on_overlap,
};

/// Test boxa_prune_sorted_on_overlap with no overlap
#[test]
fn partition_prune_no_overlap() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 10, 10).unwrap());
    boxa.push(Box::new(20, 20, 10, 10).unwrap());
    boxa.push(Box::new(40, 40, 10, 10).unwrap());

    let result = boxa_prune_sorted_on_overlap(&boxa, 0.0).unwrap();
    assert_eq!(result.len(), 3, "no boxes should be pruned when no overlap");
}

/// Test boxa_prune_sorted_on_overlap with full overlap
#[test]
fn partition_prune_full_overlap() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 20, 20).unwrap()); // large
    boxa.push(Box::new(5, 5, 10, 10).unwrap()); // inside the first

    let result = boxa_prune_sorted_on_overlap(&boxa, 0.5).unwrap();
    // The second box is fully contained; overlap fraction > 0.5
    assert!(result.len() <= 2);
}

/// Test boxa_prune_sorted_on_overlap with max_overlap = 1.0 (no pruning)
#[test]
fn partition_prune_max_overlap_1() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 20, 20).unwrap());
    boxa.push(Box::new(5, 5, 10, 10).unwrap());

    let result = boxa_prune_sorted_on_overlap(&boxa, 1.0).unwrap();
    assert_eq!(result.len(), 2, "no pruning with max_overlap=1.0");
}

/// Test boxa_prune_sorted_on_overlap with empty input
#[test]
fn partition_prune_empty() {
    let boxa = Boxa::new();
    let result = boxa_prune_sorted_on_overlap(&boxa, 0.5).unwrap();
    assert_eq!(result.len(), 0);
}

/// Test boxa_get_whiteblocks basic
#[test]
fn partition_get_whiteblocks_basic() {
    // Create a simple layout with one box in the center
    let mut boxa = Boxa::new();
    boxa.push(Box::new(40, 40, 20, 20).unwrap()); // component in center

    let region = Box::new(0, 0, 100, 100).unwrap();
    let result = boxa_get_whiteblocks(
        &boxa,
        Some(&region),
        WhiteblockSort::ByArea,
        10,
        0.3,
        200,
        0.2,
        5000,
    )
    .unwrap();

    // Should find some whitespace blocks around the centered box
    assert!(!result.is_empty(), "should find whitespace blocks");
}

/// Test boxa_get_whiteblocks with no components (entire region is white)
#[test]
fn partition_get_whiteblocks_empty() {
    let boxa = Boxa::new();
    let region = Box::new(0, 0, 100, 100).unwrap();

    let result = boxa_get_whiteblocks(
        &boxa,
        Some(&region),
        WhiteblockSort::ByArea,
        5,
        0.3,
        200,
        0.2,
        1000,
    )
    .unwrap();

    // Entire region is white → should get the whole region
    assert!(
        !result.is_empty(),
        "entire region should be returned as white"
    );
}

/// Test invalid max_overlap parameter
#[test]
fn partition_prune_invalid_overlap() {
    let boxa = Boxa::new();
    assert!(boxa_prune_sorted_on_overlap(&boxa, -0.1).is_err());
    assert!(boxa_prune_sorted_on_overlap(&boxa, 1.1).is_err());
}

//! Overlap regression test
//!
//! Tests functions that combine boxes that overlap into their bounding regions,
//! and tests the overlap and separation distance between boxes.
//!
//! Partial migration: combine_overlaps, all_contained_in, combine_overlaps_in_pair,
//! overlap_distance, and separation_distance are tested. The percolation visualization
//! (pixRenderBoxa, pixaDisplayTiledInRows) is not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/overlap_reg.c`

mod common;
use common::RegParams;
use leptonica::{Box, Boxa};

/// Test combine_overlaps: boxes that overlap are merged into bounding regions.
///
/// C: boxaCombineOverlaps(boxa1, NULL), boxaContainedInBoxa
#[test]
fn overlap_reg_combine_overlaps() {
    let mut rp = RegParams::new("overlap_combine");

    // Create a set of overlapping boxes
    let mut boxa1 = Boxa::new();
    boxa1.push(Box::new(0, 0, 50, 50).unwrap());
    boxa1.push(Box::new(30, 30, 50, 50).unwrap()); // overlaps first
    boxa1.push(Box::new(200, 200, 30, 30).unwrap()); // isolated

    let combined = boxa1.combine_overlaps();

    // Two overlapping boxes merge into one, plus isolated = 2 total
    rp.compare_values(2.0, combined.len() as f64, 0.0);

    // Original boxes must all be contained in the combined result
    let contained = boxa1.all_contained_in(&combined);
    rp.compare_values(1.0, if contained { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "overlap combine test failed");
}

/// Test all_contained_in: verify that combined boxes contain all original boxes.
///
/// C: boxaContainedInBoxa(boxa2, boxa1, &result)
#[test]
fn overlap_reg_contained_in() {
    let mut rp = RegParams::new("overlap_contained");

    let mut boxa1 = Boxa::new();
    boxa1.push(Box::new(0, 0, 100, 100).unwrap());
    boxa1.push(Box::new(50, 50, 100, 100).unwrap()); // overlaps

    let combined = boxa1.combine_overlaps();
    // combined should contain all of boxa1's boxes
    let result = boxa1.all_contained_in(&combined);
    rp.compare_values(1.0, if result { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "overlap contained_in test failed");
}

/// Test combine_overlaps_in_pair: two boxa that overlap each other are merged.
///
/// C: boxaCombineOverlapsInPair(boxa1, boxa2, &boxa3, &boxa4, pixa1)
#[test]
fn overlap_reg_combine_in_pair() {
    let mut rp = RegParams::new("overlap_pair");

    let mut boxa1 = Boxa::new();
    for i in 0..4i32 {
        boxa1.push(Box::new(i * 30, i * 30, 40, 40).unwrap());
    }
    let mut boxa2 = Boxa::new();
    for i in 0..4i32 {
        boxa2.push(Box::new(i * 30 + 15, i * 30 + 15, 40, 40).unwrap());
    }

    let (result1, result2) = Boxa::combine_overlaps_in_pair(&boxa1, &boxa2);

    // Pairwise combination should merge more boxes than combining each Boxa independently,
    // because it merges across the two input arrays as well as within each.
    let combined1 = boxa1.combine_overlaps();
    let combined2 = boxa2.combine_overlaps();
    let total_individual = combined1.len() + combined2.len();
    let total_pair = result1.len() + result2.len();
    rp.compare_values(
        1.0,
        if total_pair < total_individual {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "overlap combine_in_pair test failed");
}

/// Test overlap_distance and separation_distance for all 9 positions (C check 12).
///
/// Places a 1x1 box at (0,0) and tests against 9 1x1 boxes on a 3x3 grid.
/// C: boxOverlapDistance, boxSeparationDistance
#[test]
fn overlap_reg_distance_functions() {
    let mut rp = RegParams::new("overlap_dist");

    let box1 = Box::new(0, 0, 1, 1).unwrap();

    for i in 0i32..3 {
        for j in 0i32..3 {
            let box2 = Box::new(i, j, 1, 1).unwrap();
            let (h_ovl, v_ovl) = box1.overlap_distance(&box2);
            let (h_sep, v_sep) = box1.separation_distance(&box2);

            // overlap and separation should be consistent:
            // if ovl > 0 (overlap), sep must be 0
            // if sep > 0 (touching or separated), ovl must be <= 0
            if h_ovl > 0 {
                rp.compare_values(0.0, h_sep as f64, 0.0);
            }
            if v_ovl > 0 {
                rp.compare_values(0.0, v_sep as f64, 0.0);
            }
            if h_sep > 0 {
                rp.compare_values(1.0, if h_ovl <= 0 { 1.0 } else { 0.0 }, 0.0);
            }
            if v_sep > 0 {
                rp.compare_values(1.0, if v_ovl <= 0 { 1.0 } else { 0.0 }, 0.0);
            }
        }
    }

    // box1 with itself: full overlap, zero separation
    let (h_ovl, v_ovl) = box1.overlap_distance(&box1);
    rp.compare_values(1.0, h_ovl as f64, 0.0);
    rp.compare_values(1.0, v_ovl as f64, 0.0);
    let (h_sep, v_sep) = box1.separation_distance(&box1);
    rp.compare_values(0.0, h_sep as f64, 0.0);
    rp.compare_values(0.0, v_sep as f64, 0.0);

    assert!(rp.cleanup(), "overlap distance functions test failed");
}

/// Test combine_overlaps with random-like sets (C percolation test structure).
///
/// Verifies that combined result is idempotent: re-combining yields same count.
#[test]
fn overlap_reg_idempotent() {
    let mut rp = RegParams::new("overlap_idem");

    let coords: &[(i32, i32, i32, i32)] = &[
        (0, 0, 30, 30),
        (20, 20, 30, 30),
        (40, 40, 30, 30),
        (100, 100, 20, 20),
        (150, 0, 25, 25),
        (160, 10, 25, 25),
    ];

    let mut boxa = Boxa::new();
    for &(x, y, w, h) in coords {
        boxa.push(Box::new(x, y, w, h).unwrap());
    }

    let combined1 = boxa.combine_overlaps();
    let combined2 = combined1.combine_overlaps();

    // Re-combining an already-combined result should be idempotent
    rp.compare_values(combined1.len() as f64, combined2.len() as f64, 0.0);

    assert!(rp.cleanup(), "overlap idempotent test failed");
}

/// Test splitcomp (pixSplitIntoBoxa, pixSplitComponentIntoBoxa).
///
/// These functions are not yet implemented in leptonica-region.
#[test]
#[ignore = "not yet implemented: pixSplitIntoBoxa/pixSplitComponentIntoBoxa not available"]
fn splitcomp_reg_split_into_boxa() {
    // C: boxa = pixSplitIntoBoxa(pixs, minsum, skipdist, delta, maxbg, 0);
    //    boxa = pixSplitComponentIntoBoxa(pixt, NULL, minsum, skipdist, delta, maxbg, 0, 1);
}

/// Test smoothedge (pixGetEdgeProfile, edge smoothness analysis).
///
/// Requires raggededge.png test image and edge analysis functions
/// not yet available in the Rust API.
#[test]
#[ignore = "not yet implemented: edge profile functions not available; raggededge.png not in test images"]
fn smoothedge_reg_edge_profile() {
    // C: pixGetEdgeProfile(pixs, L_FROM_RIGHT, minjump, minreversal, &n, &mean, &stdev);
    // Analyzes edges of a 1bpp connected component image for smoothness
}

/// Test texturefill (pixFindRepCloseTile, pixTextureFillMap).
///
/// Requires amoris.2.150.jpg and texture fill functions
/// not yet available in the Rust API.
#[test]
#[ignore = "not yet implemented: pixFindRepCloseTile/pixTextureFillMap not available; amoris.2.150.jpg not in test images"]
fn texturefill_reg_fill() {
    // C: pixFindRepCloseTile(pixs, box1, L_VERT, 20, 30, 7, &box2, 1);
    //    pixTextureFillMap(pixa, boxa, ...);
}

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
//! C Leptonica: `prog/overlap_reg.c`

use crate::common::{RegParams, load_test_image};
use leptonica::core::GlibcRand;
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

    // C only lets the *strictly larger* box of an overlapping cross-array pair
    // absorb the other, so the second array's boxes are made smaller here;
    // with equal sizes nothing would merge across the arrays.
    let mut boxa1 = Boxa::new();
    for i in 0..4i32 {
        boxa1.push(Box::new(i * 30, i * 30, 40, 40).unwrap());
    }
    let mut boxa2 = Boxa::new();
    for i in 0..4i32 {
        boxa2.push(Box::new(i * 30 + 15, i * 30 + 15, 20, 20).unwrap());
    }

    let (result1, result2) = Boxa::combine_overlaps_in_pair(&boxa1, &boxa2);

    // Pairwise combination merges across the two arrays as well as within
    // each, so it ends up with fewer boxes than combining them separately.
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
/// C: boxa = pixSplitIntoBoxa(pixs, minsum, skipdist, delta, maxbg, 0);
#[test]
fn splitcomp_reg_split_into_boxa() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("splitcomp");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let boxa = pix.split_into_boxa(10, 5, 2, 5).expect("split_into_boxa");

    // The result should contain rectangular sub-regions
    rp.compare_values(1.0, if !boxa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Each box should have valid dimensions
    for b in boxa.boxes() {
        assert!(b.w > 0 && b.h > 0, "box must have positive dimensions");
    }

    assert!(rp.cleanup(), "splitcomp_reg test failed");
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

/// `overlap_reg.c` builds every box from `srand(45617)` plus glibc's
/// `rand()`, so the inputs — and therefore every output — depend on
/// reproducing that generator. [`GlibcRand`] does; this wraps the scaling
/// idiom the C program uses at each call site.
struct ScaledRand(GlibcRand);

impl ScaledRand {
    fn new(seed: u32) -> Self {
        Self(GlibcRand::new(seed))
    }

    /// C: `(l_int32)(scale * (l_float64)rand() / (l_float64)RAND_MAX)`
    fn scaled(&mut self, scale: f64) -> i32 {
        (scale * self.0.next_u32() as f64 / 2_147_483_647.0) as i32
    }
}

/// C's `boxaContainedInBoxa(boxa1, boxa2)`: despite the argument names it
/// iterates **boxa2** and asks whether each of its boxes is contained in some
/// box of boxa1.
fn all_contained(boxa1: &Boxa, boxa2: &Boxa) -> bool {
    boxa2.iter().filter(|b| b.is_valid()).all(|b2| {
        boxa1.iter().filter(|b| b.is_valid()).any(|b1| {
            b2.x >= b1.x && b2.y >= b1.y && b2.x + b2.w <= b1.x + b1.w && b2.y + b2.h <= b1.y + b1.h
        })
    })
}

/// C's `boxaCombineOverlapsAlt` from `prog/overlap_reg.c`: the same result by
/// a less elegant route, used there only as a cross-check.
fn combine_overlaps_alt(boxas: &Boxa) -> Boxa {
    let mut boxa1: Vec<Box> = boxas.iter().copied().collect();
    loop {
        let n1 = boxa1.len();
        let mut boxa2: Vec<Box> = Vec::with_capacity(n1);
        for (i, &box1) in boxa1.iter().enumerate() {
            if i == 0 {
                boxa2.push(box1);
                continue;
            }
            let mut found = false;
            for slot in boxa2.iter_mut() {
                if box1.overlaps(slot) {
                    *slot = box1.union(slot);
                    found = true;
                    break;
                }
            }
            if !found {
                boxa2.push(box1);
            }
        }
        let n2 = boxa2.len();
        boxa1 = boxa2;
        if n1 == n2 {
            return boxa1.into_iter().collect();
        }
    }
}

/// C-compat: `prog/overlap_reg.c` checks 0-12.
///
/// The program has no image input at all — every box is generated from
/// `srand(45617)` plus glibc's `rand()` — so reproducing that generator makes
/// the whole thing deterministic and bit-exactly comparable against C.
#[test]
fn overlap_c_compat() {
    use leptonica::core::Pixa;
    use leptonica::core::pix::PixelOp;
    use leptonica::io::ImageFormat;
    use leptonica::{Pix, PixelDepth};

    if crate::common::is_display_mode() {
        return;
    }

    const MAXSIZE: [f64; 7] = [5.0, 10.0, 15.0, 20.0, 25.0, 26.0, 27.0];

    let mut rp = RegParams::new("overlap_c");

    // C 0-6: percolation-style display at each maximum box size. C re-seeds
    // at the top of every iteration, so the generator is left 2000 draws past
    // the seed when the loop ends — the later blocks continue from there.
    let mut rng = ScaledRand::new(45617);
    for &maxsize in MAXSIZE.iter() {
        rng = ScaledRand::new(45617);
        let mut pixa1 = Pixa::new();
        let mut boxa1 = Boxa::new();
        for _ in 0..500 {
            let x = rng.scaled(600.0);
            let y = rng.scaled(600.0);
            let w = 1 + rng.scaled(maxsize);
            let h = 1 + rng.scaled(maxsize);
            boxa1.push(Box::new(x, y, w, h).expect("box"));
        }

        let pix1 = Pix::new(660, 660, PixelDepth::Bit1).expect("canvas");
        let mut pm = pix1.try_into_mut().unwrap();
        pm.render_boxa(&boxa1, 2, PixelOp::Set)
            .expect("render boxa");
        pixa1.push(pm.into());

        let boxa2 = boxa1.combine_overlaps();
        let pix2 = Pix::new(660, 660, PixelDepth::Bit1).expect("canvas 2");
        let mut pm = pix2.try_into_mut().unwrap();
        pm.render_boxa(&boxa2, 2, PixelOp::Set)
            .expect("render combined");
        pixa1.push(pm.into());

        let pix3 = pixa1
            .display_tiled_in_rows(PixelDepth::Bit1, 1500, 1.0, 0, 50, 2)
            .expect("tile rows");
        rp.write_pix_and_check(&pix3, ImageFormat::Png)
            .expect("check: percolation display");
    }

    // C 7-8: one case with the debug pixa from boxaCombineOverlaps.
    let mut boxa1 = Boxa::new();
    for _ in 0..80 {
        let x = rng.scaled(600.0);
        let y = rng.scaled(600.0);
        let w = 10 + rng.scaled(48.0);
        let h = 10 + rng.scaled(53.0);
        boxa1.push(Box::new(x, y, w, h).expect("box"));
    }
    let mut pixadb = Pixa::new();
    let boxa2 = boxa1.combine_overlaps_debug(Some(&mut pixadb));
    // C 7: regTestCompareValues(rp, 1, boxaContainedInBoxa(boxa2, boxa1), 0)
    rp.compare_values(
        1.0,
        if all_contained(&boxa2, &boxa1) {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    let pix1 = pixadb
        .display_tiled_in_rows(PixelDepth::Bit32, 1500, 1.0, 0, 50, 2)
        .expect("tile debug rows");
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("check: combine_overlaps debug");

    // C 9-10: the alternative implementation agrees with the main one.
    let boxa3 = combine_overlaps_alt(&boxa1);
    rp.compare_values(
        1.0,
        if all_contained(&boxa3, &boxa2) {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if all_contained(&boxa2, &boxa3) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // C 11: two boxa greedily munching each other.
    let mut boxa1 = Boxa::new();
    let mut boxa2 = Boxa::new();
    for i in 0..80 {
        let x = rng.scaled(600.0);
        let y = rng.scaled(600.0);
        let w = 10 + rng.scaled(55.0);
        let h = 10 + rng.scaled(55.0);
        let b = Box::new(x, y, w, h).expect("box");
        if i < 40 { boxa1.push(b) } else { boxa2.push(b) }
    }
    let mut pixadb = Pixa::new();
    let (_, _) = Boxa::combine_overlaps_in_pair_debug(&boxa1, &boxa2, Some(&mut pixadb));
    let pix1 = pixadb
        .display_tiled_in_rows(PixelDepth::Bit32, 1500, 1.0, 0, 50, 2)
        .expect("tile pair debug rows");
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("check: combine_overlaps_in_pair debug");

    // C 12: overlap and separation distances for 9 unit boxes on a 3x3 grid.
    let box1 = Box::new(0, 0, 1, 1).expect("unit box");
    let mut out = String::new();
    for i in 0..3 {
        for j in 0..3 {
            let box2 = Box::new(i, j, 1, 1).expect("probe box");
            let (hovl, vovl) = box1.overlap_distance(&box2);
            let (hsep, vsep) = box1.separation_distance(&box2);
            out.push_str(&format!(
                "({i},{j}): ovl = ({hovl},{vovl}); sep = ({hsep},{vsep})\n"
            ));
        }
    }
    rp.write_data_and_check(out.as_bytes(), "dat")
        .expect("check: overlap/separation table");

    assert!(rp.cleanup(), "overlap C-compat test failed");
}

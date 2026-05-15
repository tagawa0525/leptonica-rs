//! Regression tests for compare_tiles_by_histo (plan 142).

use leptonica::core::numa::{Numa, Numaa};
use leptonica::core::pix::compare_tiles_by_histo;

fn one_tile_histo(peak_bin: usize, count: f32) -> Numa {
    let mut na = Numa::new();
    for i in 0..256 {
        na.push(if i == peak_bin { count } else { 0.0 });
    }
    na
}

#[test]
fn compare_tiles_identical_naas_score_1() {
    // Two identical Numaa: score should be 1.0.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    for _ in 0..3 {
        naa1.push(one_tile_histo(100, 50.0));
        naa2.push(one_tile_histo(100, 50.0));
    }
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    assert!(
        (score - 1.0).abs() < 1e-4,
        "expected score 1.0 for identical histos, got {score}"
    );
}

#[test]
fn compare_tiles_very_different_score_low() {
    // Peak shift from bin 50 to bin 150 (distance 100) → EMD/255 ≈ 0.39.
    // 1 - 10 * 0.39 < 0 → clamped to 0.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(one_tile_histo(50, 50.0));
    naa2.push(one_tile_histo(150, 50.0));
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    assert!(
        score < 0.1,
        "expected near-0 score for very different histos, got {score}"
    );
}

#[test]
fn compare_tiles_size_mismatch_returns_zero() {
    // wratio = 50/100 = 0.5 < 0.9 → early-exit 0.0.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(one_tile_histo(100, 50.0));
    naa2.push(one_tile_histo(100, 50.0));
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 50, 100, 100, 100).unwrap();
    assert!(score.abs() < 1e-4);
}

#[test]
fn compare_tiles_tile_count_mismatch_returns_zero() {
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(one_tile_histo(100, 50.0));
    naa1.push(one_tile_histo(100, 50.0));
    naa2.push(one_tile_histo(100, 50.0));
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    assert!(score.abs() < 1e-4);
}

#[test]
fn compare_tiles_min_score_is_minimum() {
    // 3 tiles: first identical, middle slightly different, last identical.
    // Score should reflect the worst (middle) tile, not the average.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(one_tile_histo(100, 50.0));
    naa1.push(one_tile_histo(100, 50.0)); // tile 1 will be made worse
    naa1.push(one_tile_histo(100, 50.0));
    naa2.push(one_tile_histo(100, 50.0));
    naa2.push(one_tile_histo(105, 50.0)); // bin shift of 5
    naa2.push(one_tile_histo(100, 50.0));
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    // Tile 1: EMD ≈ 5/255 ≈ 0.0196, score = 1 - 10*0.0196 ≈ 0.804.
    // Tiles 0 and 2 score 1.0. Min should be ~0.804.
    assert!(
        score > 0.7 && score < 0.9,
        "expected min-score ≈ 0.8, got {score}"
    );
}

#[test]
fn compare_tiles_rejects_invalid_minratio() {
    let naa1 = Numaa::new();
    let naa2 = Numaa::new();
    assert!(compare_tiles_by_histo(&naa1, &naa2, -0.1, 100, 100, 100, 100).is_err());
    assert!(compare_tiles_by_histo(&naa1, &naa2, 1.5, 100, 100, 100, 100).is_err());
}

#[test]
fn compare_tiles_empty_returns_zero() {
    let naa1 = Numaa::new();
    let naa2 = Numaa::new();
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    assert!(score.abs() < 1e-4);
}

#[test]
fn compare_tiles_ignores_white_bin_255() {
    // Two histos that are identical except for bin 255 (white). Should
    // still score 1.0 because bin 255 is zeroed before EMD.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    let mut na1 = one_tile_histo(100, 50.0);
    na1.set(255, 1000.0).unwrap(); // lots of white in naa1
    let mut na2 = one_tile_histo(100, 50.0);
    na2.set(255, 0.0).unwrap(); // no white in naa2
    naa1.push(na1);
    naa2.push(na2);
    let score = compare_tiles_by_histo(&naa1, &naa2, 0.9, 100, 100, 100, 100).unwrap();
    assert!(
        (score - 1.0).abs() < 1e-4,
        "expected white-ignored score 1.0, got {score}"
    );
}

//! Regression tests for Numa::split_distribution (plan 133).

use leptonica::core::numa::Numa;

#[test]
fn split_bimodal_histogram() {
    // Bimodal: peaks at 1..3 (low) and 7..9 (high), valley near 5.
    let mut na = Numa::new();
    for v in [0, 10, 30, 10, 1, 0, 1, 10, 30, 10] {
        na.push(v as f32);
    }
    let r = na.split_distribution(0.1, false).unwrap();
    // Best split should land in the valley (between modes 2 and 8).
    // With C's +1 offset, expect around 5.
    assert!(
        (3..=7).contains(&r.split_index),
        "expected split near valley, got {}",
        r.split_index
    );
    // Both partitions should have positive mass at the chosen split.
    assert!(r.num1 > 0.0 && r.num2 > 0.0);
}

#[test]
fn split_distribution_returns_score_array() {
    let mut na = Numa::new();
    for v in [0, 5, 20, 5, 0, 5, 20, 5, 0] {
        na.push(v as f32);
    }
    let r = na.split_distribution(0.05, true).unwrap();
    let score = r.score.expect("want_score=true should return Some");
    assert_eq!(score.len(), na.len());
    // Scores are non-negative.
    for i in 0..score.len() {
        assert!(score.get(i).unwrap() >= 0.0);
    }
}

#[test]
fn split_distribution_no_score_when_not_requested() {
    let mut na = Numa::new();
    for v in [0, 10, 30, 10, 0] {
        na.push(v as f32);
    }
    let r = na.split_distribution(0.1, false).unwrap();
    assert!(r.score.is_none());
}

#[test]
fn split_distribution_rejects_small_input() {
    let mut na = Numa::new();
    na.push(5.0); // n = 1
    assert!(na.split_distribution(0.1, false).is_err());
    let empty = Numa::new();
    assert!(empty.split_distribution(0.1, false).is_err());
}

#[test]
fn split_distribution_rejects_zero_sum() {
    let mut na = Numa::new();
    for _ in 0..5 {
        na.push(0.0);
    }
    assert!(na.split_distribution(0.1, false).is_err());
}

#[test]
fn split_distribution_caps_index_at_255() {
    // 300-bin histogram, ensure the +1 / 255-cap rule works.
    let mut na = Numa::new();
    for i in 0..300u32 {
        // Most mass near the right end (around bin 250).
        let v = if (245..=255).contains(&i) { 100 } else { 1 };
        na.push(v as f32);
    }
    let r = na.split_distribution(0.1, false).unwrap();
    assert!(
        r.split_index <= 255,
        "split index must be capped at 255, got {}",
        r.split_index
    );
}

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
fn split_distribution_rejects_invalid_score_fract() {
    let mut na = Numa::new();
    for v in [0, 5, 20, 5, 0] {
        na.push(v as f32);
    }
    assert!(na.split_distribution(-0.1, false).is_err());
    assert!(na.split_distribution(1.5, false).is_err());
    assert!(na.split_distribution(f32::NAN, false).is_err());
}

#[test]
fn split_distribution_caps_index_at_255() {
    // Construct a 512-bin bimodal histogram so the Otsu-optimal split lies
    // well above 254. Without the cap, `split_index` would be a number in
    // the high 300s+; the cap clamps it to exactly 255.
    let mut na = Numa::new();
    for i in 0..512u32 {
        // Two clear peaks: one centred near bin 320, another near bin 480.
        // The valley between them sits around bin 400 (>> 255).
        let v = if (300..=340).contains(&i) || (460..=500).contains(&i) {
            100
        } else {
            0
        };
        na.push(v as f32);
    }
    let r = na.split_distribution(0.1, false).unwrap();
    assert_eq!(
        r.split_index, 255,
        "expected the 255 cap to clamp the split index (got {})",
        r.split_index
    );
}

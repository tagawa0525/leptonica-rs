//! Regression tests for Numa::eval_haar_sum / eval_best_haar_parameters
//! (plan 136).

use leptonica::core::numa::Numa;

#[test]
fn eval_haar_sum_zero_signal() {
    let mut na = Numa::new();
    for _ in 0..100 {
        na.push(0.0);
    }
    let score = na.eval_haar_sum(5.0, 0.0, 1.0).unwrap();
    assert!(
        score.abs() < 1e-5,
        "zero signal should give 0 score, got {score}"
    );
}

#[test]
fn eval_haar_sum_periodic_signal_maximal_at_matching_width() {
    // 100-element square wave with period 20: bar (val = 1) for 10 samples,
    // then gap (val = 0) for 10 samples, repeated. The Haar comb at
    // width = 10 alternates between bar centres and gap centres, producing
    // a strong magnitude. A mismatched width (e.g. 7) samples a mixture and
    // gives a much smaller magnitude.
    let mut na = Numa::new();
    for i in 0..100 {
        na.push(if (i / 10) % 2 == 0 { 1.0 } else { 0.0 });
    }
    let matched = na.eval_haar_sum(10.0, 0.0, 1.0).unwrap().abs();
    let mismatched = na.eval_haar_sum(7.0, 0.0, 1.0).unwrap().abs();
    assert!(
        matched > mismatched * 1.5,
        "matched width should clearly dominate: matched={matched}, mismatched={mismatched}"
    );
}

#[test]
fn eval_haar_sum_rejects_invalid_width() {
    let mut na = Numa::new();
    for _ in 0..20 {
        na.push(1.0);
    }
    assert!(na.eval_haar_sum(0.0, 0.0, 1.0).is_err());
    assert!(na.eval_haar_sum(-1.0, 0.0, 1.0).is_err());
    assert!(na.eval_haar_sum(f32::NAN, 0.0, 1.0).is_err());
}

#[test]
fn eval_haar_sum_rejects_too_short_signal() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
    // width = 5 → n < 2 * width (4 < 10), should error.
    assert!(na.eval_haar_sum(5.0, 0.0, 1.0).is_err());
}

// -- eval_best_haar_parameters ----------------------------------------------

#[test]
fn eval_best_haar_picks_periodic_width() {
    // 200-element square wave with period 20 (matched width = 10). The
    // sweep should converge on width ≈ 10.
    let mut na = Numa::new();
    for i in 0..200 {
        na.push(if (i / 10) % 2 == 0 { 1.0 } else { 0.0 });
    }
    let (best_width, _shift, best_score) =
        na.eval_best_haar_parameters(1.0, 20, 4, 4.0, 20.0).unwrap();
    assert!(best_score > 0.0, "best_score should be > 0");
    assert!(
        (best_width - 10.0).abs() < 1.5,
        "expected best_width near 10, got {best_width}"
    );
}

#[test]
fn eval_best_haar_rejects_invalid_nwidth() {
    let na = Numa::from_vec(vec![1.0; 50]);
    assert!(na.eval_best_haar_parameters(1.0, 0, 4, 5.0, 10.0).is_err());
    assert!(na.eval_best_haar_parameters(1.0, 1, 4, 5.0, 10.0).is_err());
}

#[test]
fn eval_best_haar_rejects_invalid_nshift() {
    let na = Numa::from_vec(vec![1.0; 50]);
    assert!(na.eval_best_haar_parameters(1.0, 5, 0, 5.0, 10.0).is_err());
}

#[test]
fn eval_best_haar_rejects_invalid_width_range() {
    let na = Numa::from_vec(vec![1.0; 50]);
    assert!(na.eval_best_haar_parameters(1.0, 5, 4, 0.0, 10.0).is_err());
    assert!(na.eval_best_haar_parameters(1.0, 5, 4, 10.0, 5.0).is_err());
}

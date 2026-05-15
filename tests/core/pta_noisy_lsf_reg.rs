//! Regression tests for Pta::noisy_linear_lsf / noisy_quadratic_lsf
//! (plan 138).

use leptonica::core::pta::Pta;

fn make_linear_with_outlier() -> Pta {
    // y = 2x + 1 for x in 0..=9, with outlier injected at x = 5 (y = 100).
    let mut pta = Pta::new();
    for x in 0..=9 {
        let y = if x == 5 { 100.0 } else { 2.0 * x as f32 + 1.0 };
        pta.push(x as f32, y);
    }
    pta
}

#[test]
fn noisy_linear_lsf_rejects_outlier_and_recovers_slope() {
    let pta = make_linear_with_outlier();
    let r = pta.noisy_linear_lsf(3.0, false).unwrap();
    // After outlier removal, the refit should be very close to y = 2x + 1.
    assert!((r.a - 2.0).abs() < 0.05, "expected a≈2, got {}", r.a);
    assert!((r.b - 1.0).abs() < 0.5, "expected b≈1, got {}", r.b);
    // The 10-point input had 1 outlier → inliers should have 9 points.
    assert_eq!(r.inliers.len(), 9);
    // No fit array requested.
    assert!(r.fit.is_none());
}

#[test]
fn noisy_linear_lsf_returns_fit_when_requested() {
    let pta = make_linear_with_outlier();
    let r = pta.noisy_linear_lsf(3.0, true).unwrap();
    let fit = r.fit.unwrap();
    assert_eq!(fit.len(), r.inliers.len());
}

#[test]
fn noisy_linear_lsf_rejects_invalid_factor() {
    let pta = make_linear_with_outlier();
    assert!(pta.noisy_linear_lsf(0.0, false).is_err());
    assert!(pta.noisy_linear_lsf(-1.0, false).is_err());
    assert!(pta.noisy_linear_lsf(f32::NAN, false).is_err());
}

#[test]
fn noisy_linear_lsf_rejects_too_few_points() {
    let mut pta = Pta::new();
    pta.push(0.0, 0.0);
    pta.push(1.0, 1.0);
    assert!(pta.noisy_linear_lsf(3.0, false).is_err());
}

// -- noisy_quadratic_lsf ----------------------------------------------------

fn make_quadratic_with_outlier() -> Pta {
    // y = 2x^2 + 3x + 1 for x in 0..=9, with outlier at x = 5 (y = 1000).
    let mut pta = Pta::new();
    for x in 0..=9 {
        let xf = x as f32;
        let y = if x == 5 {
            1000.0
        } else {
            2.0 * xf * xf + 3.0 * xf + 1.0
        };
        pta.push(xf, y);
    }
    pta
}

#[test]
fn noisy_quadratic_lsf_rejects_outlier_and_recovers_coeffs() {
    let pta = make_quadratic_with_outlier();
    let r = pta.noisy_quadratic_lsf(3.0, false).unwrap();
    assert!((r.a - 2.0).abs() < 0.1, "expected a≈2, got {}", r.a);
    assert!((r.b - 3.0).abs() < 0.5, "expected b≈3, got {}", r.b);
    assert!((r.c - 1.0).abs() < 1.0, "expected c≈1, got {}", r.c);
    assert_eq!(r.inliers.len(), 9);
}

#[test]
fn noisy_quadratic_lsf_rejects_invalid_factor() {
    let pta = make_quadratic_with_outlier();
    assert!(pta.noisy_quadratic_lsf(0.0, false).is_err());
    assert!(pta.noisy_quadratic_lsf(-2.0, false).is_err());
}

#[test]
fn noisy_quadratic_lsf_rejects_too_few_points() {
    let mut pta = Pta::new();
    for i in 0..3 {
        pta.push(i as f32, 0.0);
    }
    // n = 3 < 4 → Err
    assert!(pta.noisy_quadratic_lsf(3.0, false).is_err());
}

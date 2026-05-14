//! Regression tests for Numa::crossings_by_peaks (plan 134).

use leptonica::core::numa::Numa;

#[test]
fn crossings_by_peaks_simple_oscillation() {
    // Triangular wave: peak at idx 4 (val 4), trough at idx 10 (val -2),
    // end-of-array at idx 14. With find_extrema(1.0):
    //   - segment 0..4: threshold = (0 + 4) / 2 = 2 → crosses at idx 2
    //   - segment 4..10: threshold = (4 + (-2)) / 2 = 1 → crosses at idx 7
    //   - segment 10..14: threshold = (-2 + 2) / 2 = 0 → crosses at idx 12
    let na = Numa::from_vec(vec![
        0.0, 1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -2.0, -1.0, 0.0, 1.0, 2.0,
    ]);
    let crossings = na.crossings_by_peaks(None, 1.0).unwrap();
    assert_eq!(crossings.len(), 3, "expected 3 crossings");
    let eps = 1e-4;
    assert!((crossings.get(0).unwrap() - 2.0).abs() < eps);
    assert!((crossings.get(1).unwrap() - 7.0).abs() < eps);
    assert!((crossings.get(2).unwrap() - 12.0).abs() < eps);
}

#[test]
fn crossings_by_peaks_too_short_returns_empty() {
    let na = Numa::from_vec(vec![1.0]);
    let crossings = na.crossings_by_peaks(None, 1.0).unwrap();
    assert_eq!(crossings.len(), 0);
}

#[test]
fn crossings_by_peaks_uses_nax_when_provided() {
    // Half wave: peak at idx 4 (val 4), end at idx 8 (val 0).
    // With find_extrema(1.0):
    //   - segment 0..4: threshold = 2 → crosses at idx 2
    //   - segment 4..8: threshold = 2 → crosses at idx 6
    // nax scales positions by 10×, so crossings should be at 20 and 60.
    let nay = Numa::from_vec(vec![0.0, 1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
    let mut nax = Numa::new();
    for i in 0..nay.len() {
        nax.push(i as f32 * 10.0);
    }
    let crossings = nay.crossings_by_peaks(Some(&nax), 1.0).unwrap();
    assert_eq!(crossings.len(), 2);
    let eps = 1e-4;
    assert!(
        (crossings.get(0).unwrap() - 20.0).abs() < eps,
        "got {}",
        crossings.get(0).unwrap()
    );
    assert!(
        (crossings.get(1).unwrap() - 60.0).abs() < eps,
        "got {}",
        crossings.get(1).unwrap()
    );
}

#[test]
fn crossings_by_peaks_length_mismatch_errors() {
    let nay = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let nax = Numa::from_vec(vec![1.0, 2.0]);
    assert!(nay.crossings_by_peaks(Some(&nax), 0.5).is_err());
}

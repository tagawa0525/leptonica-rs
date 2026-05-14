//! Regression tests for Numa::crossings_by_peaks (plan 134).

use leptonica::core::numa::Numa;

#[test]
fn crossings_by_peaks_simple_oscillation() {
    // Triangular wave with peak at index 4 and trough at index 8:
    // 0, 1, 2, 3, 4, 3, 2, 1, 0, -1, -2, -1, 0, 1, 2.
    let na = Numa::from_vec(vec![
        0.0, 1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -2.0, -1.0, 0.0, 1.0, 2.0,
    ]);
    let crossings = na.crossings_by_peaks(None, 1.0).unwrap();
    // Should find at least 2 crossings (one in the descending segment,
    // one in the ascending segment after the trough).
    assert!(
        crossings.len() >= 2,
        "expected >= 2 crossings, got {}",
        crossings.len()
    );
}

#[test]
fn crossings_by_peaks_too_short_returns_empty() {
    let na = Numa::from_vec(vec![1.0]);
    let crossings = na.crossings_by_peaks(None, 1.0).unwrap();
    assert_eq!(crossings.len(), 0);
}

#[test]
fn crossings_by_peaks_uses_nax_when_provided() {
    // nay: same triangular wave; nax: explicit x positions x[i] = i * 10.0
    let nay = Numa::from_vec(vec![0.0, 1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
    let mut nax = Numa::new();
    for i in 0..nay.len() {
        nax.push(i as f32 * 10.0);
    }
    let crossings = nay.crossings_by_peaks(Some(&nax), 1.0).unwrap();
    // All output values should fall within the nax range [0, 80].
    for i in 0..crossings.len() {
        let v = crossings.get(i).unwrap();
        assert!((0.0..=80.0).contains(&v), "crossing {v} out of range");
    }
}

#[test]
fn crossings_by_peaks_length_mismatch_errors() {
    let nay = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let nax = Numa::from_vec(vec![1.0, 2.0]);
    assert!(nay.crossings_by_peaks(Some(&nax), 0.5).is_err());
}

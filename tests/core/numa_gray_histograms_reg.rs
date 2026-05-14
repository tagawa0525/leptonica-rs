//! Regression tests for gray_inter_histogram_stats / gray_histograms_to_emd
//! (plan 135).

use leptonica::core::numa::{Numa, Numaa};

fn flat_256(val: f32) -> Numa {
    let mut na = Numa::new();
    for _ in 0..256 {
        na.push(val);
    }
    na
}

fn shifted_256(start: usize, count: usize, val: f32) -> Numa {
    let mut na = Numa::new();
    for i in 0..256 {
        if i >= start && i < start + count {
            na.push(val);
        } else {
            na.push(0.0);
        }
    }
    na
}

// -- gray_histograms_to_emd ------------------------------------------------

#[test]
fn gray_histograms_emd_identical() {
    // Same histograms in both NAAs -> EMD = 0 for each pair.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    for _ in 0..3 {
        naa1.push(flat_256(1.0));
        naa2.push(flat_256(1.0));
    }
    let r = Numa::gray_histograms_to_emd(&naa1, &naa2).unwrap();
    assert_eq!(r.len(), 3);
    for i in 0..3 {
        assert!(r.get(i).unwrap().abs() < 1e-5);
    }
}

#[test]
fn gray_histograms_emd_normalizes_by_255() {
    // Shift mass from bin 0 to bin 255 -> raw EMD = 255, normalized = 1.0.
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(shifted_256(0, 1, 100.0));
    naa2.push(shifted_256(255, 1, 100.0));
    let r = Numa::gray_histograms_to_emd(&naa1, &naa2).unwrap();
    assert_eq!(r.len(), 1);
    let v = r.get(0).unwrap();
    assert!(
        (v - 1.0).abs() < 1e-3,
        "expected normalized EMD ~ 1.0, got {v}"
    );
}

#[test]
fn gray_histograms_emd_length_mismatch_errors() {
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    naa1.push(flat_256(1.0));
    naa1.push(flat_256(1.0));
    naa2.push(flat_256(1.0));
    assert!(Numa::gray_histograms_to_emd(&naa1, &naa2).is_err());
}

#[test]
fn gray_histograms_emd_wrong_inner_size_errors() {
    let mut naa1 = Numaa::new();
    let mut naa2 = Numaa::new();
    let short = Numa::from_vec(vec![1.0; 100]);
    naa1.push(short.clone());
    naa2.push(short);
    assert!(Numa::gray_histograms_to_emd(&naa1, &naa2).is_err());
}

// -- gray_inter_histogram_stats --------------------------------------------

#[test]
fn gray_inter_stats_identical_inputs_zero_variance() {
    let mut naa = Numaa::new();
    for _ in 0..4 {
        // Single peak histogram at bin 100, all identical.
        naa.push(shifted_256(100, 1, 10.0));
    }
    let stats = Numa::gray_inter_histogram_stats(&naa, 0, true, true, true, true).unwrap();
    let mean = stats.mean.unwrap();
    let ms = stats.mean_square.unwrap();
    let var = stats.variance.unwrap();
    let rms = stats.rms.unwrap();
    assert_eq!(mean.len(), 256);
    assert_eq!(ms.len(), 256);
    assert_eq!(var.len(), 256);
    assert_eq!(rms.len(), 256);
    // Identical inputs -> variance is 0 everywhere.
    for j in 0..256 {
        assert!(var.get(j).unwrap().abs() < 1e-3, "var[{j}] not 0");
        assert!(rms.get(j).unwrap().abs() < 1e-3, "rms[{j}] not 0");
    }
}

#[test]
fn gray_inter_stats_requires_at_least_one_output() {
    let mut naa = Numaa::new();
    naa.push(flat_256(1.0));
    assert!(Numa::gray_inter_histogram_stats(&naa, 0, false, false, false, false).is_err());
}

#[test]
fn gray_inter_stats_rejects_wrong_inner_size() {
    let mut naa = Numaa::new();
    naa.push(Numa::from_vec(vec![1.0; 100]));
    assert!(Numa::gray_inter_histogram_stats(&naa, 0, true, false, false, false).is_err());
}

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
    // Identical inputs (no smoothing) → only bin 100 carries mass. After
    // normalization to sum = 10000, bin 100 has value 10000 and others 0.
    assert!(
        (mean.get(100).unwrap() - 10000.0).abs() < 1.0,
        "expected mean[100] ≈ 10000, got {}",
        mean.get(100).unwrap()
    );
    assert!(mean.get(50).unwrap().abs() < 1.0, "expected mean[50] ≈ 0");
    // mean_square = (E[x])^2: bin 100 carries 10000^2.
    assert!(
        (ms.get(100).unwrap() - 10000.0 * 10000.0).abs() < 100.0,
        "expected mean_square[100] ≈ 1e8, got {}",
        ms.get(100).unwrap()
    );
    // Identical inputs → variance and rms are 0 everywhere.
    for j in 0..256 {
        assert!(var.get(j).unwrap().abs() < 1e-3, "var[{j}] not 0");
        assert!(rms.get(j).unwrap().abs() < 1e-3, "rms[{j}] not 0");
    }
}

#[test]
fn gray_inter_stats_differing_inputs_nonzero_variance() {
    // Two histograms with mass at *different* bins → variance > 0 at those
    // bins. Confirms mean / variance follow the standard
    // E[x^2] - (E[x])^2 identity.
    let mut naa = Numaa::new();
    naa.push(shifted_256(50, 1, 10.0));
    naa.push(shifted_256(200, 1, 10.0));
    let stats = Numa::gray_inter_histogram_stats(&naa, 0, true, false, true, false).unwrap();
    let mean = stats.mean.unwrap();
    let var = stats.variance.unwrap();
    // Mean at bin 50 = (10000 + 0) / 2 = 5000. Var = E[x^2] - mean^2 =
    // (10000^2 + 0) / 2 - 5000^2 = 50_000_000 - 25_000_000 = 25_000_000.
    assert!((mean.get(50).unwrap() - 5000.0).abs() < 1.0);
    assert!((mean.get(200).unwrap() - 5000.0).abs() < 1.0);
    assert!((var.get(50).unwrap() - 25_000_000.0).abs() < 1000.0);
    assert!((var.get(200).unwrap() - 25_000_000.0).abs() < 1000.0);
    // Bins with no mass have zero mean & variance.
    assert!(mean.get(100).unwrap().abs() < 1.0);
    assert!(var.get(100).unwrap().abs() < 1.0);
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

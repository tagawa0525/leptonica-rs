//! Regression tests for Numa::get_rank_bin_values (plan 130-extension).

use leptonica::core::numa::Numa;

#[test]
fn get_rank_bin_values_sort_path() {
    // Small data with large max -> shell-sort path.
    let na = Numa::from_i32_slice(&[1, 1000000, 500, 50, 999999]);
    let out = na.get_rank_bin_values(2).unwrap();
    assert_eq!(out.len(), 2);
    // Values are sorted: 1, 50, 500, 999999, 1000000.
    // Bin sizes from numa_uniform_bin_sizes(5, 2): one bin of 3 and one of 2.
    // The first bin should have a smaller average than the second.
    assert!(out.get(0).unwrap() < out.get(1).unwrap());
}

#[test]
fn get_rank_bin_values_histogram_path() {
    // Large data with small max (8 bpp pixel-style) -> histogram path.
    let mut na = Numa::new();
    for v in 0..256 {
        for _ in 0..10 {
            na.push(v as f32);
        }
    }
    let out = na.get_rank_bin_values(4).unwrap();
    assert_eq!(out.len(), 4);
    // Bin averages should be monotonically increasing.
    let v0 = out.get(0).unwrap();
    let v1 = out.get(1).unwrap();
    let v2 = out.get(2).unwrap();
    let v3 = out.get(3).unwrap();
    assert!(v0 < v1 && v1 < v2 && v2 < v3);
    // Roughly span the full [0, 255] range.
    assert!(v0 < 64.0);
    assert!(v3 > 192.0);
}

#[test]
fn get_rank_bin_values_rejects_invalid() {
    let na = Numa::from_i32_slice(&[1, 2, 3]);
    assert!(na.get_rank_bin_values(0).is_err());
    assert!(na.get_rank_bin_values(1).is_err());
    let empty = Numa::new();
    assert!(empty.get_rank_bin_values(4).is_err());
}

#[test]
fn get_rank_bin_values_shifted_histogram() {
    // Values shifted away from 0 should still produce averages in the
    // original value domain (not internal histogram indices). 1000..1255
    // each repeated 10x → histogram path with non-zero binstart.
    let mut na = Numa::new();
    for v in 1000..1256 {
        for _ in 0..10 {
            na.push(v as f32);
        }
    }
    let out = na.get_rank_bin_values(4).unwrap();
    assert_eq!(out.len(), 4);
    let v0 = out.get(0).unwrap();
    let v3 = out.get(3).unwrap();
    // Values should be in the 1000..1256 range, not 0..256.
    assert!(
        v0 > 999.0 && v0 < 1100.0,
        "first bin {v0} should be near 1000 (not histogram index 0)"
    );
    assert!(
        v3 > 1100.0 && v3 < 1260.0,
        "last bin {v3} should be near 1255 (not histogram index)"
    );
}

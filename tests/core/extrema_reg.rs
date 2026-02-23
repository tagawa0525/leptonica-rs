//! Extrema regression test
//!
//! Tests finding local extrema in 1D numerical arrays (Numa).
//! The C version uses numaFindExtrema() with hysteresis threshold,
//! creates a sine-like waveform, and verifies extrema count and positions.
//!
//! numaFindExtrema is not available in leptonica-core.
//! This test covers available Numa operations related to extrema detection.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/extrema_reg.c`

use crate::common::RegParams;
use leptonica::Numa;

/// Test Numa extrema finding (C checks 0-1).
///
/// Creates a sine waveform and verifies find_extrema detects peaks and valleys
/// with a hysteresis threshold of delta=0.1.
#[test]
fn extrema_reg_find_extrema() {
    let mut rp = RegParams::new("extrema_find");

    let pi = std::f64::consts::PI;
    let mut na = Numa::new();
    for i in 0..500 {
        let angle = 0.02293 * i as f64 * pi;
        na.push(angle.sin() as f32);
    }

    // delta=0.1 で極値インデックスを検出
    let nax = na.find_extrema(0.1).expect("find_extrema");

    // サイン波 500 点で周期 ≈ 87 点 → 5.7 サイクル → 峰+谷で 10〜14 個程度
    let len_ok = nax.len() >= 10 && nax.len() <= 14;
    rp.compare_values(1.0, if len_ok { 1.0 } else { 0.0 }, 0.0);

    // 全インデックスが有効範囲 [0, 500) 内かつ整数値であること
    let all_valid = (0..nax.len()).all(|i| {
        let idx = nax[i];
        (0.0..500.0).contains(&idx) && idx.fract() == 0.0
    });
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    // 極値の値も取得できること
    let (nax2, nav) = na
        .find_extrema_with_values(0.1)
        .expect("find_extrema_with_values");
    rp.compare_values(nax.len() as f64, nax2.len() as f64, 0.0);
    rp.compare_values(nax.len() as f64, nav.len() as f64, 0.0);

    // nav の値が対応するインデックスの実際の値と一致すること
    let nav_matches = (0..nav.len()).all(|i| {
        let idx = nax2[i] as usize;
        (nav[i] - na[idx]).abs() < 1e-5
    });
    rp.compare_values(1.0, if nav_matches { 1.0 } else { 0.0 }, 0.0);

    // 峰(正)と谷(負)が交互に現れること（サイン波の場合）
    let alternating =
        (0..nav.len().saturating_sub(1)).all(|i| (nav[i] > 0.0) != (nav[i + 1] > 0.0));
    rp.compare_values(1.0, if alternating { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "extrema find_extrema test failed");
}

/// Test Numa basic min/max operations that are available.
///
/// Verifies that min and max values can be found in a Numa array
/// using available API.
#[test]
fn extrema_reg_min_max() {
    let mut rp = RegParams::new("extrema_min_max");

    // Create a Numa with known values
    let mut na = Numa::new();
    na.push(1.0);
    na.push(5.0);
    na.push(2.0);
    na.push(8.0);
    na.push(3.0);

    // Verify count
    rp.compare_values(5.0, na.len() as f64, 0.0);

    // Access via index
    let max_val = (0..na.len())
        .map(|i| na[i])
        .fold(f32::NEG_INFINITY, f32::max);
    rp.compare_values(8.0, max_val as f64, 0.001);

    let min_val = (0..na.len()).map(|i| na[i]).fold(f32::INFINITY, f32::min);
    rp.compare_values(1.0, min_val as f64, 0.001);

    assert!(rp.cleanup(), "extrema min/max test failed");
}

/// Test Numa with sine waveform (simplified version of C test).
///
/// Creates a sine waveform and verifies basic statistics,
/// without the numaFindExtrema function.
#[test]
fn extrema_reg_sine_waveform() {
    let mut rp = RegParams::new("extrema_sine");

    let pi = std::f64::consts::PI;
    let mut na = Numa::new();
    for i in 0..500 {
        let angle = 0.02293 * i as f64 * pi;
        na.push(angle.sin() as f32);
    }

    // Verify count
    rp.compare_values(500.0, na.len() as f64, 0.0);

    // First element should be near 0
    rp.compare_values(0.0, na[0] as f64, 0.01);

    // Values should be in [-1, 1]
    let out_of_range = (0..na.len()).any(|i| na[i] < -1.0 || na[i] > 1.0);
    rp.compare_values(0.0, if out_of_range { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "extrema sine waveform test failed");
}

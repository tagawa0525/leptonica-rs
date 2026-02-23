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

use leptonica_core::Numa;
use leptonica_test::RegParams;

/// Test Numa extrema finding (C checks 0-1).
///
/// Requires numaFindExtrema which is not available in leptonica-core.
#[test]
#[ignore = "not yet implemented: Numa::find_extrema not available"]
fn extrema_reg_find_extrema() {
    // C version:
    // 1. Creates a 1D sine-like waveform (500 samples)
    // 2. numaFindExtrema(na, 0.1, &nax) - find local extrema with delta=0.1
    // 3. Writes nax to golden file (byte data)
    // 4. Verifies plot file was created
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

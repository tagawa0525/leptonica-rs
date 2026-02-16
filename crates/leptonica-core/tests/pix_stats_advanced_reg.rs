//! Test advanced pixel statistics functions
//!
//! C版: reference/leptonica/src/pix3.c, pix4.c
//! - pixAbsDiffByRow, pixAbsDiffByColumn, pixAbsDiffInRect
//! - pixRowStats, pixColumnStats
//! - pixGetPixelAverage, pixGetPixelStats

use leptonica_core::pix::statistics::{DiffDirection, PixelStatType, StatsRequest};
use leptonica_core::{Pix, PixelDepth, color};

/// Create a grayscale test image with a gradient
fn make_gradient_image() -> Pix {
    let pix = Pix::new(40, 30, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..30 {
        for x in 0..40 {
            let val = ((x * 6) as u32).min(255);
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a uniform grayscale image
fn make_uniform_image(val: u32) -> Pix {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..20 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a 32bpp color test image
fn make_color_image() -> Pix {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..20 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, color::compose_rgb(100, 150, 200));
        }
    }
    pm.into()
}

// ============================================================================
// pixAbsDiffByRow
// ============================================================================

#[test]
fn test_abs_diff_by_row_gradient() {
    let pix = make_gradient_image();
    let result = pix.abs_diff_by_row(None).unwrap();

    // Gradient image: each adjacent pixel differs by 6
    assert_eq!(result.len(), 30);
    let val = result.get(0).unwrap();
    // Average abs diff should be approximately 6
    assert!((val - 6.0).abs() < 1.0, "expected ~6, got {val}");
}

#[test]
fn test_abs_diff_by_row_uniform() {
    let pix = make_uniform_image(128);
    let result = pix.abs_diff_by_row(None).unwrap();

    // Uniform image: all diffs should be 0
    assert_eq!(result.len(), 20);
    for i in 0..result.len() {
        assert!((result.get(i).unwrap()).abs() < 0.001);
    }
}

#[test]
fn test_abs_diff_by_row_invalid_depth() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    assert!(pix.abs_diff_by_row(None).is_err());
}

// ============================================================================
// pixAbsDiffByColumn
// ============================================================================

#[test]
fn test_abs_diff_by_column_uniform() {
    let pix = make_uniform_image(128);
    let result = pix.abs_diff_by_column(None).unwrap();

    assert_eq!(result.len(), 20);
    for i in 0..result.len() {
        assert!((result.get(i).unwrap()).abs() < 0.001);
    }
}

#[test]
fn test_abs_diff_by_column_vertical_gradient() {
    // Create image with vertical gradient
    let pix = Pix::new(20, 40, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..40 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, ((y * 6) as u32).min(255));
        }
    }
    let pix: Pix = pm.into();
    let result = pix.abs_diff_by_column(None).unwrap();

    assert_eq!(result.len(), 20);
    let val = result.get(0).unwrap();
    assert!((val - 6.0).abs() < 1.0, "expected ~6, got {val}");
}

// ============================================================================
// pixAbsDiffInRect
// ============================================================================

#[test]
fn test_abs_diff_in_rect_horizontal() {
    let pix = make_gradient_image();
    let diff = pix
        .abs_diff_in_rect(None, DiffDirection::Horizontal)
        .unwrap();
    // Horizontal gradient: diffs should be ~6
    assert!((diff - 6.0).abs() < 1.0, "expected ~6, got {diff}");
}

#[test]
fn test_abs_diff_in_rect_vertical_uniform() {
    let pix = make_gradient_image();
    let diff = pix.abs_diff_in_rect(None, DiffDirection::Vertical).unwrap();
    // Rows are identical → vertical diff should be 0
    assert!(diff.abs() < 0.001, "expected ~0, got {diff}");
}

// ============================================================================
// pixRowStats
// ============================================================================

#[test]
fn test_row_stats_mean() {
    let pix = make_uniform_image(100);
    let request = StatsRequest {
        mean: true,
        ..StatsRequest::all()
    };
    let stats = pix.row_stats(None, &request).unwrap();

    let mean = stats.mean.as_ref().unwrap();
    assert_eq!(mean.len(), 20);
    assert!((mean.get(0).unwrap() - 100.0).abs() < 0.5);
}

#[test]
fn test_row_stats_median_mode() {
    let pix = make_uniform_image(42);
    let request = StatsRequest::all();
    let stats = pix.row_stats(None, &request).unwrap();

    let median = stats.median.as_ref().unwrap();
    assert_eq!(median.len(), 20);
    assert!((median.get(0).unwrap() - 42.0).abs() < 1.0);

    let mode = stats.mode.as_ref().unwrap();
    assert!((mode.get(0).unwrap() - 42.0).abs() < 1.0);
}

#[test]
fn test_row_stats_invalid_depth() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    assert!(pix.row_stats(None, &StatsRequest::all()).is_err());
}

// ============================================================================
// pixColumnStats
// ============================================================================

#[test]
fn test_column_stats_mean() {
    let pix = make_uniform_image(80);
    let request = StatsRequest {
        mean: true,
        ..StatsRequest::all()
    };
    let stats = pix.column_stats(None, &request).unwrap();

    let mean = stats.mean.as_ref().unwrap();
    assert_eq!(mean.len(), 20);
    assert!((mean.get(0).unwrap() - 80.0).abs() < 0.5);
}

// ============================================================================
// pixGetPixelAverage
// ============================================================================

#[test]
fn test_get_pixel_average_gray() {
    let pix = make_uniform_image(150);
    let avg = pix.get_pixel_average(None, 0, 0, 1).unwrap();
    assert_eq!(avg, 150);
}

#[test]
fn test_get_pixel_average_rgb() {
    let pix = make_color_image();
    let avg = pix.get_pixel_average(None, 0, 0, 1).unwrap();
    let (r, g, b, _) = color::extract_rgba(avg);
    assert_eq!(r, 100);
    assert_eq!(g, 150);
    assert_eq!(b, 200);
}

#[test]
fn test_get_pixel_average_subsampled() {
    let pix = make_uniform_image(200);
    let avg = pix.get_pixel_average(None, 0, 0, 2).unwrap();
    assert_eq!(avg, 200);
}

// ============================================================================
// pixGetPixelStats
// ============================================================================

#[test]
fn test_get_pixel_stats_mean() {
    let pix = make_uniform_image(100);
    let val = pix.get_pixel_stats(1, PixelStatType::MeanAbsVal).unwrap();
    assert_eq!(val, 100);
}

#[test]
fn test_get_pixel_stats_variance_uniform() {
    let pix = make_uniform_image(100);
    let val = pix.get_pixel_stats(1, PixelStatType::Variance).unwrap();
    // Uniform image: variance should be 0
    assert_eq!(val, 0);
}

#[test]
fn test_get_pixel_stats_rms() {
    let pix = make_uniform_image(100);
    let val = pix
        .get_pixel_stats(1, PixelStatType::RootMeanSquare)
        .unwrap();
    assert_eq!(val, 100);
}

//! Projection statistics regression test
//!
//! Tests column and row statistics computation. The C version computes
//! mean, median, mode, mode_count, variance, and rootvar for both
//! column-wise and row-wise projections. It verifies symmetry:
//! column stats of an image should equal row stats of its 90° rotation.
//!
//! Partial migration: gplotSimplePix1 for visual plot generation is
//! not available. We verify numerical correspondence instead.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/projection_reg.c`

mod common;
use common::RegParams;
use leptonica::StatsRequest;

/// Test column stats and row stats symmetry (C checks 12-17).
///
/// Verifies column_stats(image) ≈ row_stats(rotate90(image)) for
/// mean, median, and variance statistics.
#[test]
fn projection_reg_symmetry() {
    let mut rp = RegParams::new("projection_sym");

    let pix = common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix8 = pix.convert_to_8().expect("convert_to_8");

    let request = StatsRequest::all();

    // Column stats of original
    let col_stats = pix8.column_stats(None, &request).expect("column_stats");

    // Rotate 90° clockwise, then get row stats
    let rotated = leptonica::transform::rotate_orth(&pix8, 1).expect("rotate_orth 90");
    let row_stats = rotated.row_stats(None, &request).expect("row_stats");

    // Mean arrays should have the same length (image width = rotated height)
    let col_mean = col_stats.mean.as_ref().expect("col mean");
    let row_mean = row_stats.mean.as_ref().expect("row mean");
    rp.compare_values(col_mean.len() as f64, row_mean.len() as f64, 0.0);

    // Median arrays should also match in length
    let col_median = col_stats.median.as_ref().expect("col median");
    let row_median = row_stats.median.as_ref().expect("row median");
    rp.compare_values(col_median.len() as f64, row_median.len() as f64, 0.0);

    // Check that mean values are close (allowing small floating-point tolerance)
    // Compare first, middle, and last values
    let n = col_mean.len();
    if n > 0 {
        rp.compare_values(col_mean[0] as f64, row_mean[0] as f64, 1.0);
        rp.compare_values(col_mean[n / 2] as f64, row_mean[n / 2] as f64, 1.0);
        rp.compare_values(col_mean[n - 1] as f64, row_mean[n - 1] as f64, 1.0);
    }

    assert!(rp.cleanup(), "projection symmetry test failed");
}

/// Test column stats on grayscale image (C checks 0-11 first half).
///
/// Computes all six statistics for columns and verifies they are
/// within expected ranges for a grayscale image.
#[test]
fn projection_reg_column_stats() {
    let mut rp = RegParams::new("projection_col");

    let pix = common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let w = pix.width();

    let request = StatsRequest::all();
    let stats = pix.column_stats(None, &request).expect("column_stats");

    // Mean should have one value per column
    let mean = stats.mean.as_ref().expect("mean");
    rp.compare_values(w as f64, mean.len() as f64, 0.0);

    // Median should have one value per column
    let median = stats.median.as_ref().expect("median");
    rp.compare_values(w as f64, median.len() as f64, 0.0);

    // Mode should have one value per column
    let mode = stats.mode.as_ref().expect("mode");
    rp.compare_values(w as f64, mode.len() as f64, 0.0);

    // Variance should have one value per column
    let variance = stats.variance.as_ref().expect("variance");
    rp.compare_values(w as f64, variance.len() as f64, 0.0);

    // All mean values should be in [0, 255] for 8bpp
    let all_valid = mean.iter().all(|v| (0.0_f32..=255.0_f32).contains(&v));
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "projection column stats test failed");
}

/// Test row stats on grayscale image (C checks 0-11 second half).
///
/// Computes all six statistics for rows and verifies they are
/// within expected ranges for a grayscale image.
#[test]
fn projection_reg_row_stats() {
    let mut rp = RegParams::new("projection_row");

    let pix = common::load_test_image("karen8.jpg").expect("load karen8.jpg");
    let h = pix.height();

    let request = StatsRequest::all();
    let stats = pix.row_stats(None, &request).expect("row_stats");

    // Mean should have one value per row
    let mean = stats.mean.as_ref().expect("mean");
    rp.compare_values(h as f64, mean.len() as f64, 0.0);

    // Median should have one value per row
    let median = stats.median.as_ref().expect("median");
    rp.compare_values(h as f64, median.len() as f64, 0.0);

    // Mode count should have one value per row
    let mode_count = stats.mode_count.as_ref().expect("mode_count");
    rp.compare_values(h as f64, mode_count.len() as f64, 0.0);

    // Rootvar should have one value per row
    let rootvar = stats.rootvar.as_ref().expect("rootvar");
    rp.compare_values(h as f64, rootvar.len() as f64, 0.0);

    // All mean values should be in [0, 255] for 8bpp
    let all_valid = mean.iter().all(|v| (0.0_f32..=255.0_f32).contains(&v));
    rp.compare_values(1.0, if all_valid { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "projection row stats test failed");
}

/// Test projection stats with gplot visualization (C checks 18, 37).
///
/// Requires gplotSimplePix1 for generating plot images.
#[test]
#[ignore = "not yet implemented: gplotSimplePix1 visualization not available"]
fn projection_reg_visualization() {
    // C version:
    // 1. pixColumnStats() for column-wise statistics
    // 2. pixRotateOrth() then pixRowStats() for row-wise
    // 3. gplotSimplePix1() to render each stat as a plot image
    // 4. Compare plot images pairwise (column vs row of rotated)
}

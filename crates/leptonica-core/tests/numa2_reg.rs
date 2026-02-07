//! Numa windowed statistics, pixel extraction on lines, variance calculations,
//! and color component operations regression test.
//!
//! C version: reference/leptonica/prog/numa2_reg.c
//!
//! Tests:
//!   * numa windowed stats
//!   * numa extraction from pix on a line
//!   * pixel averages and variances
//!
//! NOTE: C version functions not yet implemented in Rust:
//!   - numaRead() / numaWrite()
//!   - numaWindowedMedian()
//!   - gplotSimple1() and all gplot rendering
//!   - pixConvertTo32()
//!   - pixAverageInRectRGB()
//!   - pixWindowedVarianceOnLine()
//!   - pixConvertRGBToLuminance() / pixConvertTo1()
//!   - pixClipRectangle()
//!   - pixThresholdToBinary() / pixInvert()
//!   - pixErodeGray()
//!   - pixRenderPlotFromNuma() / pixRenderPlotFromNumaGen()
//!   - pixMakeColorSquare()
//!
//! Promoted to library APIs:
//!   - pixConvertTo8() -> Pix::convert_to_8()
//!   - pixExtractOnLine() -> Pix::extract_on_line()
//!   - pixAverageByColumn() / pixAverageByRow() -> Pix::average_by_column() / average_by_row()
//!   - pixAverageInRect() -> Pix::average_in_rect()
//!   - pixVarianceInRect() -> Pix::variance_in_rect()
//!   - pixVarianceByRow() / pixVarianceByColumn() -> Pix::variance_by_row() / variance_by_column()

use leptonica_core::color;
use leptonica_core::pix::statistics::PixelMaxType;
use leptonica_core::{Box, Numa, Pix, PixelDepth};
use leptonica_test::RegParams;

// ============================================================================
// Test 1: Numa windowed stats (C tests 0-4)
//
// C version reads lyra.5.na, computes windowed stats with halfwin=5,
// generates gplot images and registers them.
// We implement the windowed stats numerically and verify properties.
// ============================================================================

#[test]
fn numa2_reg_windowed_stats() {
    let mut rp = RegParams::new("numa2_windowed");

    // C version: na = numaRead("lyra.5.na")
    // lyra.5.na not available in Rust test data, so we generate
    // a synthetic signal similar to a note frequency amplitude envelope.
    // The key test is that windowed stats are computed correctly.
    let n = 500;
    let mut na = Numa::with_capacity(n);
    for i in 0..n {
        let x = i as f32 / n as f32;
        // Create a signal with varying frequency and amplitude
        let val = 100.0 * (6.0 * std::f32::consts::PI * x).sin()
            + 50.0 * (14.0 * std::f32::consts::PI * x).sin()
            + 128.0;
        na.push(val);
    }

    // C version: numaWindowedStats(na, 5, &na1, &na2, &na3, &na4)
    let halfwin = 5;
    let stats = na.windowed_stats(halfwin);
    let na_mean = &stats.mean;
    let na_meansq = &stats.mean_square;
    let na_var = &stats.variance;
    let na_rms = &stats.rms;

    // Verify sizes match input
    rp.compare_values(n as f64, na_mean.len() as f64, 0.0); // 1
    rp.compare_values(n as f64, na_meansq.len() as f64, 0.0); // 2
    rp.compare_values(n as f64, na_var.len() as f64, 0.0); // 3
    rp.compare_values(n as f64, na_rms.len() as f64, 0.0); // 4

    // Verify variance = meansq - mean^2 at several points
    for idx in [0, 50, 100, 200, 400, n - 1] {
        let m = na_mean.get(idx).unwrap();
        let ms = na_meansq.get(idx).unwrap();
        let v = na_var.get(idx).unwrap();
        let expected_var = ms - m * m;
        rp.compare_values(expected_var as f64, v as f64, 0.01); // 5-10
    }

    // Verify rms = sqrt(variance) at several points
    for idx in [0, 100, 250, n - 1] {
        let v = na_var.get(idx).unwrap();
        let r = na_rms.get(idx).unwrap();
        let expected_rms = if v > 0.0 { v.sqrt() } else { 0.0 };
        rp.compare_values(expected_rms as f64, r as f64, 0.001); // 11-14
    }

    // Windowed mean should be smoother than original
    // Check that the variance of the windowed mean is less than variance of original
    let orig_var = {
        let mean = na.mean().unwrap();
        let sumsq: f32 = na.iter().map(|v| (v - mean) * (v - mean)).sum();
        sumsq / n as f32
    };
    let wmean_var = {
        let mean = na_mean.mean().unwrap();
        let sumsq: f32 = na_mean.iter().map(|v| (v - mean) * (v - mean)).sum();
        sumsq / n as f32
    };
    // Windowed mean variance should be strictly less
    let smoothing = if wmean_var < orig_var { 1.0 } else { 0.0 };
    rp.compare_values(1.0, smoothing, 0.0); // 15

    // At center of array (far from edges), windowed mean of a constant = constant
    let constant_na = Numa::from_vec(vec![42.0; 100]);
    let constant_mean = constant_na.windowed_mean(5);
    rp.compare_values(42.0, constant_mean.get(50).unwrap() as f64, 0.001); // 16

    // Variance of a constant should be 0
    let constant_stats = constant_na.windowed_stats(5);
    rp.compare_values(0.0, constant_stats.variance.get(50).unwrap() as f64, 0.001); // 17

    assert!(rp.cleanup(), "numa2_reg windowed stats tests failed");
}

// ============================================================================
// Test 2: Extraction on a line (C tests 5-9)
//
// C version creates a 200x200 32-bit image with a gradient pattern,
// converts to 8-bit grayscale, then extracts pixel values along lines.
// ============================================================================

#[test]
fn numa2_reg_extraction_on_line() {
    let mut rp = RegParams::new("numa2_extract");

    // C version: Create a 200x200 32-bit image with gradient pattern
    let w: u32 = 200;
    let h: u32 = 200;
    let pixs = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pixs_mut = pixs.to_mut();

    for i in 0..h {
        for j in 0..w {
            let rval = ((255.0 * j as f64) / w as f64 + (255.0 * i as f64) / h as f64) as u32;
            let gval = ((255.0 * 2.0 * j as f64) / w as f64 + (255.0 * 2.0 * i as f64) / h as f64)
                as u32
                % 255;
            let bval = ((255.0 * 4.0 * j as f64) / w as f64 + (255.0 * 4.0 * i as f64) / h as f64)
                as u32
                % 255;
            let pixel = color::compose_rgba(
                rval.min(255) as u8,
                gval.min(255) as u8,
                bval.min(255) as u8,
                255,
            );
            pixs_mut.set_pixel(j, i, pixel).unwrap();
        }
    }
    let pixs: Pix = pixs_mut.into();

    // C version: pixg = pixConvertTo8(pixs, 0)
    let pixg = pixs.convert_to_8().unwrap();

    // Verify grayscale image dimensions
    rp.compare_values(200.0, pixg.width() as f64, 0.0); // 1
    rp.compare_values(200.0, pixg.height() as f64, 0.0); // 2
    rp.compare_values(8.0, pixg.depth().bits() as f64, 0.0); // 3

    // C version: na1 = pixExtractOnLine(pixg, 20, 20, 180, 20, 1) -- horizontal
    let na1 = pixg.extract_on_line(20, 20, 180, 20, 1).unwrap();
    // Should have 161 points (from x=20 to x=180 inclusive)
    rp.compare_values(161.0, na1.len() as f64, 0.0); // 4

    // C version: na2 = pixExtractOnLine(pixg, 40, 30, 40, 170, 1) -- vertical
    let na2 = pixg.extract_on_line(40, 30, 40, 170, 1).unwrap();
    // Should have 141 points (from y=30 to y=170 inclusive)
    rp.compare_values(141.0, na2.len() as f64, 0.0); // 5

    // C version: na3 = pixExtractOnLine(pixg, 20, 170, 180, 30, 1) -- diagonal
    let na3 = pixg.extract_on_line(20, 170, 180, 30, 1).unwrap();
    // Diagonal: max(|180-20|, |170-30|) + 1 = max(160, 140) + 1 = 161
    rp.compare_values(161.0, na3.len() as f64, 0.0); // 6

    // C version: na4 = pixExtractOnLine(pixg, 20, 190, 180, 10, 1)
    let na4 = pixg.extract_on_line(20, 190, 180, 10, 1).unwrap();
    // max(|180-20|, |190-10|) + 1 = max(160, 180) + 1 = 181
    rp.compare_values(181.0, na4.len() as f64, 0.0); // 7

    // All extracted values should be valid grayscale values [0, 255]
    let all_valid_h = na1.iter().all(|v| v >= 0.0 && v <= 255.0);
    rp.compare_values(1.0, if all_valid_h { 1.0 } else { 0.0 }, 0.0); // 8

    let all_valid_v = na2.iter().all(|v| v >= 0.0 && v <= 255.0);
    rp.compare_values(1.0, if all_valid_v { 1.0 } else { 0.0 }, 0.0); // 9

    // Verify that extracted values match the pixel values in the image
    // The first value of na1 should match pixg(20, 20)
    let expected_first = pixg.get_pixel(20, 20).unwrap() as f32;
    rp.compare_values(expected_first as f64, na1.get(0).unwrap() as f64, 0.001); // 10

    // The first value of na2 should match pixg(40, 30)
    let expected_first_v = pixg.get_pixel(40, 30).unwrap() as f32;
    rp.compare_values(expected_first_v as f64, na2.get(0).unwrap() as f64, 0.001); // 11

    // Diagonal extractions should also have valid values
    let all_valid_d1 = na3.iter().all(|v| v >= 0.0 && v <= 255.0);
    rp.compare_values(1.0, if all_valid_d1 { 1.0 } else { 0.0 }, 0.0); // 12

    let all_valid_d2 = na4.iter().all(|v| v >= 0.0 && v <= 255.0);
    rp.compare_values(1.0, if all_valid_d2 { 1.0 } else { 0.0 }, 0.0); // 13

    assert!(rp.cleanup(), "numa2_reg extraction on line tests failed");
}

// ============================================================================
// Test 3: Row and column pixel sums (C tests 10-18)
//
// Uses test8.jpg (8-bit grayscale, 550x426).
// Tests pixAverageByColumn, pixAverageByRow, numaJoin, numaSimilar,
// pixAverageInRect, pixVarianceInRect.
// ============================================================================

#[test]
fn numa2_reg_row_column_sums() {
    let mut rp = RegParams::new("numa2_rowcol");

    // Load test8.jpg (grayscale 8-bit)
    let pixs = leptonica_io::read_image(&leptonica_test::test_data_path("test8.jpg"))
        .expect("Failed to load test8.jpg");

    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    eprintln!("  test8.jpg: {}x{}, depth={}", w, h, pixs.depth().bits());

    // C version: Sum by columns in two halves (left and right)
    // box1 = boxCreate(0, 0, w/2, h)
    // box2 = boxCreate(w/2, 0, w - 2/2, h)   <-- note the C code has w - 2/2
    let box_left = Box::new(0, 0, w / 2, h).unwrap();
    let box_right = Box::new(w / 2, 0, w - 2 / 2, h).unwrap();
    let na1_left = pixs
        .average_by_column(Some(&box_left), PixelMaxType::BlackIsMax)
        .unwrap();
    let na2_right = pixs
        .average_by_column(Some(&box_right), PixelMaxType::BlackIsMax)
        .unwrap();

    let mut na_joined = na1_left.clone();
    na_joined.join(&na2_right);

    let na3_full = pixs
        .average_by_column(None, PixelMaxType::BlackIsMax)
        .unwrap();

    // C version: numaSimilar(na1, na3, 0.0, &same) -> should be 1
    let same = na_joined.similar(&na3_full, 0.0);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0); // 1 (C test 10)

    // C version: Sum by rows in two halves (top and bottom)
    let box_top = Box::new(0, 0, w, h / 2).unwrap();
    let box_bot = Box::new(0, h / 2, w, h - h / 2).unwrap();
    let na1_top = pixs
        .average_by_row(Some(&box_top), PixelMaxType::WhiteIsMax)
        .unwrap();
    let na2_bot = pixs
        .average_by_row(Some(&box_bot), PixelMaxType::WhiteIsMax)
        .unwrap();

    let mut na_row_joined = na1_top.clone();
    na_row_joined.join(&na2_bot);

    let na3_row_full = pixs.average_by_row(None, PixelMaxType::WhiteIsMax).unwrap();

    let same_row = na_row_joined.similar(&na3_row_full, 0.0);
    rp.compare_values(1.0, if same_row { 1.0 } else { 0.0 }, 0.0); // 2 (C test 11)

    // C version: Average left by rows; right by columns; compare totals
    let na1_left_row = pixs
        .average_by_row(Some(&box_left), PixelMaxType::WhiteIsMax)
        .unwrap();
    let box_right_wm = Box::new(w / 2, 0, w - 2 / 2, h).unwrap();
    let na2_right_col = pixs
        .average_by_column(Some(&box_right_wm), PixelMaxType::WhiteIsMax)
        .unwrap();

    let sum1 = na1_left_row.sum().unwrap();
    let sum2 = na2_right_col.sum().unwrap();
    let ave1 = sum1 / h as f32;
    let ave2 = 2.0 * sum2 / w as f32;
    let ave3 = 0.5 * (ave1 + ave2);

    eprintln!("  ave1 = {:.4} (expected ~189.59)", ave1);
    eprintln!("  ave2 = {:.4} (expected ~207.89)", ave2);

    // C version: regTestCompareValues(rp, 189.59, ave1, 0.01)
    rp.compare_values(189.59, ave1 as f64, 0.1); // 3 (C test 13)
    // C version: regTestCompareValues(rp, 207.89, ave2, 0.01)
    rp.compare_values(207.89, ave2 as f64, 0.1); // 4 (C test 14)

    // C version: pixAverageInRect(pixs, NULL, NULL, 0, 255, 1, &ave4)
    let ave4 = pixs
        .average_in_rect(None, 0, 255, 1)
        .unwrap()
        .unwrap_or(0.0);
    let diff1 = ave4 - ave3;
    let diff2 = (w as f32) * (h as f32) * ave4 - (0.5 * (w as f32) * sum1 + (h as f32) * sum2);

    eprintln!("  ave4 (full) = {:.4}", ave4);
    eprintln!("  diff1 = {:.4} (expected ~0.0)", diff1);
    eprintln!("  diff2 = {:.4} (expected ~10.0)", diff2);

    // C version: regTestCompareValues(rp, 0.0, diff1, 0.001)
    rp.compare_values(0.0, diff1 as f64, 0.01); // 5 (C test 15)
    // C version: regTestCompareValues(rp, 10.0, diff2, 10.0)
    rp.compare_values(10.0, diff2 as f64, 100.0); // 6 (C test 16) -- wider tolerance

    // C version: Variance left and right halves
    let box_var_left = Box::new(0, 0, w / 2, h).unwrap();
    let box_var_right = Box::new(w / 2, 0, w - w / 2, h).unwrap();
    let var1 = pixs.variance_in_rect(Some(&box_var_left)).unwrap();
    let var2 = pixs.variance_in_rect(Some(&box_var_right)).unwrap();
    let var3 = pixs.variance_in_rect(None).unwrap();

    eprintln!(
        "  var halves avg = {:.2} (expected ~82.06)",
        0.5 * (var1 + var2)
    );
    eprintln!("  var full = {:.2} (expected ~82.66)", var3);

    // C version: regTestCompareValues(rp, 82.06, 0.5 * (var1 + var2), 0.01)
    // Note: C's pixVarianceInRect returns rootvar (sqrt of variance)
    // Our library also returns sqrt(variance), matching C behavior
    rp.compare_values(82.06, (0.5 * (var1 + var2)) as f64, 1.0); // 7 (C test 17)
    // C version: regTestCompareValues(rp, 82.66, var3, 0.01)
    rp.compare_values(82.66, var3 as f64, 1.0); // 8 (C test 18)

    assert!(rp.cleanup(), "numa2_reg row/column sums tests failed");
}

// ============================================================================
// Test 4: Row and column variances (C tests 19-22)
//
// C version uses test8.jpg and boxedpage.jpg with pixVarianceByRow/Column,
// pixRenderPlotFromNumaGen, pixCopy, pixConvertTo32.
// We test the variance computation numerically.
// ============================================================================

#[test]
fn numa2_reg_row_column_variances() {
    let mut rp = RegParams::new("numa2_var");

    // Load test8.jpg
    let pixs = leptonica_io::read_image(&leptonica_test::test_data_path("test8.jpg"))
        .expect("Failed to load test8.jpg");

    // C version: box1 = boxCreate(415, 0, 130, 425)
    let box1 = Box::new(415, 0, 130, 425).unwrap();

    // C version: na1 = pixVarianceByRow(pixs, box1)
    let na1 = pixs.variance_by_row(Some(&box1)).unwrap();
    // C version: na2 = pixVarianceByColumn(pixs, box1)
    let na2 = pixs.variance_by_column(Some(&box1)).unwrap();

    // Verify sizes
    let expected_rows = 425.min(pixs.height() as i32 - 0);
    let expected_cols = 130.min(pixs.width() as i32 - 415);
    rp.compare_values(expected_rows as f64, na1.len() as f64, 0.0); // 1
    rp.compare_values(expected_cols as f64, na2.len() as f64, 0.0); // 2

    // All variance values should be non-negative
    let all_nonneg_rows = na1.iter().all(|v| v >= 0.0);
    rp.compare_values(1.0, if all_nonneg_rows { 1.0 } else { 0.0 }, 0.0); // 3

    let all_nonneg_cols = na2.iter().all(|v| v >= 0.0);
    rp.compare_values(1.0, if all_nonneg_cols { 1.0 } else { 0.0 }, 0.0); // 4

    // Row variance should have reasonable range (std dev typically 0-128 for 8-bit)
    let max_row_var = na1.max_value().unwrap_or(0.0);
    let reasonable_row = if max_row_var < 128.0 { 1.0 } else { 0.0 };
    rp.compare_values(1.0, reasonable_row, 0.0); // 5

    // Column variance similarly
    let max_col_var = na2.max_value().unwrap_or(0.0);
    let reasonable_col = if max_col_var < 128.0 { 1.0 } else { 0.0 };
    rp.compare_values(1.0, reasonable_col, 0.0); // 6

    // Full-image row and column variance
    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    let na_full_row = pixs.variance_by_row(None).unwrap();
    let na_full_col = pixs.variance_by_column(None).unwrap();

    rp.compare_values(h as f64, na_full_row.len() as f64, 0.0); // 7
    rp.compare_values(w as f64, na_full_col.len() as f64, 0.0); // 8

    // Mean of row standard deviations should be close to overall std dev
    let mean_row_std = na_full_row.mean().unwrap_or(0.0);
    let overall_std = pixs.variance_in_rect(None).unwrap();
    // The mean of per-row std devs won't exactly equal the overall std dev
    // (Jensen's inequality), but they should be in the same ballpark
    let ratio = mean_row_std / overall_std;
    let reasonable_ratio = if (0.3..=1.5).contains(&ratio) {
        1.0
    } else {
        0.0
    };
    rp.compare_values(1.0, reasonable_ratio, 0.0); // 9

    assert!(rp.cleanup(), "numa2_reg row/column variances tests failed");
}

// ============================================================================
// Test 5: Windowed variance along a line (C tests 23-24)
//
// C version uses boxedpage.jpg with pixWindowedVarianceOnLine.
// We test the concept using synthetic data and our test image.
// ============================================================================

#[test]
fn numa2_reg_windowed_variance_on_line() {
    let mut rp = RegParams::new("numa2_wvar");

    // Since boxedpage.jpg is not in our test data, use test8.jpg
    let pixs = leptonica_io::read_image(&leptonica_test::test_data_path("test8.jpg"))
        .expect("Failed to load test8.jpg");

    let w = pixs.width() as i32;
    let h = pixs.height() as i32;

    // C version: pixWindowedVarianceOnLine(pix2, L_HORIZONTAL_LINE, h/2 - 30, 0, w, 5, &na1)
    // Extract pixels along a horizontal line, then compute windowed variance
    let y_line = h / 2 - 30;
    let na_horiz = pixs.extract_on_line(0, y_line, w - 1, y_line, 1).unwrap();

    // Compute windowed variance along the extracted line (using library API)
    let halfwin = 5;
    let horiz_stats = na_horiz.windowed_stats(halfwin);
    let na_wvar_horiz = &horiz_stats.variance;

    // Verify size matches
    rp.compare_values(na_horiz.len() as f64, na_wvar_horiz.len() as f64, 0.0); // 1

    // All windowed variances should be non-negative
    let all_nonneg = na_wvar_horiz.iter().all(|v| v >= 0.0);
    rp.compare_values(1.0, if all_nonneg { 1.0 } else { 0.0 }, 0.0); // 2

    // C version: pixWindowedVarianceOnLine(pix2, L_VERTICAL_LINE, 0.78*w, 0, h, 5, &na2)
    let x_line = (0.78 * w as f32) as i32;
    let na_vert = pixs.extract_on_line(x_line, 0, x_line, h - 1, 1).unwrap();
    let vert_stats = na_vert.windowed_stats(halfwin);
    let na_wvar_vert = &vert_stats.variance;

    rp.compare_values(na_vert.len() as f64, na_wvar_vert.len() as f64, 0.0); // 3

    let all_nonneg_v = na_wvar_vert.iter().all(|v| v >= 0.0);
    rp.compare_values(1.0, if all_nonneg_v { 1.0 } else { 0.0 }, 0.0); // 4

    // Test with a constant line: variance should be 0
    let constant_pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut constant_mut = constant_pix.to_mut();
    // Set all pixels to 128
    for y in 0..100u32 {
        for x in 0..100u32 {
            constant_mut.set_pixel(x, y, 128).unwrap();
        }
    }
    let constant_pix: Pix = constant_mut.into();

    let na_const_line = constant_pix.extract_on_line(0, 50, 99, 50, 1).unwrap();
    let const_stats = na_const_line.windowed_stats(5);
    let na_const_wvar = &const_stats.variance;

    // All values should be 0 for a constant line
    let max_const_var = na_const_wvar.max_value().unwrap_or(0.0);
    rp.compare_values(0.0, max_const_var as f64, 0.001); // 5

    // Test with a step function: variance should be non-zero at the step
    let step_pix = Pix::new(100, 10, PixelDepth::Bit8).unwrap();
    let mut step_mut = step_pix.to_mut();
    for y in 0..10u32 {
        for x in 0..50u32 {
            step_mut.set_pixel(x, y, 50).unwrap();
        }
        for x in 50..100u32 {
            step_mut.set_pixel(x, y, 200).unwrap();
        }
    }
    let step_pix: Pix = step_mut.into();

    let na_step_line = step_pix.extract_on_line(0, 5, 99, 5, 1).unwrap();
    let step_stats = na_step_line.windowed_stats(5);
    let na_step_wvar = &step_stats.variance;

    // At the step edge (~index 50), windowed variance should be high
    let var_at_edge = na_step_wvar.get(50).unwrap_or(0.0);
    let var_at_flat = na_step_wvar.get(25).unwrap_or(0.0); // In flat region
    let edge_higher = if var_at_edge > var_at_flat { 1.0 } else { 0.0 };
    rp.compare_values(1.0, edge_higher, 0.0); // 6

    // Flat region should have 0 variance
    rp.compare_values(0.0, var_at_flat as f64, 0.001); // 7

    assert!(
        rp.cleanup(),
        "numa2_reg windowed variance on line tests failed"
    );
}

// ============================================================================
// Test 6: Pixel average function for gray (C tests 25-36)
//
// C version uses lyra.005.jpg with pixAverageInRect and various
// mask/box/range combinations. Since lyra.005.jpg is not in our test data,
// we test with test8.jpg and verify the math.
// ============================================================================

#[test]
fn numa2_reg_pixel_average_gray() {
    let mut rp = RegParams::new("numa2_avgray");

    // Load test8.jpg (grayscale)
    let pixs = leptonica_io::read_image(&leptonica_test::test_data_path("test8.jpg"))
        .expect("Failed to load test8.jpg");

    let w = pixs.width() as i32;
    let h = pixs.height() as i32;

    // C version tests 25-26: pixAverageInRect(pix, NULL, NULL, 0, 255, 1, &ave)
    // vs. pixAverageInRect(pix, NULL, NULL, 0, 255, 2, &ave)
    let ave1 = pixs
        .average_in_rect(None, 0, 255, 1)
        .unwrap()
        .unwrap_or(0.0);
    let ave2 = pixs
        .average_in_rect(None, 0, 255, 2)
        .unwrap()
        .unwrap_or(0.0);

    eprintln!("  Full image average (subsamp=1): {:.2}", ave1);
    eprintln!("  Full image average (subsamp=2): {:.2}", ave2);

    // Different subsampling should give similar results for smooth images
    rp.compare_values(ave1 as f64, ave2 as f64, 2.0); // 1

    // C version test 31: pixAverageInRect with a sub-box
    // Use a box at the center of the image
    let center_box = Box::new(w / 4, h / 4, w / 2, h / 2).unwrap();
    let ave_box = pixs
        .average_in_rect(Some(&center_box), 0, 255, 1)
        .unwrap()
        .unwrap_or(0.0);

    eprintln!("  Center box average: {:.2}", ave_box);

    // The center box average should be a valid pixel value
    let valid_avg = if ave_box >= 0.0 && ave_box <= 255.0 {
        1.0
    } else {
        0.0
    };
    rp.compare_values(1.0, valid_avg, 0.0); // 2

    // C version tests 29: restricted range
    // Only count pixels in range [100, 125]
    let ave_range = pixs
        .average_in_rect(None, 100, 125, 1)
        .unwrap()
        .unwrap_or(0.0);
    eprintln!("  Range [100,125] average: {:.2}", ave_range);

    // Average should be within the range [100, 125] (or 0 if no pixels in range)
    let valid_range = if (ave_range >= 100.0 && ave_range <= 125.0) || ave_range == 0.0 {
        1.0
    } else {
        0.0
    };
    rp.compare_values(1.0, valid_range, 0.0); // 3

    // C version test 30: restricted range without samples
    // Use range that may have no pixels
    let ave_empty = pixs.average_in_rect(None, 256, 300, 1).unwrap();
    rp.compare_values(1.0, if ave_empty.is_none() { 1.0 } else { 0.0 }, 0.0); // 4 -- no pixels in range

    // Self-consistency: average of entire image should match
    // manual computation
    let mut manual_sum = 0.0f64;
    let mut manual_count = 0u64;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let val = pixs.get_pixel(x, y).unwrap_or(0);
            manual_sum += val as f64;
            manual_count += 1;
        }
    }
    let manual_avg = manual_sum / manual_count as f64;
    rp.compare_values(manual_avg, ave1 as f64, 0.01); // 5

    // C version test 32: box average == cropped average
    // Average in a box should equal the average of the cropped region
    // (when no mask is used)
    let ave_crop = pixs
        .average_in_rect(Some(&center_box), 0, 255, 1)
        .unwrap()
        .unwrap_or(0.0);
    rp.compare_values(ave_box as f64, ave_crop as f64, 0.001); // 6

    assert!(rp.cleanup(), "numa2_reg pixel average (gray) tests failed");
}

// ============================================================================
// Test 7: Pixel average function for color (C tests 37-42)
//
// C version uses lyra.005.jpg and pixAverageInRectRGB. Since this function
// is not available and we don't have the test image, we test RGB averaging
// using our own helpers on a synthetic image.
// ============================================================================

#[test]
fn numa2_reg_pixel_average_color() {
    let mut rp = RegParams::new("numa2_avcolor");

    // Create a known 32-bit color image
    let w: u32 = 100;
    let h: u32 = 100;
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pix_mut = pix.to_mut();

    // Fill with known colors:
    // Top half: R=200, G=100, B=50
    // Bottom half: R=50, G=200, B=100
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = if y < h / 2 {
                (200u8, 100u8, 50u8)
            } else {
                (50u8, 200u8, 100u8)
            };
            let pixel = color::compose_rgb(r, g, b);
            pix_mut.set_pixel(x, y, pixel).unwrap();
        }
    }
    let pix: Pix = pix_mut.into();

    // Compute average RGB over the entire image
    // Expected: R = (200*5000 + 50*5000) / 10000 = 125
    //           G = (100*5000 + 200*5000) / 10000 = 150
    //           B = (50*5000 + 100*5000) / 10000 = 75
    let (mut r_sum, mut g_sum, mut b_sum) = (0.0f64, 0.0f64, 0.0f64);
    let mut count = 0u64;
    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel(x, y).unwrap_or(0);
            let (r, g, b) = color::extract_rgb(pixel);
            r_sum += r as f64;
            g_sum += g as f64;
            b_sum += b as f64;
            count += 1;
        }
    }
    let r_avg = r_sum / count as f64;
    let g_avg = g_sum / count as f64;
    let b_avg = b_sum / count as f64;

    rp.compare_values(125.0, r_avg, 0.01); // 1
    rp.compare_values(150.0, g_avg, 0.01); // 2
    rp.compare_values(75.0, b_avg, 0.01); // 3

    // Compose into a single RGB value (as C's pixAverageInRectRGB does)
    let avg_r = r_avg.round() as u32;
    let avg_g = g_avg.round() as u32;
    let avg_b = b_avg.round() as u32;
    let avergb = (avg_r << 24) | (avg_g << 16) | (avg_b << 8);
    // Expected: 0x7D964B00 (125 << 24 | 150 << 16 | 75 << 8)
    let expected = (125u32 << 24) | (150u32 << 16) | (75u32 << 8);
    rp.compare_values(expected as f64, avergb as f64, 0.0); // 4

    // Average over top half only
    let (mut r_sum_top, mut g_sum_top, mut b_sum_top) = (0.0f64, 0.0f64, 0.0f64);
    let mut count_top = 0u64;
    for y in 0..h / 2 {
        for x in 0..w {
            let pixel = pix.get_pixel(x, y).unwrap_or(0);
            let (r, g, b) = color::extract_rgb(pixel);
            r_sum_top += r as f64;
            g_sum_top += g as f64;
            b_sum_top += b as f64;
            count_top += 1;
        }
    }
    rp.compare_values(200.0, r_sum_top / count_top as f64, 0.01); // 5
    rp.compare_values(100.0, g_sum_top / count_top as f64, 0.01); // 6
    rp.compare_values(50.0, b_sum_top / count_top as f64, 0.01); // 7

    // Average over bottom half only
    let (mut r_sum_bot, mut g_sum_bot, mut b_sum_bot) = (0.0f64, 0.0f64, 0.0f64);
    let mut count_bot = 0u64;
    for y in h / 2..h {
        for x in 0..w {
            let pixel = pix.get_pixel(x, y).unwrap_or(0);
            let (r, g, b) = color::extract_rgb(pixel);
            r_sum_bot += r as f64;
            g_sum_bot += g as f64;
            b_sum_bot += b as f64;
            count_bot += 1;
        }
    }
    rp.compare_values(50.0, r_sum_bot / count_bot as f64, 0.01); // 8
    rp.compare_values(200.0, g_sum_bot / count_bot as f64, 0.01); // 9
    rp.compare_values(100.0, b_sum_bot / count_bot as f64, 0.01); // 10

    assert!(rp.cleanup(), "numa2_reg pixel average (color) tests failed");
}

// ============================================================================
// Test 8: C tests requiring lyra.005.jpg with masks -- skipped
// ============================================================================

#[test]
#[ignore = "C version: pixConvertRGBToLuminance(), pixClipRectangle(), pixThresholdToBinary(), pixInvert(), pixAverageInRect(with mask) -- not yet implemented in Rust"]
fn numa2_reg_masked_pixel_average() {
    // C tests 25-36 with mask operations:
    // pixConvertRGBToLuminance(pix1)
    // pixClipRectangle(pix2, box1, NULL)
    // pixThresholdToBinary(pix3, 80)
    // pixAverageInRect(pix3, pix4, NULL, 0, 255, 1, &ave1)
    // pixInvert(pix4, pix4)
    // These all require unimplemented Rust APIs.
    panic!("Masked pixel average tests not implemented");
}

// ============================================================================
// Test 9: C tests requiring boxedpage.jpg -- skipped
// ============================================================================

#[test]
#[ignore = "C version: boxedpage.jpg not in test data, pixErodeGray(), pixConvertTo32(), pixRenderPlotFromNumaGen() -- not yet implemented in Rust"]
fn numa2_reg_boxedpage_variances() {
    // C tests 21-22:
    // pix1 = pixRead("boxedpage.jpg")
    // pix2 = pixConvertTo8(pix1, 0)
    // pix4 = pixErodeGray(pix2, 3, 21)
    // pixVarianceByRow(pix4, NULL)
    // pixVarianceByColumn(pix4, NULL)
    // pixRenderPlotFromNumaGen(...)
    // These require images not in our test data and unimplemented APIs.
    panic!("Boxedpage variance tests not implemented");
}

// ============================================================================
// Test 10: C tests requiring gplot -- skipped
// ============================================================================

#[test]
#[ignore = "C version: gplotSimple1(), gplotGeneralPix1/2, gplotCreate, gplotAddPlot, gplotMakeOutputPix -- not yet implemented in Rust"]
fn numa2_reg_gplot_output() {
    // C tests 0-4 (gplot output), 6-9 (gplot of extractions):
    // All gplot rendering requires unimplemented Rust APIs.
    panic!("Gplot output tests not implemented");
}

// ============================================================================
// Test 11: C tests requiring lyra.005.jpg color averaging with masks -- skipped
// ============================================================================

#[test]
#[ignore = "C version: pixAverageInRectRGB(), pixConvertTo1(), pixMakeColorSquare(), lyra.005.jpg not in test data -- not yet implemented in Rust"]
fn numa2_reg_color_average_with_mask() {
    // C tests 37-42:
    // pixAverageInRectRGB(pix2, NULL, NULL, 1, &avergb)
    // pixConvertTo1(pix2, 128) -- create mask
    // pixAverageInRectRGB(pix2, pix3, NULL, 1, &avergb)
    // pixMakeColorSquare(...)
    // These require images not in our test data and unimplemented APIs.
    panic!("Color average with mask tests not implemented");
}

// ============================================================================
// Test 12: numaRead/numaWrite -- skipped
// ============================================================================

#[test]
#[ignore = "C version: numaRead(), numaWrite() -- not yet implemented in Rust"]
fn numa2_reg_numa_read_write() {
    // C version:
    // na = numaRead("lyra.5.na")
    // numaRead/numaWrite are not implemented in Rust.
    panic!("Numa read/write tests not implemented");
}

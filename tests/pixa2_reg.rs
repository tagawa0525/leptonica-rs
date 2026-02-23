//! Pix operations regression test
//!
//! Tests arithmetic, blend, raster operations, graphics drawing, border
//! operations, comparison, histogram, statistics, and line extraction.
//!
//! Named pixa2_reg for consistency with C Leptonica's test naming, but
//! this covers the full set of Phase 4 Pix operations.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixa2_reg.c`

use leptonica_core::pix::statistics::PixelMaxType;
use leptonica_core::{BlendMode, Box, Color, Pix, PixelDepth, PixelOp};
use leptonica_test::RegParams;

/// Helper: get pixel value, panicking on out-of-bounds.
fn px(pix: &Pix, x: u32, y: u32) -> u32 {
    pix.get_pixel(x, y).unwrap()
}

fn make_gray_ramp(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..height {
        for x in 0..width {
            let val = (x * 255) / width.max(1);
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

fn make_uniform_gray(width: u32, height: u32, val: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..height {
        for x in 0..width {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

fn make_binary_checkerboard(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 2 == 0 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    pm.into()
}

fn make_32bit_color(width: u32, height: u32) -> Pix {
    let pix = Pix::new(width, height, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255) / width.max(1);
            let g = (y * 255) / height.max(1);
            let b = 128u32;
            let pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

// ==========================================================================
// Test 1: Arithmetic operations -- add_constant, multiply_constant
// ==========================================================================

#[test]
fn pixa2_reg_arith_constants() {
    let mut rp = RegParams::new("pixa2_arith_const");

    let pix = make_uniform_gray(100, 100, 100);

    // Add constant
    let added = pix.add_constant(50).unwrap();
    rp.compare_values(150.0, px(&added, 50, 50) as f64, 0.0);

    // Add negative constant
    let subtracted = pix.add_constant(-30).unwrap();
    rp.compare_values(70.0, px(&subtracted, 50, 50) as f64, 0.0);

    // Add constant clipping at max
    let clipped = pix.add_constant(200).unwrap();
    rp.compare_values(255.0, px(&clipped, 50, 50) as f64, 0.0);

    // Subtract below zero clips to 0
    let clipped_low = pix.add_constant(-200).unwrap();
    rp.compare_values(0.0, px(&clipped_low, 50, 50) as f64, 0.0);

    // Multiply constant
    let doubled = pix.multiply_constant(2.0).unwrap();
    rp.compare_values(200.0, px(&doubled, 50, 50) as f64, 0.0);

    // Multiply constant clips
    let tripled = pix.multiply_constant(3.0).unwrap();
    rp.compare_values(255.0, px(&tripled, 50, 50) as f64, 0.0);

    // 1bpp not supported
    let binary = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    assert!(binary.add_constant(1).is_err());

    assert!(rp.cleanup(), "pixa2_reg arith constants tests failed");
}

// ==========================================================================
// Test 2: Arithmetic operations -- add, subtract, abs_diff, min, max
// ==========================================================================

#[test]
fn pixa2_reg_arith_binary_ops() {
    let mut rp = RegParams::new("pixa2_arith_binops");

    let pix1 = make_uniform_gray(100, 100, 100);
    let pix2 = make_uniform_gray(100, 100, 50);

    // Add two images (clipped)
    let sum = pix1.arith_add(&pix2).unwrap();
    rp.compare_values(150.0, px(&sum, 50, 50) as f64, 0.0);

    // Subtract
    let diff = pix1.arith_subtract(&pix2).unwrap();
    rp.compare_values(50.0, px(&diff, 50, 50) as f64, 0.0);

    // Abs diff
    let adiff = pix2.arith_abs_diff(&pix1).unwrap();
    rp.compare_values(50.0, px(&adiff, 50, 50) as f64, 0.0);

    // Min
    let mn = pix1.arith_min(&pix2).unwrap();
    rp.compare_values(50.0, px(&mn, 50, 50) as f64, 0.0);

    // Max
    let mx = pix1.arith_max(&pix2).unwrap();
    rp.compare_values(100.0, px(&mx, 50, 50) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg arith binary ops tests failed");
}

// ==========================================================================
// Test 3: In-place arithmetic
// ==========================================================================

#[test]
fn pixa2_reg_arith_inplace() {
    let mut rp = RegParams::new("pixa2_arith_inplace");

    let pix = make_uniform_gray(100, 100, 100);
    let pix2 = make_uniform_gray(100, 100, 25);

    // Add constant in-place
    let mut pm = pix.to_mut();
    pm.add_constant_inplace(55);
    let result: Pix = pm.into();
    rp.compare_values(155.0, px(&result, 50, 50) as f64, 0.0);

    // Multiply constant in-place
    let pix3 = make_uniform_gray(100, 100, 50);
    let mut pm3 = pix3.to_mut();
    pm3.multiply_constant_inplace(2.0).unwrap();
    let result3: Pix = pm3.into();
    rp.compare_values(100.0, px(&result3, 50, 50) as f64, 0.0);

    // Add inplace
    let mut pm4 = pix.to_mut();
    pm4.arith_add_inplace(&pix2).unwrap();
    let result4: Pix = pm4.into();
    rp.compare_values(125.0, px(&result4, 50, 50) as f64, 0.0);

    // Subtract inplace
    let mut pm5 = pix.to_mut();
    pm5.arith_subtract_inplace(&pix2).unwrap();
    let result5: Pix = pm5.into();
    rp.compare_values(75.0, px(&result5, 50, 50) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg arith inplace tests failed");
}

// ==========================================================================
// Test 4: ROP operations -- AND, OR, XOR, invert
// ==========================================================================

#[test]
fn pixa2_reg_rop_ops() {
    let mut rp = RegParams::new("pixa2_rop");

    let all_ones = {
        let p = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        pm.set_all();
        let r: Pix = pm.into();
        r
    };
    let all_zeros = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
    let checker = make_binary_checkerboard(32, 32);

    // AND: all_ones & checker == checker
    let result = all_ones.and(&checker).unwrap();
    rp.compare_values(px(&checker, 0, 0) as f64, px(&result, 0, 0) as f64, 0.0);
    rp.compare_values(px(&checker, 1, 0) as f64, px(&result, 1, 0) as f64, 0.0);

    // OR: all_zeros | checker == checker
    let result2 = all_zeros.or(&checker).unwrap();
    rp.compare_values(px(&checker, 0, 0) as f64, px(&result2, 0, 0) as f64, 0.0);

    // XOR: checker ^ checker == all_zeros
    let result3 = checker.xor(&checker).unwrap();
    rp.compare_values(0.0, px(&result3, 0, 0) as f64, 0.0);
    rp.compare_values(0.0, px(&result3, 1, 0) as f64, 0.0);

    // Invert
    let inverted = all_zeros.invert();
    rp.compare_values(1.0, px(&inverted, 0, 0) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg rop tests failed");
}

// ==========================================================================
// Test 5: ROP in-place and region operations
// ==========================================================================

#[test]
fn pixa2_reg_rop_inplace() {
    let mut rp = RegParams::new("pixa2_rop_inplace");

    let checker = make_binary_checkerboard(32, 32);

    // Invert in-place
    let mut pm = checker.to_mut();
    pm.invert_inplace();
    let inverted: Pix = pm.into();
    let orig_val = px(&checker, 0, 0);
    let inv_val = px(&inverted, 0, 0);
    rp.compare_values(1.0, if orig_val != inv_val { 1.0 } else { 0.0 }, 0.0);

    // Clear region
    let gray = make_uniform_gray(100, 100, 200);
    let mut pm2 = gray.to_mut();
    pm2.clear_region(10, 10, 20, 20);
    let cleared: Pix = pm2.into();
    rp.compare_values(0.0, px(&cleared, 15, 15) as f64, 0.0);
    rp.compare_values(200.0, px(&cleared, 0, 0) as f64, 0.0);

    // Set region
    let empty = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pm3 = empty.to_mut();
    pm3.set_region(10, 10, 20, 20);
    let set_pix: Pix = pm3.into();
    rp.compare_values(255.0, px(&set_pix, 15, 15) as f64, 0.0);
    rp.compare_values(0.0, px(&set_pix, 0, 0) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg rop inplace tests failed");
}

// ==========================================================================
// Test 6: Blend operations
// ==========================================================================

#[test]
fn pixa2_reg_blend() {
    let mut rp = RegParams::new("pixa2_blend");

    let pix1 = make_32bit_color(100, 100);
    let pix2 = make_32bit_color(100, 100);

    // Multiply blend
    let blended = pix1.blend_multiply(&pix2).unwrap();
    rp.compare_values(100.0, blended.width() as f64, 0.0);

    // Screen blend
    let screened = pix1.blend_screen(&pix2).unwrap();
    rp.compare_values(100.0, screened.width() as f64, 0.0);

    // Overlay blend
    let overlaid = pix1.blend_overlay(&pix2).unwrap();
    rp.compare_values(100.0, overlaid.width() as f64, 0.0);

    // Blend with mode
    let normal_blend = pix1.blend(&pix2, BlendMode::Normal, 0.5).unwrap();
    rp.compare_values(100.0, normal_blend.width() as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg blend tests failed");
}

// ==========================================================================
// Test 7: Border operations
// ==========================================================================

#[test]
fn pixa2_reg_border() {
    let mut rp = RegParams::new("pixa2_border");

    let pix = make_uniform_gray(100, 100, 128);

    // Add uniform border
    let bordered = pix.add_border(10, 0).unwrap();
    rp.compare_values(120.0, bordered.width() as f64, 0.0);
    rp.compare_values(120.0, bordered.height() as f64, 0.0);
    // Border pixel should be 0
    rp.compare_values(0.0, px(&bordered, 0, 0) as f64, 0.0);
    // Interior pixel should be 128
    rp.compare_values(128.0, px(&bordered, 10, 10) as f64, 0.0);

    // Remove border
    let removed = bordered.remove_border(10).unwrap();
    rp.compare_values(100.0, removed.width() as f64, 0.0);
    rp.compare_values(100.0, removed.height() as f64, 0.0);
    rp.compare_values(128.0, px(&removed, 0, 0) as f64, 0.0);

    // Add general border
    let gen_bordered = pix.add_border_general(5, 10, 15, 20, 255).unwrap();
    rp.compare_values(115.0, gen_bordered.width() as f64, 0.0); // 100 + 5 + 10
    rp.compare_values(135.0, gen_bordered.height() as f64, 0.0); // 100 + 15 + 20
    // Top-left border pixel should be 255
    rp.compare_values(255.0, px(&gen_bordered, 0, 0) as f64, 0.0);

    // Remove general border
    let gen_removed = gen_bordered.remove_border_general(5, 10, 15, 20).unwrap();
    rp.compare_values(100.0, gen_removed.width() as f64, 0.0);
    rp.compare_values(100.0, gen_removed.height() as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg border tests failed");
}

// ==========================================================================
// Test 8: Comparison operations
// ==========================================================================

#[test]
fn pixa2_reg_compare() {
    let mut rp = RegParams::new("pixa2_compare");

    let pix1 = make_uniform_gray(100, 100, 100);
    let pix2 = make_uniform_gray(100, 100, 100);
    let pix3 = make_uniform_gray(100, 100, 150);

    // Equal images
    rp.compare_values(1.0, if pix1.equals(&pix2) { 1.0 } else { 0.0 }, 0.0);

    // Unequal images
    rp.compare_values(0.0, if pix1.equals(&pix3) { 1.0 } else { 0.0 }, 0.0);

    // Pixel diffs
    let diff_result = pix1.count_pixel_diffs(&pix3).unwrap();
    rp.compare_values(10000.0, diff_result.n_diff as f64, 0.0);
    rp.compare_values(1.0, diff_result.fract_diff, 0.0);

    // RMS diff
    let rms = pix1.rms_diff(&pix3).unwrap();
    rp.compare_values(50.0, rms, 0.5);

    // Mean abs diff
    let mad = pix1.mean_abs_diff(&pix3).unwrap();
    rp.compare_values(50.0, mad, 0.5);

    // Abs diff image
    let adiff = pix1.abs_diff(&pix3).unwrap();
    rp.compare_values(50.0, px(&adiff, 50, 50) as f64, 0.0);

    // Full compare
    let cmp = pix1.compare(&pix2).unwrap();
    rp.compare_values(1.0, if cmp.equal { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(0.0, cmp.n_diff as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg compare tests failed");
}

// ==========================================================================
// Test 9: Binary correlation
// ==========================================================================

#[test]
fn pixa2_reg_correlation() {
    let mut rp = RegParams::new("pixa2_correlation");

    let checker = make_binary_checkerboard(64, 64);

    // Self-correlation should be 1.0
    let corr = leptonica_core::correlation_binary(&checker, &checker).unwrap();
    rp.compare_values(1.0, corr, 0.001);

    // Inverted should have low correlation
    let inverted = checker.invert();
    let corr_inv = leptonica_core::correlation_binary(&checker, &inverted).unwrap();
    rp.compare_values(0.0, corr_inv, 0.01);

    assert!(rp.cleanup(), "pixa2_reg correlation tests failed");
}

// ==========================================================================
// Test 10: Graphics -- line rendering
// ==========================================================================

#[test]
fn pixa2_reg_graphics_lines() {
    let mut rp = RegParams::new("pixa2_graphics_lines");

    let pix = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();

    // Render horizontal line
    pm.render_line(0, 100, 199, 100, 1, PixelOp::Set).unwrap();
    let result: Pix = pm.into();
    // Pixel on the line should be set
    rp.compare_values(1.0, if px(&result, 100, 100) > 0 { 1.0 } else { 0.0 }, 0.0);
    // Pixel off the line should be clear
    rp.compare_values(0.0, px(&result, 100, 50) as f64, 0.0);

    // Render on 32-bit with color
    let pix32 = Pix::new(200, 200, PixelDepth::Bit32).unwrap();
    let mut pm32 = pix32.try_into_mut().unwrap();
    pm32.render_line_color(10, 10, 190, 190, 3, Color::RED)
        .unwrap();
    let result32: Pix = pm32.into();
    rp.compare_values(200.0, result32.width() as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg graphics lines tests failed");
}

// ==========================================================================
// Test 11: Graphics -- box rendering
// ==========================================================================

#[test]
fn pixa2_reg_graphics_boxes() {
    let mut rp = RegParams::new("pixa2_graphics_boxes");

    let pix = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();

    let b = Box::new(20, 20, 60, 60).unwrap();
    pm.render_box(&b, 1, PixelOp::Set).unwrap();
    let result: Pix = pm.into();

    // Corner of box should be set
    rp.compare_values(1.0, if px(&result, 20, 20) > 0 { 1.0 } else { 0.0 }, 0.0);
    // Center should be clear (only outline)
    rp.compare_values(0.0, px(&result, 50, 50) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg graphics boxes tests failed");
}

// ==========================================================================
// Test 12: Graphics -- circle rendering
// ==========================================================================

#[test]
fn pixa2_reg_graphics_circles() {
    let mut rp = RegParams::new("pixa2_graphics_circles");

    let pix = Pix::new(200, 200, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();

    // Render filled circle at center
    pm.render_filled_circle(100, 100, 30, PixelOp::Set).unwrap();
    let result: Pix = pm.into();

    // Center should be set
    rp.compare_values(1.0, if px(&result, 100, 100) > 0 { 1.0 } else { 0.0 }, 0.0);
    // Far corner should be clear
    rp.compare_values(0.0, px(&result, 0, 0) as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg graphics circles tests failed");
}

// ==========================================================================
// Test 13: Histogram
// ==========================================================================

#[test]
fn pixa2_reg_histogram() {
    let mut rp = RegParams::new("pixa2_histogram");

    // Uniform gray image -- all pixels at value 128
    let pix = make_uniform_gray(100, 100, 128);

    let hist = pix.gray_histogram(1).unwrap();
    // Histogram should have 256 bins
    rp.compare_values(256.0, hist.len() as f64, 0.0);
    // Bin 128 should have 10000 counts
    rp.compare_values(10000.0, f64::from(hist.get(128).unwrap()), 0.0);
    // Other bins should be 0
    rp.compare_values(0.0, f64::from(hist.get(0).unwrap()), 0.0);
    rp.compare_values(0.0, f64::from(hist.get(255).unwrap()), 0.0);

    // Color histogram on 32-bit
    let color_pix = make_32bit_color(50, 50);
    let chist = color_pix.color_histogram(1).unwrap();
    rp.compare_values(256.0, chist.red.len() as f64, 0.0);
    rp.compare_values(256.0, chist.green.len() as f64, 0.0);
    rp.compare_values(256.0, chist.blue.len() as f64, 0.0);

    assert!(rp.cleanup(), "pixa2_reg histogram tests failed");
}

// ==========================================================================
// Test 14: Statistics -- count, average, variance
// ==========================================================================

#[test]
fn pixa2_reg_statistics() {
    let mut rp = RegParams::new("pixa2_statistics");

    // Uniform gray image at 100
    let pix = make_uniform_gray(100, 100, 100);

    // Average in rect (whole image)
    let avg = pix.average_in_rect(None).unwrap();
    rp.compare_values(100.0, f64::from(avg), 0.5);

    // Variance in rect (should be 0 for uniform image)
    let var = pix.variance_in_rect(None).unwrap();
    rp.compare_values(0.0, f64::from(var), 0.5);

    // Average by row
    let row_avgs = pix.average_by_row(None, PixelMaxType::WhiteIsMax).unwrap();
    rp.compare_values(100.0, row_avgs.len() as f64, 0.0);

    // Average by column
    let col_avgs = pix
        .average_by_column(None, PixelMaxType::WhiteIsMax)
        .unwrap();
    rp.compare_values(100.0, col_avgs.len() as f64, 0.0);

    // Count pixels on binary checkerboard
    let checker = make_binary_checkerboard(64, 64);
    let count = checker.count_pixels();
    // 64x64 = 4096 pixels, half should be set
    rp.compare_values(2048.0, count as f64, 0.0);

    // Variance on a ramp should be non-zero
    let ramp = make_gray_ramp(100, 100);
    let ramp_var = ramp.variance_in_rect(None).unwrap();
    rp.compare_values(1.0, if ramp_var > 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixa2_reg statistics tests failed");
}

// ==========================================================================
// Test 15: Extract on line
// ==========================================================================

#[test]
fn pixa2_reg_extract() {
    let mut rp = RegParams::new("pixa2_extract");

    let pix = make_gray_ramp(100, 100);

    // Extract along horizontal line at y=50
    let vals = pix.extract_on_line(0, 50, 99, 50, 1).unwrap();
    rp.compare_values(100.0, vals.len() as f64, 0.0);

    // First value should be near 0
    rp.compare_values(0.0, f64::from(vals.get(0).unwrap()), 3.0);
    // Last value should be near 255
    rp.compare_values(255.0, f64::from(vals.get(99).unwrap()), 3.0);

    assert!(rp.cleanup(), "pixa2_reg extract tests failed");
}

// ==========================================================================
// Test 16: Graphics Pta generation
// ==========================================================================

#[test]
fn pixa2_reg_pta_generation() {
    let mut rp = RegParams::new("pixa2_pta_gen");

    // Generate a line Pta
    let pta = leptonica_core::pix::graphics::generate_line_pta(0, 0, 100, 0);
    rp.compare_values(1.0, if !pta.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Generate a filled circle Pta
    let circle_pta = leptonica_core::pix::graphics::generate_filled_circle_pta(10);
    rp.compare_values(1.0, if !circle_pta.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixa2_reg pta generation tests failed");
}

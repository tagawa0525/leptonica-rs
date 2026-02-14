//! FPix (floating-point image) regression test
//!
//! Tests FPix creation, pixel access, arithmetic operations, Pix conversion,
//! and statistics.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/fpix1_reg.c`

use leptonica_core::{FPix, NegativeHandling, Pix, PixelDepth};
use leptonica_test::RegParams;

// ==========================================================================
// Test 1: FPix creation and basic properties
// ==========================================================================

#[test]

fn fpix1_reg_creation() {
    let mut rp = RegParams::new("fpix1_creation");

    let fpix = FPix::new(640, 480).expect("FPix::new failed");
    rp.compare_values(640.0, fpix.width() as f64, 0.0);
    rp.compare_values(480.0, fpix.height() as f64, 0.0);

    // All pixels should be zero initially
    let all_zero = fpix.data().iter().all(|&v| v == 0.0);
    rp.compare_values(1.0, if all_zero { 1.0 } else { 0.0 }, 0.0);

    // FPix creation with initial value
    let fpix_v = FPix::new_with_value(100, 100, 42.5).expect("FPix::new_with_value failed");
    let all_match = fpix_v.data().iter().all(|&v| v == 42.5);
    rp.compare_values(1.0, if all_match { 1.0 } else { 0.0 }, 0.0);

    // FPix from raw data
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let fpix_d = FPix::from_data(3, 2, data).expect("FPix::from_data failed");
    rp.compare_values(1.0, fpix_d.get_pixel(0, 0).unwrap() as f64, 0.0);
    rp.compare_values(6.0, fpix_d.get_pixel(2, 1).unwrap() as f64, 0.0);

    // Invalid dimensions
    let invalid = FPix::new(0, 100);
    rp.compare_values(1.0, if invalid.is_err() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "fpix1_reg creation tests failed");
}

// ==========================================================================
// Test 2: Pixel access
// ==========================================================================

#[test]

fn fpix1_reg_pixel_access() {
    let mut rp = RegParams::new("fpix1_access");

    let mut fpix = FPix::new(100, 100).unwrap();

    // Set and get
    fpix.set_pixel(50, 50, 3.14).unwrap();
    rp.compare_values(3.14, fpix.get_pixel(50, 50).unwrap() as f64, 0.001);

    // Negative values
    fpix.set_pixel(0, 0, -1.5).unwrap();
    rp.compare_values(-1.5, fpix.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Out-of-bounds access should error
    let oob = fpix.get_pixel(100, 0);
    rp.compare_values(1.0, if oob.is_err() { 1.0 } else { 0.0 }, 0.0);

    // Row access
    let mut fpix2 = FPix::new(5, 3).unwrap();
    for x in 0..5 {
        fpix2.set_pixel(x, 1, (x + 1) as f32).unwrap();
    }
    let row = fpix2.row(1);
    rp.compare_values(5.0, row.len() as f64, 0.0);
    rp.compare_values(1.0, row[0] as f64, 0.0);
    rp.compare_values(5.0, row[4] as f64, 0.0);

    // Resolution
    let mut fpix3 = FPix::new(10, 10).unwrap();
    fpix3.set_resolution(300, 300);
    rp.compare_values(300.0, fpix3.xres() as f64, 0.0);
    rp.compare_values(300.0, fpix3.yres() as f64, 0.0);

    assert!(rp.cleanup(), "fpix1_reg pixel access tests failed");
}

// ==========================================================================
// Test 3: Arithmetic operations
// ==========================================================================

#[test]

fn fpix1_reg_arithmetic() {
    let mut rp = RegParams::new("fpix1_arith");

    let fpix1 = FPix::new_with_value(50, 50, 3.0).unwrap();
    let fpix2 = FPix::new_with_value(50, 50, 2.0).unwrap();

    // Add
    let result = fpix1.add(&fpix2).unwrap();
    rp.compare_values(5.0, result.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Sub
    let result = fpix1.sub(&fpix2).unwrap();
    rp.compare_values(1.0, result.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Mul
    let result = fpix1.mul(&fpix2).unwrap();
    rp.compare_values(6.0, result.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Div
    let result = fpix1.div(&fpix2).unwrap();
    rp.compare_values(1.5, result.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Operator overloading
    let result = (&fpix1 + &fpix2).unwrap();
    rp.compare_values(5.0, result.data()[0] as f64, 0.001);

    let result = (&fpix1 - &fpix2).unwrap();
    rp.compare_values(1.0, result.data()[0] as f64, 0.001);

    // Size mismatch should error
    let fpix3 = FPix::new(10, 10).unwrap();
    let err = fpix1.add(&fpix3);
    rp.compare_values(1.0, if err.is_err() { 1.0 } else { 0.0 }, 0.0);

    // Constant operations
    let mut fpix4 = FPix::new_with_value(10, 10, 2.0).unwrap();
    fpix4.add_constant(3.0);
    rp.compare_values(5.0, fpix4.get_pixel(0, 0).unwrap() as f64, 0.001);

    fpix4.mul_constant(2.0);
    rp.compare_values(10.0, fpix4.get_pixel(0, 0).unwrap() as f64, 0.001);

    // Linear combination
    let fpix5 = FPix::new_with_value(10, 10, 2.0).unwrap();
    let lc = fpix5.linear_combination(3.0, 1.0); // 3*2 + 1 = 7
    rp.compare_values(7.0, lc.data()[0] as f64, 0.001);

    assert!(rp.cleanup(), "fpix1_reg arithmetic tests failed");
}

// ==========================================================================
// Test 4: Statistics
// ==========================================================================

#[test]

fn fpix1_reg_statistics() {
    let mut rp = RegParams::new("fpix1_stats");

    let mut fpix = FPix::new_with_value(10, 10, 5.0).unwrap();
    fpix.set_pixel(3, 7, -2.0).unwrap();
    fpix.set_pixel(8, 2, 100.0).unwrap();

    // Min
    let (min_val, min_x, min_y) = fpix.min().unwrap();
    rp.compare_values(-2.0, min_val as f64, 0.0);
    rp.compare_values(3.0, min_x as f64, 0.0);
    rp.compare_values(7.0, min_y as f64, 0.0);

    // Max
    let (max_val, max_x, max_y) = fpix.max().unwrap();
    rp.compare_values(100.0, max_val as f64, 0.0);
    rp.compare_values(8.0, max_x as f64, 0.0);
    rp.compare_values(2.0, max_y as f64, 0.0);

    // Mean: (98*5 + (-2) + 100) / 100 = (490 + 98) / 100 = 5.88
    let mean = fpix.mean().unwrap();
    rp.compare_values(5.88, mean as f64, 0.001);

    // Sum
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let fpix2 = FPix::from_data(2, 2, data).unwrap();
    rp.compare_values(10.0, fpix2.sum() as f64, 0.001);

    // Mean of uniform
    let fpix3 = FPix::new_with_value(10, 10, 4.0).unwrap();
    rp.compare_values(4.0, fpix3.mean().unwrap() as f64, 0.001);

    assert!(rp.cleanup(), "fpix1_reg statistics tests failed");
}

// ==========================================================================
// Test 5: Pix conversion (FPix <-> Pix roundtrip)
// ==========================================================================

#[test]

fn fpix1_reg_pix_conversion() {
    let mut rp = RegParams::new("fpix1_pix");

    // Create an 8-bit Pix with known values
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.to_mut();

    // Set row 0: 100, 200, 0, 255
    let data = pix_mut.data_mut();
    data[0] = 0x64_C8_00_FF; // 100, 200, 0, 255

    let pix: Pix = pix_mut.into();
    let fpix = FPix::from_pix(&pix).unwrap();

    rp.compare_values(100.0, fpix.get_pixel(0, 0).unwrap() as f64, 0.0);
    rp.compare_values(200.0, fpix.get_pixel(1, 0).unwrap() as f64, 0.0);
    rp.compare_values(0.0, fpix.get_pixel(2, 0).unwrap() as f64, 0.0);
    rp.compare_values(255.0, fpix.get_pixel(3, 0).unwrap() as f64, 0.0);

    // FPix to Pix (8-bit)
    let mut fpix2 = FPix::new(4, 2).unwrap();
    fpix2.set_pixel(0, 0, 0.0).unwrap();
    fpix2.set_pixel(1, 0, 127.5).unwrap();
    fpix2.set_pixel(2, 0, 255.0).unwrap();
    fpix2.set_pixel(3, 0, 300.0).unwrap(); // should clamp to 255

    let pix2 = fpix2.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(8.0, pix2.depth().bits() as f64, 0.0);

    // Negative handling: ClipToZero
    let mut fpix3 = FPix::new(2, 1).unwrap();
    fpix3.set_pixel(0, 0, -10.0).unwrap();
    fpix3.set_pixel(1, 0, 10.0).unwrap();

    let pix3 = fpix3.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    let data3 = pix3.data();
    let val0 = (data3[0] >> 24) & 0xff;
    rp.compare_values(0.0, val0 as f64, 0.0); // -10 clipped to 0

    // Auto-detect depth
    let fpix8 = FPix::new_with_value(10, 10, 200.0).unwrap();
    let auto8 = fpix8.to_pix(0, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(8.0, auto8.depth().bits() as f64, 0.0);

    let fpix16 = FPix::new_with_value(10, 10, 1000.0).unwrap();
    let auto16 = fpix16.to_pix(0, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(16.0, auto16.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "fpix1_reg pix conversion tests failed");
}

// ==========================================================================
// Test 6: Set all / clear
// ==========================================================================

#[test]

fn fpix1_reg_set_clear() {
    let mut rp = RegParams::new("fpix1_setclear");

    let mut fpix = FPix::new(10, 10).unwrap();

    fpix.set_all(5.0);
    let all_five = fpix.data().iter().all(|&v| v == 5.0);
    rp.compare_values(1.0, if all_five { 1.0 } else { 0.0 }, 0.0);

    fpix.clear();
    let all_zero = fpix.data().iter().all(|&v| v == 0.0);
    rp.compare_values(1.0, if all_zero { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "fpix1_reg set/clear tests failed");
}

// ==========================================================================
// Test 7: Clone independence
// ==========================================================================

#[test]

fn fpix1_reg_clone() {
    let mut rp = RegParams::new("fpix1_clone");

    let fpix1 = FPix::new_with_value(10, 10, 5.0).unwrap();
    let fpix2 = fpix1.clone();

    // Should be independent copies
    let data_eq = fpix1.data() == fpix2.data();
    let ptr_ne = fpix1.data().as_ptr() != fpix2.data().as_ptr();
    rp.compare_values(1.0, if data_eq { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if ptr_ne { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "fpix1_reg clone tests failed");
}

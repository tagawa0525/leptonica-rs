//! Test FPix/DPix extension functions
//!
//! # See also
//!
//! C Leptonica: `fpix1.c`, `fpix2.c`

use leptonica_core::{DPix, FPix, NegativeHandling};

// ============================================================================
// FPix::create_template
// ============================================================================

#[test]
fn test_fpix_create_template() {
    let mut fpix = FPix::new_with_value(100, 200, 42.0).unwrap();
    fpix.set_resolution(300, 300);

    let tmpl = fpix.create_template();
    assert_eq!(tmpl.width(), 100);
    assert_eq!(tmpl.height(), 200);
    assert_eq!(tmpl.resolution(), (300, 300));
    // All values should be zero
    assert_eq!(tmpl.get_pixel(0, 0).unwrap(), 0.0);
    assert_eq!(tmpl.get_pixel(50, 100).unwrap(), 0.0);
}

// ============================================================================
// FPix::linear_combination_two
// ============================================================================

#[test]
fn test_fpix_linear_combination_two() {
    let f1 = FPix::new_with_value(10, 10, 3.0).unwrap();
    let f2 = FPix::new_with_value(10, 10, 5.0).unwrap();

    // result = 2.0 * f1 + 3.0 * f2 = 6.0 + 15.0 = 21.0
    let result = FPix::linear_combination_two(2.0, &f1, 3.0, &f2).unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.height(), 10);
    assert!((result.get_pixel(0, 0).unwrap() - 21.0).abs() < 1e-6);
}

#[test]
fn test_fpix_linear_combination_two_size_mismatch() {
    let f1 = FPix::new(10, 10).unwrap();
    let f2 = FPix::new(20, 20).unwrap();
    assert!(FPix::linear_combination_two(1.0, &f1, 1.0, &f2).is_err());
}

// ============================================================================
// DPix::new
// ============================================================================

#[test]
fn test_dpix_new() {
    let dpix = DPix::new(50, 30).unwrap();
    assert_eq!(dpix.width(), 50);
    assert_eq!(dpix.height(), 30);
    // All values should be zero
    for &v in dpix.data() {
        assert_eq!(v, 0.0);
    }
}

#[test]
fn test_dpix_invalid_dimensions() {
    assert!(DPix::new(0, 10).is_err());
    assert!(DPix::new(10, 0).is_err());
}

// ============================================================================
// DPix pixel access
// ============================================================================

#[test]
fn test_dpix_pixel_access() {
    let mut dpix = DPix::new(10, 10).unwrap();
    dpix.set_pixel(5, 5, 3.14159).unwrap();
    assert!((dpix.get_pixel(5, 5).unwrap() - 3.14159).abs() < 1e-10);

    // Out of bounds
    assert!(dpix.get_pixel(10, 0).is_err());
    assert!(dpix.set_pixel(0, 10, 1.0).is_err());
}

// ============================================================================
// DPix::to_pix
// ============================================================================

#[test]
fn test_dpix_to_pix() {
    let mut dpix = DPix::new(5, 5).unwrap();
    for y in 0..5 {
        for x in 0..5 {
            dpix.set_pixel(x, y, (x * 50) as f64).unwrap();
        }
    }

    let pix = dpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    assert_eq!(pix.width(), 5);
    assert_eq!(pix.height(), 5);
    assert_eq!(pix.get_pixel(0, 0).unwrap(), 0);
    assert_eq!(pix.get_pixel(4, 0).unwrap(), 200);
}

#[test]
fn test_dpix_to_pix_negative_handling() {
    let mut dpix = DPix::new(3, 1).unwrap();
    dpix.set_pixel(0, 0, -10.0).unwrap();
    dpix.set_pixel(1, 0, 100.0).unwrap();
    dpix.set_pixel(2, 0, -50.0).unwrap();

    // ClipToZero
    let pix = dpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    assert_eq!(pix.get_pixel(0, 0).unwrap(), 0);
    assert_eq!(pix.get_pixel(1, 0).unwrap(), 100);
    assert_eq!(pix.get_pixel(2, 0).unwrap(), 0);

    // TakeAbsValue
    let pix = dpix.to_pix(8, NegativeHandling::TakeAbsValue).unwrap();
    assert_eq!(pix.get_pixel(0, 0).unwrap(), 10);
    assert_eq!(pix.get_pixel(2, 0).unwrap(), 50);
}

// ============================================================================
// DPix::to_fpix / from_fpix
// ============================================================================

#[test]
fn test_dpix_to_fpix() {
    let mut dpix = DPix::new(5, 5).unwrap();
    dpix.set_pixel(2, 2, 123.456).unwrap();

    let fpix = dpix.to_fpix();
    assert_eq!(fpix.width(), 5);
    assert_eq!(fpix.height(), 5);
    assert!((fpix.get_pixel(2, 2).unwrap() - 123.456).abs() < 0.001);
}

#[test]
fn test_dpix_from_fpix() {
    let mut fpix = FPix::new(5, 5).unwrap();
    fpix.set_pixel(3, 3, 99.5).unwrap();

    let dpix = DPix::from_fpix(&fpix);
    assert_eq!(dpix.width(), 5);
    assert_eq!(dpix.height(), 5);
    assert!((dpix.get_pixel(3, 3).unwrap() - 99.5).abs() < 1e-6);
}

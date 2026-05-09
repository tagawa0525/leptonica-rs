//! Test FPix/DPix extension functions
//!
//! C version fpix2_reg.c tests:
//! - Orthogonal rotation of FPix (checks 0-2): fpixRotateOrth 90/180/270
//! - Border operations on FPix (checks 3-4): fpixAddMirroredBorder,
//!   fpixAddContinuedBorder
//!
//! # See also
//!
//! C Leptonica: `fpix1.c`, `fpix2.c`

use leptonica::core::fpix::transform::RotateDirection;
use leptonica::{DPix, FPix, NegativeHandling};

/// Build an FPix where the (x, y) value encodes its position so we can verify
/// rotations and flips by inspecting individual pixel values.
fn make_positional_fpix(w: u32, h: u32) -> FPix {
    let mut fpix = FPix::new(w, h).unwrap();
    for y in 0..h {
        for x in 0..w {
            fpix.set_pixel(x, y, (y * w + x) as f32).unwrap();
        }
    }
    fpix
}

// ============================================================================
// C-equivalent regression test skeletons (C checks 0–4)
// ============================================================================

/// FPix orthogonal rotation by 90 degrees (C check 0).
///
/// rotate_orth(1) is one clockwise quarter-turn: result[i, j] = src[j, h-1-i]
/// where the source has dimensions (w, h) and the destination (h, w).
#[test]
fn fpix2_reg_rotate_orth_90() {
    let src = make_positional_fpix(5, 3);
    let dst = src.rotate_orth(1).expect("rotate_orth(1)");
    assert_eq!((dst.width(), dst.height()), (3, 5));
    for y in 0..dst.height() {
        for x in 0..dst.width() {
            let expected = src.get_pixel(y, src.height() - 1 - x).unwrap();
            assert_eq!(dst.get_pixel(x, y).unwrap(), expected, "pixel ({x}, {y})");
        }
    }
}

/// FPix orthogonal rotation by 180 degrees (C check 1).
#[test]
fn fpix2_reg_rotate_orth_180() {
    let src = make_positional_fpix(5, 3);
    let dst = src.rotate_orth(2).expect("rotate_orth(2)");
    assert_eq!((dst.width(), dst.height()), (5, 3));
    for y in 0..dst.height() {
        for x in 0..dst.width() {
            let expected = src
                .get_pixel(src.width() - 1 - x, src.height() - 1 - y)
                .unwrap();
            assert_eq!(dst.get_pixel(x, y).unwrap(), expected, "pixel ({x}, {y})");
        }
    }
}

/// FPix orthogonal rotation by 270 degrees (C check 2).
#[test]
fn fpix2_reg_rotate_orth_270() {
    let src = make_positional_fpix(5, 3);
    let dst = src.rotate_orth(3).expect("rotate_orth(3)");
    assert_eq!((dst.width(), dst.height()), (3, 5));
    for y in 0..dst.height() {
        for x in 0..dst.width() {
            let expected = src.get_pixel(src.width() - 1 - y, x).unwrap();
            assert_eq!(dst.get_pixel(x, y).unwrap(), expected, "pixel ({x}, {y})");
        }
    }

    // rotate_90 cw + ccw round-trip restores the source.
    let cw = src.rotate_90(RotateDirection::Cw).unwrap();
    let restored = cw.rotate_90(RotateDirection::Ccw).unwrap();
    assert_eq!(restored.data(), src.data(), "cw + ccw should round-trip");

    // flip_lr applied twice is identity.
    let lr = src.flip_lr().unwrap();
    let lr_lr = lr.flip_lr().unwrap();
    assert_eq!(lr_lr.data(), src.data());
}

/// FPix mirrored border addition (C check 3).
#[test]
fn fpix2_reg_add_mirrored_border() {
    let src = make_positional_fpix(5, 3);
    let bordered = src.add_mirrored_border(2, 2, 1, 1).expect("mirror border");
    assert_eq!((bordered.width(), bordered.height()), (9, 5));
    // Top-left of the original pixel block sits at (2, 1).
    for y in 0..3 {
        for x in 0..5 {
            assert_eq!(
                bordered.get_pixel(x + 2, y + 1).unwrap(),
                src.get_pixel(x, y).unwrap(),
                "interior pixel ({x}, {y})",
            );
        }
    }
    // Mirror columns: column 1 (one in from left) should mirror original col 0.
    for y in 0..3 {
        assert_eq!(
            bordered.get_pixel(1, y + 1).unwrap(),
            src.get_pixel(0, y).unwrap(),
            "mirror left col @ y={y}",
        );
        assert_eq!(
            bordered.get_pixel(0, y + 1).unwrap(),
            src.get_pixel(1, y).unwrap(),
            "mirror left col2 @ y={y}",
        );
    }
}

/// FPix continued border addition (C check 4).
#[test]
fn fpix2_reg_add_continued_border() {
    let src = make_positional_fpix(5, 3);
    let bordered = src
        .add_continued_border(2, 2, 1, 1)
        .expect("continued border");
    assert_eq!((bordered.width(), bordered.height()), (9, 5));
    // Each border column repeats the nearest-edge value of the source.
    for y in 0..3 {
        let left_edge = src.get_pixel(0, y).unwrap();
        let right_edge = src.get_pixel(4, y).unwrap();
        for bx in 0..2 {
            assert_eq!(bordered.get_pixel(bx, y + 1).unwrap(), left_edge);
            assert_eq!(bordered.get_pixel(7 + bx, y + 1).unwrap(), right_edge);
        }
    }
}

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
    dpix.set_pixel(5, 5, std::f64::consts::PI).unwrap();
    assert!((dpix.get_pixel(5, 5).unwrap() - std::f64::consts::PI).abs() < 1e-10);

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

#[test]
fn test_dpix_to_pix_invalid_depth() {
    let dpix = DPix::new(1, 1).unwrap();
    assert!(dpix.to_pix(7, NegativeHandling::ClipToZero).is_err());
    assert!(dpix.to_pix(15, NegativeHandling::ClipToZero).is_err());
    assert!(dpix.to_pix(4, NegativeHandling::ClipToZero).is_err());
}

#[test]
fn test_dpix_to_pix_preserves_resolution() {
    let mut dpix = DPix::new(5, 5).unwrap();
    dpix.set_resolution(300, 600);
    dpix.set_pixel(0, 0, 100.0).unwrap();

    let pix = dpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    assert_eq!(pix.xres(), 300);
    assert_eq!(pix.yres(), 600);
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

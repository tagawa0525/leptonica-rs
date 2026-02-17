//! Test pixel arithmetic and rasterop extension functions
//!
//! # See also
//!
//! C Leptonica: `pixarith.c`, `rop.c`

use leptonica_core::{InColor, Pix, PixelDepth};

// ============================================================================
// Pix::mult_const_accumulate
// ============================================================================

#[test]

fn test_mult_const_accumulate_basic() {
    // 32bpp image with offset=0x40000000
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let offset = 0x40000000u32;

    // Set all pixels to offset + 100
    let mut pm = pix.to_mut();
    for y in 0..10 {
        for x in 0..10 {
            pm.set_pixel_unchecked(x, y, offset + 100);
        }
    }

    // Multiply by 2.0: (val - offset) * factor + offset = (100) * 2.0 + offset = offset + 200
    pm.mult_const_accumulate(2.0, offset).unwrap();

    let result = pm.get_pixel_unchecked(5, 5);
    assert_eq!(result, offset + 200);
}

#[test]

fn test_mult_const_accumulate_fractional() {
    let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
    let offset = 0x40000000u32;
    let mut pm = pix.to_mut();

    for y in 0..5 {
        for x in 0..5 {
            pm.set_pixel_unchecked(x, y, offset + 1000);
        }
    }

    // Multiply by 0.5: (1000) * 0.5 + offset = offset + 500
    pm.mult_const_accumulate(0.5, offset).unwrap();

    let result = pm.get_pixel_unchecked(2, 2);
    assert_eq!(result, offset + 500);
}

#[test]

fn test_mult_const_accumulate_not_32bpp() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Should return error for non-32bpp
    assert!(pm.mult_const_accumulate(2.0, 0).is_err());
}

// ============================================================================
// PixMut::rasterop_vip
// ============================================================================

#[test]

fn test_rasterop_vip_shift_down() {
    // 8bpp 10x10 image with row 0 set to 200
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for x in 0..10 {
        pm.set_pixel_unchecked(x, 0, 200);
    }

    // Shift vertical band (x=0..10, full width) down by 3, bring in white
    pm.rasterop_vip(0, 10, 3, InColor::White);

    // Row 0 should now be white (255 for 8bpp)
    assert_eq!(pm.get_pixel_unchecked(5, 0), 255);
    // Row 3 should have the original value
    assert_eq!(pm.get_pixel_unchecked(5, 3), 200);
}

#[test]

fn test_rasterop_vip_shift_up() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Set row 5 to 150
    for x in 0..10 {
        pm.set_pixel_unchecked(x, 5, 150);
    }

    // Shift up by 3 (negative vshift), bring in black
    pm.rasterop_vip(0, 10, -3, InColor::Black);

    // Row 2 should have original row 5's value
    assert_eq!(pm.get_pixel_unchecked(5, 2), 150);
    // Row 5 should now be black (0)
    assert_eq!(pm.get_pixel_unchecked(5, 7), 0);
}

#[test]

fn test_rasterop_vip_partial_band() {
    let pix = Pix::new(20, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Set all pixels in column 5 to 100
    for y in 0..10 {
        pm.set_pixel_unchecked(5, y, 100);
    }

    // Shift only band x=3..8 (bx=3, bw=5) down by 2
    pm.rasterop_vip(3, 5, 2, InColor::White);

    // Column 5 row 0 should be white (in the shifted band)
    assert_eq!(pm.get_pixel_unchecked(5, 0), 255);
    // Column 5 row 2 should have the original value
    assert_eq!(pm.get_pixel_unchecked(5, 2), 100);
    // Column 0 (outside band) should be unchanged
    assert_eq!(pm.get_pixel_unchecked(0, 0), 0);
}

// ============================================================================
// PixMut::rasterop_hip
// ============================================================================

#[test]

fn test_rasterop_hip_shift_right() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Set column 0 to 200
    for y in 0..10 {
        pm.set_pixel_unchecked(0, y, 200);
    }

    // Shift horizontal band (y=0..10, full height) right by 3, bring in white
    pm.rasterop_hip(0, 10, 3, InColor::White);

    // Column 0 should now be white
    assert_eq!(pm.get_pixel_unchecked(0, 5), 255);
    // Column 3 should have the original value
    assert_eq!(pm.get_pixel_unchecked(3, 5), 200);
}

#[test]

fn test_rasterop_hip_shift_left() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Set column 5 to 150
    for y in 0..10 {
        pm.set_pixel_unchecked(5, y, 150);
    }

    // Shift left by 3, bring in black
    pm.rasterop_hip(0, 10, -3, InColor::Black);

    // Column 2 should have original column 5's value
    assert_eq!(pm.get_pixel_unchecked(2, 5), 150);
    // Last 3 columns should be black
    assert_eq!(pm.get_pixel_unchecked(9, 5), 0);
}

#[test]

fn test_rasterop_hip_partial_band() {
    let pix = Pix::new(10, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    // Set row 5 to 100
    for x in 0..10 {
        pm.set_pixel_unchecked(x, 5, 100);
    }

    // Shift only band y=3..8 (by=3, bh=5) right by 2
    pm.rasterop_hip(3, 5, 2, InColor::White);

    // Row 5 column 0 should be white (in the shifted band)
    assert_eq!(pm.get_pixel_unchecked(0, 5), 255);
    // Row 5 column 2 should have the original value
    assert_eq!(pm.get_pixel_unchecked(2, 5), 100);
    // Row 0 (outside band) should be unchanged
    assert_eq!(pm.get_pixel_unchecked(0, 0), 0);
}

// ============================================================================
// Pix::translate
// ============================================================================

#[test]

fn test_translate_right_down() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(0, 0, 200);
    let pix: Pix = pm.into();

    let result = pix.translate(5, 3, InColor::White);

    // Original (0,0) should now be at (5,3)
    assert_eq!(result.get_pixel(5, 3).unwrap(), 200);
    // Origin should be white
    assert_eq!(result.get_pixel(0, 0).unwrap(), 255);
}

#[test]

fn test_translate_left_up() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(10, 10, 200);
    let pix: Pix = pm.into();

    let result = pix.translate(-3, -5, InColor::Black);

    // Original (10,10) should now be at (7,5)
    assert_eq!(result.get_pixel(7, 5).unwrap(), 200);
    // Bottom-right should be black
    assert_eq!(result.get_pixel(19, 19).unwrap(), 0);
}

#[test]

fn test_translate_zero_shift() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(5, 5, 100);
    let pix: Pix = pm.into();

    let result = pix.translate(0, 0, InColor::White);

    assert_eq!(result.get_pixel(5, 5).unwrap(), 100);
}

#[test]

fn test_translate_binary() {
    let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel(5, 5, 1).unwrap();
    let pix: Pix = pm.into();

    let result = pix.translate(3, 2, InColor::White);

    // Original (5,5) moved to (8,7)
    assert_eq!(result.get_pixel(8, 7).unwrap(), 1);
    // Top-left fill: first 2 rows are white (1 for binary), (0,0) is white
    assert_eq!(result.get_pixel(0, 0).unwrap(), 1);
    // Left fill: first 3 columns are white, (0,5) is white
    assert_eq!(result.get_pixel(0, 5).unwrap(), 1);
}

//! Test additional clip and histogram extension functions
//!
//! # See also
//!
//! C Leptonica: `pix4.c`, `pix5.c`

use leptonica_core::{Box, Pix, PixelDepth, ScanDirection};

// ============================================================================
// Pix::scan_for_edge
// ============================================================================

#[test]
fn test_scan_for_edge_from_left() {
    // 8bpp 100x50, set a vertical edge at x=30
    let pix = Pix::new(100, 50, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..50 {
        for x in 30..100u32 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pix: Pix = pm.into();

    let region = Box::new(0, 0, 100, 50).unwrap();
    let loc = pix
        .scan_for_edge(&region, 50, 150, 5, 1, ScanDirection::FromLeft)
        .unwrap();
    // Edge should be near x=30
    assert!((loc as i32 - 30).abs() <= 2);
}

#[test]
fn test_scan_for_edge_from_right() {
    let pix = Pix::new(100, 50, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..50 {
        for x in 0..70u32 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pix: Pix = pm.into();

    let region = Box::new(0, 0, 100, 50).unwrap();
    let loc = pix
        .scan_for_edge(&region, 50, 150, 5, 1, ScanDirection::FromRight)
        .unwrap();
    // Edge should be near x=70
    assert!((loc as i32 - 70).abs() <= 2);
}

#[test]
fn test_scan_for_edge_from_top() {
    let pix = Pix::new(50, 100, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 25..100u32 {
        for x in 0..50 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pix: Pix = pm.into();

    let region = Box::new(0, 0, 50, 100).unwrap();
    let loc = pix
        .scan_for_edge(&region, 50, 150, 5, 1, ScanDirection::FromTop)
        .unwrap();
    assert!((loc as i32 - 25).abs() <= 2);
}

// ============================================================================
// Pix::clip_box_to_edges
// ============================================================================

#[test]
fn test_clip_box_to_edges_basic() {
    // 8bpp 100x100, with content in center (30,30)-(70,70)
    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 30..70u32 {
        for x in 30..70u32 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pix: Pix = pm.into();

    let input_box = Box::new(0, 0, 100, 100).unwrap();
    let (clipped, result_box) = pix.clip_box_to_edges(&input_box, 50, 150, 10, 1).unwrap();

    // Result box should be close to the content area
    assert!(result_box.x >= 25 && result_box.x <= 35);
    assert!(result_box.y >= 25 && result_box.y <= 35);
    assert!(clipped.width() >= 30 && clipped.width() <= 50);
}

// ============================================================================
// Pix::clip_masked
// ============================================================================

#[test]
fn test_clip_masked_basic() {
    // 8bpp 20x20 with all pixels = 100
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..20 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, 100);
        }
    }
    let pix: Pix = pm.into();

    // Full 10x10 mask at (5,5)
    let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut mm = mask.to_mut();
    for y in 0..10 {
        for x in 0..10 {
            mm.set_pixel_unchecked(x, y, 1);
        }
    }
    let mask: Pix = mm.into();

    let result = pix.clip_masked(&mask, 5, 5, 255).unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.height(), 10);
    // All masked -> original value
    assert_eq!(result.get_pixel(0, 0).unwrap(), 100);
}

#[test]
fn test_clip_masked_outval() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..20 {
        for x in 0..20 {
            pm.set_pixel_unchecked(x, y, 100);
        }
    }
    let pix: Pix = pm.into();

    // Partial mask: only center 6x6 pixels
    let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let mut mm = mask.to_mut();
    for y in 2..8 {
        for x in 2..8 {
            mm.set_pixel_unchecked(x, y, 1);
        }
    }
    let mask: Pix = mm.into();

    let result = pix.clip_masked(&mask, 5, 5, 0).unwrap();
    // Unmasked pixels should be outval (0)
    assert_eq!(result.get_pixel(0, 0).unwrap(), 0);
    // Masked pixels should keep original value
    assert_eq!(result.get_pixel(5, 5).unwrap(), 100);
}

// ============================================================================
// Pix::make_symmetric_mask
// ============================================================================

#[test]
fn test_make_symmetric_mask_inner() {
    let mask = Pix::make_symmetric_mask(100, 100, 0.5, 0.5, true).unwrap();
    assert_eq!(mask.width(), 100);
    assert_eq!(mask.height(), 100);
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    // Center should be foreground (inner rectangle)
    assert_eq!(mask.get_pixel(50, 50).unwrap(), 1);
    // Corner should be background
    assert_eq!(mask.get_pixel(0, 0).unwrap(), 0);
}

#[test]
fn test_make_symmetric_mask_outer() {
    let mask = Pix::make_symmetric_mask(100, 100, 0.5, 0.5, false).unwrap();
    assert_eq!(mask.width(), 100);
    // Center should be background (frame has hole)
    assert_eq!(mask.get_pixel(50, 50).unwrap(), 0);
    // Edge area should be foreground
    assert_eq!(mask.get_pixel(10, 10).unwrap(), 1);
}

// ============================================================================
// Pix::threshold_for_fg_bg
// ============================================================================

#[test]
fn test_threshold_for_fg_bg_basic() {
    // Bimodal image: top half dark (fg=40), bottom half light (bg=200)
    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pm = pix.to_mut();
    for y in 0..50 {
        for x in 0..100 {
            pm.set_pixel_unchecked(x, y, 40);
        }
    }
    for y in 50..100 {
        for x in 0..100 {
            pm.set_pixel_unchecked(x, y, 200);
        }
    }
    let pix: Pix = pm.into();

    let (fg_val, bg_val) = pix.threshold_for_fg_bg(1, 128).unwrap();
    assert!(fg_val < 80);
    assert!(bg_val > 150);
}

//! Test advanced clipping, foreground detection, and mask generation
//!
//! # See also
//!
//! C Leptonica: `pix5.c`
//! - pixClipRectangleWithBorder, pixCropToMatch, pixClipMasked
//! - pixClipToForeground, pixClipBoxToForeground, pixScanForForeground
//! - pixMakeFrameMask, pixFractionFgInMask
//! - pixAverageOnLine

use leptonica_core::{Box, Pix, PixelDepth, ScanDirection};

/// Create a 1bpp image with a foreground rectangle
fn make_fg_image(w: u32, h: u32, fx: u32, fy: u32, fw: u32, fh: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in fy..fy + fh {
        for x in fx..fx + fw {
            if x < w && y < h {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    pm.into()
}

/// Create a uniform 8bpp image
fn make_gray(w: u32, h: u32, val: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

// ============================================================================
// pixClipRectangleWithBorder
// ============================================================================

#[test]

fn test_clip_rectangle_with_border() {
    let pix = make_gray(100, 100, 128);
    let region = Box::new(20, 20, 40, 40).unwrap();
    let (clipped, _box) = pix.clip_rectangle_with_border(&region, 5).unwrap();
    // With border 5 on each side: 40+10=50 x 40+10=50
    assert_eq!(clipped.width(), 50);
    assert_eq!(clipped.height(), 50);
}

#[test]

fn test_clip_rectangle_with_border_clamped() {
    let pix = make_gray(100, 100, 128);
    // Region near edge: border should be clamped
    let region = Box::new(2, 2, 40, 40).unwrap();
    let (clipped, _box) = pix.clip_rectangle_with_border(&region, 10).unwrap();
    // Border clamped by distance to edge (2 on left/top)
    assert!(clipped.width() >= 40);
    assert!(clipped.height() >= 40);
}

// ============================================================================
// pixCropToMatch
// ============================================================================

#[test]

fn test_crop_to_match_same_size() {
    let pix1 = make_gray(50, 50, 100);
    let pix2 = make_gray(50, 50, 200);
    let (r1, r2) = pix1.crop_to_match(&pix2).unwrap();
    assert_eq!(r1.width(), 50);
    assert_eq!(r2.width(), 50);
}

#[test]

fn test_crop_to_match_different_size() {
    let pix1 = make_gray(60, 40, 100);
    let pix2 = make_gray(40, 60, 200);
    let (r1, r2) = pix1.crop_to_match(&pix2).unwrap();
    assert_eq!(r1.width(), 40);
    assert_eq!(r1.height(), 40);
    assert_eq!(r2.width(), 40);
    assert_eq!(r2.height(), 40);
}

// ============================================================================
// pixClipToForeground
// ============================================================================

#[test]

fn test_clip_to_foreground_basic() {
    let pix = make_fg_image(100, 100, 20, 30, 40, 50);
    let result = pix.clip_to_foreground().unwrap();
    assert!(result.is_some());
    let (clipped, bbox) = result.unwrap();
    assert_eq!(bbox.x, 20);
    assert_eq!(bbox.y, 30);
    assert_eq!(bbox.w, 40);
    assert_eq!(bbox.h, 50);
    assert_eq!(clipped.width(), 40);
    assert_eq!(clipped.height(), 50);
}

#[test]

fn test_clip_to_foreground_empty() {
    let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let result = pix.clip_to_foreground().unwrap();
    assert!(result.is_none());
}

#[test]

fn test_clip_to_foreground_invalid_depth() {
    let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
    assert!(pix.clip_to_foreground().is_err());
}

// ============================================================================
// pixScanForForeground
// ============================================================================

#[test]

fn test_scan_for_foreground_from_left() {
    let pix = make_fg_image(100, 100, 30, 20, 40, 60);
    let region = Box::new(0, 0, 100, 100).unwrap();
    let loc = pix
        .scan_for_foreground(&region, ScanDirection::FromLeft)
        .unwrap();
    assert_eq!(loc, 30);
}

#[test]

fn test_scan_for_foreground_from_top() {
    let pix = make_fg_image(100, 100, 30, 20, 40, 60);
    let region = Box::new(0, 0, 100, 100).unwrap();
    let loc = pix
        .scan_for_foreground(&region, ScanDirection::FromTop)
        .unwrap();
    assert_eq!(loc, 20);
}

// ============================================================================
// pixClipBoxToForeground
// ============================================================================

#[test]

fn test_clip_box_to_foreground() {
    let pix = make_fg_image(100, 100, 20, 30, 40, 50);
    let result = pix.clip_box_to_foreground(None).unwrap();
    assert!(result.is_some());
    let (_clipped, bbox) = result.unwrap();
    assert_eq!(bbox.x, 20);
    assert_eq!(bbox.y, 30);
    assert_eq!(bbox.w, 40);
    assert_eq!(bbox.h, 50);
}

// ============================================================================
// pixMakeFrameMask
// ============================================================================

#[test]

fn test_make_frame_mask_basic() {
    let mask = Pix::make_frame_mask(100, 100, 0.1, 0.4, 0.1, 0.4).unwrap();
    assert_eq!(mask.width(), 100);
    assert_eq!(mask.height(), 100);
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    // Ring pixel should be ON, corner and center (hole) should be OFF
    assert_eq!(mask.get_pixel_unchecked(10, 10), 1); // in the ring
    assert_eq!(mask.get_pixel_unchecked(0, 0), 0); // outside outer boundary
    assert_eq!(mask.get_pixel_unchecked(50, 50), 0); // inside the hole
}

// ============================================================================
// pixFractionFgInMask
// ============================================================================

#[test]

fn test_fraction_fg_in_mask_full_overlap() {
    let pix1 = make_fg_image(50, 50, 10, 10, 30, 30);
    let pix2 = make_fg_image(50, 50, 0, 0, 50, 50);
    let fract = pix1.fraction_fg_in_mask(&pix2).unwrap();
    assert!((fract - 1.0).abs() < 0.01, "expected ~1.0, got {fract}");
}

#[test]

fn test_fraction_fg_in_mask_partial() {
    let pix1 = make_fg_image(50, 50, 0, 0, 50, 50); // all foreground
    let pix2 = make_fg_image(50, 50, 0, 0, 25, 50); // left half
    let fract = pix1.fraction_fg_in_mask(&pix2).unwrap();
    assert!((fract - 0.5).abs() < 0.01, "expected ~0.5, got {fract}");
}

// ============================================================================
// pixAverageOnLine
// ============================================================================

#[test]

fn test_average_on_line_horizontal() {
    let pix = make_gray(100, 100, 128);
    let avg = pix.average_on_line(0, 50, 99, 50, 1).unwrap();
    assert!((avg - 128.0).abs() < 0.5, "expected ~128, got {avg}");
}

#[test]

fn test_average_on_line_vertical() {
    let pix = make_gray(100, 100, 200);
    let avg = pix.average_on_line(50, 0, 50, 99, 1).unwrap();
    assert!((avg - 200.0).abs() < 0.5, "expected ~200, got {avg}");
}

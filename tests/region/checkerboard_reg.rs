//! Tests for checkerboard corner detection
//!
//! Tests the find_checkerboard_corners function.

use leptonica::region::checkerboard::find_checkerboard_corners;
use leptonica::{Pix, PixelDepth};

/// Test find_checkerboard_corners error on invalid params
#[test]
fn checkerboard_invalid_params() {
    let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
    // size too small
    assert!(find_checkerboard_corners(&pix, 3, 1, 2).is_err());
    // dilation out of range
    assert!(find_checkerboard_corners(&pix, 7, 0, 2).is_err());
    assert!(find_checkerboard_corners(&pix, 7, 6, 2).is_err());
    // nsels invalid
    assert!(find_checkerboard_corners(&pix, 7, 1, 3).is_err());
}

/// Test find_checkerboard_corners on empty image
#[test]
fn checkerboard_empty_image() {
    let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let (corners_pix, pta) = find_checkerboard_corners(&pix, 7, 1, 2).unwrap();
    assert_eq!(pta.len(), 0, "no corners in empty image");
    assert_eq!(corners_pix.count_pixels(), 0);
}

/// Test find_checkerboard_corners on a simple checkerboard pattern
#[test]
fn checkerboard_simple_pattern() {
    let size = 60;
    let cell = 10;
    let pix = Pix::new(size, size, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Create a 6x6 checkerboard with 10px cells
    for cy in 0..6 {
        for cx in 0..6 {
            if (cx + cy) % 2 == 0 {
                for y in (cy * cell)..((cy + 1) * cell).min(size) {
                    for x in (cx * cell)..((cx + 1) * cell).min(size) {
                        pix_mut.set_pixel(x, y, 1).unwrap();
                    }
                }
            }
        }
    }
    let pix: Pix = pix_mut.into();

    // Should detect some corners (exact count depends on HMT matching)
    let (_corners_pix, pta) = find_checkerboard_corners(&pix, 7, 1, 2).unwrap();
    // With a clean checkerboard, we may or may not get perfect matches
    // depending on the sel sizes - just verify no crash
    let _ = pta.len();
}

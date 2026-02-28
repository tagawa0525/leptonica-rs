//! Tests for binary reduction (binreduce) module
//!
//! Tests 2x binary image reduction by subsampling.

use leptonica::morph::binreduce::{make_subsample_tab_2x, reduce_binary_2};
use leptonica::{Pix, PixelDepth};

/// Test reduce_binary_2 basic dimensions
#[test]
fn binreduce_basic_dimensions() {
    let pix = Pix::new(100, 80, PixelDepth::Bit1).unwrap();
    let result = reduce_binary_2(&pix).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 40);
    assert_eq!(result.depth(), PixelDepth::Bit1);
}

/// Test reduce_binary_2 preserves foreground
#[test]
fn binreduce_preserves_foreground() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Fill a 4x4 block at (4,4) - should map to (2,2) in reduced
    for y in 4..8 {
        for x in 4..8 {
            pix_mut.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pix_mut.into();

    let result = reduce_binary_2(&pix).unwrap();
    // The reduced pixel at (2,2) corresponds to source (4,4)
    assert_eq!(result.get_pixel(2, 2).unwrap(), 1);
    // Pixel at (0,0) was empty
    assert_eq!(result.get_pixel(0, 0).unwrap(), 0);
}

/// Test reduce_binary_2 with odd dimensions
#[test]
fn binreduce_odd_dimensions() {
    let pix = Pix::new(11, 9, PixelDepth::Bit1).unwrap();
    let result = reduce_binary_2(&pix).unwrap();
    assert_eq!(result.width(), 5);
    assert_eq!(result.height(), 4);
}

/// Test reduce_binary_2 error on non-1bpp
#[test]
fn binreduce_error_on_wrong_depth() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    assert!(reduce_binary_2(&pix).is_err());
}

/// Test reduce_binary_2 error on height 1
#[test]
fn binreduce_error_on_height_1() {
    let pix = Pix::new(20, 1, PixelDepth::Bit1).unwrap();
    assert!(reduce_binary_2(&pix).is_err());
}

/// Test make_subsample_tab_2x produces correct size
#[test]
fn binreduce_subsample_tab_size() {
    let tab = make_subsample_tab_2x();
    assert_eq!(tab.len(), 256);
}

/// Test make_subsample_tab_2x preserves identity
#[test]
fn binreduce_subsample_tab_zero() {
    let tab = make_subsample_tab_2x();
    assert_eq!(tab[0], 0);
}

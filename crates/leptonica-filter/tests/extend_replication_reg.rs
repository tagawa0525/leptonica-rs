//! Test pixExtendByReplication()
//!
//! C版: reference/leptonica/src/adaptmap.c:pixExtendByReplication()

use leptonica_core::{Pix, PixelDepth};
use leptonica_filter::extend_by_replication;

#[test]
fn test_extend_by_replication_basic() {
    // Create a 3x3 test image
    let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Set distinct values
    pix_mut.set_pixel_unchecked(0, 0, 10);
    pix_mut.set_pixel_unchecked(1, 0, 20);
    pix_mut.set_pixel_unchecked(2, 0, 30);
    pix_mut.set_pixel_unchecked(0, 1, 40);
    pix_mut.set_pixel_unchecked(1, 1, 50);
    pix_mut.set_pixel_unchecked(2, 1, 60);
    pix_mut.set_pixel_unchecked(0, 2, 70);
    pix_mut.set_pixel_unchecked(1, 2, 80);
    pix_mut.set_pixel_unchecked(2, 2, 90);

    let pix: Pix = pix_mut.into();

    // Extend by 1 pixel in each direction
    let result = extend_by_replication(&pix, 1, 1).unwrap();

    // Result should be 5x5
    assert_eq!(result.width(), 5);
    assert_eq!(result.height(), 5);

    // Check that edges are replicated
    // Top-left corner should be replicated from (0,0)
    assert_eq!(result.get_pixel_unchecked(0, 0), 10);
    // Top edge should be replicated from row 0
    assert_eq!(result.get_pixel_unchecked(2, 0), 20);
    // Left edge should be replicated from column 0
    assert_eq!(result.get_pixel_unchecked(0, 2), 40);
    // Center should be unchanged (but shifted by extend_x, extend_y)
    assert_eq!(result.get_pixel_unchecked(2, 2), 50);
    // Right edge should be replicated from column 2
    assert_eq!(result.get_pixel_unchecked(4, 2), 60);
    // Bottom edge should be replicated from row 2
    assert_eq!(result.get_pixel_unchecked(2, 4), 80);
}

#[test]
fn test_extend_by_replication_32bpp() {
    use leptonica_core::color;

    // Test with 32bpp RGB image
    let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    pix_mut.set_pixel_unchecked(0, 0, color::compose_rgb(255, 0, 0));
    pix_mut.set_pixel_unchecked(1, 0, color::compose_rgb(0, 255, 0));
    pix_mut.set_pixel_unchecked(0, 1, color::compose_rgb(0, 0, 255));
    pix_mut.set_pixel_unchecked(1, 1, color::compose_rgb(255, 255, 0));

    let pix: Pix = pix_mut.into();

    let result = extend_by_replication(&pix, 2, 2).unwrap();

    // Result should be 6x6
    assert_eq!(result.width(), 6);
    assert_eq!(result.height(), 6);
    assert_eq!(result.depth(), PixelDepth::Bit32);

    // Check corner replication
    assert_eq!(
        result.get_pixel_unchecked(0, 0),
        color::compose_rgb(255, 0, 0)
    );
    assert_eq!(
        result.get_pixel_unchecked(5, 0),
        color::compose_rgb(0, 255, 0)
    );
}

#[test]
fn test_extend_by_replication_zero_extend() {
    // Zero extension should return a clone
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
    let result = extend_by_replication(&pix, 0, 0).unwrap();

    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
}

#[test]
fn test_extend_by_replication_colormap_preserved() {
    use leptonica_core::colormap::{PixColormap, RgbaQuad};

    // Create 8bpp image with colormap
    let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad::rgb(255, 0, 0)).unwrap(); // index 0: red
    cmap.add_color(RgbaQuad::rgb(0, 255, 0)).unwrap(); // index 1: green
    pix_mut.set_colormap(Some(cmap)).unwrap();

    // Set pixel values (colormap indices)
    for y in 0..3 {
        for x in 0..3 {
            pix_mut.set_pixel_unchecked(x, y, x % 2);
        }
    }

    let pix: Pix = pix_mut.into();
    assert!(pix.has_colormap());

    let result = extend_by_replication(&pix, 1, 1).unwrap();

    // Colormap should be preserved
    assert!(result.has_colormap());
    let result_cmap = result.colormap().unwrap();
    assert_eq!(result_cmap.len(), 2);
}

#[test]
fn test_extend_by_replication_overflow_protection() {
    let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();

    // Extremely large extension should return error, not panic
    let result = extend_by_replication(&pix, u32::MAX / 2, 1);
    assert!(result.is_err());
}

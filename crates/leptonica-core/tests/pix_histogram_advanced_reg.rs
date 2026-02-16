//! Test advanced histogram and tile-based statistics functions
//!
//! # See also
//!
//! C Leptonica: `pix4.c`
//! - pixGetGrayHistogramTiled, pixGetCmapHistogram, pixCountRGBColors
//! - pixGetAverageMasked, pixGetAverageMaskedRGB
//! - pixGetAverageTiled, pixGetAverageTiledRGB
//! - pixGetRankValueMasked, pixGetRankValueMaskedRGB

use leptonica_core::pix::statistics::PixelStatType;
use leptonica_core::{Numaa, Pix, PixColormap, PixelDepth, RgbaQuad, color};

/// Create a uniform 8bpp image
fn make_uniform_gray(val: u32, w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a gradient 8bpp image (0..255 across width)
fn make_gradient_gray(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = ((x as f32 / w as f32) * 255.0) as u32;
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a uniform 32bpp RGB image
fn make_uniform_rgb(r: u8, g: u8, b: u8, w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let pixel = color::compose_rgb(r, g, b);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

/// Create a 1bpp mask with left half set
fn make_left_half_mask(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w / 2 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

/// Create a colormapped image
fn make_cmap_image() -> Pix {
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_color(RgbaQuad {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    })
    .unwrap();
    cmap.add_color(RgbaQuad {
        red: 128,
        green: 128,
        blue: 128,
        alpha: 255,
    })
    .unwrap();
    cmap.add_color(RgbaQuad {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    })
    .unwrap();

    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..20 {
        for x in 0..20 {
            let idx = if x < 7 {
                0
            } else if x < 14 {
                1
            } else {
                2
            };
            pm.set_pixel_unchecked(x, y, idx);
        }
    }
    pm.set_colormap(Some(cmap)).unwrap();
    pm.into()
}

// ============================================================================
// pixGetGrayHistogramTiled
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_gray_histogram_tiled_basic() {
    let pix = make_uniform_gray(100, 40, 40);
    let result: Numaa = pix.gray_histogram_tiled(1, 2, 2).unwrap();
    // 2x2 tiles = 4 histograms
    assert_eq!(result.len(), 4);
}

#[test]
#[ignore = "not yet implemented"]
fn test_gray_histogram_tiled_values() {
    let pix = make_uniform_gray(100, 40, 40);
    let result = pix.gray_histogram_tiled(1, 2, 2).unwrap();
    // Each tile is 20x20 = 400 pixels, all value 100
    let hist = result.get(0).unwrap();
    assert_eq!(hist.len(), 256);
    assert_eq!(hist.get(100).unwrap(), 400.0);
    assert_eq!(hist.get(0).unwrap(), 0.0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_gray_histogram_tiled_invalid_depth() {
    let pix = Pix::new(40, 40, PixelDepth::Bit32).unwrap();
    assert!(pix.gray_histogram_tiled(1, 2, 2).is_err());
}

// ============================================================================
// pixGetCmapHistogram
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_cmap_histogram_basic() {
    let pix = make_cmap_image();
    let hist = pix.cmap_histogram(1).unwrap();
    // 8bpp colormap → 256-bin histogram of indices
    assert_eq!(hist.len(), 256);
    // Index 0: 7 cols * 20 rows = 140
    assert_eq!(hist.get(0).unwrap(), 140.0);
    // Index 1: 7 cols * 20 rows = 140
    assert_eq!(hist.get(1).unwrap(), 140.0);
    // Index 2: 6 cols * 20 rows = 120
    assert_eq!(hist.get(2).unwrap(), 120.0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_cmap_histogram_no_colormap() {
    let pix = make_uniform_gray(100, 20, 20);
    assert!(pix.cmap_histogram(1).is_err());
}

// ============================================================================
// pixCountRGBColors
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_count_rgb_colors_uniform() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let count = pix.count_rgb_colors(1).unwrap();
    assert_eq!(count, 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_count_rgb_colors_gradient() {
    // Create image with distinct colors
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..10 {
        for x in 0..10 {
            pm.set_pixel_unchecked(x, y, color::compose_rgb(x as u8 * 25, y as u8 * 25, 0));
        }
    }
    let pix: Pix = pm.into();
    let count = pix.count_rgb_colors(1).unwrap();
    assert_eq!(count, 100);
}

#[test]
#[ignore = "not yet implemented"]
fn test_count_rgb_colors_invalid_depth() {
    let pix = make_uniform_gray(100, 20, 20);
    assert!(pix.count_rgb_colors(1).is_err());
}

// ============================================================================
// pixGetAverageMasked
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_average_masked_no_mask() {
    let pix = make_uniform_gray(120, 20, 20);
    let val = pix
        .average_masked(None, 0, 0, 1, PixelStatType::MeanAbsVal)
        .unwrap();
    assert!((val - 120.0).abs() < 0.5);
}

#[test]
#[ignore = "not yet implemented"]
fn test_average_masked_with_mask() {
    // Left half = 50, right half = 200
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..20 {
        for x in 0..20 {
            let val = if x < 10 { 50u32 } else { 200 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let mask = make_left_half_mask(20, 20);

    // With mask covering left half only
    let val = pix
        .average_masked(Some(&mask), 0, 0, 1, PixelStatType::MeanAbsVal)
        .unwrap();
    assert!((val - 50.0).abs() < 0.5, "expected ~50, got {val}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_average_masked_variance() {
    let pix = make_uniform_gray(100, 20, 20);
    let val = pix
        .average_masked(None, 0, 0, 1, PixelStatType::Variance)
        .unwrap();
    assert!(
        val.abs() < 0.001,
        "uniform image variance should be 0, got {val}"
    );
}

// ============================================================================
// pixGetAverageMaskedRGB
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_average_masked_rgb_no_mask() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let (r, g, b) = pix
        .average_masked_rgb(None, 0, 0, 1, PixelStatType::MeanAbsVal)
        .unwrap();
    assert!((r - 100.0).abs() < 0.5, "r: expected 100, got {r}");
    assert!((g - 150.0).abs() < 0.5, "g: expected 150, got {g}");
    assert!((b - 200.0).abs() < 0.5, "b: expected 200, got {b}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_average_masked_rgb_invalid_depth() {
    let pix = make_uniform_gray(100, 20, 20);
    assert!(
        pix.average_masked_rgb(None, 0, 0, 1, PixelStatType::MeanAbsVal)
            .is_err()
    );
}

// ============================================================================
// pixGetAverageTiled
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_average_tiled_uniform() {
    let pix = make_uniform_gray(150, 40, 40);
    let result = pix
        .average_tiled(10, 10, PixelStatType::MeanAbsVal)
        .unwrap();
    // Output: 40/10 = 4 x 40/10 = 4
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
    assert_eq!(result.depth(), PixelDepth::Bit8);
    assert_eq!(result.get_pixel_unchecked(0, 0), 150);
}

#[test]
#[ignore = "not yet implemented"]
fn test_average_tiled_invalid_depth() {
    let pix = Pix::new(40, 40, PixelDepth::Bit32).unwrap();
    assert!(
        pix.average_tiled(10, 10, PixelStatType::MeanAbsVal)
            .is_err()
    );
}

// ============================================================================
// pixGetAverageTiledRGB
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_average_tiled_rgb_uniform() {
    let pix = make_uniform_rgb(100, 150, 200, 40, 40);
    let (pr, pg, pb) = pix
        .average_tiled_rgb(10, 10, PixelStatType::MeanAbsVal)
        .unwrap();
    assert_eq!(pr.width(), 4);
    assert_eq!(pr.height(), 4);
    assert_eq!(pr.get_pixel_unchecked(0, 0), 100);
    assert_eq!(pg.get_pixel_unchecked(0, 0), 150);
    assert_eq!(pb.get_pixel_unchecked(0, 0), 200);
}

#[test]
#[ignore = "not yet implemented"]
fn test_average_tiled_rgb_invalid_depth() {
    let pix = make_uniform_gray(100, 40, 40);
    assert!(
        pix.average_tiled_rgb(10, 10, PixelStatType::MeanAbsVal)
            .is_err()
    );
}

// ============================================================================
// pixGetRankValueMasked
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_rank_value_masked_median() {
    let pix = make_uniform_gray(128, 20, 20);
    let (val, _hist) = pix.rank_value_masked(None, 0, 0, 1, 0.5).unwrap();
    assert!((val - 128.0).abs() < 1.0, "expected ~128, got {val}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_rank_value_masked_min_max() {
    let pix = make_gradient_gray(100, 10);
    let (min_val, _) = pix.rank_value_masked(None, 0, 0, 1, 0.0).unwrap();
    let (max_val, _) = pix.rank_value_masked(None, 0, 0, 1, 1.0).unwrap();
    assert!(min_val < 5.0, "min should be near 0, got {min_val}");
    assert!(max_val > 250.0, "max should be near 255, got {max_val}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_rank_value_masked_invalid_rank() {
    let pix = make_uniform_gray(100, 20, 20);
    assert!(pix.rank_value_masked(None, 0, 0, 1, -0.1).is_err());
    assert!(pix.rank_value_masked(None, 0, 0, 1, 1.1).is_err());
}

// ============================================================================
// pixGetRankValueMaskedRGB
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_rank_value_masked_rgb_uniform() {
    let pix = make_uniform_rgb(100, 150, 200, 20, 20);
    let (r, g, b) = pix.rank_value_masked_rgb(None, 0, 0, 1, 0.5).unwrap();
    assert!((r - 100.0).abs() < 1.0, "r: expected 100, got {r}");
    assert!((g - 150.0).abs() < 1.0, "g: expected 150, got {g}");
    assert!((b - 200.0).abs() < 1.0, "b: expected 200, got {b}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_rank_value_masked_rgb_invalid_depth() {
    let pix = make_uniform_gray(100, 20, 20);
    assert!(pix.rank_value_masked_rgb(None, 0, 0, 1, 0.5).is_err());
}

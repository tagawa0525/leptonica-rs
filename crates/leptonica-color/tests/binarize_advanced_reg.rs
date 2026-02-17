//! Test advanced binarization and quantization functions
//!
//! # See also
//!
//! C Leptonica: `binarize.c`, `grayquant.c`
//! - pixVarThresholdToBinary, pixGenerateMaskByValue, pixGenerateMaskByBand
//! - pixThresholdTo2bpp, pixThresholdTo4bpp
//! - pixOtsuAdaptiveThreshold, pixSauvolaBinarizeTiled

use leptonica_color::threshold::{
    generate_mask_by_band, generate_mask_by_value, otsu_adaptive_threshold, sauvola_binarize_tiled,
    threshold_to_2bpp, threshold_to_4bpp, var_threshold_to_binary,
};
use leptonica_core::{Pix, PixelDepth};

/// Create an 8bpp gradient image (0..255 across width)
fn make_gradient_8bpp(w: u32, h: u32) -> Pix {
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

/// Create a uniform 8bpp image
fn make_uniform_8bpp(w: u32, h: u32, val: u32) -> Pix {
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
// var_threshold_to_binary
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_var_threshold_all_below() {
    // Image val=100, threshold=200 → all foreground (1)
    let pix = make_uniform_8bpp(20, 20, 100);
    let thresh = make_uniform_8bpp(20, 20, 200);
    let binary = var_threshold_to_binary(&pix, &thresh).unwrap();
    assert_eq!(binary.depth(), PixelDepth::Bit1);
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| binary.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
#[ignore = "not yet implemented"]
fn test_var_threshold_all_above() {
    // Image val=200, threshold=100 → all background (0)
    let pix = make_uniform_8bpp(20, 20, 200);
    let thresh = make_uniform_8bpp(20, 20, 100);
    let binary = var_threshold_to_binary(&pix, &thresh).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| binary.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

#[test]
#[ignore = "not yet implemented"]
fn test_var_threshold_gradient() {
    let pix = make_gradient_8bpp(256, 10);
    let thresh = make_uniform_8bpp(256, 10, 128);
    let binary = var_threshold_to_binary(&pix, &thresh).unwrap();
    // Pixels with val < 128 should be ON (foreground)
    let val_0 = binary.get_pixel_unchecked(0, 0);
    assert_eq!(val_0, 1); // val=0 < 128
    let val_255 = binary.get_pixel_unchecked(255, 0);
    assert_eq!(val_255, 0); // val=255 >= 128
}

// ============================================================================
// generate_mask_by_value
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_mask_by_value_match() {
    let pix = make_uniform_8bpp(20, 20, 128);
    let mask = generate_mask_by_value(&pix, 128).unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
#[ignore = "not yet implemented"]
fn test_mask_by_value_no_match() {
    let pix = make_uniform_8bpp(20, 20, 128);
    let mask = generate_mask_by_value(&pix, 64).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

// ============================================================================
// generate_mask_by_band
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_mask_by_band_in_range() {
    let pix = make_uniform_8bpp(20, 20, 128);
    let mask = generate_mask_by_band(&pix, 100, 200, true).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400); // 128 is in [100, 200]
}

#[test]
#[ignore = "not yet implemented"]
fn test_mask_by_band_out_of_range() {
    let pix = make_uniform_8bpp(20, 20, 128);
    let mask = generate_mask_by_band(&pix, 100, 200, false).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0); // 128 is in [100, 200], so excluded
}

#[test]
#[ignore = "not yet implemented"]
fn test_mask_by_band_gradient() {
    let pix = make_gradient_8bpp(256, 1);
    let mask = generate_mask_by_band(&pix, 100, 200, true).unwrap();
    // Count should be approximately 101 pixels (values 100..=200)
    let on_count: u32 = (0..256).map(|x| mask.get_pixel_unchecked(x, 0)).sum();
    assert!(on_count > 90 && on_count < 110, "on_count = {on_count}");
}

// ============================================================================
// threshold_to_2bpp
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_2bpp_2levels() {
    let pix = make_gradient_8bpp(256, 1);
    let quantized = threshold_to_2bpp(&pix, 2, false).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit2);
    // With 2 levels: values < 128 → 0, values >= 128 → 1
    assert_eq!(quantized.get_pixel_unchecked(0, 0), 0);
    assert_eq!(quantized.get_pixel_unchecked(255, 0), 1);
}

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_2bpp_4levels() {
    let pix = make_gradient_8bpp(256, 1);
    let quantized = threshold_to_2bpp(&pix, 4, false).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit2);
    // With 4 levels, max value = 3
    let max_val = (0..256)
        .map(|x| quantized.get_pixel_unchecked(x, 0))
        .max()
        .unwrap();
    assert_eq!(max_val, 3);
}

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_2bpp_invalid_levels() {
    let pix = make_gradient_8bpp(256, 1);
    assert!(threshold_to_2bpp(&pix, 1, false).is_err());
    assert!(threshold_to_2bpp(&pix, 5, false).is_err());
}

// ============================================================================
// threshold_to_4bpp
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_4bpp_4levels() {
    let pix = make_gradient_8bpp(256, 1);
    let quantized = threshold_to_4bpp(&pix, 4, false).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit4);
    let max_val = (0..256)
        .map(|x| quantized.get_pixel_unchecked(x, 0))
        .max()
        .unwrap();
    assert_eq!(max_val, 3);
}

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_4bpp_16levels() {
    let pix = make_gradient_8bpp(256, 1);
    let quantized = threshold_to_4bpp(&pix, 16, false).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit4);
    let max_val = (0..256)
        .map(|x| quantized.get_pixel_unchecked(x, 0))
        .max()
        .unwrap();
    assert_eq!(max_val, 15);
}

#[test]
#[ignore = "not yet implemented"]
fn test_threshold_to_4bpp_invalid_levels() {
    let pix = make_gradient_8bpp(256, 1);
    assert!(threshold_to_4bpp(&pix, 1, false).is_err());
    assert!(threshold_to_4bpp(&pix, 17, false).is_err());
}

// ============================================================================
// otsu_adaptive_threshold
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_otsu_adaptive_basic() {
    // Image: left half dark (30), right half bright (220)
    let pix = Pix::new(200, 100, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..200u32 {
            let val = if x < 100 { 30 } else { 220 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let (thresh_map, binary) = otsu_adaptive_threshold(&pix, 50, 50, 0, 0, 0.0).unwrap();
    assert_eq!(thresh_map.depth(), PixelDepth::Bit8);
    assert_eq!(binary.depth(), PixelDepth::Bit1);
    // Dark pixels should be foreground (ON), bright pixels background (OFF)
    // At (10, 50) dark side
    let dark_px = binary.get_pixel_unchecked(10, 50);
    // At (150, 50) bright side
    let bright_px = binary.get_pixel_unchecked(150, 50);
    assert_ne!(dark_px, bright_px, "dark and bright regions should differ");
}

// ============================================================================
// sauvola_binarize_tiled
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_sauvola_tiled_basic() {
    // Create a gradient image
    let pix = make_gradient_8bpp(200, 100);
    let (thresh_map, binary) = sauvola_binarize_tiled(&pix, 7, 0.35, 2, 2).unwrap();
    assert_eq!(thresh_map.depth(), PixelDepth::Bit8);
    assert_eq!(binary.depth(), PixelDepth::Bit1);
    assert_eq!(binary.width(), 200);
    assert_eq!(binary.height(), 100);
}

#[test]
#[ignore = "not yet implemented"]
fn test_sauvola_tiled_single_tile() {
    // nx=1, ny=1 should work like non-tiled version
    let pix = make_gradient_8bpp(100, 100);
    let (_thresh_map, binary) = sauvola_binarize_tiled(&pix, 7, 0.35, 1, 1).unwrap();
    assert_eq!(binary.depth(), PixelDepth::Bit1);
    assert_eq!(binary.width(), 100);
}

//! Test extended quantization functions
//!
//! # See also
//!
//! C Leptonica: `colorquant1.c`, `colorquant2.c`
//! - pixFixedOctcubeQuant256, pixOctreeQuantByPopulation
//! - pixOctreeQuantNumColors, pixMedianCutQuantMixed
//! - pixQuantFromCmap, pixRemoveUnusedColors

use leptonica_color::quantize::{
    fixed_octcube_quant_256, median_cut_quant_mixed, octree_quant_by_population,
    octree_quant_num_colors, quant_from_cmap, remove_unused_colors,
};
use leptonica_core::{Pix, PixColormap, PixelDepth, color};

/// Create a tricolor image (red/green/blue vertical stripes)
fn make_tricolor(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let third = w / 3;
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < third {
                color::compose_rgb(255, 0, 0)
            } else if x < 2 * third {
                color::compose_rgb(0, 255, 0)
            } else {
                color::compose_rgb(0, 0, 255)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

/// Create a color gradient image
fn make_color_gradient(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let r = ((x as f32 / w as f32) * 255.0) as u8;
            let g = ((y as f32 / h as f32) * 255.0) as u8;
            let b = 128u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

/// Create a mixed gray/color image
fn make_mixed_gray_color(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let half = w / 2;
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < half {
                // Left half: grayscale gradient
                let g = ((y as f32 / h as f32) * 255.0) as u8;
                color::compose_rgb(g, g, g)
            } else {
                // Right half: color
                color::compose_rgb(255, 0, 0)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

// ============================================================================
// fixed_octcube_quant_256
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_fixed_octcube_quant_256_basic() {
    let pix = make_color_gradient(64, 64);
    let quantized = fixed_octcube_quant_256(&pix).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit8);
    assert!(quantized.colormap().is_some());
    let cmap = quantized.colormap().unwrap();
    assert_eq!(cmap.len(), 256);
}

#[test]
#[ignore = "not yet implemented"]
fn test_fixed_octcube_quant_256_tricolor() {
    let pix = make_tricolor(30, 10);
    let quantized = fixed_octcube_quant_256(&pix).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit8);
    assert!(quantized.colormap().is_some());
    // Check that red, green, blue are distinct in quantized output
    let left = quantized.get_pixel_unchecked(5, 5);
    let mid = quantized.get_pixel_unchecked(15, 5);
    let right = quantized.get_pixel_unchecked(25, 5);
    assert_ne!(left, mid);
    assert_ne!(mid, right);
}

#[test]
#[ignore = "not yet implemented"]
fn test_fixed_octcube_quant_256_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(fixed_octcube_quant_256(&pix).is_err());
}

// ============================================================================
// octree_quant_by_population
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_by_population_basic() {
    let pix = make_color_gradient(64, 64);
    let quantized = octree_quant_by_population(&pix, 4).unwrap();
    assert!(quantized.colormap().is_some());
    let cmap = quantized.colormap().unwrap();
    assert!(cmap.len() <= 256);
}

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_by_population_few_colors() {
    let pix = make_tricolor(30, 10);
    let quantized = octree_quant_by_population(&pix, 4).unwrap();
    assert!(quantized.colormap().is_some());
    let cmap = quantized.colormap().unwrap();
    // 3 input colors should result in ≤3 output colors
    assert!(cmap.len() <= 4, "expected ≤4 colors, got {}", cmap.len());
}

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_by_population_level3() {
    let pix = make_color_gradient(64, 64);
    let quantized = octree_quant_by_population(&pix, 3).unwrap();
    assert!(quantized.colormap().is_some());
}

// ============================================================================
// octree_quant_num_colors
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_num_colors_16() {
    let pix = make_color_gradient(64, 64);
    let quantized = octree_quant_num_colors(&pix, 16, 0).unwrap();
    assert!(quantized.colormap().is_some());
    let cmap = quantized.colormap().unwrap();
    assert!(cmap.len() <= 16, "expected ≤16 colors, got {}", cmap.len());
}

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_num_colors_64() {
    let pix = make_color_gradient(64, 64);
    let quantized = octree_quant_num_colors(&pix, 64, 0).unwrap();
    assert!(quantized.colormap().is_some());
    let cmap = quantized.colormap().unwrap();
    assert!(cmap.len() <= 64, "expected ≤64 colors, got {}", cmap.len());
}

#[test]
#[ignore = "not yet implemented"]
fn test_octree_quant_num_colors_invalid() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(octree_quant_num_colors(&pix, 16, 0).is_err());
}

// ============================================================================
// median_cut_quant_mixed
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_median_cut_quant_mixed_basic() {
    let pix = make_mixed_gray_color(100, 50);
    let quantized = median_cut_quant_mixed(&pix, 128, 64, 20, 236, 15).unwrap();
    assert_eq!(quantized.depth(), PixelDepth::Bit8);
    assert!(quantized.colormap().is_some());
}

#[test]
#[ignore = "not yet implemented"]
fn test_median_cut_quant_mixed_all_gray() {
    // Pure grayscale image
    let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..50u32 {
        for x in 0..50u32 {
            let g = ((y as f32 / 50.0) * 255.0) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(g, g, g));
        }
    }
    let pix: Pix = pm.into();
    let quantized = median_cut_quant_mixed(&pix, 128, 64, 20, 236, 15).unwrap();
    assert!(quantized.colormap().is_some());
}

// ============================================================================
// quant_from_cmap
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_quant_from_cmap_rgb() {
    let pix = make_tricolor(30, 10);
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap();
    cmap.add_rgb(0, 255, 0).unwrap();
    cmap.add_rgb(0, 0, 255).unwrap();
    let quantized = quant_from_cmap(&pix, &cmap, 2).unwrap();
    assert!(quantized.colormap().is_some());
    // Left third should map to red (index 0)
    let left = quantized.get_pixel_unchecked(5, 5);
    let mid = quantized.get_pixel_unchecked(15, 5);
    let right = quantized.get_pixel_unchecked(25, 5);
    assert_eq!(left, 0); // Red
    assert_eq!(mid, 1); // Green
    assert_eq!(right, 2); // Blue
}

#[test]
#[ignore = "not yet implemented"]
fn test_quant_from_cmap_gray() {
    // 8bpp grayscale → quantize to 4 gray levels
    let pix = Pix::new(100, 1, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for x in 0..100u32 {
        pm.set_pixel_unchecked(x, 0, (x * 255 / 99) as u32);
    }
    let pix: Pix = pm.into();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(0, 0, 0).unwrap();
    cmap.add_rgb(85, 85, 85).unwrap();
    cmap.add_rgb(170, 170, 170).unwrap();
    cmap.add_rgb(255, 255, 255).unwrap();
    let quantized = quant_from_cmap(&pix, &cmap, 2).unwrap();
    assert!(quantized.colormap().is_some());
}

#[test]
#[ignore = "not yet implemented"]
fn test_quant_from_cmap_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let cmap = PixColormap::new(8).unwrap();
    assert!(quant_from_cmap(&pix, &cmap, 2).is_err());
}

// ============================================================================
// remove_unused_colors
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_remove_unused_colors_basic() {
    // Create 8bpp image with colormap of 4 entries but only use 2
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap(); // index 0: used
    cmap.add_rgb(0, 255, 0).unwrap(); // index 1: unused
    cmap.add_rgb(0, 0, 255).unwrap(); // index 2: used
    cmap.add_rgb(255, 255, 0).unwrap(); // index 3: unused
    pm.set_colormap(Some(cmap)).unwrap();
    for y in 0..10u32 {
        for x in 0..10u32 {
            let idx = if x < 5 { 0 } else { 2 };
            pm.set_pixel_unchecked(x, y, idx);
        }
    }
    let pix: Pix = pm.into();
    let result = remove_unused_colors(&pix).unwrap();
    let cmap = result.colormap().unwrap();
    assert_eq!(cmap.len(), 2); // Only 2 colors used
}

#[test]
#[ignore = "not yet implemented"]
fn test_remove_unused_colors_all_used() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap();
    cmap.add_rgb(0, 0, 255).unwrap();
    pm.set_colormap(Some(cmap)).unwrap();
    for y in 0..10u32 {
        for x in 0..10u32 {
            pm.set_pixel_unchecked(x, y, if x < 5 { 0 } else { 1 });
        }
    }
    let pix: Pix = pm.into();
    let result = remove_unused_colors(&pix).unwrap();
    let cmap = result.colormap().unwrap();
    assert_eq!(cmap.len(), 2); // All colors used, no change
}

#[test]
#[ignore = "not yet implemented"]
fn test_remove_unused_colors_no_colormap() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    assert!(remove_unused_colors(&pix).is_err());
}

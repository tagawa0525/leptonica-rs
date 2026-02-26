//! Coverage tests for 45 unimplemented color functions
//!
//!
//! # See also
//!
//! C Leptonica: colorspace.c, colorquant1.c, colorquant2.c, colorseg.c,
//!              colorcontent.c, colorfill.c, coloring.c, binarize.c, grayquant.c

#![allow(unused_imports, unused_variables, dead_code)]

use leptonica::core::pixel;
use leptonica::{Pix, PixelDepth};

/// Create a uniform RGB image
fn make_uniform_rgb(r: u8, g: u8, b: u8, w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let val = pixel::compose_rgb(r, g, b);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a 3-color block image (red/green/blue thirds)
fn make_tricolor(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let third = w / 3;
    for y in 0..h {
        for x in 0..w {
            let val = if x < third {
                pixel::compose_rgb(255, 0, 0)
            } else if x < 2 * third {
                pixel::compose_rgb(0, 255, 0)
            } else {
                pixel::compose_rgb(0, 0, 255)
            };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a 6-color block image for quantization testing
fn make_6color(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let colors = [
        (200, 50, 50),
        (50, 200, 50),
        (50, 50, 200),
        (200, 200, 50),
        (200, 50, 200),
        (50, 200, 200),
    ];
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * 2 / h) * 3 + x * 3 / w) as usize % 6;
            let (r, g, b) = colors[idx];
            pm.set_pixel_unchecked(x, y, pixel::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

/// Create an 8bpp gradient image
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

/// Create a bimodal 8bpp image (dark + bright halves)
fn make_bimodal_8bpp(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = if x < w / 2 { 40 } else { 210 };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    pm.into()
}

/// Create a color gradient image for quantization
fn make_color_gradient(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = 128u8;
            pm.set_pixel_unchecked(x, y, pixel::compose_rgb(r, g, b));
        }
    }
    pm.into()
}

// ============================================================================
// 1. pixcmapConvertRGBToHSV (colorspace.rs)
// ============================================================================

#[test]
fn test_pix_colormap_convert_rgb_to_hsv() {
    use leptonica::color::colorspace::pix_colormap_convert_rgb_to_hsv;
    use leptonica::core::PixColormap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap(); // pure red
    cmap.add_rgb(0, 255, 0).unwrap(); // pure green
    cmap.add_rgb(0, 0, 255).unwrap(); // pure blue

    let result = pix_colormap_convert_rgb_to_hsv(&mut cmap);
    assert!(result.is_ok());
    // After conversion, entries should contain HSV values packed as (h,s,v)
    let (h, _s, _v) = cmap.get_rgb(0).unwrap();
    // Red in HSV: h=0 (or near 0), s=255, v=255 in leptonica's [0,240] hue range
    assert!(h <= 10 || h >= 230, "red hue should be near 0, got {h}");
}

// ============================================================================
// 2. pixcmapConvertHSVToRGB (colorspace.rs)
// ============================================================================

#[test]
fn test_pix_colormap_convert_hsv_to_rgb() {
    use leptonica::color::colorspace::{
        pix_colormap_convert_hsv_to_rgb, pix_colormap_convert_rgb_to_hsv,
    };
    use leptonica::core::PixColormap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(200, 100, 50).unwrap();
    cmap.add_rgb(50, 200, 100).unwrap();

    let (r0, g0, b0) = cmap.get_rgb(0).unwrap();
    pix_colormap_convert_rgb_to_hsv(&mut cmap).unwrap();
    pix_colormap_convert_hsv_to_rgb(&mut cmap).unwrap();
    let (r1, g1, b1) = cmap.get_rgb(0).unwrap();
    // Round-trip should be close
    assert!((r0 as i32 - r1 as i32).unsigned_abs() <= 2);
    assert!((g0 as i32 - g1 as i32).unsigned_abs() <= 2);
    assert!((b0 as i32 - b1 as i32).unsigned_abs() <= 2);
}

// ============================================================================
// 3. pixFindHistoPeaksHSV (colorspace.rs)
// ============================================================================

#[test]
fn test_find_histo_peaks_hsv() {
    use leptonica::color::colorspace::{HsvHistoType, find_histo_peaks_hsv, make_histo_hs};

    let pix = make_tricolor(120, 60);
    let histo = make_histo_hs(&pix, 1).unwrap();
    let result = find_histo_peaks_hsv(&histo, HsvHistoType::HS, 5, 5, 3, 1.5);
    assert!(result.is_ok());
    let (points, areas) = result.unwrap();
    assert!(!points.is_empty(), "should find at least one peak");
    assert_eq!(points.len(), areas.len());
}

// ============================================================================
// 4. pixConvertRGBToXYZ (colorspace.rs)
// ============================================================================

#[test]
fn test_pix_convert_rgb_to_xyz() {
    use leptonica::color::colorspace::pix_convert_rgb_to_xyz;
    use leptonica::core::FPix;

    let pix = make_uniform_rgb(128, 64, 32, 20, 20);
    let result = pix_convert_rgb_to_xyz(&pix);
    assert!(result.is_ok());
    let (fx, fy, fz) = result.unwrap();
    assert_eq!(fx.width(), 20);
    assert_eq!(fx.height(), 20);
    assert_eq!(fy.width(), 20);
    assert_eq!(fz.width(), 20);
    // X, Y, Z values should be positive for non-black pixels
    assert!(fx.get_pixel(0, 0).unwrap() > 0.0);
    assert!(fy.get_pixel(0, 0).unwrap() > 0.0);
    assert!(fz.get_pixel(0, 0).unwrap() > 0.0);
}

// ============================================================================
// 5. fpixaConvertXYZToRGB (colorspace.rs)
// ============================================================================

#[test]
fn test_fpixa_convert_xyz_to_rgb() {
    use leptonica::color::colorspace::{fpixa_convert_xyz_to_rgb, pix_convert_rgb_to_xyz};

    let pix = make_uniform_rgb(180, 90, 45, 20, 20);
    let (fx, fy, fz) = pix_convert_rgb_to_xyz(&pix).unwrap();
    let result = fpixa_convert_xyz_to_rgb(&fx, &fy, &fz);
    assert!(result.is_ok());
    let pix2 = result.unwrap();
    assert_eq!(pix2.width(), 20);
    assert_eq!(pix2.depth(), PixelDepth::Bit32);
    // Round-trip should be close
    let (r, g, b) = pixel::extract_rgb(pix2.get_pixel_unchecked(0, 0));
    assert!((r as i32 - 180).unsigned_abs() <= 2);
    assert!((g as i32 - 90).unsigned_abs() <= 2);
    assert!((b as i32 - 45).unsigned_abs() <= 2);
}

// ============================================================================
// 6. pixConvertRGBToLAB (colorspace.rs)
// ============================================================================

#[test]
fn test_pix_convert_rgb_to_lab() {
    use leptonica::color::colorspace::pix_convert_rgb_to_lab;

    let pix = make_uniform_rgb(128, 64, 32, 20, 20);
    let result = pix_convert_rgb_to_lab(&pix);
    assert!(result.is_ok());
    let (fl, fa, fb) = result.unwrap();
    assert_eq!(fl.width(), 20);
    assert_eq!(fl.height(), 20);
    // L should be in [0, 100] range for a normal pixel
    let l_val = fl.get_pixel(0, 0).unwrap();
    assert!(
        (0.0..=100.0).contains(&l_val),
        "L value out of range: {l_val}"
    );
}

// ============================================================================
// 7. fpixaConvertLABToRGB (colorspace.rs)
// ============================================================================

#[test]
fn test_fpixa_convert_lab_to_rgb() {
    use leptonica::color::colorspace::{fpixa_convert_lab_to_rgb, pix_convert_rgb_to_lab};

    let pix = make_uniform_rgb(180, 90, 45, 20, 20);
    let (fl, fa, fb) = pix_convert_rgb_to_lab(&pix).unwrap();
    let result = fpixa_convert_lab_to_rgb(&fl, &fa, &fb);
    assert!(result.is_ok());
    let pix2 = result.unwrap();
    let (r, g, b) = pixel::extract_rgb(pix2.get_pixel_unchecked(0, 0));
    assert!((r as i32 - 180).unsigned_abs() <= 2);
    assert!((g as i32 - 90).unsigned_abs() <= 2);
    assert!((b as i32 - 45).unsigned_abs() <= 2);
}

// ============================================================================
// 8-9. pixcmapConvertRGBToYUV / pixcmapConvertYUVToRGB (colorspace.rs)
// ============================================================================

#[test]
fn test_pix_colormap_convert_rgb_yuv_roundtrip() {
    use leptonica::color::colorspace::{
        pix_colormap_convert_rgb_to_yuv, pix_colormap_convert_yuv_to_rgb,
    };
    use leptonica::core::PixColormap;

    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(200, 100, 50).unwrap();
    let (r0, g0, b0) = cmap.get_rgb(0).unwrap();
    pix_colormap_convert_rgb_to_yuv(&mut cmap).unwrap();
    pix_colormap_convert_yuv_to_rgb(&mut cmap).unwrap();
    let (r1, g1, b1) = cmap.get_rgb(0).unwrap();
    assert!((r0 as i32 - r1 as i32).unsigned_abs() <= 2);
    assert!((g0 as i32 - g1 as i32).unsigned_abs() <= 2);
    assert!((b0 as i32 - b1 as i32).unsigned_abs() <= 2);
}

// ============================================================================
// 10-11. fpixaConvertXYZToLAB / fpixaConvertLABToXYZ (colorspace.rs)
// ============================================================================

#[test]
fn test_fpixa_convert_xyz_lab_roundtrip() {
    use leptonica::color::colorspace::{
        fpixa_convert_lab_to_xyz, fpixa_convert_xyz_to_lab, pix_convert_rgb_to_xyz,
    };

    let pix = make_uniform_rgb(128, 64, 32, 10, 10);
    let (fx, fy, fz) = pix_convert_rgb_to_xyz(&pix).unwrap();
    let (fl, fa, fb) = fpixa_convert_xyz_to_lab(&fx, &fy, &fz).unwrap();
    let (fx2, fy2, fz2) = fpixa_convert_lab_to_xyz(&fl, &fa, &fb).unwrap();
    // Round-trip should be close
    let diff_x = (fx.get_pixel(0, 0).unwrap() - fx2.get_pixel(0, 0).unwrap()).abs();
    assert!(diff_x < 0.1, "X round-trip diff too large: {diff_x}");
}

// ============================================================================
// 12. pixOctcubeQuantMixedWithGray (quantize.rs)
// ============================================================================

#[test]
fn test_octcube_quant_mixed_with_gray() {
    use leptonica::color::quantize::octcube_quant_mixed_with_gray;

    let pix = make_color_gradient(100, 80);
    let result = octcube_quant_mixed_with_gray(&pix, 8, 4, 40);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit8);
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 13. pixFewColorsOctcubeQuant1 (quantize.rs)
// ============================================================================

#[test]
fn test_few_colors_octcube_quant1() {
    use leptonica::color::quantize::few_colors_octcube_quant1;

    let pix = make_6color(120, 90);
    let result = few_colors_octcube_quant1(&pix, 3);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
    let ncolors = pixd.colormap().unwrap().len();
    assert!(ncolors <= 256, "too many colors: {ncolors}");
}

// ============================================================================
// 14. pixFewColorsOctcubeQuant2 (quantize.rs)
// ============================================================================

#[test]
fn test_few_colors_octcube_quant2() {
    use leptonica::color::quantize::few_colors_octcube_quant2;

    let pix = make_6color(120, 90);
    let result = few_colors_octcube_quant2(&pix, 3, 6);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 15. pixFewColorsOctcubeQuantMixed (quantize.rs)
// ============================================================================

#[test]
fn test_few_colors_octcube_quant_mixed() {
    use leptonica::color::quantize::few_colors_octcube_quant_mixed;

    let pix = make_6color(120, 90);
    let result = few_colors_octcube_quant_mixed(&pix, 3, 10);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 16. pixFixedOctcubeQuantGenRGB (quantize.rs)
// ============================================================================

#[test]
fn test_fixed_octcube_quant_gen_rgb() {
    use leptonica::color::quantize::fixed_octcube_quant_gen_rgb;

    let pix = make_color_gradient(100, 80);
    let result = fixed_octcube_quant_gen_rgb(&pix, 2);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit32);
    assert_eq!(pixd.width(), 100);
}

// ============================================================================
// 17. pixOctcubeQuantFromCmap (quantize.rs)
// ============================================================================

#[test]
fn test_octcube_quant_from_cmap() {
    use leptonica::color::quantize::octcube_quant_from_cmap;
    use leptonica::core::PixColormap;

    let pix = make_tricolor(60, 60);
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap();
    cmap.add_rgb(0, 255, 0).unwrap();
    cmap.add_rgb(0, 0, 255).unwrap();
    let result = octcube_quant_from_cmap(&pix, &cmap, 2);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 18. pixOctcubeQuantFromCmapLUT (quantize.rs)
// ============================================================================

#[test]
fn test_octcube_quant_from_cmap_lut() {
    use leptonica::color::quantize::octcube_quant_from_cmap_lut;
    use leptonica::core::PixColormap;

    let pix = make_tricolor(60, 60);
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(255, 0, 0).unwrap();
    cmap.add_rgb(0, 255, 0).unwrap();
    cmap.add_rgb(0, 0, 255).unwrap();
    let result = octcube_quant_from_cmap_lut(&pix, &cmap, 2);
    assert!(result.is_ok());
}

// ============================================================================
// 19. pixOctcubeTree (quantize.rs)
// ============================================================================

#[test]
fn test_octcube_tree() {
    use leptonica::color::quantize::octcube_tree;

    let pix = make_6color(60, 60);
    let result = octcube_tree(&pix, 3);
    assert!(result.is_ok());
    let tree = result.unwrap();
    // Should have histogram data about occupied octcubes
    assert!(!tree.histogram.is_empty());
}

// ============================================================================
// 20. pixNumberOccupiedOctcubes (quantize.rs)
// ============================================================================

#[test]
fn test_number_occupied_octcubes() {
    use leptonica::color::quantize::number_occupied_octcubes;

    let pix = make_6color(60, 60);
    let result = number_occupied_octcubes(&pix, 3, 0);
    assert!(result.is_ok());
    let count = result.unwrap();
    assert!((3..=512).contains(&count), "unexpected count: {count}");
}

// ============================================================================
// 21. pixFewColorsMedianCutQuantMixed (quantize.rs)
// ============================================================================

#[test]
fn test_few_colors_median_cut_quant_mixed() {
    use leptonica::color::quantize::few_colors_median_cut_quant_mixed;

    let pix = make_6color(120, 90);
    let result = few_colors_median_cut_quant_mixed(&pix, 6, 10);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 22. pixColorSegmentClean (segment.rs)
// ============================================================================

#[test]
fn test_color_segment_clean() {
    use leptonica::color::segment::{color_segment_clean, color_segment_cluster};

    let pix = make_6color(120, 90);
    let (segmented, counts) = color_segment_cluster(&pix, 75, 10)
        .map(|p| {
            let cmap = p.colormap().unwrap();
            let ncolors = cmap.len();
            // Create mock count array
            let counts = vec![100u32; ncolors];
            (p, counts)
        })
        .unwrap();
    let mut seg_mut = segmented.try_into_mut().unwrap();
    let result = color_segment_clean(&mut seg_mut, 3, &counts);
    assert!(result.is_ok());
}

// ============================================================================
// 23. pixColorShiftWhitePoint (analysis.rs)
// ============================================================================

#[test]
fn test_color_shift_white_point() {
    use leptonica::color::analysis::color_shift_white_point;

    let pix = make_uniform_rgb(200, 180, 160, 20, 20);
    let result = color_shift_white_point(&pix, 200, 180, 160);
    assert!(result.is_ok());
    let shifted = result.unwrap();
    assert_eq!(shifted.depth(), PixelDepth::Bit32);
    // After shift with ref=actual values, pixels should be near white
    let (r, g, b) = pixel::extract_rgb(shifted.get_pixel_unchecked(10, 10));
    assert!(r >= 250, "r should be near 255, got {r}");
    assert!(g >= 250, "g should be near 255, got {g}");
    assert!(b >= 250, "b should be near 255, got {b}");
}

// ============================================================================
// 24. pixFindColorRegions (analysis.rs)
// ============================================================================

#[test]
fn test_find_color_regions() {
    use leptonica::color::analysis::find_color_regions;

    // Image with a colored region and white background
    let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let val = if x > 30 && x < 70 && y > 30 && y < 70 {
                pixel::compose_rgb(255, 0, 0) // red center
            } else {
                pixel::compose_rgb(240, 240, 240) // light bg
            };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let result = find_color_regions(&pix, None, 1, 210, 70, 10, 90, 0.05);
    assert!(result.is_ok());
    let frac = result.unwrap();
    assert!(frac > 0.0, "should detect some color fraction");
}

// ============================================================================
// 25. pixConvertRGBToCmapLossless (analysis.rs)
// ============================================================================

#[test]
fn test_convert_rgb_to_cmap_lossless() {
    use leptonica::color::analysis::convert_rgb_to_cmap_lossless;

    let pix = make_tricolor(60, 60);
    let result = convert_rgb_to_cmap_lossless(&pix);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
    let ncolors = pixd.colormap().unwrap().len();
    assert_eq!(ncolors, 3, "tricolor should have 3 colors, got {ncolors}");
}

#[test]
fn test_convert_rgb_to_cmap_lossless_too_many_colors() {
    use leptonica::color::analysis::convert_rgb_to_cmap_lossless;

    let pix = make_color_gradient(100, 100);
    let result = convert_rgb_to_cmap_lossless(&pix);
    // Gradient has > 256 colors, should fail
    assert!(result.is_err());
}

// ============================================================================
// 26. pixSimpleColorQuantize (analysis.rs)
// ============================================================================

#[test]
fn test_simple_color_quantize() {
    use leptonica::color::analysis::simple_color_quantize;

    let pix = make_color_gradient(100, 80);
    let result = simple_color_quantize(&pix, 3, 16);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 27. pixHasHighlightRed (analysis.rs)
// ============================================================================

#[test]
fn test_has_highlight_red() {
    use leptonica::color::analysis::has_highlight_red;

    // Image with some bright red pixels
    let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let val = if x > 40 && x < 60 && y > 40 && y < 60 {
                pixel::compose_rgb(255, 30, 30) // bright red center
            } else {
                pixel::compose_rgb(200, 200, 200) // gray
            };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let result = has_highlight_red(&pix, 1);
    assert!(result.is_ok());
}

// ============================================================================
// 28. pixColorContentByLocation (colorfill.rs)
// ============================================================================

#[test]
fn test_color_content_by_location() {
    use leptonica::color::colorfill::color_content_by_location;

    let pix = make_6color(120, 90);
    let result = color_content_by_location(&pix, 4, 10, 20);
    assert!(result.is_ok());
}

// ============================================================================
// 29. pixColorGrayRegions (coloring.rs)
// ============================================================================

#[test]
fn test_color_gray_regions() {
    use leptonica::color::coloring::color_gray_regions;

    // Create image with gray and colored regions
    let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let val = if x < 50 {
                pixel::compose_rgb(128, 128, 128) // gray
            } else {
                pixel::compose_rgb(255, 0, 0) // red
            };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let result = color_gray_regions(&pix, None, 20, 0, 200, (0, 255, 0));
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit32);
}

// ============================================================================
// 30. pixSnapColorCmap (coloring.rs)
// ============================================================================

#[test]
fn test_snap_color_cmap() {
    use leptonica::color::coloring::snap_color_cmap;
    use leptonica::core::PixColormap;

    // Create 8bpp colormapped image
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(250, 245, 240).unwrap(); // near white
    cmap.add_rgb(10, 5, 15).unwrap(); // near black
    cmap.add_rgb(128, 128, 128).unwrap(); // gray

    let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..30u32 {
        for x in 0..30u32 {
            pm.set_pixel_unchecked(x, y, x % 3);
        }
    }
    let pixd: Pix = pm.into();
    let mut pixd_mut = pixd.try_into_mut().unwrap();
    pixd_mut.set_colormap(Some(cmap)).unwrap();
    let pixd: Pix = pixd_mut.into();

    let result = snap_color_cmap(&pixd, 0xffffff00, 20);
    assert!(result.is_ok());
}

// ============================================================================
// 31. pixOtsuThreshOnBackgroundNorm (threshold.rs)
// ============================================================================

#[test]
fn test_otsu_thresh_on_background_norm() {
    use leptonica::color::threshold::otsu_thresh_on_background_norm;

    let pix = make_bimodal_8bpp(200, 100);
    let result = otsu_thresh_on_background_norm(&pix, None, 10, 15, 100, 50, 255, 2, 2, 0.1);
    assert!(result.is_ok());
    let (pixd, _thresh) = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 32. pixMaskedThreshOnBackgroundNorm (threshold.rs)
// ============================================================================

#[test]
fn test_masked_thresh_on_background_norm() {
    use leptonica::color::threshold::masked_thresh_on_background_norm;

    let pix = make_bimodal_8bpp(200, 100);
    let result = masked_thresh_on_background_norm(&pix, None, 10, 15, 100, 50, 2, 2, 0.1);
    assert!(result.is_ok());
    let (pixd, _thresh) = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 33. pixSauvolaOnContrastNorm (threshold.rs)
// ============================================================================

#[test]
fn test_sauvola_on_contrast_norm() {
    use leptonica::color::threshold::sauvola_on_contrast_norm;

    let pix = make_bimodal_8bpp(200, 100);
    let result = sauvola_on_contrast_norm(&pix, 7);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 34. pixThreshOnDoubleNorm (threshold.rs)
// ============================================================================

#[test]
fn test_thresh_on_double_norm() {
    use leptonica::color::threshold::thresh_on_double_norm;

    let pix = make_bimodal_8bpp(200, 100);
    let result = thresh_on_double_norm(&pix, 7);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 35. pixThresholdByConnComp (threshold.rs)
// ============================================================================

#[test]
fn test_threshold_by_conn_comp() {
    use leptonica::color::threshold::threshold_by_conn_comp;

    let pix = make_bimodal_8bpp(200, 100);
    let result = threshold_by_conn_comp(&pix, None, 80, 200, 10, 0.9, None);
    assert!(result.is_ok());
    let (pixd, thresh) = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
    assert!((80..=200).contains(&thresh), "thresh={thresh}");
}

// ============================================================================
// 36. pixThresholdByHisto (threshold.rs)
// ============================================================================

#[test]
fn test_threshold_by_histo() {
    use leptonica::color::threshold::threshold_by_histo;

    let pix = make_bimodal_8bpp(200, 100);
    let result = threshold_by_histo(&pix, 1, 2);
    assert!(result.is_ok());
    let (pixd, thresh) = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
    assert!(thresh > 0 && thresh < 255, "thresh={thresh}");
}

// ============================================================================
// 37. pixAdaptThresholdToBinaryGen (threshold.rs)
// ============================================================================

#[test]
fn test_adapt_threshold_to_binary_gen() {
    use leptonica::color::threshold::adapt_threshold_to_binary_gen;

    let pix = make_gradient_8bpp(200, 100);
    let result = adapt_threshold_to_binary_gen(
        &pix, None, /* gamma */ 1.0, /* blackval */ 70, /* whiteval */ 190,
    );
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 38. pixDitherTo2bpp (threshold.rs)
// ============================================================================

#[test]
fn test_dither_to_2bpp() {
    use leptonica::color::threshold::dither_to_2bpp;

    let pix = make_gradient_8bpp(200, 100);
    let result = dither_to_2bpp(&pix);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit2);
}

// ============================================================================
// 39. pixDitherTo2bppSpec (threshold.rs)
// ============================================================================

#[test]
fn test_dither_to_2bpp_spec() {
    use leptonica::color::threshold::dither_to_2bpp_spec;

    let pix = make_gradient_8bpp(200, 100);
    let result = dither_to_2bpp_spec(&pix, 51, 85, 170);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit2);
}

// ============================================================================
// 40. pixThresholdOn8bpp (threshold.rs)
// ============================================================================

#[test]
fn test_threshold_on_8bpp() {
    use leptonica::color::threshold::threshold_on_8bpp;

    let pix = make_gradient_8bpp(256, 10);
    let result = threshold_on_8bpp(&pix, 4, true);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert_eq!(pixd.depth(), PixelDepth::Bit8);
    // Should have a colormap when with_colormap=true
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 41. pixThresholdGrayArb (threshold.rs)
// ============================================================================

#[test]
fn test_threshold_gray_arb() {
    use leptonica::color::threshold::threshold_gray_arb;

    let pix = make_gradient_8bpp(256, 10);
    let result = threshold_gray_arb(&pix, "80 160 240");
    assert!(result.is_ok());
    let pixd = result.unwrap();
    // With 3 thresholds, output should have 4 levels
    let max_val = (0..256)
        .map(|x| pixd.get_pixel_unchecked(x, 0))
        .max()
        .unwrap();
    assert!(max_val <= 3, "max_val={max_val}");
}

// ============================================================================
// 42. pixGenerateMaskByBand32 (threshold.rs)
// ============================================================================

#[test]
fn test_generate_mask_by_band_32() {
    use leptonica::color::threshold::generate_mask_by_band_32;

    let pix = make_uniform_rgb(128, 64, 32, 20, 20);
    // Band that includes all pixels
    let result = generate_mask_by_band_32(&pix, 0xff000000, 200);
    assert!(result.is_ok());
    let mask = result.unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 43. pixGenerateMaskByDiscr32 (threshold.rs)
// ============================================================================

#[test]
fn test_generate_mask_by_discr_32() {
    use leptonica::color::threshold::generate_mask_by_discr_32;

    // Image with distinct red and blue regions
    let pix = Pix::new(100, 50, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..50u32 {
        for x in 0..100u32 {
            let val = if x < 50 {
                pixel::compose_rgb(200, 50, 50)
            } else {
                pixel::compose_rgb(50, 50, 200)
            };
            pm.set_pixel_unchecked(x, y, val);
        }
    }
    let pix: Pix = pm.into();
    let result = generate_mask_by_discr_32(&pix, 0xc8323200, 0x3232c800, 128);
    assert!(result.is_ok());
    let mask = result.unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
}

// ============================================================================
// 44. pixGrayQuantFromHisto (threshold.rs)
// ============================================================================

#[test]
fn test_gray_quant_from_histo() {
    use leptonica::color::threshold::gray_quant_from_histo;

    let pix = make_bimodal_8bpp(200, 100);
    let result = gray_quant_from_histo(&pix, None, 0.01, 8);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

// ============================================================================
// 45. pixGrayQuantFromCmap (threshold.rs)
// ============================================================================

#[test]
fn test_gray_quant_from_cmap() {
    use leptonica::color::threshold::gray_quant_from_cmap;
    use leptonica::core::PixColormap;

    let pix = make_gradient_8bpp(256, 10);
    let mut cmap = PixColormap::new(8).unwrap();
    cmap.add_rgb(0, 0, 0).unwrap();
    cmap.add_rgb(85, 85, 85).unwrap();
    cmap.add_rgb(170, 170, 170).unwrap();
    cmap.add_rgb(255, 255, 255).unwrap();
    let result = gray_quant_from_cmap(&pix, &cmap, 2);
    assert!(result.is_ok());
    let pixd = result.unwrap();
    assert!(pixd.colormap().is_some());
}

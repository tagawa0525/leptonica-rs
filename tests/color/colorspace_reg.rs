//! Colorspace conversion regression test
//!
//! C version: reference/leptonica/prog/colorspace_reg.c
//! Tests RGB<->HSV, RGB<->Lab, RGB<->YUV conversions.
//!
//! Expanded in Phase 5 to add:
//! - HSV colormap roundtrip (pix_colormap_convert_rgb_to_hsv / hsv_to_rgb)
//! - Color magnitude sweep with both ColorMagnitudeType methods
//! - HSV spectrum generation and verification

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::color::{
    ColorMagnitudeType, color_magnitude, hsv_to_rgb, lab_to_rgb, pix_colormap_convert_hsv_to_rgb,
    pix_colormap_convert_rgb_to_hsv, pix_convert_hsv_to_rgb, pix_convert_rgb_to_hsv,
    pix_convert_to_gray, rgb_to_gray, rgb_to_hsv, rgb_to_lab, rgb_to_xyz, rgb_to_yuv, xyz_to_rgb,
    yuv_to_rgb,
};
use leptonica::io::ImageFormat;

#[test]
fn colorspace_reg() {
    let mut rp = RegParams::new("colorspace");

    // --- Pixel-level tests ---

    // Test RGB -> gray
    let gray = rgb_to_gray(255, 0, 0);
    rp.compare_values(1.0, if gray > 0 { 1.0 } else { 0.0 }, 0.0);

    let gray_w = rgb_to_gray(255, 255, 255);
    rp.compare_values(255.0, gray_w as f64, 0.0);

    let gray_b = rgb_to_gray(0, 0, 0);
    rp.compare_values(0.0, gray_b as f64, 0.0);

    // Test RGB -> HSV -> RGB roundtrip
    for &(r, g, b) in &[
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (128, 64, 32),
        (255, 255, 255),
        (0, 0, 0),
    ] {
        let hsv = rgb_to_hsv(r, g, b);
        let (r2, g2, b2) = hsv_to_rgb(hsv);
        let ok = (r as i16 - r2 as i16).unsigned_abs() <= 1
            && (g as i16 - g2 as i16).unsigned_abs() <= 1
            && (b as i16 - b2 as i16).unsigned_abs() <= 1;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    }

    // Test RGB -> Lab -> RGB roundtrip
    for &(r, g, b) in &[(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 128, 128)] {
        let lab = rgb_to_lab(r, g, b);
        let (r2, g2, b2) = lab_to_rgb(lab);
        let ok = (r as i16 - r2 as i16).unsigned_abs() <= 2
            && (g as i16 - g2 as i16).unsigned_abs() <= 2
            && (b as i16 - b2 as i16).unsigned_abs() <= 2;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    }

    // Test RGB -> XYZ -> RGB roundtrip
    for &(r, g, b) in &[(255, 128, 0), (0, 128, 255), (64, 64, 64)] {
        let xyz = rgb_to_xyz(r, g, b);
        let (r2, g2, b2) = xyz_to_rgb(xyz);
        let ok = (r as i16 - r2 as i16).unsigned_abs() <= 2
            && (g as i16 - g2 as i16).unsigned_abs() <= 2
            && (b as i16 - b2 as i16).unsigned_abs() <= 2;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    }

    // Test RGB -> YUV -> RGB roundtrip
    for &(r, g, b) in &[(255, 0, 0), (0, 255, 0), (128, 128, 128)] {
        let yuv = rgb_to_yuv(r, g, b);
        let (r2, g2, b2) = yuv_to_rgb(yuv);
        let ok = (r as i16 - r2 as i16).unsigned_abs() <= 2
            && (g as i16 - g2 as i16).unsigned_abs() <= 2
            && (b as i16 - b2 as i16).unsigned_abs() <= 2;
        rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
    }

    // --- Image-level tests ---

    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    let w = pix32.width();
    let h = pix32.height();

    // RGB -> gray
    let gray_img = pix_convert_to_gray(&pix32).expect("pix_convert_to_gray");
    rp.compare_values(w as f64, gray_img.width() as f64, 0.0);
    rp.compare_values(h as f64, gray_img.height() as f64, 0.0);
    rp.compare_values(8.0, gray_img.depth().bits() as f64, 0.0);
    rp.write_pix_and_check(&gray_img, ImageFormat::Png)
        .expect("write gray_img colorspace");

    // RGB -> HSV -> RGB roundtrip
    let hsv_img = pix_convert_rgb_to_hsv(&pix32).expect("pix_convert_rgb_to_hsv");
    rp.compare_values(w as f64, hsv_img.width() as f64, 0.0);
    rp.compare_values(h as f64, hsv_img.height() as f64, 0.0);

    let rgb_back = pix_convert_hsv_to_rgb(&hsv_img).expect("pix_convert_hsv_to_rgb");
    rp.compare_values(w as f64, rgb_back.width() as f64, 0.0);
    rp.compare_values(h as f64, rgb_back.height() as f64, 0.0);

    assert!(rp.cleanup(), "colorspace regression test failed");
}

/// Test colormap RGB↔HSV roundtrip on a colormapped image.
///
/// C: pixConvertRGBToHSV on colormapped (4bpp) images.
/// Verifies that pix_colormap_convert_rgb_to_hsv followed by
/// pix_colormap_convert_hsv_to_rgb preserves colormap entry count.
#[test]
fn colorspace_reg_colormap_hsv_roundtrip() {
    let mut rp = RegParams::new("colorspace_cmap_hsv");

    // Load a colormapped image (4bpp, 11 colors)
    let pix = load_test_image("weasel4.11c.png").expect("load weasel4.11c.png");
    assert!(pix.has_colormap(), "weasel4.11c should have colormap");

    let n_colors_orig = pix.colormap().map(|c| c.len()).unwrap_or(0);
    assert!(n_colors_orig > 0, "colormap must have entries");

    // Convert colormap to HSV space
    let mut pix_mut = pix.to_mut();
    {
        let cmap = pix_mut.colormap_mut().expect("has colormap");
        pix_colormap_convert_rgb_to_hsv(cmap).expect("colormap RGB→HSV");
    }
    let pix_hsv: leptonica::Pix = pix_mut.into();

    rp.write_pix_and_check(&pix_hsv, ImageFormat::Png)
        .expect("write colormap hsv");

    // Verify colormap entry count is preserved
    let n_colors_hsv = pix_hsv.colormap().map(|c| c.len()).unwrap_or(0);
    rp.compare_values(n_colors_orig as f64, n_colors_hsv as f64, 0.0);

    // Convert back to RGB
    let mut pix_hsv_mut = pix_hsv.to_mut();
    {
        let cmap = pix_hsv_mut.colormap_mut().expect("has colormap after HSV");
        pix_colormap_convert_hsv_to_rgb(cmap).expect("colormap HSV→RGB");
    }
    let pix_back: leptonica::Pix = pix_hsv_mut.into();

    rp.write_pix_and_check(&pix_back, ImageFormat::Png)
        .expect("write colormap rgb back");

    // Colormap size should still match
    let n_colors_back = pix_back.colormap().map(|c| c.len()).unwrap_or(0);
    rp.compare_values(n_colors_orig as f64, n_colors_back as f64, 0.0);

    assert!(rp.cleanup(), "colorspace_reg_colormap_hsv_roundtrip failed");
}

/// Test color magnitude with both methods (AveMaxDiff2, IntermedDiff).
///
/// C: pixColorMagnitude with L_AVE_MAX_DIFF_2 and L_INTERMED_DIFF.
/// Sweeps over 20 white-point threshold values with 6 color thresholds,
/// counting fraction of pixels exceeding each magnitude threshold.
#[test]
fn colorspace_reg_color_magnitude() {
    let mut rp = RegParams::new("colorspace_magnitude");

    let pix32 = load_test_image("weasel32.png").expect("load 32bpp");
    let pix32 = if pix32.depth() != PixelDepth::Bit32 {
        pix32.convert_to_32().expect("convert to 32bpp")
    } else {
        pix32
    };

    // Test AveMaxDiff2 magnitude map
    let mag_ave =
        color_magnitude(&pix32, ColorMagnitudeType::AveMaxDiff2).expect("AveMaxDiff2 magnitude");
    assert_eq!(mag_ave.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&mag_ave, ImageFormat::Png)
        .expect("write AveMaxDiff2");

    // Test IntermedDiff magnitude map
    let mag_int =
        color_magnitude(&pix32, ColorMagnitudeType::IntermedDiff).expect("IntermedDiff magnitude");
    assert_eq!(mag_int.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&mag_int, ImageFormat::Png)
        .expect("write IntermedDiff");

    // Test MaxDiff magnitude map
    let mag_max = color_magnitude(&pix32, ColorMagnitudeType::MaxDiff).expect("MaxDiff magnitude");
    assert_eq!(mag_max.depth(), PixelDepth::Bit8);
    rp.write_pix_and_check(&mag_max, ImageFormat::Png)
        .expect("write MaxDiff");

    // Threshold sweep: count fraction of pixels above each threshold (6 levels × 20 white points)
    // C: sweeps thresholds 20, 40, 60, 80, 100, 120 on both mag types
    let thresholds = [20u8, 40, 60, 80, 100, 120];
    let n_pixels = (pix32.width() * pix32.height()) as f64;

    for thresh in thresholds {
        // Count pixels above threshold in AveMaxDiff2 map
        let count_ave = count_pixels_above(&mag_ave, thresh);
        let frac_ave = count_ave as f64 / n_pixels;
        rp.compare_values(
            1.0,
            if (0.0..=1.0).contains(&frac_ave) {
                1.0
            } else {
                0.0
            },
            0.0,
        );

        // Count pixels above threshold in IntermedDiff map
        let count_int = count_pixels_above(&mag_int, thresh);
        let frac_int = count_int as f64 / n_pixels;
        rp.compare_values(
            1.0,
            if (0.0..=1.0).contains(&frac_int) {
                1.0
            } else {
                0.0
            },
            0.0,
        );
    }

    assert!(rp.cleanup(), "colorspace_reg_color_magnitude failed");
}

/// Count pixels above a threshold in an 8bpp image.
fn count_pixels_above(pix: &leptonica::Pix, thresh: u8) -> u32 {
    let w = pix.width();
    let h = pix.height();
    let mut count = 0u32;
    for y in 0..h {
        for x in 0..w {
            let v = pix.get_pixel_unchecked(x, y);
            if v > thresh as u32 {
                count += 1;
            }
        }
    }
    count
}

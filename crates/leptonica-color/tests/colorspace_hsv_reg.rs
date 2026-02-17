//! Test HSV range mask, histogram, and YUV image conversion functions
//!
//! # See also
//!
//! C Leptonica: `colorspace.c`
//! - pixMakeRangeMaskHS, pixMakeRangeMaskHV, pixMakeRangeMaskSV
//! - pixMakeHistoHS, pixMakeHistoHV, pixMakeHistoSV
//! - pixConvertRGBToYUV, pixConvertYUVToRGB

use leptonica_color::colorspace::{
    RegionFlag, make_histo_hs, make_histo_hv, make_histo_sv, make_range_mask_hs,
    make_range_mask_hv, make_range_mask_sv, pix_convert_rgb_to_yuv, pix_convert_yuv_to_rgb,
};
use leptonica_core::{Pix, PixelDepth, color};

/// Create a uniform RGB image
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

/// Create a 3-color image: red (left), green (middle), blue (right)
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

// ============================================================================
// make_range_mask_hs
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hs_include_red() {
    // Pure red: H=0 in Leptonica HSV, S=255
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    // Hue center=0, half-width=20 → range wraps: [220..239] + [0..20]
    // Sat center=200, half-width=100 → range [100..255]
    let mask = make_range_mask_hs(&pix, 0, 20, 200, 100, RegionFlag::Include).unwrap();
    assert_eq!(mask.depth(), PixelDepth::Bit1);
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400); // All red pixels match
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hs_exclude_red() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    let mask = make_range_mask_hs(&pix, 0, 20, 200, 100, RegionFlag::Exclude).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0); // All red pixels excluded → mask OFF
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hs_no_match() {
    // Pure red: H=0
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    // Select green hue range: H=80 ± 20 → [60..100]
    let mask = make_range_mask_hs(&pix, 80, 20, 128, 128, RegionFlag::Include).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0); // Red not in green range
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hs_gray_excluded() {
    // Gray: S=0, so saturation range [100..255] shouldn't match
    let pix = make_uniform_rgb(128, 128, 128, 20, 20);
    let mask = make_range_mask_hs(&pix, 0, 120, 200, 55, RegionFlag::Include).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0); // Gray has S=0, not in [145..255]
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hs_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(make_range_mask_hs(&pix, 0, 20, 128, 128, RegionFlag::Include).is_err());
}

// ============================================================================
// make_range_mask_hv
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hv_include_red() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    // Red: H=0, V=255
    let mask = make_range_mask_hv(&pix, 0, 20, 200, 100, RegionFlag::Include).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_hv_exclude() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    let mask = make_range_mask_hv(&pix, 0, 20, 200, 100, RegionFlag::Exclude).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

// ============================================================================
// make_range_mask_sv
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_sv_include() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    // Red: S=255, V=255
    let mask = make_range_mask_sv(&pix, 200, 100, 200, 100, RegionFlag::Include).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 400);
}

#[test]
#[ignore = "not yet implemented"]
fn test_range_mask_sv_gray_low_saturation() {
    let pix = make_uniform_rgb(128, 128, 128, 20, 20);
    // Gray: S=0, V=128. Select high saturation [200..255] → no match
    let mask = make_range_mask_sv(&pix, 228, 28, 128, 128, RegionFlag::Include).unwrap();
    let on_count: u32 = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .map(|(x, y)| mask.get_pixel_unchecked(x, y))
        .sum();
    assert_eq!(on_count, 0);
}

// ============================================================================
// make_histo_hs
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_histo_hs_uniform() {
    // All pixels are red: H=0, S=255
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    let histo = make_histo_hs(&pix, 1).unwrap();
    assert_eq!(histo.width(), 256); // saturation axis
    assert_eq!(histo.height(), 240); // hue axis
    assert_eq!(histo.depth(), PixelDepth::Bit32);
    // Bin at (hue=0, sat=255) should have count=400
    let count = histo.get_pixel_unchecked(255, 0);
    assert_eq!(count, 400);
}

#[test]
#[ignore = "not yet implemented"]
fn test_histo_hs_tricolor() {
    let pix = make_tricolor(30, 10);
    let histo = make_histo_hs(&pix, 1).unwrap();
    // Three distinct peaks at H=0(red), H=80(green), H=160(blue)
    let red_count = histo.get_pixel_unchecked(255, 0); // H=0, S=255
    let green_count = histo.get_pixel_unchecked(255, 80); // H=80, S=255
    let blue_count = histo.get_pixel_unchecked(255, 160); // H=160, S=255
    assert!(red_count > 0, "red count = {red_count}");
    assert!(green_count > 0, "green count = {green_count}");
    assert!(blue_count > 0, "blue count = {blue_count}");
}

#[test]
#[ignore = "not yet implemented"]
fn test_histo_hs_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(make_histo_hs(&pix, 1).is_err());
}

// ============================================================================
// make_histo_hv
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_histo_hv_uniform() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    let histo = make_histo_hv(&pix, 1).unwrap();
    assert_eq!(histo.width(), 256); // value axis
    assert_eq!(histo.height(), 240); // hue axis
    // Red: H=0, V=255 → bin at (val=255, hue=0)
    let count = histo.get_pixel_unchecked(255, 0);
    assert_eq!(count, 400);
}

// ============================================================================
// make_histo_sv
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_histo_sv_uniform() {
    let pix = make_uniform_rgb(255, 0, 0, 20, 20);
    let histo = make_histo_sv(&pix, 1).unwrap();
    assert_eq!(histo.width(), 256); // value axis
    assert_eq!(histo.height(), 256); // saturation axis
    // Red: S=255, V=255 → bin at (val=255, sat=255)
    let count = histo.get_pixel_unchecked(255, 255);
    assert_eq!(count, 400);
}

// ============================================================================
// pix_convert_rgb_to_yuv / pix_convert_yuv_to_rgb
// ============================================================================

#[test]
#[ignore = "not yet implemented"]
fn test_yuv_roundtrip() {
    let pix = make_tricolor(30, 10);
    let yuv = pix_convert_rgb_to_yuv(&pix).unwrap();
    assert_eq!(yuv.depth(), PixelDepth::Bit32);
    let rgb = pix_convert_yuv_to_rgb(&yuv).unwrap();

    // Check roundtrip for a few pixels
    for (x, y) in [(5, 5), (15, 5), (25, 5)] {
        let orig = pix.get_pixel_unchecked(x, y);
        let back = rgb.get_pixel_unchecked(x, y);
        let (r1, g1, b1) = color::extract_rgb(orig);
        let (r2, g2, b2) = color::extract_rgb(back);
        assert!(
            (r1 as i32 - r2 as i32).abs() <= 2
                && (g1 as i32 - g2 as i32).abs() <= 2
                && (b1 as i32 - b2 as i32).abs() <= 2,
            "roundtrip failed at ({x},{y}): ({r1},{g1},{b1}) -> ({r2},{g2},{b2})"
        );
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_yuv_known_values() {
    // White (255,255,255) → Y≈235, U≈128, V≈128 in video range
    let pix = make_uniform_rgb(255, 255, 255, 10, 10);
    let yuv = pix_convert_rgb_to_yuv(&pix).unwrap();
    let pixel = yuv.get_pixel_unchecked(0, 0);
    let y_val = (pixel >> 24) & 0xff;
    let u_val = (pixel >> 16) & 0xff;
    let v_val = (pixel >> 8) & 0xff;
    // Y should be close to 235
    assert!((y_val as i32 - 235).abs() <= 2, "Y={y_val}, expected ~235");
    // U and V should be close to 128
    assert!((u_val as i32 - 128).abs() <= 2, "U={u_val}, expected ~128");
    assert!((v_val as i32 - 128).abs() <= 2, "V={v_val}, expected ~128");
}

#[test]
#[ignore = "not yet implemented"]
fn test_yuv_black_known_values() {
    // Black (0,0,0) → Y≈16, U≈128, V≈128
    let pix = make_uniform_rgb(0, 0, 0, 10, 10);
    let yuv = pix_convert_rgb_to_yuv(&pix).unwrap();
    let pixel = yuv.get_pixel_unchecked(0, 0);
    let y_val = (pixel >> 24) & 0xff;
    let u_val = (pixel >> 16) & 0xff;
    let v_val = (pixel >> 8) & 0xff;
    assert!((y_val as i32 - 16).abs() <= 1, "Y={y_val}, expected ~16");
    assert!((u_val as i32 - 128).abs() <= 1, "U={u_val}, expected ~128");
    assert!((v_val as i32 - 128).abs() <= 1, "V={v_val}, expected ~128");
}

#[test]
#[ignore = "not yet implemented"]
fn test_yuv_invalid_depth() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(pix_convert_rgb_to_yuv(&pix).is_err());
    assert!(pix_convert_yuv_to_rgb(&pix).is_err());
}

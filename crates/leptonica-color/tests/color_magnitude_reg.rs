//! Tests for pixColorMagnitude
//!
//! # See also
//!
//! C Leptonica: `colorcontent.c` — `pixColorMagnitude`

use leptonica_color::{ColorMagnitudeType, analysis::color_magnitude};
use leptonica_core::{Pix, PixelDepth, color};

#[test]
#[ignore = "not yet implemented"]
fn test_color_magnitude_gray_pixel() {
    // A pure gray pixel (r=g=b) should have magnitude 0 for all types
    let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(0, 0, color::compose_rgba(128, 128, 128, 255));
    let pix: Pix = pm.into();

    for mag_type in [
        ColorMagnitudeType::IntermedDiff,
        ColorMagnitudeType::AveMaxDiff2,
        ColorMagnitudeType::MaxDiff,
    ] {
        let result = color_magnitude(&pix, mag_type).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel(0, 0).unwrap(), 0);
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test_color_magnitude_pure_red() {
    // Pure red (255, 0, 0)
    let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(0, 0, color::compose_rgba(255, 0, 0, 255));
    let pix: Pix = pm.into();

    // MaxDiff: max(255,0,0) - min(255,0,0) = 255
    let result = color_magnitude(&pix, ColorMagnitudeType::MaxDiff).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 255);

    // IntermedDiff: median of |255-0|=255, |255-0|=255, |0-0|=0 → 255
    let result = color_magnitude(&pix, ColorMagnitudeType::IntermedDiff).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 255);

    // AveMaxDiff2: max(|255-(0+0)/2|=255, |0-(255+0)/2|=127, |0-(255+0)/2|=127) = 255
    let result = color_magnitude(&pix, ColorMagnitudeType::AveMaxDiff2).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 255);
}

#[test]
#[ignore = "not yet implemented"]
fn test_color_magnitude_known_values() {
    // R=100, G=120, B=140 → known expected values
    let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(0, 0, color::compose_rgba(100, 120, 140, 255));
    let pix: Pix = pm.into();

    // IntermedDiff: |100-120|=20, |100-140|=40, |120-140|=20
    // sorted: 20, 20, 40 → median = 20
    let result = color_magnitude(&pix, ColorMagnitudeType::IntermedDiff).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 20);

    // MaxDiff: 140-100 = 40
    let result = color_magnitude(&pix, ColorMagnitudeType::MaxDiff).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 40);

    // AveMaxDiff2: |100-(120+140)/2|=30, |120-(100+140)/2|=0, |140-(100+120)/2|=30
    // max = 30
    let result = color_magnitude(&pix, ColorMagnitudeType::AveMaxDiff2).unwrap();
    assert_eq!(result.get_pixel(0, 0).unwrap(), 30);
}

#[test]
#[ignore = "not yet implemented"]
fn test_color_magnitude_multiple_pixels() {
    // 3x1 image with different colors
    let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
    let mut pm = pix.to_mut();
    pm.set_pixel_unchecked(0, 0, color::compose_rgba(100, 100, 100, 255)); // gray
    pm.set_pixel_unchecked(1, 0, color::compose_rgba(200, 0, 0, 255)); // red
    pm.set_pixel_unchecked(2, 0, color::compose_rgba(50, 100, 150, 255)); // blue-ish
    let pix: Pix = pm.into();

    let result = color_magnitude(&pix, ColorMagnitudeType::MaxDiff).unwrap();
    assert_eq!(result.width(), 3);
    assert_eq!(result.height(), 1);
    assert_eq!(result.get_pixel(0, 0).unwrap(), 0); // gray
    assert_eq!(result.get_pixel(1, 0).unwrap(), 200); // 200 - 0
    assert_eq!(result.get_pixel(2, 0).unwrap(), 100); // 150 - 50
}

#[test]
#[ignore = "not yet implemented"]
fn test_color_magnitude_rejects_non_32bpp() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    assert!(color_magnitude(&pix, ColorMagnitudeType::MaxDiff).is_err());
}

//! Colorspace conversion regression test
//!
//! C version: reference/leptonica/prog/colorspace_reg.c
//! Tests RGB<->HSV, RGB<->Lab, RGB<->YUV conversions.

use leptonica_color::{
    hsv_to_rgb, lab_to_rgb, pix_convert_hsv_to_rgb, pix_convert_rgb_to_hsv, pix_convert_to_gray,
    rgb_to_gray, rgb_to_hsv, rgb_to_lab, rgb_to_xyz, rgb_to_yuv, xyz_to_rgb, yuv_to_rgb,
};
use leptonica_test::{RegParams, load_test_image};

#[test]
#[ignore = "not yet implemented"]
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

    // RGB -> HSV -> RGB roundtrip
    let hsv_img = pix_convert_rgb_to_hsv(&pix32).expect("pix_convert_rgb_to_hsv");
    rp.compare_values(w as f64, hsv_img.width() as f64, 0.0);
    rp.compare_values(h as f64, hsv_img.height() as f64, 0.0);

    let rgb_back = pix_convert_hsv_to_rgb(&hsv_img).expect("pix_convert_hsv_to_rgb");
    rp.compare_values(w as f64, rgb_back.width() as f64, 0.0);
    rp.compare_values(h as f64, rgb_back.height() as f64, 0.0);

    assert!(rp.cleanup(), "colorspace regression test failed");
}

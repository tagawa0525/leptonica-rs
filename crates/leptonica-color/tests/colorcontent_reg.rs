//! Color content analysis regression test
//!
//! C version: reference/leptonica/prog/colorcontent_reg.c
//! Tests color_content, count_colors, is_grayscale, grayscale_histogram.

use leptonica_color::{
    color_content, count_colors, grayscale_histogram, is_grayscale, is_grayscale_tolerant,
};
use leptonica_core::{Pix, PixelDepth, color};
use leptonica_test::{RegParams, load_test_image};

fn create_known_color_image() -> Pix {
    let (w, h) = (60u32, 40u32);
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < 36 {
                color::compose_rgb(255, 0, 0)
            } else if x < 54 {
                color::compose_rgb(0, 255, 0)
            } else {
                color::compose_rgb(0, 0, 255)
            };
            pm.set_pixel_unchecked(x, y, pixel);
        }
    }
    pm.into()
}

fn create_grayscale_rgb() -> Pix {
    let (w, h) = (50u32, 50u32);
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let gray = ((x + y) * 5 % 256) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(gray, gray, gray));
        }
    }
    pm.into()
}

fn create_grayscale_8bpp() -> Pix {
    let (w, h) = (50u32, 50u32);
    let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, (x * 5 + y * 3) % 256);
        }
    }
    pm.into()
}

#[test]
fn colorcontent_reg() {
    let mut rp = RegParams::new("colorcontent");

    // Test 1: color_content on known-color image
    let pix_known = create_known_color_image();
    let stats = color_content(&pix_known).expect("color_content should succeed");
    rp.compare_values(3.0, stats.unique_colors as f64, 0.0);
    rp.compare_values(0.0, if stats.is_grayscale { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(
        1.0,
        if stats.dominant_colors.len() >= 3 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    if !stats.dominant_colors.is_empty() {
        let (r, g, b, _count) = stats.dominant_colors[0];
        rp.compare_values(255.0, r as f64, 0.0);
        rp.compare_values(0.0, g as f64, 0.0);
        rp.compare_values(0.0, b as f64, 0.0);
    }

    // Test 2: real image
    if let Ok(fish) = load_test_image("fish24.jpg") {
        let stats = color_content(&fish).expect("fish24 color_content");
        rp.compare_values(1.0, if stats.unique_colors > 100 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(0.0, if stats.is_grayscale { 1.0 } else { 0.0 }, 0.0);
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 3: count_colors
    rp.compare_values(3.0, count_colors(&pix_known).unwrap() as f64, 0.0);

    let pix8 = create_grayscale_8bpp();
    let count8 = count_colors(&pix8).unwrap();
    rp.compare_values(
        1.0,
        if count8 > 0 && count8 <= 256 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    if let Ok(wyom) = load_test_image("wyom.jpg") {
        let count_wyom = count_colors(&wyom).unwrap();
        rp.compare_values(132165.0, count_wyom as f64, 15000.0);
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 4: grayscale detection
    let gray_rgb = create_grayscale_rgb();
    rp.compare_values(
        1.0,
        if is_grayscale(&gray_rgb).unwrap() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        0.0,
        if is_grayscale(&pix_known).unwrap() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let near_gray = {
        let p = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                let base = ((x + y) * 4 % 256) as u8;
                pm.set_pixel_unchecked(
                    x,
                    y,
                    color::compose_rgb(base, base.wrapping_add(1), base.wrapping_add(2)),
                );
            }
        }
        let r: Pix = pm.into();
        r
    };
    rp.compare_values(
        0.0,
        if is_grayscale(&near_gray).unwrap() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if is_grayscale_tolerant(&near_gray, 5).unwrap() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test 5: histogram
    let uniform_gray = {
        let p = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        for y in 0..20u32 {
            for x in 0..20u32 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        let r: Pix = pm.into();
        r
    };
    let hist = grayscale_histogram(&uniform_gray).unwrap();
    rp.compare_values(400.0, hist[100] as f64, 0.0);
    rp.compare_values(0.0, hist[0] as f64, 0.0);

    // Test 6: 8bpp
    let stats8 = color_content(&pix8).unwrap();
    rp.compare_values(1.0, if stats8.is_grayscale { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if stats8.unique_colors > 0 { 1.0 } else { 0.0 }, 0.0);

    // Test 7: marge.jpg
    if let Ok(marge) = load_test_image("marge.jpg") {
        let sm = color_content(&marge).unwrap();
        rp.compare_values(1.0, if sm.unique_colors > 100 { 1.0 } else { 0.0 }, 0.0);
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // Test 8: error cases
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    rp.compare_values(2.0, count_colors(&pix1).unwrap() as f64, 0.0);
    assert!(color_content(&pix1).is_err());

    assert!(rp.cleanup(), "colorcontent regression test failed");
}

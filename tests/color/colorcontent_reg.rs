//! Color content analysis regression test
//!
//! C version: prog/colorcontent_reg.c
//! Tests color_content, count_colors, is_grayscale, grayscale_histogram.

use crate::common::{RegParams, load_test_image};
use leptonica::color::{
    color_content, color_content_by_location, count_colors, grayscale_histogram, is_grayscale,
    is_grayscale_tolerant,
};
use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::{Pix, PixelDepth};

fn create_known_color_image() -> Pix {
    let (w, h) = (60u32, 40u32);
    let pix = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let pixel = if x < 36 {
                pixel::compose_rgb(255, 0, 0)
            } else if x < 54 {
                pixel::compose_rgb(0, 255, 0)
            } else {
                pixel::compose_rgb(0, 0, 255)
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
            pm.set_pixel_unchecked(x, y, pixel::compose_rgb(gray, gray, gray));
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

    // WPAC: known-color image
    rp.write_pix_and_check(&pix_known, ImageFormat::Png)
        .expect("check: known-color image");

    // Test 2: real image
    if let Ok(fish) = load_test_image("fish24.jpg") {
        let stats = color_content(&fish).expect("fish24 color_content");
        rp.compare_values(1.0, if stats.unique_colors > 100 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(0.0, if stats.is_grayscale { 1.0 } else { 0.0 }, 0.0);

        // WPAC: color_content_by_location on fish24
        let loc_result =
            color_content_by_location(&fish, 4, 10, 30).expect("color_content_by_location fish24");
        rp.write_pix_and_check(&loc_result, ImageFormat::Png)
            .expect("check: fish24 color by location");
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0); // index sync for write_pix_and_check
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

        // WPAC: color_content_by_location on wyom
        let loc_wyom =
            color_content_by_location(&wyom, 4, 10, 30).expect("color_content_by_location wyom");
        rp.write_pix_and_check(&loc_wyom, ImageFormat::Png)
            .expect("check: wyom color by location");
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0); // index sync for write_pix_and_check
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
                    pixel::compose_rgb(base, base.wrapping_add(1), base.wrapping_add(2)),
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

        // WPAC: color_content_by_location on marge
        let loc_marge =
            color_content_by_location(&marge, 4, 10, 30).expect("color_content_by_location marge");
        rp.write_pix_and_check(&loc_marge, ImageFormat::Png)
            .expect("check: marge color by location");
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0); // index sync for write_pix_and_check
    }

    // Test 8: error cases
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    rp.compare_values(2.0, count_colors(&pix1).unwrap() as f64, 0.0);
    assert!(color_content(&pix1).is_err());

    assert!(rp.cleanup(), "colorcontent regression test failed");
}

/// C-compat: `prog/colorcontent_reg.c` checks 10-17.
///
/// These are the RGB-gamut classification steps. They take no input image
/// (the gamut is synthesized), so unlike the earlier checks in that program
/// they are free of JPEG decode differences and can be compared bit-exactly
/// against the C output.
#[test]
fn colorcontent_c_compat() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("colorcontent_c");

    // C 10-12: binary classification of RGB colors using a single plane.
    let gamut = Pix::make_gamut_rgb(3).expect("gamut");
    let mask = gamut
        .make_arb_mask_from_rgb(-0.5, -0.5, 1.0, 20.0)
        .expect("arb mask");
    let combined = combine_over_white(&gamut, &mask);
    rp.write_pix_and_check(&gamut, ImageFormat::Png)
        .expect("check: gamut");
    rp.write_pix_and_check(&mask, ImageFormat::Png)
        .expect("check: single-plane mask");
    rp.write_pix_and_check(&combined, ImageFormat::Png)
        .expect("check: single-plane selection");

    // C 13-17: more than one plane, further restricting the allowed region.
    let mask2 = gamut
        .make_arb_mask_from_rgb(1.5, -0.5, -1.0, 0.0)
        .expect("arb mask 2");
    let mask3 = gamut
        .make_arb_mask_from_rgb(0.4, 0.3, 0.3, 60.0)
        .expect("arb mask 3")
        .invert();
    let sub1 = mask.subtract(&mask2).expect("mask - mask2");
    let sub2 = sub1.subtract(&mask3).expect("sub1 - mask3");
    let combined2 = combine_over_white(&gamut, &sub2);
    rp.write_pix_and_check(&mask2, ImageFormat::Png)
        .expect("check: mask2");
    rp.write_pix_and_check(&mask3, ImageFormat::Png)
        .expect("check: mask3 inverted");
    rp.write_pix_and_check(&sub1, ImageFormat::Png)
        .expect("check: mask - mask2");
    rp.write_pix_and_check(&sub2, ImageFormat::Png)
        .expect("check: sub1 - mask3");
    rp.write_pix_and_check(&combined2, ImageFormat::Png)
        .expect("check: multi-plane selection");

    assert!(rp.cleanup(), "colorcontent C-compat test failed");
}

/// C: `pixCreate(w, h, 32); pixSetAll(pix); pixCombineMasked(pix, src, mask)`.
fn combine_over_white(src: &Pix, mask: &Pix) -> Pix {
    let white = Pix::new(src.width(), src.height(), PixelDepth::Bit32).expect("white canvas");
    let mut wm = white.try_into_mut().unwrap();
    wm.set_all();
    wm.combine_masked(src, mask).expect("combine masked");
    wm.into()
}

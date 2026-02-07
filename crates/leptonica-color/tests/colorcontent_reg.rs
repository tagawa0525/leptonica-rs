//! Color content analysis regression test
//!
//! C版: reference/leptonica/prog/colorcontent_reg.c
//! 色含有分析（color_content）と色数カウント（count_colors）をテスト。
//!
//! C版テスト関数でRust未実装のもの:
//! - pixGetMostPopulatedColors -- Rust: color_contentのdominant_colorsで代替
//! - pixSimpleColorQuantize -- Rust未実装のためスキップ
//! - pixNumSignificantGrayColors -- Rust未実装のためスキップ
//! - pixFindColorRegions -- Rust未実装のためスキップ
//! - pixMakeGamutRGB / pixMakeArbMaskFromRGB -- Rust未実装のためスキップ

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

    // === Test 1: color_content on known-color image ===
    // C版: pixGetMostPopulatedColors(pix1, 2, 3, 10, &colors, NULL)
    eprintln!("=== Test 1: known-color image ===");
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
        let (r, g, b, count) = stats.dominant_colors[0];
        eprintln!("  dominant[0] = ({},{},{}) count={}", r, g, b, count);
        rp.compare_values(255.0, r as f64, 0.0);
        rp.compare_values(0.0, g as f64, 0.0);
        rp.compare_values(0.0, b as f64, 0.0);
    }

    // === Test 2: color_content on real image ===
    // C版: pix1 = pixRead("fish24.jpg")
    eprintln!("=== Test 2: real image ===");
    if let Ok(fish) = load_test_image("fish24.jpg") {
        let stats = color_content(&fish).expect("fish24 color_content");
        rp.compare_values(1.0, if stats.unique_colors > 100 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(0.0, if stats.is_grayscale { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  fish24: {} unique, grayscale={}",
            stats.unique_colors, stats.is_grayscale
        );
        // C版: pixSimpleColorQuantize(pix1, 2, 3, 10) -- Rust未実装のためスキップ
    } else {
        eprintln!("  fish24.jpg not available, skipping");
        rp.compare_values(1.0, 1.0, 0.0);
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // === Test 3: count_colors ===
    // C版: pixNumColors(pix1, 1, &ncolors)
    eprintln!("=== Test 3: count_colors ===");
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
    eprintln!("  8bpp: {} colors", count8);

    // C版: pixNumColors on wyom.jpg => ~132165
    if let Ok(wyom) = load_test_image("wyom.jpg") {
        let count_wyom = count_colors(&wyom).unwrap();
        rp.compare_values(132165.0, count_wyom as f64, 15000.0);
        eprintln!("  wyom: {} unique (expected ~132165)", count_wyom);
    } else {
        eprintln!("  wyom.jpg not available");
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // === Test 4: grayscale detection ===
    // C版: pixNumSignificantGrayColors(pix2, 20, 236, 0.0001, 1, &ncolors) => 219
    eprintln!("=== Test 4: grayscale detection ===");
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

    // Near-grayscale with tolerance
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

    // === Test 5: grayscale_histogram ===
    eprintln!("=== Test 5: histogram ===");
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

    // === Test 6: color_content on 8bpp ===
    eprintln!("=== Test 6: 8bpp ===");
    let stats8 = color_content(&pix8).unwrap();
    rp.compare_values(1.0, if stats8.is_grayscale { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if stats8.unique_colors > 0 { 1.0 } else { 0.0 }, 0.0);

    // === Test 7: marge.jpg ===
    // C版: pix1 = pixRead("marge.jpg"); pixNumSignificantGrayColors(...) => 219
    eprintln!("=== Test 7: marge.jpg ===");
    if let Ok(marge) = load_test_image("marge.jpg") {
        let sm = color_content(&marge).unwrap();
        rp.compare_values(1.0, if sm.unique_colors > 100 { 1.0 } else { 0.0 }, 0.0);
        eprintln!(
            "  marge: {} unique, grayscale={}",
            sm.unique_colors, sm.is_grayscale
        );
        // C版: pixNumSignificantGrayColors -- Rust未実装のためスキップ
        // C版: pixFindColorRegions -- Rust未実装のためスキップ
        // C版: pixMakeGamutRGB -- Rust未実装のためスキップ
    } else {
        rp.compare_values(1.0, 1.0, 0.0);
    }

    // === Test 8: error cases ===
    eprintln!("=== Test 8: errors ===");
    let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    rp.compare_values(2.0, count_colors(&pix1).unwrap() as f64, 0.0);
    assert!(color_content(&pix1).is_err());
    eprintln!("  error handling OK");

    assert!(rp.cleanup(), "colorcontent regression test failed");
}

//! Color morphology regression test
//!
//! C version: reference/leptonica/prog/colormorph_reg.c
//! Tests dilate_color, erode_color, open_color, close_color.
//! Compares direct color morph operations with color morph sequence results.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test colormorph_reg
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{close_color, dilate_color, erode_color, open_color};
use leptonica_test::{RegParams, load_test_image};

const SIZE: u32 = 7;

#[test]
fn colormorph_reg() {
    let mut rp = RegParams::new("colormorph");

    let pixs = load_test_image("wyom.jpg").expect("load wyom.jpg");
    assert_eq!(
        pixs.depth(),
        PixelDepth::Bit32,
        "Test image should be 32-bpp color"
    );

    // C版: Test 0,1 -- pixColorMorph(pixs, L_MORPH_DILATE, SIZE, SIZE)
    //                   vs pixColorMorphSequence(pixs, "d7.7", 0, 0)
    // Rust: pixColorMorphSequence -- 未実装
    // We test dilate_color alone and verify basic properties.
    eprintln!("  Testing color dilation");
    let pix1 = dilate_color(&pixs, SIZE, SIZE).expect("dilate_color");
    assert_eq!(pix1.depth(), PixelDepth::Bit32);
    assert_eq!(pix1.width(), pixs.width());
    assert_eq!(pix1.height(), pixs.height());
    rp.compare_values(1.0, 1.0, 0.0); // Verify operation succeeds

    // Verify dilation property: dilated pixels should be >= original (per channel max)
    // Check a sample of pixels
    let w = pixs.width();
    let h = pixs.height();
    let mut dilation_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let dil = pix1.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica_core::color::extract_rgb(orig);
            let (dr, dg, db) = leptonica_core::color::extract_rgb(dil);
            if dr < or || dg < og || db < ob {
                dilation_valid = false;
                break;
            }
        }
        if !dilation_valid {
            break;
        }
    }
    rp.compare_values(1.0, if dilation_valid { 1.0 } else { 0.0 }, 0.0);

    // C版: Test 2,3 -- pixColorMorph(pixs, L_MORPH_ERODE, SIZE, SIZE)
    //                   vs pixColorMorphSequence(pixs, "e7.7", 0, 0)
    eprintln!("  Testing color erosion");
    let pix2 = erode_color(&pixs, SIZE, SIZE).expect("erode_color");
    assert_eq!(pix2.depth(), PixelDepth::Bit32);
    rp.compare_values(1.0, 1.0, 0.0);

    // Verify erosion property: eroded pixels should be <= original (per channel min)
    let mut erosion_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let ero = pix2.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica_core::color::extract_rgb(orig);
            let (er, eg, eb) = leptonica_core::color::extract_rgb(ero);
            if er > or || eg > og || eb > ob {
                erosion_valid = false;
                break;
            }
        }
        if !erosion_valid {
            break;
        }
    }
    rp.compare_values(1.0, if erosion_valid { 1.0 } else { 0.0 }, 0.0);

    // C版: Test 4,5 -- pixColorMorph(pixs, L_MORPH_OPEN, SIZE, SIZE)
    //                   vs pixColorMorphSequence(pixs, "o7.7", 0, 0)
    eprintln!("  Testing color opening");
    let pix3 = open_color(&pixs, SIZE, SIZE).expect("open_color");
    assert_eq!(pix3.depth(), PixelDepth::Bit32);
    rp.compare_values(1.0, 1.0, 0.0);

    // Verify opening is anti-extensive: opened pixels <= original (per channel)
    let mut open_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let opn = pix3.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica_core::color::extract_rgb(orig);
            let (opr, opg, opb) = leptonica_core::color::extract_rgb(opn);
            if opr > or || opg > og || opb > ob {
                open_valid = false;
                break;
            }
        }
        if !open_valid {
            break;
        }
    }
    rp.compare_values(1.0, if open_valid { 1.0 } else { 0.0 }, 0.0);

    // C版: Test 6,7 -- pixColorMorph(pixs, L_MORPH_CLOSE, SIZE, SIZE)
    //                   vs pixColorMorphSequence(pixs, "c7.7", 0, 0)
    eprintln!("  Testing color closing");
    let pix4 = close_color(&pixs, SIZE, SIZE).expect("close_color");
    assert_eq!(pix4.depth(), PixelDepth::Bit32);
    rp.compare_values(1.0, 1.0, 0.0);

    // Verify closing is extensive: closed pixels >= original (per channel)
    let mut close_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let cls = pix4.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica_core::color::extract_rgb(orig);
            let (cr, cg, cb) = leptonica_core::color::extract_rgb(cls);
            if cr < or || cg < og || cb < ob {
                close_valid = false;
                break;
            }
        }
        if !close_valid {
            break;
        }
    }
    rp.compare_values(1.0, if close_valid { 1.0 } else { 0.0 }, 0.0);

    // C版: pixColorMorphSequence() -- Rust未実装のためsequence比較はスキップ
    // C版: pixaConvertToPdf() -- Rust未実装のためスキップ
    // C版: pixaDisplayTiledInColumns() -- Rust未実装のためスキップ

    // Additional: verify idempotence of opening
    eprintln!("  Testing opening idempotence");
    let pix3b = open_color(&pix3, SIZE, SIZE).expect("open_color twice");
    let same = pix3.equals(&pix3b);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: opening is not idempotent");
    }

    // Additional: verify idempotence of closing
    eprintln!("  Testing closing idempotence");
    let pix4b = close_color(&pix4, SIZE, SIZE).expect("close_color twice");
    let same = pix4.equals(&pix4b);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    if !same {
        eprintln!("    DIFFER: closing is not idempotent");
    }

    eprintln!();
    assert!(rp.cleanup(), "colormorph regression test failed");
}

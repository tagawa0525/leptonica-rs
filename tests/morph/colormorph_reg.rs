//! Color morphology regression test
//!
//! C version: prog/colormorph_reg.c
//! Tests dilate_color, erode_color, open_color, close_color and compares
//! direct operations with color_morph_sequence results.
//!
//! C checkpoint mapping (8 total):
//!   0: write_pix_and_check dilate_color
//!   1: compare_pix dilate vs sequence "d7.7"
//!   2: write_pix_and_check erode_color
//!   3: compare_pix erode vs sequence "e7.7"
//!   4: write_pix_and_check open_color
//!   5: compare_pix open vs sequence "o7.7"
//!   6: write_pix_and_check close_color
//!   7: compare_pix close vs sequence "c7.7"

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{close_color, color_morph_sequence, dilate_color, erode_color, open_color};

const SIZE: u32 = 7;

#[test]
fn colormorph_reg() {
    let mut rp = RegParams::new("colormorph");

    let pixs = load_test_image("wyom.jpg").expect("load wyom.jpg");
    assert_eq!(pixs.depth(), PixelDepth::Bit32);

    // C check 0: write_pix_and_check dilate_color(7,7)
    let pix_dil = dilate_color(&pixs, SIZE, SIZE).expect("dilate_color");
    rp.write_pix_and_check(&pix_dil, ImageFormat::Jpeg)
        .expect("write dilate result");

    // C check 1: compare_pix dilate vs color_morph_sequence("d7.7")
    let pix_dil_seq = color_morph_sequence(&pixs, "d7.7").expect("color_morph_sequence d7.7");
    rp.compare_pix(&pix_dil, &pix_dil_seq);

    // C check 2: write_pix_and_check erode_color(7,7)
    let pix_ero = erode_color(&pixs, SIZE, SIZE).expect("erode_color");
    rp.write_pix_and_check(&pix_ero, ImageFormat::Jpeg)
        .expect("write erode result");

    // C check 3: compare_pix erode vs color_morph_sequence("e7.7")
    let pix_ero_seq = color_morph_sequence(&pixs, "e7.7").expect("color_morph_sequence e7.7");
    rp.compare_pix(&pix_ero, &pix_ero_seq);

    // C check 4: write_pix_and_check open_color(7,7)
    let pix_opn = open_color(&pixs, SIZE, SIZE).expect("open_color");
    rp.write_pix_and_check(&pix_opn, ImageFormat::Jpeg)
        .expect("write open result");

    // C check 5: compare_pix open vs color_morph_sequence("o7.7")
    let pix_opn_seq = color_morph_sequence(&pixs, "o7.7").expect("color_morph_sequence o7.7");
    rp.compare_pix(&pix_opn, &pix_opn_seq);

    // C check 6: write_pix_and_check close_color(7,7)
    let pix_cls = close_color(&pixs, SIZE, SIZE).expect("close_color");
    rp.write_pix_and_check(&pix_cls, ImageFormat::Jpeg)
        .expect("write close result");

    // C check 7: compare_pix close vs color_morph_sequence("c7.7")
    let pix_cls_seq = color_morph_sequence(&pixs, "c7.7").expect("color_morph_sequence c7.7");
    rp.compare_pix(&pix_cls, &pix_cls_seq);

    // Additional: channel monotonicity and idempotence
    let w = pixs.width();
    let h = pixs.height();

    // Dilation: per-channel max (dilated >= original)
    let mut dilation_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let dil = pix_dil.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica::core::pixel::extract_rgb(orig);
            let (dr, dg, db) = leptonica::core::pixel::extract_rgb(dil);
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

    // Erosion: per-channel min (eroded <= original)
    let mut erosion_valid = true;
    for y in (0..h).step_by(10) {
        for x in (0..w).step_by(10) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let ero = pix_ero.get_pixel(x, y).unwrap_or(0);
            let (or, og, ob) = leptonica::core::pixel::extract_rgb(orig);
            let (er, eg, eb) = leptonica::core::pixel::extract_rgb(ero);
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

    // Opening idempotence
    let pix_opn2 = open_color(&pix_opn, SIZE, SIZE).expect("open_color twice");
    rp.compare_values(1.0, if pix_opn.equals(&pix_opn2) { 1.0 } else { 0.0 }, 0.0);

    // Closing idempotence
    let pix_cls2 = close_color(&pix_cls, SIZE, SIZE).expect("close_color twice");
    rp.compare_values(1.0, if pix_cls.equals(&pix_cls2) { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "colormorph regression test failed");
}

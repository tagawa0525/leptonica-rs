//! Write-text regression test
//!
//! Uses bitmap-font rendering and image I/O round-trip.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/writetext_reg.c`

use crate::common::{RegParams, load_test_image};
use leptonica::color::{threshold_to_2bpp, threshold_to_4bpp, threshold_to_binary};
use leptonica::io::{ImageFormat, read_image_mem, write_image_mem};
use leptonica::{Bmf, Pix, PixelDepth, TextLocation};

#[test]
fn writetext_reg() {
    let mut rp = RegParams::new("writetext");

    // --- Part 1: Synthetic image + text ---
    let src = Pix::new(120, 60, PixelDepth::Bit8).expect("create src");
    let bmf = Bmf::new(10).expect("create bmf");
    let with_text = bmf
        .add_textlines(&src, "writetext regression", 0, TextLocation::Below)
        .expect("add_textlines");

    rp.write_pix_and_check(&with_text, ImageFormat::Png)
        .expect("check with_text");
    rp.compare_values(src.width() as f64, with_text.width() as f64, 0.0);
    rp.compare_values(
        1.0,
        if with_text.height() > src.height() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let png = write_image_mem(&with_text, ImageFormat::Png).expect("write_image_mem png");
    let restored = read_image_mem(&png).expect("read_image_mem png");
    rp.write_pix_and_check(&restored, ImageFormat::Png)
        .expect("check restored");
    rp.compare_values(with_text.width() as f64, restored.width() as f64, 0.0);
    rp.compare_values(with_text.height() as f64, restored.height() as f64, 0.0);

    let mut rm = restored.to_mut();
    rm.set_text(Some("writetext-spix".to_string()));
    let restored: Pix = rm.into();
    let spix = restored.write_spix_to_bytes().expect("write_spix_to_bytes");
    let restored_spix = Pix::read_spix_from_bytes(&spix).expect("read_spix_from_bytes");
    rp.compare_values(restored.width() as f64, restored_spix.width() as f64, 0.0);
    rp.compare_values(restored.height() as f64, restored_spix.height() as f64, 0.0);

    // --- Part 2: Real image + text below ---
    let pix = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let bmf6 = Bmf::new(6).expect("create bmf size 6");
    let with_text2 = bmf6
        .add_textlines(&pix, "regression test", 0, TextLocation::Below)
        .expect("add_textlines to real image");
    rp.write_pix_and_check(&with_text2, ImageFormat::Png)
        .expect("check real image with text");

    assert!(rp.cleanup(), "writetext regression test failed");
}

/// Test add_textlines on 8 different image depths (C: main loop in writetext_reg.c).
///
/// C version creates 8 image types and calls AddTextAndSave (pixAddSingleTextblock)
/// for 4 locations.  Rust's add_textlines covers Above and Below; Left and Right
/// are Rust-only additions.  Cmapped images are tested via threshold_to_2bpp and
/// threshold_to_4bpp.
#[test]
fn writetext_reg_multi_depth() {
    let mut rp = RegParams::new("writetext_multi");

    let pix8 = load_test_image("lucasta.150.jpg").expect("load lucasta.150.jpg");
    let bmf = Bmf::new(6).expect("create bmf size 6");
    let text = "text rendering test";
    let w = pix8.width();
    let h = pix8.height();

    // 8 bpp grayscale — Above and Below
    for &loc in &[TextLocation::Above, TextLocation::Below] {
        let r = bmf
            .add_textlines(&pix8, text, 0, loc)
            .expect("add_textlines 8bpp");
        rp.compare_values(w as f64, r.width() as f64, 0.0);
        rp.compare_values(1.0, if r.height() > h { 1.0 } else { 0.0 }, 0.0);
        rp.write_pix_and_check(&r, ImageFormat::Png)
            .expect("write 8bpp");
    }

    // 32 bpp RGB
    let pix32 = pix8.convert_to_32().expect("convert_to_32");
    for &loc in &[TextLocation::Above, TextLocation::Below] {
        let r = bmf
            .add_textlines(&pix32, text, 0xff00_0000, loc)
            .expect("add_textlines 32bpp");
        rp.compare_values(w as f64, r.width() as f64, 0.0);
        rp.write_pix_and_check(&r, ImageFormat::Png)
            .expect("write 32bpp");
    }

    // 4 bpp cmapped
    let pix4c = threshold_to_4bpp(&pix8, 10, true).expect("threshold_to_4bpp");
    assert_eq!(pix4c.depth(), PixelDepth::Bit4);
    let r = bmf
        .add_textlines(&pix4c, text, 0, TextLocation::Below)
        .expect("add_textlines 4bpp cmapped");
    rp.compare_values(w as f64, r.width() as f64, 0.0);
    rp.write_pix_and_check(&r, ImageFormat::Png)
        .expect("write 4bpp");

    // 2 bpp cmapped
    let pix2c = threshold_to_2bpp(&pix8, 3, true).expect("threshold_to_2bpp");
    assert_eq!(pix2c.depth(), PixelDepth::Bit2);
    let r = bmf
        .add_textlines(&pix2c, text, 0, TextLocation::Below)
        .expect("add_textlines 2bpp cmapped");
    rp.compare_values(w as f64, r.width() as f64, 0.0);
    rp.write_pix_and_check(&r, ImageFormat::Png)
        .expect("write 2bpp");

    // 1 bpp binary
    let pix1 = threshold_to_binary(&pix8, 160).expect("threshold_to_binary");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);
    let r = bmf
        .add_textlines(&pix1, text, 0, TextLocation::Below)
        .expect("add_textlines 1bpp");
    rp.compare_values(w as f64, r.width() as f64, 0.0);
    rp.write_pix_and_check(&r, ImageFormat::Png)
        .expect("write 1bpp");

    assert!(rp.cleanup(), "writetext multi-depth test failed");
}

/// Test set_textline for per-line colored text (C part 2: pixSetTextline loop).
///
/// C version loads weasel4.11c.png, scales 8×, quantizes, then writes 6 text
/// lines at different y positions with 6 different colors.
#[test]
fn writetext_reg_set_textline() {
    let mut rp = RegParams::new("writetext_setline");

    // Use a 32bpp canvas for multi-color text
    let canvas = Pix::new(200, 400, PixelDepth::Bit32).expect("create canvas");
    let bmf = Bmf::new(10).expect("create bmf");

    let colors = [
        0x4090_e000u32,
        0x40e0_9000,
        0x9040_e000,
        0x90e0_4000,
        0xe040_9000,
        0xe090_4000,
    ];

    let mut current = canvas;
    for (i, &color) in colors.iter().enumerate() {
        let line = format!("This is textline {i}");
        let y = 50 + 50 * i as i32;
        let (rendered, _overflow) = bmf
            .set_textline(&current, &line, 10, y, color)
            .expect("set_textline");
        current = rendered;
    }

    rp.write_pix_and_check(&current, ImageFormat::Png)
        .expect("write multi-color text");
    rp.compare_values(200.0, current.width() as f64, 0.0);
    rp.compare_values(400.0, current.height() as f64, 0.0);

    assert!(rp.cleanup(), "writetext set_textline test failed");
}

#[test]
#[ignore = "pixAddSingleTextblock not implemented"]
fn writetext_reg_single_textblock() {}

#[test]
#[ignore = "pixaDisplayTiledInColumns not implemented"]
fn writetext_reg_tiled_display() {}

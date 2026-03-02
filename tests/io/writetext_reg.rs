//! Write-text regression test
//!
//! Uses bitmap-font rendering and image I/O round-trip.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/writetext_reg.c`

use crate::common::{RegParams, load_test_image};
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

#[test]
#[ignore = "pixAddSingleTextblock not implemented"]
fn writetext_reg_single_textblock() {}

#[test]
#[ignore = "pixaDisplayTiledInColumns not implemented"]
fn writetext_reg_tiled_display() {}

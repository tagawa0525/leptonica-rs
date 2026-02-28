//! Write-text regression test
//!
//! Uses bitmap-font rendering and image I/O round-trip.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/writetext_reg.c`

use crate::common::RegParams;
use leptonica::io::{ImageFormat, read_image_mem, write_image_mem};
use leptonica::{Bmf, Pix, PixelDepth, TextLocation};

#[test]
fn writetext_reg() {
    let mut rp = RegParams::new("writetext");

    let src = Pix::new(120, 60, PixelDepth::Bit8).expect("create src");
    let bmf = Bmf::new(10).expect("create bmf");
    let with_text = bmf
        .add_textlines(&src, "writetext regression", 0, TextLocation::Below)
        .expect("add_textlines");

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
    rp.compare_values(with_text.width() as f64, restored.width() as f64, 0.0);
    rp.compare_values(with_text.height() as f64, restored.height() as f64, 0.0);

    let mut rm = restored.to_mut();
    rm.set_text(Some("writetext-spix".to_string()));
    let restored: Pix = rm.into();
    let spix = restored.write_spix_to_bytes().expect("write_spix_to_bytes");
    let restored_spix = Pix::read_spix_from_bytes(&spix).expect("read_spix_from_bytes");
    rp.compare_values(restored.width() as f64, restored_spix.width() as f64, 0.0);
    rp.compare_values(restored.height() as f64, restored_spix.height() as f64, 0.0);

    assert!(rp.cleanup(), "writetext regression test failed");
}

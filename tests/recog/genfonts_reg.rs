//! Generated-font regression test
//!
//! Trains `Recog` with generated bitmap-font glyphs.
//!
//! # See also
//!
//! C Leptonica: `prog/genfonts_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::recog::recog::create;
use leptonica::{Bmf, Pixa};

#[test]
fn genfonts_reg() {
    let mut rp = RegParams::new("genfonts");

    let bmf = Bmf::new(12).expect("create bitmap font");
    let mut recog = create(0, 40, 0, 180, 1).expect("create recog");

    for ch in ['A', 'B', 'C', 'D'] {
        let glyph = bmf.get_pix(ch).expect("glyph");
        recog
            .train_labeled(&glyph, &ch.to_string())
            .expect("train glyph");
    }
    recog.finish_training().expect("finish training");

    let sample_counts = recog.get_sample_counts();
    let total_samples: usize = sample_counts.iter().sum();
    rp.compare_values(4.0, total_samples as f64, 0.0);

    // C checks 0-8 equivalent: collect glyphs into Pixa and display tiled
    let mut pixa = Pixa::new();
    for ch in b'A'..=b'Z' {
        if let Some(glyph) = bmf.get_pix(ch as char) {
            pixa.push(glyph);
        }
    }
    let tiled = pixa
        .display_tiled(500, 0, 10)
        .expect("display tiled glyphs");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: genfonts glyph tiled");

    let probe = bmf.get_pix('B').expect("probe glyph");
    let rch = recog.identify_pix(&probe).expect("identify");
    rp.compare_values(1.0, if !rch.text.is_empty() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if rch.score >= 0.0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "genfonts regression test failed");
}

/// C-comparable font pixa displays (plan 902 PR 12).
///
/// Mirrors the string-generated section of C genfonts_reg (checks 9-17):
/// for each font size, tile the 95-glyph font pixa with
/// pixaDisplayTiled(1500, 0, 15). Pairs with C genfonts.09-17; the C
/// manifest itself proves the file path (checks 0-8) produces identical
/// output, so one series covers both.
#[test]
fn genfonts_c_compat() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("genfonts_c");

    for size in [4u32, 6, 8, 10, 12, 14, 16, 18, 20] {
        let bmf = Bmf::new(size).expect("create bmf");
        let tiled = bmf
            .get_font_pixa()
            .display_tiled(1500, 0, 15)
            .expect("display tiled font");
        rp.write_pix_and_check(&tiled, ImageFormat::Png)
            .expect("check: tiled font pixa");
    }

    assert!(rp.cleanup(), "genfonts c-compat test failed");
}

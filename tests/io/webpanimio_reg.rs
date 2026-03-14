//! Animated WebP regression test
//!
//! Covers memory/stream/file animation writers.
//!
//! # See also
//!
//! C Leptonica: `prog/webpanimio_reg.c`

use crate::common::RegParams;
use leptonica::core::pixel;
use leptonica::io::webp::{
    WebPAnimOptions, write_webp_anim, write_webp_anim_file, write_webp_anim_mem,
};
use leptonica::{Pix, Pixa, PixelDepth};

fn make_frame(w: u32, h: u32, rgb: (u8, u8, u8)) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).expect("create frame");
    let mut pm = pix.try_into_mut().expect("mutable frame");
    let p = pixel::compose_rgb(rgb.0, rgb.1, rgb.2);
    for y in 0..h {
        for x in 0..w {
            pm.set_pixel_unchecked(x, y, p);
        }
    }
    pm.into()
}

#[test]
fn webpanimio_reg() {
    let mut rp = RegParams::new("webpanimio");

    let mut pixa = Pixa::new();
    pixa.push(make_frame(32, 20, (255, 0, 0)));
    pixa.push(make_frame(32, 20, (0, 255, 0)));
    pixa.push(make_frame(32, 20, (0, 0, 255)));

    let options = WebPAnimOptions {
        loop_count: 1,
        duration_ms: 80,
        quality: 75,
        lossless: true,
    };

    let data = write_webp_anim_mem(&pixa, &options).expect("write_webp_anim_mem");
    rp.compare_values(1.0, if data.len() > 12 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if &data[0..4] == b"RIFF" { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if &data[8..12] == b"WEBP" { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(
        1.0,
        if data.windows(4).any(|w| w == b"ANIM") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let mut stream = Vec::new();
    write_webp_anim(&pixa, &mut stream, &options).expect("write_webp_anim");
    rp.compare_values(1.0, if stream.len() > 12 { 1.0 } else { 0.0 }, 0.0);

    let dir = std::env::temp_dir().join("leptonica_webpanimio_reg");
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("anim.webp");
    write_webp_anim_file(&pixa, &path, &options).expect("write_webp_anim_file");
    rp.compare_values(1.0, if path.exists() { 1.0 } else { 0.0 }, 0.0);
    std::fs::remove_dir_all(&dir).ok();

    assert!(rp.cleanup(), "webpanimio regression test failed");
}

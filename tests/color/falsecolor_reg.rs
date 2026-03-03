//! False-color regression test
//!
//! Uses color-mapping transforms as the Rust mapping for false-color workflows.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/falsecolor_reg.c`

use crate::common::RegParams;
use leptonica::color::{
    pix_linear_map_to_target_color, pix_map_with_invariant_hue, pix_shift_by_component,
};
use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::{Pix, PixelDepth};

fn make_gray_gradient(w: u32, h: u32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit32).expect("create gradient");
    let mut pm = pix.try_into_mut().expect("mutable gradient");
    for y in 0..h {
        for x in 0..w {
            let v = ((x * 255) / (w.saturating_sub(1).max(1))) as u8;
            pm.set_pixel_unchecked(x, y, pixel::compose_rgb(v, v, v));
        }
    }
    pm.into()
}

#[test]
fn falsecolor_reg() {
    let mut rp = RegParams::new("falsecolor");

    let pix = make_gray_gradient(16, 4);

    let mapped = pix_linear_map_to_target_color(&pix, 0x80808000, 0xff400000).expect("linear map");
    rp.compare_values(16.0, mapped.width() as f64, 0.0);
    rp.compare_values(4.0, mapped.height() as f64, 0.0);
    rp.write_pix_and_check(&mapped, ImageFormat::Png)
        .expect("write mapped falsecolor");

    let p0 = mapped.get_pixel_unchecked(0, 0);
    let p1 = mapped.get_pixel_unchecked(15, 0);
    rp.compare_values(1.0, if p0 != p1 { 1.0 } else { 0.0 }, 0.0);

    let shifted = pix_shift_by_component(&pix, 0xffffff00, 0x80c0ff00).expect("shift by component");
    rp.compare_values(
        1.0,
        if shifted.get_pixel_unchecked(10, 2) != pix.get_pixel_unchecked(10, 2) {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let inv_hue = pix_map_with_invariant_hue(&mapped, 0xff000000, 0.4).expect("invariant hue");
    rp.compare_values(mapped.width() as f64, inv_hue.width() as f64, 0.0);
    rp.compare_values(mapped.height() as f64, inv_hue.height() as f64, 0.0);

    let pix8 = Pix::new(8, 8, PixelDepth::Bit8).expect("create 8bpp");
    rp.compare_values(
        1.0,
        if pix_linear_map_to_target_color(&pix8, 0x80808000, 0xff000000).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "falsecolor regression test failed");
}

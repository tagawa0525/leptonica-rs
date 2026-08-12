//! False-color regression test
//!
//! Uses color-mapping transforms as the Rust mapping for false-color workflows.
//!
//! # See also
//!
//! C Leptonica: `prog/falsecolor_reg.c`

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
    rp.write_pix_and_check(&shifted, ImageFormat::Png)
        .expect("check: shift by component");

    let inv_hue = pix_map_with_invariant_hue(&mapped, 0xff000000, 0.4).expect("invariant hue");
    rp.compare_values(mapped.width() as f64, inv_hue.width() as f64, 0.0);
    rp.compare_values(mapped.height() as f64, inv_hue.height() as f64, 0.0);
    rp.write_pix_and_check(&inv_hue, ImageFormat::Png)
        .expect("check: invariant hue");

    // Additional: linear map with different target
    let mapped2 =
        pix_linear_map_to_target_color(&pix, 0x80808000, 0x0040ff00).expect("linear map blue");
    rp.write_pix_and_check(&mapped2, ImageFormat::Png)
        .expect("check: linear map blue target");

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

/// C-comparable false color conversion (plan 902 PR 11).
///
/// Mirrors C falsecolor_reg exactly: 768x100 synthetic 8bpp / 16bpp
/// horizontal gradients, then `convert_gray_to_false_color` with
/// gamma in {1.0, 2.0, 3.0}. All eight outputs pair with C
/// falsecolor.00-07 (lossless PNG, no codec ambiguity).
#[test]
fn falsecolor_c_compat() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("falsecolor_c");

    // C: pixCreate(768, 100, 8/16) with val = 0xff * j / 768 (0xffff for 16)
    let pix8 = {
        let p = Pix::new(768, 100, PixelDepth::Bit8).expect("create 8bpp gradient");
        let mut pm = p.try_into_mut().expect("mutable 8bpp gradient");
        for y in 0..100 {
            for x in 0..768u32 {
                pm.set_pixel_unchecked(x, y, 0xff * x / 768);
            }
        }
        let p: Pix = pm.into();
        p
    };
    let pix16 = {
        let p = Pix::new(768, 100, PixelDepth::Bit16).expect("create 16bpp gradient");
        let mut pm = p.try_into_mut().expect("mutable 16bpp gradient");
        for y in 0..100 {
            for x in 0..768u32 {
                pm.set_pixel_unchecked(x, y, 0xffff * x / 768);
            }
        }
        let p: Pix = pm.into();
        p
    };

    // C checks 0-1: the raw gradients
    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("check: 8bpp gradient");
    rp.write_pix_and_check(&pix16, ImageFormat::Png)
        .expect("check: 16bpp gradient");

    // C checks 2-4 (8bpp) and 5-7 (16bpp): false color with gamma sweep
    for gamma in [1.0f32, 2.0, 3.0] {
        let fc = pix8
            .convert_gray_to_false_color(gamma)
            .expect("false color 8bpp");
        rp.write_pix_and_check(&fc, ImageFormat::Png)
            .expect("check: false color 8bpp");
    }
    for gamma in [1.0f32, 2.0, 3.0] {
        let fc = pix16
            .convert_gray_to_false_color(gamma)
            .expect("false color 16bpp");
        rp.write_pix_and_check(&fc, ImageFormat::Png)
            .expect("check: false color 16bpp");
    }

    assert!(rp.cleanup(), "falsecolor c-compat test failed");
}

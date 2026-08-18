//! Scale regression test
//!
//! C version: `prog/scale_reg.c`
//!
//! Tests various scaling operations:
//!   1. Scale up by 2x — dimensions double
//!   2. Scale down by 0.5x — dimensions halve
//!   3. Scale to specific target size
//!   4. Scale by sampling (nearest-neighbor)
//!   5. Scale by 1.0 preserves dimensions
//!   6. Anisotropic scaling (different x/y factors)
//!   7. Scale with different methods (Linear, Sampling)
//!   8. Scale on binary (1bpp) image
//!
//! C version tests `pixScale` on 10 images of varying depth/colormap,
//! and also tests `pixScaleToGray*`, `pixScaleSmoothToSize`, etc.

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::transform::{
    ScaleMethod, scale, scale_by_sampling, scale_to_gray_2, scale_to_gray_3, scale_to_gray_4,
    scale_to_gray_6, scale_to_gray_8, scale_to_size,
};

/// Test scaling operations on grayscale and binary images
///
/// C version: tests `pixScale` at factors [2.3, 1.5, 1.1, 0.6, 0.3] on each
/// of 10 image types (1bpp, 2bpp, 4bpp, 8bpp, 16bpp, 32bpp with/without cmap).
#[test]
#[ignore = "not yet implemented"]
fn scale_reg() {
    let mut rp = RegParams::new("scale");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Scale up 2x ---
    // C version: pixc = pixScale(pixs, 2.25, 2.25)
    let up2 = scale(&pixs, 2.0, 2.0, ScaleMethod::Linear).expect("scale 2x");
    rp.compare_values((w * 2) as f64, up2.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, up2.height() as f64, 1.0);
    rp.write_pix_and_check(&up2, ImageFormat::Png)
        .expect("write up2");
    eprintln!("  scale 2x: {}x{}", up2.width(), up2.height());

    // --- Test 2: Scale down 0.5x ---
    // C version: pixc = pixScale(pixs, 0.65, 0.65) etc.
    let down2 = scale(&pixs, 0.5, 0.5, ScaleMethod::Linear).expect("scale 0.5x");
    rp.compare_values((w / 2) as f64, down2.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, down2.height() as f64, 1.0);
    rp.write_pix_and_check(&down2, ImageFormat::Png)
        .expect("write down2");
    eprintln!("  scale 0.5x: {}x{}", down2.width(), down2.height());

    // --- Test 3: Scale to specific size ---
    let target_w = 200u32;
    let target_h = 150u32;
    let sized = scale_to_size(&pixs, target_w, target_h).expect("scale_to_size");
    rp.compare_values(target_w as f64, sized.width() as f64, 0.0);
    rp.compare_values(target_h as f64, sized.height() as f64, 0.0);
    rp.write_pix_and_check(&sized, ImageFormat::Png)
        .expect("write sized");
    eprintln!(
        "  scale_to_size(200,150): {}x{}",
        sized.width(),
        sized.height()
    );

    // --- Test 4: Scale by sampling ---
    // C version: pixScaleBySampling used internally for 1bpp upscaling
    let sampled = scale_by_sampling(&pixs, 2.0, 2.0).expect("scale_by_sampling 2x");
    rp.compare_values((w * 2) as f64, sampled.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, sampled.height() as f64, 1.0);

    // --- Test 5: Scale 1.0 should preserve dimensions ---
    let s1 = scale(&pixs, 1.0, 1.0, ScaleMethod::Linear).expect("scale 1x");
    rp.compare_values(w as f64, s1.width() as f64, 0.0);
    rp.compare_values(h as f64, s1.height() as f64, 0.0);

    // --- Test 6: Anisotropic scaling ---
    let aniso = scale(&pixs, 2.0, 0.5, ScaleMethod::Linear).expect("aniso scale");
    rp.compare_values((w * 2) as f64, aniso.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, aniso.height() as f64, 1.0);
    rp.write_pix_and_check(&aniso, ImageFormat::Png)
        .expect("write aniso");
    eprintln!(
        "  aniso scale(2.0, 0.5): {}x{}",
        aniso.width(),
        aniso.height()
    );

    // --- Test 7: Scale with different methods ---
    for method in [ScaleMethod::Linear, ScaleMethod::Sampling] {
        let s = scale(&pixs, 1.5, 1.5, method).expect("scale method");
        rp.compare_values(
            1.0,
            if s.width() > 0 && s.height() > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!("  scale {:?} 1.5x: {}x{}", method, s.width(), s.height());
    }

    // --- Test 8: Scale with binary image ---
    // C version: pixs = pixRead("feyn-fract.tif"); pixc = pixScale(pixs, 0.32, 0.32)
    let pixb = load_test_image("feyn-fract.tif").expect("load binary");
    let sb = scale(&pixb, 2.0, 2.0, ScaleMethod::Sampling).expect("scale binary");
    rp.compare_values((pixb.width() * 2) as f64, sb.width() as f64, 1.0);
    rp.compare_values((pixb.height() * 2) as f64, sb.height() as f64, 1.0);

    // --- Test 9: pixScaleToGray at fixed reduction factors ---
    // C version tests pixScaleToGray2/3/4/6/8 on 1bpp images
    let stg2 = scale_to_gray_2(&pixb).expect("scale_to_gray_2");
    rp.compare_values((pixb.width() / 2) as f64, stg2.width() as f64, 1.0);
    rp.write_pix_and_check(&stg2, ImageFormat::Png)
        .expect("check: scale_to_gray_2");

    // C truncates the 3x and 6x destination widths to a multiple of 8, and
    // the 4x width to an even number (pixScaleToGray{3,4,6}).
    let stg3 = scale_to_gray_3(&pixb).expect("scale_to_gray_3");
    rp.compare_values(((pixb.width() / 3) & !7) as f64, stg3.width() as f64, 0.0);
    rp.write_pix_and_check(&stg3, ImageFormat::Png)
        .expect("check: scale_to_gray_3");

    let stg4 = scale_to_gray_4(&pixb).expect("scale_to_gray_4");
    rp.compare_values(((pixb.width() / 4) & !1) as f64, stg4.width() as f64, 0.0);
    rp.write_pix_and_check(&stg4, ImageFormat::Png)
        .expect("check: scale_to_gray_4");

    let stg6 = scale_to_gray_6(&pixb).expect("scale_to_gray_6");
    rp.compare_values(((pixb.width() / 6) & !7) as f64, stg6.width() as f64, 0.0);
    rp.write_pix_and_check(&stg6, ImageFormat::Png)
        .expect("check: scale_to_gray_6");

    let stg8 = scale_to_gray_8(&pixb).expect("scale_to_gray_8");
    rp.compare_values((pixb.width() / 8) as f64, stg8.width() as f64, 1.0);
    rp.write_pix_and_check(&stg8, ImageFormat::Png)
        .expect("check: scale_to_gray_8");

    assert!(rp.cleanup(), "scale regression test failed");
}

/// scale_general must follow C pixScaleGeneral (plan 902 PR 17).
///
/// C dispatches to the public `pixScaleAreaMap` / `pixScaleGrayLI` /
/// `pixScaleColorLI` entry points (so their special cases and
/// `(int)(scale * ws + 0.5)` sizing apply) and then sharpens with
/// `pixUnsharpMasking` when `sharpfract`/`sharpwidth` are positive:
/// for reductions when `maxscale > 0.2`, for magnifications when
/// `maxscale < 1.4`.
#[test]
fn scale_general_matches_c_dispatch() {
    use leptonica::transform::{scale, scale_area_map, scale_general};
    use leptonica::{Pix, PixelDepth};

    // A 465-tall image at 0.5 must use the area-map 1/2 special case,
    // giving 232 (not the 233 that rounding would produce).
    let pix = Pix::new(300, 465, PixelDepth::Bit32).expect("create");
    let out = scale_general(&pix, 0.5, 0.5, 0.0, 0).expect("scale_general");
    assert_eq!((out.width(), out.height()), (150, 232));
    let direct = scale_area_map(&pix, 0.5, 0.5).expect("scale_area_map");
    assert_eq!((direct.width(), direct.height()), (150, 232));

    // With sharpening disabled the result must equal the bare area map.
    // A blocky pattern: a linear ramp would make the box lowpass equal the
    // centre value, so sharpening would be a no-op.
    let gradient = {
        let p = Pix::new(64, 64, PixelDepth::Bit8).expect("create gray");
        let mut pm = p.try_into_mut().unwrap();
        for y in 0..64u32 {
            for x in 0..64u32 {
                let v = if ((x / 4) + (y / 4)) % 2 == 0 {
                    40
                } else {
                    210
                };
                pm.set_pixel(x, y, v).unwrap();
            }
        }
        let p: Pix = pm.into();
        p
    };
    let plain = scale_general(&gradient, 0.5, 0.5, 0.0, 0).expect("no sharpening");
    let area = scale_area_map(&gradient, 0.5, 0.5).expect("area map");
    for y in 0..plain.height() {
        for x in 0..plain.width() {
            assert_eq!(plain.get_pixel(x, y), area.get_pixel(x, y), "({x}, {y})");
        }
    }

    // pixScale applies sharpening by default, so it must differ from the
    // unsharpened path on an image with edges.
    let sharpened =
        scale(&gradient, 0.5, 0.5, leptonica::transform::ScaleMethod::Auto).expect("scale");
    assert_eq!(
        (sharpened.width(), sharpened.height()),
        (plain.width(), plain.height())
    );
    let differs = (0..plain.height())
        .any(|y| (0..plain.width()).any(|x| sharpened.get_pixel(x, y) != plain.get_pixel(x, y)));
    assert!(differs, "pixScale must sharpen by default");
}

/// scale_area_map_2 must truncate like C (plan 902 PR 17).
///
/// C `scaleAreaMapLow2` averages each 2x2 block with `val >>= 2` — a
/// truncating shift, not a `+2` round-half-up — and composes only the
/// three colour bytes (alpha stays 0).
#[test]
fn scale_area_map_2_truncates_like_c() {
    use leptonica::transform::scale_area_map_2;
    use leptonica::{Pix, PixelDepth};

    // 2x2 gray block summing to 402: C gives 402 >> 2 = 100,
    // rounding would give (402 + 2) / 4 = 101.
    let gray = {
        let p = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        pm.set_pixel(0, 0, 100).unwrap();
        pm.set_pixel(1, 0, 101).unwrap();
        pm.set_pixel(0, 1, 100).unwrap();
        pm.set_pixel(1, 1, 101).unwrap();
        let p: Pix = pm.into();
        p
    };
    let out = scale_area_map_2(&gray).expect("scale_area_map_2 gray");
    assert_eq!((out.width(), out.height()), (1, 1));
    assert_eq!(out.get_pixel(0, 0).unwrap(), 100);

    // Same for each colour channel of a 32bpp block.
    let color = {
        let p = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        for (x, y, v) in [
            (0u32, 0u32, 0x6465_66ffu32),
            (1, 0, 0x6566_67ff),
            (0, 1, 0x6465_66ff),
            (1, 1, 0x6566_67ff),
        ] {
            pm.set_pixel(x, y, v).unwrap();
        }
        let p: Pix = pm.into();
        p
    };
    let out = scale_area_map_2(&color).expect("scale_area_map_2 color");
    // r: (100+101)*2 = 402 >> 2 = 100 = 0x64, likewise g = 0x65, b = 0x66.
    // C composeRGBPixel leaves the alpha byte 0.
    assert_eq!(out.get_pixel(0, 0).unwrap(), 0x6465_6600);
}

/// C-compat: `prog/scale_reg.c`, every PNG output from a lossless input.
///
/// The program writes most of its results as JPEG, but the 1 bpp block
/// (checks 0-5), the 2 and 4 bpp blocks (20-22, 24-26, 28-30) and the
/// `scale_to_size` check (35) are PNG and read only lossless images, so they
/// are bit-exactly comparable against C.
#[test]
#[ignore = "not yet implemented"]
fn scale_c_compat() {
    use leptonica::transform::{
        ScaleMethod, scale, scale_to_gray_3, scale_to_gray_4, scale_to_gray_6, scale_to_gray_8,
        scale_to_gray_16, scale_to_size,
    };

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("scale_c");
    let sc = |p: &leptonica::Pix, f: f32| scale(p, f, f, ScaleMethod::Auto).expect("scale");

    // C 0-5: 1 bpp, scaled down and reduced to gray at several factors.
    let pixs = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    rp.write_pix_and_check(&sc(&pixs, 0.32), ImageFormat::Png)
        .expect("check: 1bpp scale 0.32");
    for reduced in [
        scale_to_gray_3(&pixs).expect("to gray 3"),
        scale_to_gray_4(&pixs).expect("to gray 4"),
        scale_to_gray_6(&pixs).expect("to gray 6"),
        scale_to_gray_8(&pixs).expect("to gray 8"),
        scale_to_gray_16(&pixs).expect("to gray 16"),
    ] {
        rp.write_pix_and_check(&reduced, ImageFormat::Png)
            .expect("check: scale to gray");
    }

    // C 20-22, 24-26, 28-30: 2 bpp with cmap, 4 bpp without, 4 bpp with.
    for (name, up) in [
        ("weasel2.4c.png", 2.25f32),
        ("weasel4.png", 1.72),
        ("weasel4.16c.png", 1.72),
    ] {
        let pixs = load_test_image(name).expect("load weasel");
        for f in [up, 0.85, 0.65] {
            rp.write_pix_and_check(&sc(&pixs, f), ImageFormat::Png)
                .expect("check: weasel scale");
        }
    }

    // C 35: scale_to_size with a free dimension (uses fast unsharp masking).
    let pixs = load_test_image("graytext.png").expect("load graytext.png");
    let scaled = scale_to_size(&pixs, 0, 32).expect("scale to size");
    rp.write_pix_and_check(&scaled, ImageFormat::Png)
        .expect("check: scale_to_size");

    assert!(rp.cleanup(), "scale C-compat test failed");
}

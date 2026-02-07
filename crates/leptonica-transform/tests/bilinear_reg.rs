//! Bilinear transform regression test
//!
//! C version: reference/leptonica/prog/bilinear_reg.c
//! Tests bilinear transforms, including invertability and large distortions.
//!
//! C version test structure:
//!   1. Test invertability of sampling (pixBilinearSampledPta) -- i=1,2
//!   2. Test invertability of grayscale interpolation (pixBilinearPta on 8bpp) -- i=1,2
//!   3. Test invertability of color interpolation (pixBilinearPta on 32bpp)
//!      -- test24.jpg未提供のためスキップ (use weasel32.png instead)
//!   4. Comparison between sampling and interpolated
//!   5. Large distortion with inversion (pixBilinearSampledPta, pixBilinearPta)

use leptonica_core::{Pix, PixelDepth};
use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{
    AffineFill, Point, ScaleMethod, bilinear_pta, bilinear_sampled_pta, scale,
};

// Point data from C version (bilinear_reg.c MakePtas function)
// Source points (4 per test set, i=0..2)
const X1: [i32; 3] = [32, 32, 32];
const Y1: [i32; 3] = [150, 150, 150];
const X2: [i32; 3] = [520, 520, 520];
const Y2: [i32; 3] = [150, 150, 150];
const X3: [i32; 3] = [32, 32, 32];
const Y3: [i32; 3] = [612, 612, 612];
const X4: [i32; 3] = [520, 520, 520];
const Y4: [i32; 3] = [612, 612, 612];

// Destination points
const XP1: [i32; 3] = [32, 32, 32];
const YP1: [i32; 3] = [150, 150, 150];
const XP2: [i32; 3] = [520, 520, 520];
const YP2: [i32; 3] = [44, 124, 140];
const XP3: [i32; 3] = [32, 32, 32];
const YP3: [i32; 3] = [612, 612, 612];
const XP4: [i32; 3] = [520, 520, 520];
const YP4: [i32; 3] = [694, 624, 622];

fn make_pts(i: usize) -> ([Point; 4], [Point; 4]) {
    let src = [
        Point::new(X1[i] as f32, Y1[i] as f32),
        Point::new(X2[i] as f32, Y2[i] as f32),
        Point::new(X3[i] as f32, Y3[i] as f32),
        Point::new(X4[i] as f32, Y4[i] as f32),
    ];
    let dst = [
        Point::new(XP1[i] as f32, YP1[i] as f32),
        Point::new(XP2[i] as f32, YP2[i] as f32),
        Point::new(XP3[i] as f32, YP3[i] as f32),
        Point::new(XP4[i] as f32, YP4[i] as f32),
    ];
    (src, dst)
}

fn count_diff_pixels(pix: &Pix) -> u64 {
    let w = pix.width();
    let h = pix.height();
    let mut count = 0u64;
    for y in 0..h {
        for x in 0..w {
            if let Some(v) = pix.get_pixel(x, y) {
                if v != 0 {
                    count += 1;
                }
            }
        }
    }
    count
}

fn pix_xor(pix1: &Pix, pix2: &Pix) -> Pix {
    let w = pix1.width();
    let h = pix1.height();
    let depth = pix1.depth();
    let out = Pix::new(w, h, depth).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel(x, y).unwrap_or(0);
            let v2 = pix2.get_pixel(x, y).unwrap_or(0);
            let _ = out_mut.set_pixel(x, y, v1 ^ v2);
        }
    }
    out_mut.into()
}

fn remove_border(pix: &Pix, border: u32) -> Pix {
    let w = pix.width();
    let h = pix.height();
    if w <= 2 * border || h <= 2 * border {
        return Pix::new(1, 1, pix.depth()).unwrap();
    }
    let new_w = w - 2 * border;
    let new_h = h - 2 * border;
    let out = Pix::new(new_w, new_h, pix.depth()).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..new_h {
        for x in 0..new_w {
            let val = pix.get_pixel(x + border, y + border).unwrap_or(0);
            let _ = out_mut.set_pixel(x, y, val);
        }
    }
    out_mut.into()
}

fn add_border(pix: &Pix, border: u32, fill_val: u32) -> Pix {
    let w = pix.width();
    let h = pix.height();
    let new_w = w + 2 * border;
    let new_h = h + 2 * border;
    let out = Pix::new(new_w, new_h, pix.depth()).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..new_h {
        for x in 0..new_w {
            let _ = out_mut.set_pixel(x, y, fill_val);
        }
    }
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            let _ = out_mut.set_pixel(x + border, y + border, val);
        }
    }
    out_mut.into()
}

fn convert_1bpp_to_8bpp(pix: &Pix) -> Pix {
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit8).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            let gray = if val == 0 { 255u32 } else { 0u32 };
            let _ = out_mut.set_pixel(x, y, gray);
        }
    }
    out_mut.into()
}

// C version: Test invertability of sampling (pixBilinearSampledPta) -- i=1,2
#[test]
fn bilinear_reg_sampling_invertability() {
    let mut rp = RegParams::new("bilinear_sampling");

    // C version: pixs = pixRead("feyn.tif"); pixg = pixScaleToGray(pixs, 0.2);
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale 0.2");
    // C version: pixScaleToGray produces 8bpp -- Rust未実装, convert manually
    let pixg = convert_1bpp_to_8bpp(&pixs);
    let added_border = 250u32;

    // C version: for (i = 1; i < 3; i++)
    for i in 1..3 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixg, ADDED_BORDER_PIXELS, 255);
        let pixb = add_border(&pixg, added_border, 255);

        // C version: pix1 = pixBilinearSampledPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = bilinear_sampled_pta(&pixb, ptad, ptas, AffineFill::White)
            .expect("bilinear_sampled_pta forward");
        rp.compare_values(pixb.width() as f64, pix1.width() as f64, 0.0);

        // C version: pix2 = pixBilinearSampledPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = bilinear_sampled_pta(&pix1, ptas, ptad, AffineFill::White)
            .expect("bilinear_sampled_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS);
        let pixd = remove_border(&pix2, added_border);
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
            // C version: pixInvert(pixd, pixd); pixXor(pixd, pixd, pixg);
            let xor_result = pix_xor(&pixd, &pixg);
            let diff_count = count_diff_pixels(&xor_result);
            let total = pixg.width() as u64 * pixg.height() as u64;
            let diff_frac = diff_count as f64 / total as f64;
            eprintln!(
                "  Sampling invertability set {}: diff_frac={:.4}",
                i, diff_frac
            );
            rp.compare_values(0.0, diff_frac, 0.25);
        }
    }
    assert!(rp.cleanup(), "bilinear sampling invertability test failed");
}

// C version: Test invertability of grayscale interpolation (pixBilinearPta)
#[test]
fn bilinear_reg_grayscale_interpolation_invertability() {
    let mut rp = RegParams::new("bilinear_gray_interp");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = convert_1bpp_to_8bpp(&pixs);
    let added_border = 250u32;

    // C version: for (i = 1; i < 3; i++)
    for i in 1..3 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixg, ADDED_BORDER_PIXELS, 255);
        let pixb = add_border(&pixg, added_border, 255);

        // C version: pix1 = pixBilinearPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 =
            bilinear_pta(&pixb, ptad, ptas, AffineFill::White).expect("bilinear_pta forward");
        rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

        // C version: pix2 = pixBilinearPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 =
            bilinear_pta(&pix1, ptas, ptad, AffineFill::White).expect("bilinear_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS);
        let pixd = remove_border(&pix2, added_border);
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
            // C version: pixInvert(pixd, pixd); pixXor(pixd, pixd, pixg);
            let mut total_diff = 0u64;
            let total = pixg.width() as u64 * pixg.height() as u64;
            for y in 0..pixd.height() {
                for x in 0..pixd.width() {
                    let v1 = pixg.get_pixel(x, y).unwrap_or(0) as i64;
                    let v2 = pixd.get_pixel(x, y).unwrap_or(0) as i64;
                    total_diff += (v1 - v2).unsigned_abs();
                }
            }
            let mean_diff = total_diff as f64 / total as f64;
            eprintln!(
                "  Grayscale interp invertability set {}: mean_abs_diff={:.2}",
                i, mean_diff
            );
            rp.compare_values(0.0, mean_diff, 40.0);
        }
    }
    assert!(
        rp.cleanup(),
        "bilinear grayscale interpolation invertability test failed"
    );
}

// C version: Comparison between sampling and interpolated (point set 2)
#[test]
fn bilinear_reg_compare_sampling_interpolated() {
    let mut rp = RegParams::new("bilinear_compare");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = convert_1bpp_to_8bpp(&pixs);

    // C version: MakePtas(2, &ptas, &ptad);
    let (ptas, ptad) = make_pts(2);

    // C version: pix1 = pixBilinearSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_sampled =
        bilinear_sampled_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_sampled_pta");

    // C version: pix2 = pixBilinearPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_interp = bilinear_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_pta");

    rp.compare_values(pixg.width() as f64, pix_sampled.width() as f64, 0.0);
    rp.compare_values(pixg.width() as f64, pix_interp.width() as f64, 0.0);

    // Both should have some content
    let sampled_nonzero = count_diff_pixels(&pix_sampled);
    let interp_nonzero = count_diff_pixels(&pix_interp);
    let total = pixg.width() as u64 * pixg.height() as u64;
    eprintln!(
        "  Compare: sampled nonzero={}/{}, interp nonzero={}/{}",
        sampled_nonzero, total, interp_nonzero, total
    );
    rp.compare_values(1.0, if sampled_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if interp_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);

    // C version: pixXor(pix2, pix2, pix1); pixInvert(pix2, pix2);
    let xor_result = pix_xor(&pix_sampled, &pix_interp);
    let diff_count = count_diff_pixels(&xor_result);
    let diff_frac = diff_count as f64 / total as f64;
    eprintln!("  Sampled vs interp diff: {:.4}", diff_frac);
    rp.compare_values(0.0, diff_frac, 0.50);

    assert!(
        rp.cleanup(),
        "bilinear compare sampling/interpolated test failed"
    );
}

// C version: Large distortion with inversion (point set 0)
#[test]
fn bilinear_reg_large_distortion() {
    let mut rp = RegParams::new("bilinear_large_distort");

    // C version: pixs = pixRead("marge.jpg"); pixg = pixConvertTo8(pixs, 0);
    let pixs = load_test_image("marge.jpg").expect("load marge.jpg");

    // Convert to 8bpp grayscale if needed
    let pixg = if pixs.depth() == PixelDepth::Bit8 {
        pixs
    } else if pixs.depth() == PixelDepth::Bit32 {
        // Convert 32bpp to 8bpp grayscale
        let w = pixs.width();
        let h = pixs.height();
        let out = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        let mut out_mut = out.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let rgba = pixs.get_pixel(x, y).unwrap_or(0);
                let r = ((rgba >> 24) & 0xFF) as u32;
                let g = ((rgba >> 16) & 0xFF) as u32;
                let b = ((rgba >> 8) & 0xFF) as u32;
                let gray = (r * 77 + g * 150 + b * 29) >> 8;
                let _ = out_mut.set_pixel(x, y, gray);
            }
        }
        out_mut.into()
    } else {
        convert_1bpp_to_8bpp(&pixs)
    };

    // C version: MakePtas(0, &ptas, &ptad);
    let (ptas, ptad) = make_pts(0);

    // C version: pix1 = pixBilinearSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix1 = bilinear_sampled_pta(&pixg, ptas, ptad, AffineFill::White)
        .expect("bilinear_sampled_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix1.width() as f64, 0.0);

    // C version: pix2 = pixBilinearPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix2 =
        bilinear_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix2.width() as f64, 0.0);

    // C version: pix3 = pixBilinearSampledPta(pix1, ptad, ptas, L_BRING_IN_WHITE);
    // Forward then inverse
    let pix3 = bilinear_sampled_pta(&pix1, ptad, ptas, AffineFill::White)
        .expect("bilinear_sampled_pta inverse");
    rp.compare_values(pixg.width() as f64, pix3.width() as f64, 0.0);

    // C version: pix4 = pixBilinearPta(pix2, ptad, ptas, L_BRING_IN_WHITE);
    let pix4 = bilinear_pta(&pix2, ptad, ptas, AffineFill::White).expect("bilinear_pta inverse");
    rp.compare_values(pixg.width() as f64, pix4.width() as f64, 0.0);

    // Results should be non-blank
    rp.compare_values(
        1.0,
        if count_diff_pixels(&pix1) > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if count_diff_pixels(&pix2) > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if count_diff_pixels(&pix3) > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if count_diff_pixels(&pix4) > 0 {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "bilinear large distortion test failed");
}

// Basic API test: bilinear_pta and bilinear_sampled_pta on 8bpp
#[test]
fn bilinear_reg_pta_basic() {
    let mut rp = RegParams::new("bilinear_pta_basic");

    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let _ = pix_mut.set_pixel(x, y, (x + y) % 256);
        }
    }
    let pix: Pix = pix_mut.into();

    let src = [
        Point::new(0.0, 0.0),
        Point::new(99.0, 0.0),
        Point::new(0.0, 99.0),
        Point::new(99.0, 99.0),
    ];
    let dst = [
        Point::new(10.0, 10.0),
        Point::new(89.0, 10.0),
        Point::new(10.0, 89.0),
        Point::new(89.0, 89.0),
    ];

    let out = bilinear_pta(&pix, src, dst, AffineFill::White).expect("bilinear_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);
    rp.compare_values(8.0, out.depth().bits() as f64, 0.0);

    let out =
        bilinear_sampled_pta(&pix, src, dst, AffineFill::White).expect("bilinear_sampled_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);

    assert!(rp.cleanup(), "bilinear pta basic test failed");
}

// C version: Test invertability of color interpolation (32bpp)
// test24.jpg not available, using weasel32.png
#[test]
fn bilinear_reg_color_interpolation() {
    let mut rp = RegParams::new("bilinear_color");

    let pixc = load_test_image("weasel32.png").expect("load weasel32.png");

    let src = [
        Point::new(0.0, 0.0),
        Point::new(pixc.width() as f32 - 1.0, 0.0),
        Point::new(0.0, pixc.height() as f32 - 1.0),
        Point::new(pixc.width() as f32 - 1.0, pixc.height() as f32 - 1.0),
    ];
    let dst = [
        Point::new(10.0, 5.0),
        Point::new(pixc.width() as f32 - 10.0, 10.0),
        Point::new(5.0, pixc.height() as f32 - 10.0),
        Point::new(pixc.width() as f32 - 5.0, pixc.height() as f32 - 5.0),
    ];

    let out = bilinear_pta(&pixc, src, dst, AffineFill::White).expect("bilinear_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(pixc.height() as f64, out.height() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    let out = bilinear_sampled_pta(&pixc, src, dst, AffineFill::White)
        .expect("bilinear_sampled_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "bilinear color interpolation test failed");
}

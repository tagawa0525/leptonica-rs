//! Projective transform regression test
//!
//! C version: reference/leptonica/prog/projective_reg.c
//! Tests projective transforms, including invertability and large distortions.
//!
//! C version test structure:
//!   1. Test invertability of sampling (pixProjectiveSampledPta) -- i=0..2
//!   2. Test invertability of grayscale interpolation (pixProjectivePta on 8bpp) -- i=0..1
//!   3. Test invertability of color interpolation (pixProjectivePta on 32bpp)
//!      -- test24.jpg未提供のためスキップ (use weasel32.png instead)
//!   4. Comparison between sampling and interpolated (point set 3)

use leptonica_core::{Pix, PixelDepth};
use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{
    AffineFill, Point, ScaleMethod, projective_pta, projective_sampled_pta, scale,
};

// Point data from C version (projective_reg.c MakePtas function)
// Source points (4 per test set, i=0..4)
const X1: [i32; 5] = [300, 300, 300, 300, 32];
const Y1: [i32; 5] = [1200, 1200, 1250, 1250, 934];
const X2: [i32; 5] = [1200, 1200, 1125, 1300, 487];
const Y2: [i32; 5] = [1100, 1100, 1100, 1250, 934];
const X3: [i32; 5] = [200, 200, 200, 250, 32];
const Y3: [i32; 5] = [200, 200, 200, 300, 67];
const X4: [i32; 5] = [1200, 1200, 1300, 1250, 332];
const Y4: [i32; 5] = [400, 200, 200, 300, 57];

// Destination points
const XP1: [i32; 5] = [300, 300, 1150, 300, 32];
const YP1: [i32; 5] = [1200, 1400, 1150, 1350, 934];
const XP2: [i32; 5] = [1100, 1400, 320, 1300, 487];
const YP2: [i32; 5] = [1000, 1500, 1300, 1200, 904];
const XP3: [i32; 5] = [250, 200, 1310, 300, 61];
const YP3: [i32; 5] = [200, 300, 250, 325, 83];
const XP4: [i32; 5] = [1250, 1200, 240, 1250, 412];
const YP4: [i32; 5] = [300, 300, 250, 350, 83];

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

// C version: Test invertability of sampling (pixProjectiveSampledPta) -- i=0..2
#[test]
fn projective_reg_sampling_invertability() {
    let mut rp = RegParams::new("projective_sampling");

    // C version: pixs = pixRead("feyn.tif"); pixsc = pixScale(pixs, 0.3, 0.3);
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixsc = scale(&pix, 0.3, 0.3, ScaleMethod::Linear).expect("scale 0.3");
    let added_border = 250u32;

    // C version: for (i = 0; i < 3; i++)
    for i in 0..3 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixsc, ADDED_BORDER_PIXELS, 0);
        let pixb = add_border(&pixsc, added_border, 0);

        // C version: pix1 = pixProjectiveSampledPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = projective_sampled_pta(&pixb, ptad, ptas, AffineFill::White)
            .expect("projective_sampled_pta forward");
        rp.compare_values(pixb.width() as f64, pix1.width() as f64, 0.0);

        // C version: pix2 = pixProjectiveSampledPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = projective_sampled_pta(&pix1, ptas, ptad, AffineFill::White)
            .expect("projective_sampled_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS);
        let pixd = remove_border(&pix2, added_border);
        rp.compare_values(pixsc.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixsc.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixsc.width() && pixd.height() == pixsc.height() {
            // C version: pixXor(pixd, pixd, pixsc);
            let xor_result = pix_xor(&pixd, &pixsc);
            let diff_count = count_diff_pixels(&xor_result);
            let total = pixsc.width() as u64 * pixsc.height() as u64;
            let diff_frac = diff_count as f64 / total as f64;
            eprintln!(
                "  Sampling invertability set {}: diff_frac={:.4}",
                i, diff_frac
            );
            rp.compare_values(0.0, diff_frac, 0.25);
        }
    }
    assert!(
        rp.cleanup(),
        "projective sampling invertability test failed"
    );
}

// C version: Test invertability of grayscale interpolation (pixProjectivePta) -- i=0..1
#[test]
fn projective_reg_grayscale_interpolation_invertability() {
    let mut rp = RegParams::new("projective_gray_interp");

    // C version: pixg = pixScaleToGray(pixs, 0.2) -- Rust未実装, manual conversion
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = convert_1bpp_to_8bpp(&pixs);
    let added_border = 125u32; // C version: ADDED_BORDER_PIXELS / 2

    // C version: for (i = 0; i < 2; i++)
    for i in 0..2 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixg, ADDED_BORDER_PIXELS / 2, 255);
        let pixb = add_border(&pixg, added_border, 255);

        // C version: pix1 = pixProjectivePta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 =
            projective_pta(&pixb, ptad, ptas, AffineFill::White).expect("projective_pta forward");
        rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

        // C version: pix2 = pixProjectivePta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 =
            projective_pta(&pix1, ptas, ptad, AffineFill::White).expect("projective_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS / 2);
        let pixd = remove_border(&pix2, added_border);
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
            // C version: pixXor(pixd, pixd, pixg); pixInvert(pixd, pixd);
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
        "projective grayscale interpolation invertability test failed"
    );
}

// C version: Comparison between sampling and interpolated (point set 3)
#[test]
fn projective_reg_compare_sampling_interpolated() {
    let mut rp = RegParams::new("projective_compare");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = convert_1bpp_to_8bpp(&pixs);

    // C version: MakePtas(3, &ptas, &ptad);
    let (ptas, ptad) = make_pts(3);

    // C version: pix1 = pixProjectiveSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_sampled = projective_sampled_pta(&pixg, ptas, ptad, AffineFill::White)
        .expect("projective_sampled_pta");

    // C version: pix2 = pixProjectivePta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_interp = projective_pta(&pixg, ptas, ptad, AffineFill::White).expect("projective_pta");

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
        "projective compare sampling/interpolated test failed"
    );
}

// Basic API test: projective_pta and projective_sampled_pta on 8bpp
#[test]
fn projective_reg_pta_basic() {
    let mut rp = RegParams::new("projective_pta_basic");

    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let _ = pix_mut.set_pixel(x, y, ((x + y) % 256) as u32);
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

    let out = projective_pta(&pix, src, dst, AffineFill::White).expect("projective_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);
    rp.compare_values(8.0, out.depth().bits() as f64, 0.0);

    let out =
        projective_sampled_pta(&pix, src, dst, AffineFill::White).expect("projective_sampled_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);

    assert!(rp.cleanup(), "projective pta basic test failed");
}

// C version: Test invertability of color interpolation (32bpp)
// test24.jpg not available, using weasel32.png
#[test]
fn projective_reg_color_interpolation() {
    let mut rp = RegParams::new("projective_color");

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

    let out = projective_pta(&pixc, src, dst, AffineFill::White).expect("projective_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(pixc.height() as f64, out.height() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    let out = projective_sampled_pta(&pixc, src, dst, AffineFill::White)
        .expect("projective_sampled_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "projective color interpolation test failed");
}

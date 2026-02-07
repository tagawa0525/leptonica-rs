//! Affine transform regression test
//!
//! C version: reference/leptonica/prog/affine_reg.c
//! Tests affine transforms, including invertability and large distortions.
//!
//! C version test structure:
//!   1. Test invertability of sequential (pixAffineSequential) -- Rust未実装のためスキップ
//!   2. Test invertability of sampling (pixAffineSampledPta)
//!   3. Test invertability of grayscale interpolation (pixAffinePta on 8bpp)
//!   4. Test invertability of color interpolation (pixAffinePta on 32bpp)
//!      -- test24.jpg未提供のためスキップ
//!   5. Comparison between sequential and sampling
//!      -- pixAffineSequential未実装のためスキップ
//!   6. Test with large distortion (pixAffineSampledPta, pixAffinePta)
//!   7. Test affine transforms on pix and boxa
//!      -- boxaAffineTransform, pixCloseBrick, pixConnComp等未実装のためスキップ

use leptonica_core::{Pix, PixelDepth};
use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{AffineFill, Point, ScaleMethod, affine_pta, affine_sampled_pta, scale};

// Point data from C version (affine_reg.c MakePtas function)
const X1: [i32; 5] = [300, 300, 300, 95, 32];
const Y1: [i32; 5] = [1200, 1200, 1250, 2821, 934];
const X2: [i32; 5] = [1200, 1200, 1125, 1432, 487];
const Y2: [i32; 5] = [1100, 1100, 1100, 2682, 934];
const X3: [i32; 5] = [200, 200, 200, 232, 32];
const Y3: [i32; 5] = [200, 200, 200, 657, 67];

const XP1: [i32; 5] = [500, 300, 350, 117, 32];
const YP1: [i32; 5] = [1700, 1400, 1400, 2629, 934];
const XP2: [i32; 5] = [850, 1400, 1400, 1464, 487];
const YP2: [i32; 5] = [850, 1500, 1500, 2432, 804];
const XP3: [i32; 5] = [450, 200, 400, 183, 61];
const YP3: [i32; 5] = [300, 300, 400, 490, 83];

fn make_pts(i: usize) -> ([Point; 3], [Point; 3]) {
    let src = [
        Point::new(X1[i] as f32, Y1[i] as f32),
        Point::new(X2[i] as f32, Y2[i] as f32),
        Point::new(X3[i] as f32, Y3[i] as f32),
    ];
    let dst = [
        Point::new(XP1[i] as f32, YP1[i] as f32),
        Point::new(XP2[i] as f32, YP2[i] as f32),
        Point::new(XP3[i] as f32, YP3[i] as f32),
    ];
    (src, dst)
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

// C version: Test invertability of sampling (pixAffineSampledPta)
#[test]
fn affine_reg_sampling_invertability() {
    let mut rp = RegParams::new("affine_sampling");

    // C version: pix = pixRead("feyn.tif"); pixs = pixScale(pix, 0.22, 0.22);
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.22, 0.22, ScaleMethod::Linear).expect("scale 0.22");
    let added_border = 1000u32;

    // C version: for (i = 0; i < 3; i++)
    for i in 0..3 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixs, ADDED_BORDER_PIXELS, 0);
        let pixb = add_border(&pixs, added_border, 0);

        // C version: pix1 = pixAffineSampledPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = affine_sampled_pta(&pixb, ptad, ptas, AffineFill::White)
            .expect("affine_sampled_pta forward");
        rp.compare_values(pixb.width() as f64, pix1.width() as f64, 0.0);

        // C version: pix2 = pixAffineSampledPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = affine_sampled_pta(&pix1, ptas, ptad, AffineFill::White)
            .expect("affine_sampled_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS);
        let pixd = remove_border(&pix2, added_border);
        rp.compare_values(pixs.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixs.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixs.width() && pixd.height() == pixs.height() {
            // C version: pixXor(pixd, pixd, pixs);
            let xor_result = pix_xor(&pixd, &pixs);
            let diff_count = xor_result.count_pixels();
            let total = pixs.width() as u64 * pixs.height() as u64;
            let diff_frac = diff_count as f64 / total as f64;
            eprintln!(
                "  Sampling invertability set {}: diff_frac={:.4}",
                i, diff_frac
            );
            rp.compare_values(0.0, diff_frac, 0.20);
        }
    }
    assert!(rp.cleanup(), "affine sampling invertability test failed");
}

// C version: Test invertability of grayscale interpolation (pixAffinePta)
#[test]
fn affine_reg_grayscale_interpolation_invertability() {
    let mut rp = RegParams::new("affine_gray_interp");

    // C version: pixg = pixScaleToGray3(pix) -- Rust未実装, manual conversion
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.22, 0.22, ScaleMethod::Linear).expect("scale");
    let pixg = convert_1bpp_to_8bpp(&pixs);
    let added_border = 333u32; // C version: ADDED_BORDER_PIXELS / 3

    for i in 0..3 {
        let (ptas, ptad) = make_pts(i);

        // C version: pixb = pixAddBorder(pixg, ADDED_BORDER_PIXELS / 3, 255);
        let pixb = add_border(&pixg, added_border, 255);

        // C version: pix1 = pixAffinePta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = affine_pta(&pixb, ptad, ptas, AffineFill::White).expect("affine_pta forward");
        rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

        // C version: pix2 = pixAffinePta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = affine_pta(&pix1, ptas, ptad, AffineFill::White).expect("affine_pta inverse");

        // C version: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS / 3);
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
        "affine grayscale interpolation invertability test failed"
    );
}

// C version: Test with large distortion (point set 4)
#[test]
fn affine_reg_large_distortion() {
    let mut rp = RegParams::new("affine_large_distort");

    // C version: pixg = pixScaleToGray6(pix) -- Rust未実装
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.167, 0.167, ScaleMethod::Linear).expect("scale ~1/6");
    let pixg = convert_1bpp_to_8bpp(&pixs);

    // C version: MakePtas(4, &ptas, &ptad);
    let (ptas, ptad) = make_pts(4);

    // C version: pixAffineSequential(pixg, ptas, ptad, 0, 0) -- Rust未実装のためスキップ

    // C version: pix2 = pixAffineSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_sampled = affine_sampled_pta(&pixg, ptas, ptad, AffineFill::White)
        .expect("affine_sampled_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix_sampled.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix_sampled.height() as f64, 0.0);

    // C version: pix3 = pixAffinePta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_interp =
        affine_pta(&pixg, ptas, ptad, AffineFill::White).expect("affine_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix_interp.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix_interp.height() as f64, 0.0);

    // Both should have some content (not blank)
    let sampled_nonzero = pix_sampled.count_pixels();
    let interp_nonzero = pix_interp.count_pixels();
    let total = pixg.width() as u64 * pixg.height() as u64;
    eprintln!(
        "  Large distortion: sampled nonzero={}/{}, interp nonzero={}/{}",
        sampled_nonzero, total, interp_nonzero, total
    );
    rp.compare_values(1.0, if sampled_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if interp_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);

    // C version: pixXor(pix1, pix1, pix2) -- compare sampled vs interpolated
    let xor_result = pix_xor(&pix_sampled, &pix_interp);
    let diff_count = xor_result.count_pixels();
    let diff_frac = diff_count as f64 / total as f64;
    eprintln!("  Sampled vs interp diff: {:.4}", diff_frac);
    rp.compare_values(0.0, diff_frac, 0.50);

    assert!(rp.cleanup(), "affine large distortion test failed");
}

// Basic API test: affine_pta and affine_sampled_pta on 8bpp
#[test]
fn affine_reg_pta_basic() {
    let mut rp = RegParams::new("affine_pta_basic");

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
    ];
    let dst = [
        Point::new(10.0, 10.0),
        Point::new(89.0, 10.0),
        Point::new(10.0, 89.0),
    ];

    let out = affine_pta(&pix, src, dst, AffineFill::White).expect("affine_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);
    rp.compare_values(8.0, out.depth().bits() as f64, 0.0);

    let out = affine_sampled_pta(&pix, src, dst, AffineFill::White).expect("affine_sampled_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);

    assert!(rp.cleanup(), "affine pta basic test failed");
}

// Basic API test: affine_sampled_pta on 1bpp
#[test]
fn affine_reg_sampled_1bpp() {
    let mut rp = RegParams::new("affine_sampled_1bpp");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.22, 0.22, ScaleMethod::Sampling).expect("scale");

    let src = [
        Point::new(300.0, 1200.0),
        Point::new(1200.0, 1100.0),
        Point::new(200.0, 200.0),
    ];
    let dst = [
        Point::new(500.0, 1700.0),
        Point::new(850.0, 850.0),
        Point::new(450.0, 300.0),
    ];

    let out =
        affine_sampled_pta(&pixs, src, dst, AffineFill::White).expect("affine_sampled_pta 1bpp");
    rp.compare_values(pixs.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, out.height() as f64, 0.0);
    rp.compare_values(1.0, out.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "affine sampled 1bpp test failed");
}

// C version: Test invertability of color interpolation (32bpp)
// test24.jpg not available, using weasel32.png
#[test]
fn affine_reg_color_interpolation() {
    let mut rp = RegParams::new("affine_color");

    let pixc = load_test_image("weasel32.png").expect("load weasel32.png");

    let src = [
        Point::new(0.0, 0.0),
        Point::new(pixc.width() as f32 - 1.0, 0.0),
        Point::new(0.0, pixc.height() as f32 - 1.0),
    ];
    let dst = [
        Point::new(10.0, 5.0),
        Point::new(pixc.width() as f32 - 10.0, 10.0),
        Point::new(5.0, pixc.height() as f32 - 10.0),
    ];

    let out = affine_pta(&pixc, src, dst, AffineFill::White).expect("affine_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(pixc.height() as f64, out.height() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    let out =
        affine_sampled_pta(&pixc, src, dst, AffineFill::White).expect("affine_sampled_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "affine color interpolation test failed");
}

// C version: pixAffineSequential -- Rust未実装
#[test]
#[ignore = "pixAffineSequential未実装: C版ではpixAffineSequentialで逐次的affine変換をテスト"]
fn affine_reg_sequential_invertability() {
    // C version: pixAffineSequential(pixb, ptad, ptas, 0, 0)
}

// C version: boxaAffineTransform test -- Rust未実装
#[test]
#[ignore = "boxaAffineTransform, createMatrix2d*, pixCloseBrick, pixConnComp等未実装"]
fn affine_reg_boxa_transform() {
    // C version: Tests affine transforms and inverses on pix and boxa
    // Requires: createMatrix2dTranslate, createMatrix2dScale, createMatrix2dRotate,
    //           l_productMat3, boxaAffineTransform, affineInvertXform,
    //           pixCloseBrick, pixOpenBrick, pixConnComp, pixConvertTo32,
    //           pixRenderHashBoxArb
}

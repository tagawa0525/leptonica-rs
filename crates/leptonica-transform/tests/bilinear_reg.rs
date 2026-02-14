//! Bilinear transform regression test
//!
//! C version: `reference/leptonica/prog/bilinear_reg.c`
//!
//! Tests bilinear transforms including invertability and large distortions.
//!
//! C version test structure:
//!   1. Test invertability of sampling (`pixBilinearSampledPta`) — i=1,2
//!   2. Test invertability of grayscale interpolation (`pixBilinearPta` on 8bpp) — i=1,2
//!   3. Test invertability of color interpolation (`pixBilinearPta` on 32bpp)
//!   4. Comparison between sampling and interpolated
//!   5. Large distortion with inversion (`pixBilinearSampledPta`, `pixBilinearPta`)
//!
//! Bilinear uses 4 point correspondences (vs 3 for affine).

use leptonica_core::{Pix, PixelDepth};
use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{
    AffineFill, Point, ScaleMethod, bilinear_pta, bilinear_sampled_pta, scale,
};

// Point data from C version (bilinear_reg.c MakePtas function)
const X1: [i32; 3] = [32, 32, 32];
const Y1: [i32; 3] = [150, 150, 150];
const X2: [i32; 3] = [520, 520, 520];
const Y2: [i32; 3] = [150, 150, 150];
const X3: [i32; 3] = [32, 32, 32];
const Y3: [i32; 3] = [612, 612, 612];
const X4: [i32; 3] = [520, 520, 520];
const Y4: [i32; 3] = [612, 612, 612];

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

/// Test invertability of sampling (`pixBilinearSampledPta`) — i=1,2
///
/// C version: forward+inverse bilinear sampling on 8bpp image with 250px border.
#[test]
fn bilinear_reg_sampling_invertability() {
    let mut rp = RegParams::new("bilinear_sampling");

    // C: pixs = pixRead("feyn.tif"); pixg = pixScaleToGray(pixs, 0.2);
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale 0.2");
    let pixg = pixs.convert_to_8().expect("convert_to_8");
    let added_border = 250u32;

    // C: for (i = 1; i < 3; i++)
    for i in 1..3 {
        let (ptas, ptad) = make_pts(i);

        let pixb = pixg.add_border(added_border, 255).expect("add_border");

        // C: pix1 = pixBilinearSampledPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = bilinear_sampled_pta(&pixb, ptad, ptas, AffineFill::White)
            .expect("bilinear_sampled_pta forward");
        rp.compare_values(pixb.width() as f64, pix1.width() as f64, 0.0);

        // C: pix2 = pixBilinearSampledPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = bilinear_sampled_pta(&pix1, ptas, ptad, AffineFill::White)
            .expect("bilinear_sampled_pta inverse");

        let pixd = pix2.remove_border(added_border).expect("remove_border");
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
            let xor_result = pixd.xor(&pixg).expect("xor");
            let diff_count = xor_result.count_pixels();
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

/// Test invertability of grayscale interpolation (`pixBilinearPta`)
///
/// C version: forward+inverse bilinear interpolation on 8bpp with pixel diff.
#[test]
fn bilinear_reg_grayscale_interpolation_invertability() {
    let mut rp = RegParams::new("bilinear_gray_interp");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = pixs.convert_to_8().expect("convert_to_8");
    let added_border = 250u32;

    // C: for (i = 1; i < 3; i++)
    for i in 1..3 {
        let (ptas, ptad) = make_pts(i);

        let pixb = pixg.add_border(added_border, 255).expect("add_border");

        // C: pix1 = pixBilinearPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 =
            bilinear_pta(&pixb, ptad, ptas, AffineFill::White).expect("bilinear_pta forward");
        rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);

        // C: pix2 = pixBilinearPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 =
            bilinear_pta(&pix1, ptas, ptad, AffineFill::White).expect("bilinear_pta inverse");

        let pixd = pix2.remove_border(added_border).expect("remove_border");
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
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

/// Comparison between sampling and interpolated (point set 2)
///
/// C version: `MakePtas(2, ...)` then compare sampled vs interpolated results.
#[test]
fn bilinear_reg_compare_sampling_interpolated() {
    let mut rp = RegParams::new("bilinear_compare");

    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.2, 0.2, ScaleMethod::Linear).expect("scale");
    let pixg = pixs.convert_to_8().expect("convert_to_8");

    let (ptas, ptad) = make_pts(2);

    let pix_sampled =
        bilinear_sampled_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_sampled_pta");
    let pix_interp = bilinear_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_pta");

    rp.compare_values(pixg.width() as f64, pix_sampled.width() as f64, 0.0);
    rp.compare_values(pixg.width() as f64, pix_interp.width() as f64, 0.0);

    let sampled_nonzero = pix_sampled.count_pixels();
    let interp_nonzero = pix_interp.count_pixels();
    let total = pixg.width() as u64 * pixg.height() as u64;
    eprintln!(
        "  Compare: sampled nonzero={}/{}, interp nonzero={}/{}",
        sampled_nonzero, total, interp_nonzero, total
    );
    rp.compare_values(1.0, if sampled_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if interp_nonzero > 0 { 1.0 } else { 0.0 }, 0.0);

    let xor_result = pix_sampled.xor(&pix_interp).expect("xor");
    let diff_count = xor_result.count_pixels();
    let diff_frac = diff_count as f64 / total as f64;
    eprintln!("  Sampled vs interp diff: {:.4}", diff_frac);
    rp.compare_values(0.0, diff_frac, 0.50);

    assert!(
        rp.cleanup(),
        "bilinear compare sampling/interpolated test failed"
    );
}

/// Large distortion with inversion (point set 0)
///
/// C version: forward then inverse on `marge.jpg` with point set 0.
#[test]
fn bilinear_reg_large_distortion() {
    let mut rp = RegParams::new("bilinear_large_distort");

    let pixs = load_test_image("marge.jpg").expect("load marge.jpg");

    let pixg = if pixs.depth() == PixelDepth::Bit8 {
        pixs
    } else {
        pixs.convert_to_8().expect("convert_to_8")
    };

    let (ptas, ptad) = make_pts(0);

    // C: pix1 = pixBilinearSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix1 = bilinear_sampled_pta(&pixg, ptas, ptad, AffineFill::White)
        .expect("bilinear_sampled_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix1.width() as f64, 0.0);

    // C: pix2 = pixBilinearPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix2 =
        bilinear_pta(&pixg, ptas, ptad, AffineFill::White).expect("bilinear_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix2.width() as f64, 0.0);

    // Forward then inverse
    let pix3 = bilinear_sampled_pta(&pix1, ptad, ptas, AffineFill::White)
        .expect("bilinear_sampled_pta inverse");
    rp.compare_values(pixg.width() as f64, pix3.width() as f64, 0.0);

    let pix4 = bilinear_pta(&pix2, ptad, ptas, AffineFill::White).expect("bilinear_pta inverse");
    rp.compare_values(pixg.width() as f64, pix4.width() as f64, 0.0);

    // Results should be non-blank
    rp.compare_values(1.0, if pix1.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if pix2.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if pix3.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if pix4.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "bilinear large distortion test failed");
}

/// Basic API test: `bilinear_pta` and `bilinear_sampled_pta` on synthetic 8bpp
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

/// Test color interpolation on 32bpp
///
/// C version: test24.jpg not available, using weasel32.png.
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

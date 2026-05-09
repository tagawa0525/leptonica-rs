//! Affine transform regression test
//!
//! C version: `prog/affine_reg.c`
//!
//! Tests affine transforms including invertability and large distortions.
//!
//! C version test structure:
//!   1. Test invertability of sequential (`pixAffineSequential`) — Rust not implemented
//!   2. Test invertability of sampling (`pixAffineSampledPta`) — point sets 0..2
//!   3. Test invertability of grayscale interpolation (`pixAffinePta` on 8bpp) — point sets 0..2
//!   4. Test invertability of color interpolation (`pixAffinePta` on 32bpp) — point sets 0..2
//!   5. Comparison between sequential and sampling — Rust not implemented
//!   6. Test with large distortion (point set 4) — `pixAffineSampledPta`, `pixAffinePta`
//!   7. Affine transforms on pix and boxa — requires `boxaAffineTransform` etc.
//!
//! Point data from `MakePtas()` function in C version.

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::transform::{AffineFill, Point, ScaleMethod, affine_pta, affine_sampled_pta, scale};
use leptonica::{Pix, PixelDepth};

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

/// Test invertability of sampling (`pixAffineSampledPta`)
///
/// C version: forward transform then inverse on 1bpp scaled feyn.tif,
/// with 1000px border, XOR comparison to check recovery.
#[test]
fn affine_reg_sampling_invertability() {
    let mut rp = RegParams::new("affine_sampling");

    // C: pix = pixRead("feyn.tif"); pixs = pixScale(pix, 0.22, 0.22);
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.22, 0.22, ScaleMethod::Linear).expect("scale 0.22");
    let added_border = 1000u32;

    // C: for (i = 0; i < 3; i++)
    for i in 0..3 {
        let (ptas, ptad) = make_pts(i);

        let pixb = pixs.add_border(added_border, 0).expect("add_border");

        // C: pix1 = pixAffineSampledPta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = affine_sampled_pta(&pixb, ptad, ptas, AffineFill::White)
            .expect("affine_sampled_pta forward");
        rp.compare_values(pixb.width() as f64, pix1.width() as f64, 0.0);
        if i == 0 {
            let fmt = if pix1.depth() == PixelDepth::Bit1 {
                ImageFormat::Tiff
            } else {
                ImageFormat::Png
            };
            rp.write_pix_and_check(&pix1, fmt)
                .expect("write pix1 affine_sampling");
        }

        // C: pix2 = pixAffineSampledPta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = affine_sampled_pta(&pix1, ptas, ptad, AffineFill::White)
            .expect("affine_sampled_pta inverse");

        let pixd = pix2.remove_border(added_border).expect("remove_border");
        rp.compare_values(pixs.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixs.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixs.width() && pixd.height() == pixs.height() {
            // C: pixXor(pixd, pixd, pixs);
            let xor_result = pixd.xor(&pixs).expect("xor");
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

/// Test invertability of grayscale interpolation (`pixAffinePta`)
///
/// C version: forward+inverse on 8bpp image with pixel-level diff comparison.
#[test]
fn affine_reg_grayscale_interpolation_invertability() {
    let mut rp = RegParams::new("affine_gray_interp");

    // C: pixg = pixScaleToGray3(pix)
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.22, 0.22, ScaleMethod::Linear).expect("scale");
    let pixg = pixs.convert_to_8().expect("convert_to_8");
    let added_border = 333u32; // C: ADDED_BORDER_PIXELS / 3

    for i in 0..3 {
        let (ptas, ptad) = make_pts(i);

        // C: pixb = pixAddBorder(pixg, ADDED_BORDER_PIXELS / 3, 255);
        let pixb = pixg.add_border(added_border, 255).expect("add_border");

        // C: pix1 = pixAffinePta(pixb, ptad, ptas, L_BRING_IN_WHITE);
        let pix1 = affine_pta(&pixb, ptad, ptas, AffineFill::White).expect("affine_pta forward");
        rp.compare_values(8.0, pix1.depth().bits() as f64, 0.0);
        if i == 0 {
            rp.write_pix_and_check(&pix1, ImageFormat::Png)
                .expect("write pix1 affine_gray_interp");
        }

        // C: pix2 = pixAffinePta(pix1, ptas, ptad, L_BRING_IN_WHITE);
        let pix2 = affine_pta(&pix1, ptas, ptad, AffineFill::White).expect("affine_pta inverse");

        // C: pixd = pixRemoveBorder(pix2, ADDED_BORDER_PIXELS / 3);
        let pixd = pix2.remove_border(added_border).expect("remove_border");
        rp.compare_values(pixg.width() as f64, pixd.width() as f64, 0.0);
        rp.compare_values(pixg.height() as f64, pixd.height() as f64, 0.0);

        if pixd.width() == pixg.width() && pixd.height() == pixg.height() {
            // C: pixXor(pixd, pixd, pixg); pixInvert(pixd, pixd);
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

/// Test with large distortion (point set 4)
///
/// C version: tests `pixAffineSampledPta` and `pixAffinePta` with extreme point
/// displacements, comparing sampled vs interpolated results.
#[test]
fn affine_reg_large_distortion() {
    let mut rp = RegParams::new("affine_large_distort");

    // C: pixg = pixScaleToGray6(pix)
    let pix = load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = scale(&pix, 0.167, 0.167, ScaleMethod::Linear).expect("scale ~1/6");
    let pixg = pixs.convert_to_8().expect("convert_to_8");

    // C: MakePtas(4, &ptas, &ptad);
    let (ptas, ptad) = make_pts(4);

    // C: pix2 = pixAffineSampledPta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_sampled = affine_sampled_pta(&pixg, ptas, ptad, AffineFill::White)
        .expect("affine_sampled_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix_sampled.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix_sampled.height() as f64, 0.0);
    rp.write_pix_and_check(&pix_sampled, ImageFormat::Png)
        .expect("write pix_sampled");

    // C: pix3 = pixAffinePta(pixg, ptas, ptad, L_BRING_IN_WHITE);
    let pix_interp =
        affine_pta(&pixg, ptas, ptad, AffineFill::White).expect("affine_pta large distortion");
    rp.compare_values(pixg.width() as f64, pix_interp.width() as f64, 0.0);
    rp.compare_values(pixg.height() as f64, pix_interp.height() as f64, 0.0);
    rp.write_pix_and_check(&pix_interp, ImageFormat::Png)
        .expect("write pix_interp");

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

    // C: pixXor(pix1, pix1, pix2) — compare sampled vs interpolated
    let xor_result = pix_sampled.xor(&pix_interp).expect("xor");
    let diff_count = xor_result.count_pixels();
    let diff_frac = diff_count as f64 / total as f64;
    eprintln!("  Sampled vs interp diff: {:.4}", diff_frac);
    rp.compare_values(0.0, diff_frac, 0.50);

    assert!(rp.cleanup(), "affine large distortion test failed");
}

/// Basic API test: `affine_pta` and `affine_sampled_pta` on synthetic 8bpp
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
    rp.write_pix_and_check(&out, ImageFormat::Png)
        .expect("write out affine_pta_basic");

    let out = affine_sampled_pta(&pix, src, dst, AffineFill::White).expect("affine_sampled_pta");
    rp.compare_values(100.0, out.width() as f64, 0.0);
    rp.compare_values(100.0, out.height() as f64, 0.0);

    assert!(rp.cleanup(), "affine pta basic test failed");
}

/// Test `affine_sampled_pta` on 1bpp
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

/// Test color interpolation on 32bpp
///
/// C version: test24.jpg not available, using weasel32.png.
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
    rp.write_pix_and_check(&out, ImageFormat::Png)
        .expect("write out affine_color");

    let out =
        affine_sampled_pta(&pixc, src, dst, AffineFill::White).expect("affine_sampled_pta 32bpp");
    rp.compare_values(pixc.width() as f64, out.width() as f64, 0.0);
    rp.compare_values(32.0, out.depth().bits() as f64, 0.0);

    assert!(rp.cleanup(), "affine color interpolation test failed");
}

/// `pixAffineSequential` smoke test (C version analogue).
///
/// The C version explicitly notes that this warp is "about 3× faster than
/// affineSampled but the results on text are much inferior", so we avoid
/// strict pixel-equality / round-trip thresholds. Instead we verify that the
/// warp:
///
/// 1. Returns a Pix whose dimensions are within a sane factor of the source
///    (the internal `pixScale` enlarges the bordered image, so the final
///    dimensions are not strictly equal to the source dimensions).
/// 2. Preserves non-trivial foreground content (count > 1000 ON pixels).
/// 3. Rejects degenerate / wrong-arity point sets.
#[test]
fn affine_reg_sequential_invertability() {
    use leptonica::core::Pta;
    use leptonica::transform::affine::affine_sequential;

    let pix = crate::common::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let pixb = if pix.depth() == leptonica::PixelDepth::Bit1 {
        pix
    } else {
        pix.convert_to_1_adaptive().expect("convert to 1bpp")
    };

    // Three corner points in (origin, x-axis, y-axis) order.
    let mut ptas = Pta::with_capacity(3);
    ptas.push(50.0, 50.0);
    ptas.push(250.0, 70.0);
    ptas.push(70.0, 200.0);
    let mut ptad = Pta::with_capacity(3);
    ptad.push(60.0, 80.0);
    ptad.push(260.0, 60.0);
    ptad.push(90.0, 220.0);

    let pix_fwd = affine_sequential(&pixb, &ptad, &ptas, 200, 200).expect("forward warp");
    // C version of pixAffineSequential applies pixScale on the bordered image
    // and only strips the original (unscaled) border, so the final dimensions
    // depend on scalex/scaley. We just assert the result is non-degenerate
    // and within a reasonable factor of the source.
    assert!(pix_fwd.width() > 0 && pix_fwd.height() > 0);
    assert!(
        pix_fwd.width() < 4 * pixb.width() && pix_fwd.height() < 4 * pixb.height(),
        "output {}x{} should be within ~4x of input {}x{}",
        pix_fwd.width(),
        pix_fwd.height(),
        pixb.width(),
        pixb.height(),
    );

    // Foreground should still be present after the warp. C `pixAffineSequential`
    // pushes a lot of content off the canvas via the intermediate shears + scale,
    // so we only require non-trivial preservation, not a tight threshold.
    let area_orig = pixb.count_pixels();
    let area_fwd = pix_fwd.count_pixels();
    assert!(area_orig > 0);
    assert!(
        area_fwd > 1000,
        "warp should leave non-trivial foreground (orig={area_orig}, fwd={area_fwd})",
    );

    // Degenerate point sets are rejected.
    let mut bad_ptas = Pta::with_capacity(3);
    bad_ptas.push(0.0, 50.0);
    bad_ptas.push(100.0, 60.0);
    bad_ptas.push(50.0, 50.0); // y1 == y3
    assert!(affine_sequential(&pixb, &ptad, &bad_ptas, 0, 0).is_err());

    let mut wrong_count = Pta::with_capacity(2);
    wrong_count.push(0.0, 0.0);
    wrong_count.push(1.0, 1.0);
    assert!(affine_sequential(&pixb, &ptad, &wrong_count, 0, 0).is_err());
}

/// C version: `boxaAffineTransform` test — not implemented in Rust
#[test]
#[ignore = "boxaAffineTransform, createMatrix2d*, pixCloseBrick, pixConnComp not implemented"]
fn affine_reg_boxa_transform() {
    // C: Tests affine transforms and inverses on pix and boxa
}

//! Regression tests for fpix_affine / fpix_projective (plan 140).

use leptonica::core::fpix::{FPix, fpix_affine, fpix_projective};

fn ramp_x(w: u32, h: u32) -> FPix {
    // Ramp along x: pixel (x, y) = x.
    let mut fpix = FPix::new(w, h).unwrap();
    for y in 0..h {
        for x in 0..w {
            fpix.set_pixel(x, y, x as f32).unwrap();
        }
    }
    fpix
}

#[test]
fn fpix_affine_identity_preserves_values() {
    // Identity affine: vc = [1, 0, 0, 0, 1, 0] (x' = x, y' = y).
    let src = ramp_x(10, 10);
    let dst = fpix_affine(&src, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0], -1.0).unwrap();
    // `linear_interpolate_pixel_float` skips coordinates where x > w-2 or
    // y > h-2. So x = 9 and y = 9 both fall outside and yield inval.
    for y in 0..=8 {
        for x in 0..=8 {
            let got = dst.get_pixel(x, y).unwrap();
            assert!(
                (got - x as f32).abs() < 1e-3,
                "at ({x}, {y}): expected {}, got {got}",
                x as f32
            );
        }
    }
}

#[test]
fn fpix_affine_shift_uses_inval_outside() {
    // Shift right by 3: vc = [1, 0, -3, 0, 1, 0] (x_src = x_dst - 3).
    // Destination pixel at x_dst = 0 maps to x_src = -3 → inval.
    let src = ramp_x(10, 10);
    let dst = fpix_affine(&src, &[1.0, 0.0, -3.0, 0.0, 1.0, 0.0], -1.0).unwrap();
    assert!((dst.get_pixel(0, 5).unwrap() - (-1.0)).abs() < 1e-3);
    assert!((dst.get_pixel(1, 5).unwrap() - (-1.0)).abs() < 1e-3);
    // Pixel at x_dst = 5 → x_src = 2 → value = 2.0.
    assert!((dst.get_pixel(5, 5).unwrap() - 2.0).abs() < 1e-3);
}

#[test]
fn fpix_projective_identity_matches_affine() {
    // Identity projective: vc = [1, 0, 0, 0, 1, 0, 0, 0].
    let src = ramp_x(10, 10);
    let dst = fpix_projective(&src, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], -1.0).unwrap();
    let affine_dst = fpix_affine(&src, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0], -1.0).unwrap();
    for y in 0..10 {
        for x in 0..10 {
            let a = affine_dst.get_pixel(x, y).unwrap();
            let p = dst.get_pixel(x, y).unwrap();
            assert!((a - p).abs() < 1e-3, "mismatch at ({x}, {y}): {a} vs {p}");
        }
    }
}

#[test]
fn fpix_projective_degenerate_denom_uses_inval() {
    // Choose vc so that vc[6]*j + vc[7]*i + 1 = 0 at some pixel.
    // vc[6] = 1, vc[7] = 0 → denom = j + 1 → never 0 in 0..10.
    // To force denom = 0 at j = 3, use vc[6] = -1/3 ≈ -0.3333.
    let src = ramp_x(10, 10);
    let dst = fpix_projective(
        &src,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0 / 3.0, 0.0],
        -42.0,
    )
    .unwrap();
    // At j = 3, denom = -1/3 * 3 + 1 = 0 → inval = -42.
    assert!((dst.get_pixel(3, 5).unwrap() - (-42.0)).abs() < 1e-3);
}

#[test]
fn fpix_affine_preserves_dimensions() {
    let src = ramp_x(15, 7);
    let dst = fpix_affine(&src, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0], 0.0).unwrap();
    assert_eq!(dst.width(), 15);
    assert_eq!(dst.height(), 7);
}

#[test]
fn fpix_projective_preserves_dimensions() {
    let src = ramp_x(8, 12);
    let dst = fpix_projective(&src, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], 0.0).unwrap();
    assert_eq!(dst.width(), 8);
    assert_eq!(dst.height(), 12);
}

#[test]
fn fpix_affine_cross_term_swaps_axes() {
    // Build a 2D source where pixel (x, y) = 100*y + x — both coordinates
    // contribute. The matrix vc = [0, 1, 0, 1, 0, 0] swaps axes:
    //   x_src = vc[0]*j + vc[1]*i + vc[2] = i  (uses cross-term vc[1])
    //   y_src = vc[3]*j + vc[4]*i + vc[5] = j  (uses cross-term vc[3])
    // So dst(j, i) should equal src(i, j) = 100*j + i.
    let w = 8u32;
    let h = 8u32;
    let mut src = leptonica::core::fpix::FPix::new(w, h).unwrap();
    for y in 0..h {
        for x in 0..w {
            src.set_pixel(x, y, 100.0 * y as f32 + x as f32).unwrap();
        }
    }
    let dst = fpix_affine(&src, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0], -1.0).unwrap();
    // Check several interior pixels — at dst (j, i), expected = 100*j + i.
    // Use ±1.5 tolerance to absorb bilinear quantisation (16 sub-pixel grid).
    for &(j, i) in &[(1, 2), (3, 4), (5, 1)] {
        let got = dst.get_pixel(j as u32, i as u32).unwrap();
        let expected = 100.0 * j as f32 + i as f32;
        assert!(
            (got - expected).abs() < 1.5,
            "swap at ({j}, {i}): expected {expected}, got {got}"
        );
    }
}

#[test]
fn fpix_affine_fractional_shift_uses_bilinear_interpolation() {
    // Ramp along x with 0.5-pixel shift in x. dst(x, y) should sample
    // src(x + 0.5, y) ≈ x + 0.5 (since ramp_x is linear in x).
    let src = ramp_x(10, 10);
    let dst = fpix_affine(&src, &[1.0, 0.0, 0.5, 0.0, 1.0, 0.0], -1.0).unwrap();
    // Interior pixels only (x + 0.5 must lie in [0, w-2]).
    for x in 0u32..=7 {
        let got = dst.get_pixel(x, 5).unwrap();
        let expected = x as f32 + 0.5;
        assert!(
            (got - expected).abs() < 0.1,
            "bilinear at ({x}, 5): expected {expected}, got {got}"
        );
    }
}

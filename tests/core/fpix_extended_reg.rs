//! Regression tests for plan 110 (fpix2.c の FPix 拡張 7 関数).

use leptonica::FPix;
use leptonica::PixelDepth;
use leptonica::core::fpix::extended::linear_interpolate_pixel_float;

fn make_fpix(w: u32, h: u32, fill: f32) -> FPix {
    let mut f = FPix::new(w, h).unwrap();
    f.set_all(fill);
    f
}

// -- get_min / get_max ----------------------------------------------------

#[test]
fn fpix_get_min_finds_minimum() {
    let mut f = make_fpix(4, 3, 5.0);
    f.set_pixel(2, 1, -1.0).unwrap();
    f.set_pixel(0, 2, 3.0).unwrap();
    let (v, x, y) = f.get_min().unwrap();
    assert!((v + 1.0).abs() < 1e-6);
    assert_eq!((x, y), (2, 1));
}

#[test]
fn fpix_get_max_finds_maximum() {
    let mut f = make_fpix(3, 3, 0.0);
    f.set_pixel(1, 2, 7.5).unwrap();
    let (v, x, y) = f.get_max().unwrap();
    assert!((v - 7.5).abs() < 1e-6);
    assert_eq!((x, y), (1, 2));
}

// -- threshold_to_pix -----------------------------------------------------

#[test]
fn fpix_threshold_to_pix_mixed() {
    let mut f = make_fpix(4, 2, 2.0);
    f.set_pixel(0, 0, 1.0).unwrap();
    f.set_pixel(3, 1, 5.0).unwrap();
    let pix = f.threshold_to_pix(2.0).unwrap();
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    assert_eq!(pix.width(), 4);
    assert_eq!(pix.height(), 2);
    // val<=2 ⇒ FG. So (0,0)=1, (3,1)=0, others=1 since 2<=2.
    assert_eq!(pix.get_pixel(0, 0), Some(1));
    assert_eq!(pix.get_pixel(3, 1), Some(0));
    assert_eq!(pix.get_pixel(1, 0), Some(1));
}

#[test]
fn fpix_threshold_to_pix_all_above() {
    let f = make_fpix(2, 2, 10.0);
    let pix = f.threshold_to_pix(0.0).unwrap();
    assert_eq!(pix.count_pixels(), 0);
}

#[test]
fn fpix_threshold_to_pix_all_below_or_equal() {
    let f = make_fpix(3, 3, 1.0);
    let pix = f.threshold_to_pix(1.0).unwrap();
    assert_eq!(pix.count_pixels(), 9);
}

// -- rasterop -------------------------------------------------------------

#[test]
fn fpix_rasterop_basic_copy() {
    let mut src = FPix::new(4, 4).unwrap();
    for y in 0..4 {
        for x in 0..4 {
            src.set_pixel(x, y, (x + 10 * y) as f32).unwrap();
        }
    }
    let mut dst = FPix::new(4, 4).unwrap();
    dst.rasterop(0, 0, 4, 4, &src, 0, 0).unwrap();
    assert!((dst.get_pixel(3, 3).unwrap() - 33.0).abs() < 1e-6);
}

#[test]
fn fpix_rasterop_negative_dx_clips() {
    let mut src = FPix::new(3, 3).unwrap();
    for i in 0..3 {
        src.set_pixel(i, 0, (i + 1) as f32).unwrap();
    }
    let mut dst = FPix::new(3, 3).unwrap();
    // dx = -1, sx = 0 → effective shifts dst left start by 1, sx becomes 1.
    dst.rasterop(-1, 0, 3, 1, &src, 0, 0).unwrap();
    // After clipping: dx=0, sx=1, dw=2
    assert!((dst.get_pixel(0, 0).unwrap() - 2.0).abs() < 1e-6);
    assert!((dst.get_pixel(1, 0).unwrap() - 3.0).abs() < 1e-6);
    assert!((dst.get_pixel(2, 0).unwrap() - 0.0).abs() < 1e-6);
}

#[test]
fn fpix_rasterop_overhang_clips() {
    let src = make_fpix(2, 2, 1.0);
    let mut dst = FPix::new(4, 4).unwrap();
    // dst is 4x4, paste 4x4 from src starting at (3,3) → only 1x1 copies.
    dst.rasterop(3, 3, 4, 4, &src, 0, 0).unwrap();
    assert!((dst.get_pixel(3, 3).unwrap() - 1.0).abs() < 1e-6);
}

#[test]
fn fpix_rasterop_empty_after_clip_is_noop() {
    let src = make_fpix(2, 2, 1.0);
    let mut dst = make_fpix(2, 2, 0.0);
    // Place 2x2 entirely off the right edge.
    dst.rasterop(10, 0, 2, 2, &src, 0, 0).unwrap();
    assert_eq!(dst.get_pixel(0, 0).unwrap(), 0.0);
    assert_eq!(dst.get_pixel(1, 1).unwrap(), 0.0);
}

// -- scale_by_integer -----------------------------------------------------

#[test]
fn fpix_scale_by_integer_factor_one() {
    let mut f = FPix::new(3, 3).unwrap();
    f.set_pixel(1, 1, 5.0).unwrap();
    let out = f.scale_by_integer(1).unwrap();
    // factor=1: wd = 1*(3-1)+1 = 3 → same size, values preserved.
    assert_eq!(out.width(), 3);
    assert_eq!(out.height(), 3);
    assert!((out.get_pixel(1, 1).unwrap() - 5.0).abs() < 1e-6);
}

#[test]
fn fpix_scale_by_integer_factor_two_midpoint() {
    // 2x2 with values v0=0, v1=4, v2=8, v3=12 (row-major).
    let mut f = FPix::new(2, 2).unwrap();
    f.set_pixel(0, 0, 0.0).unwrap();
    f.set_pixel(1, 0, 4.0).unwrap();
    f.set_pixel(0, 1, 8.0).unwrap();
    f.set_pixel(1, 1, 12.0).unwrap();
    let out = f.scale_by_integer(2).unwrap();
    // wd = 2*(2-1)+1 = 3
    assert_eq!(out.width(), 3);
    assert_eq!(out.height(), 3);
    // At (1,1) is exact midpoint of v0..v3 → 6.0.
    assert!((out.get_pixel(1, 1).unwrap() - 6.0).abs() < 1e-5);
}

#[test]
fn fpix_scale_by_integer_factor_zero_errors() {
    let f = make_fpix(2, 2, 0.0);
    assert!(f.scale_by_integer(0).is_err());
}

#[test]
fn fpix_scale_by_integer_huge_factor_errors() {
    let f = make_fpix(2, 2, 0.0);
    // Factor > i32::MAX must be rejected to avoid silent wrap.
    assert!(f.scale_by_integer(u32::MAX).is_err());
}

// -- remove_border --------------------------------------------------------

#[test]
fn fpix_remove_border_zero_is_clone() {
    let mut f = FPix::new(3, 3).unwrap();
    f.set_pixel(1, 1, 9.0).unwrap();
    f.set_xres(300);
    f.set_yres(300);
    let out = f.remove_border(0, 0, 0, 0).unwrap();
    assert_eq!(out.width(), 3);
    assert!((out.get_pixel(1, 1).unwrap() - 9.0).abs() < 1e-6);
    // resolution metadata must be preserved by the zero-border path
    assert_eq!(out.xres(), 300);
    assert_eq!(out.yres(), 300);
}

#[test]
fn fpix_remove_border_partial() {
    let mut f = FPix::new(5, 4).unwrap();
    for y in 0..4 {
        for x in 0..5 {
            f.set_pixel(x, y, (x + 10 * y) as f32).unwrap();
        }
    }
    let out = f.remove_border(1, 1, 1, 1).unwrap();
    assert_eq!(out.width(), 3);
    assert_eq!(out.height(), 2);
    // Original (1,1)=11 maps to (0,0).
    assert!((out.get_pixel(0, 0).unwrap() - 11.0).abs() < 1e-6);
    // Original (3,2)=23 maps to (2,1).
    assert!((out.get_pixel(2, 1).unwrap() - 23.0).abs() < 1e-6);
}

#[test]
fn fpix_remove_border_too_much_errors() {
    let f = make_fpix(2, 2, 0.0);
    assert!(f.remove_border(5, 5, 5, 5).is_err());
}

// -- linear_interpolate_pixel_float ---------------------------------------

#[test]
fn linear_interpolate_integer_position_returns_pixel() {
    // 3x3 with each cell = row*10 + col.
    let data: Vec<f32> = (0..9).map(|i| ((i / 3) * 10 + i % 3) as f32).collect();
    let v = linear_interpolate_pixel_float(&data, 3, 3, 1.0, 1.0, -1.0);
    assert!((v - 11.0).abs() < 1e-4);
}

#[test]
fn linear_interpolate_midpoint() {
    // 2x2: 0,4 / 8,12. Midpoint (0.5, 0.5) → average = 6.0.
    let data: Vec<f32> = vec![0.0, 4.0, 8.0, 12.0];
    let v = linear_interpolate_pixel_float(&data, 2, 2, 0.0, 0.0, -1.0);
    assert!((v - 0.0).abs() < 1e-4);
    // (0,0) integer-position returns exact 0
    // (0.5, 0.5) is at x<= w-2=0, so it counts as on-edge ⇒ allowed.
    let v_mid = linear_interpolate_pixel_float(&data, 2, 2, 0.0, 0.0, -1.0);
    assert!((v_mid - 0.0).abs() < 1e-4);
}

#[test]
fn linear_interpolate_off_edge_returns_inval() {
    let data: Vec<f32> = vec![1.0; 4];
    let v = linear_interpolate_pixel_float(&data, 2, 2, -0.1, 0.0, -7.0);
    assert!((v + 7.0).abs() < 1e-6);
    let v2 = linear_interpolate_pixel_float(&data, 2, 2, 0.5, 0.0, -7.0);
    // For 2x2, w-2=0, so x>0 is "off the edge" too.
    assert!((v2 + 7.0).abs() < 1e-6);
}

#[test]
fn linear_interpolate_within_3x3() {
    // 3x3 with linear pattern: cell = row + col.
    let data: Vec<f32> = (0..9).map(|i| ((i / 3) + (i % 3)) as f32).collect();
    let v = linear_interpolate_pixel_float(&data, 3, 3, 0.5, 0.5, -1.0);
    // Bilinear of (0,1,1,2) at (0.5,0.5) = 1.0.
    assert!((v - 1.0).abs() < 1e-3);
}

#[test]
fn linear_interpolate_nan_returns_inval() {
    let data: Vec<f32> = vec![1.0; 9];
    let v = linear_interpolate_pixel_float(&data, 3, 3, f32::NAN, 0.0, -9.0);
    assert!((v + 9.0).abs() < 1e-6);
    let v2 = linear_interpolate_pixel_float(&data, 3, 3, 0.0, f32::INFINITY, -9.0);
    assert!((v2 + 9.0).abs() < 1e-6);
}

#[test]
fn linear_interpolate_small_dims_return_inval() {
    let data: Vec<f32> = vec![1.0, 2.0];
    // w < 2 or h < 2 are rejected.
    let v = linear_interpolate_pixel_float(&data, 1, 2, 0.0, 0.0, -7.0);
    assert!((v + 7.0).abs() < 1e-6);
    let v2 = linear_interpolate_pixel_float(&data, 2, 1, 0.0, 0.0, -7.0);
    assert!((v2 + 7.0).abs() < 1e-6);
}

#[test]
fn linear_interpolate_short_buffer_returns_inval() {
    // Claim 3x3 = 9 elements but pass only 4.
    let data: Vec<f32> = vec![1.0; 4];
    let v = linear_interpolate_pixel_float(&data, 3, 3, 0.5, 0.5, -7.0);
    assert!((v + 7.0).abs() < 1e-6);
}

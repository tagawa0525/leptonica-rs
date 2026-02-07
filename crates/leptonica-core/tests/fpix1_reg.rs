//! FPix (floating-point image) regression test
//!
//! C版: reference/leptonica/prog/fpix1_reg.c
//! FPixの作成、ピクセルアクセス、算術演算、Pix変換をテスト。
//!
//! C版テストは以下の大きなセクションから構成される:
//!   0-3:   ガウシアンカーネルの作成と表示
//!   4-9:   Pixおよび FPixでの畳み込み
//!   10-12: 算術演算（回転180°、線形結合、スケーリング）
//!   13-21: サンプル出力付き畳み込み（RGB含む）
//!   22-26: FPixの拡張（continued/slope border）、DPix変換
//!   27-29: FPixのアフィン/射影変換
//!
//! Rust実装では FPix コアAPI（作成、ピクセルアクセス、算術、Pix変換、統計）の
//! テストに集中する。C版の高度な機能（畳み込み、カーネル表示、rasterop、
//! dewarp、DPix、ボーダー拡張、アフィン/射影変換）は未実装のためスキップ。
//!
//! Run with:
//! ```
//! cargo test -p leptonica-core --test fpix1_reg
//! ```

use leptonica_core::{FPix, NegativeHandling, Pix, PixelDepth};
use leptonica_test::RegParams;

// ==========================================================================
// Test 1: FPix creation and basic properties
// C版: fpixCreate() に相当
// ==========================================================================
#[test]
fn fpix1_reg_creation() {
    let mut rp = RegParams::new("fpix1_creation");

    // --- FPix creation ---
    let fpix = FPix::new(640, 480).expect("FPix::new failed");
    rp.compare_values(640.0, fpix.width() as f64, 0.0);
    rp.compare_values(480.0, fpix.height() as f64, 0.0);
    rp.compare_values(640.0, fpix.dimensions().0 as f64, 0.0);
    rp.compare_values(480.0, fpix.dimensions().1 as f64, 0.0);

    // All pixels should be zero initially
    let all_zero = fpix.data().iter().all(|&v| v == 0.0);
    rp.compare_values(1.0, if all_zero { 1.0 } else { 0.0 }, 0.0);

    // --- FPix creation with initial value ---
    let fpix_v = FPix::new_with_value(100, 100, 42.5).expect("FPix::new_with_value failed");
    let all_match = fpix_v.data().iter().all(|&v| v == 42.5);
    rp.compare_values(1.0, if all_match { 1.0 } else { 0.0 }, 0.0);

    // --- FPix from raw data ---
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let fpix_d = FPix::from_data(3, 2, data).expect("FPix::from_data failed");
    rp.compare_values(3.0, fpix_d.width() as f64, 0.0);
    rp.compare_values(2.0, fpix_d.height() as f64, 0.0);
    rp.compare_values(1.0, fpix_d.get_pixel(0, 0).unwrap() as f64, 0.0);
    rp.compare_values(6.0, fpix_d.get_pixel(2, 1).unwrap() as f64, 0.0);

    // --- Invalid creation ---
    rp.compare_values(1.0, if FPix::new(0, 100).is_err() { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if FPix::new(100, 0).is_err() { 1.0 } else { 0.0 }, 0.0);

    // --- from_data with wrong size ---
    let bad_data = vec![1.0, 2.0, 3.0];
    rp.compare_values(
        1.0,
        if FPix::from_data(3, 2, bad_data).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "fpix1_creation regression test failed");
}

// ==========================================================================
// Test 2: Pixel access
// C版: fpixGetPixel(), fpixSetPixel() に相当
// ==========================================================================
#[test]
fn fpix1_reg_pixel_access() {
    let mut rp = RegParams::new("fpix1_pixel_access");

    let mut fpix = FPix::new(100, 100).expect("FPix::new failed");

    // Set and get various positions
    fpix.set_pixel(0, 0, 1.5).expect("set_pixel(0,0)");
    fpix.set_pixel(99, 99, -3.14).expect("set_pixel(99,99)");
    fpix.set_pixel(50, 25, 255.0).expect("set_pixel(50,25)");
    fpix.set_pixel(10, 20, 0.001).expect("set_pixel(10,20)");

    rp.compare_values(1.5, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(-3.14, fpix.get_pixel(99, 99).unwrap() as f64, 1e-6);
    rp.compare_values(255.0, fpix.get_pixel(50, 25).unwrap() as f64, 1e-6);
    rp.compare_values(0.001, fpix.get_pixel(10, 20).unwrap() as f64, 1e-6);

    // Out of bounds should error
    rp.compare_values(
        1.0,
        if fpix.get_pixel(100, 0).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if fpix.get_pixel(0, 100).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if fpix.set_pixel(100, 100, 0.0).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Unsafe unchecked access
    unsafe {
        fpix.set_pixel_unchecked(5, 5, 77.7);
        let val = fpix.get_pixel_unchecked(5, 5);
        rp.compare_values(77.7, val as f64, 1e-5);
    }

    // Row access
    for x in 0..100u32 {
        fpix.set_pixel(x, 10, x as f32 * 0.1).unwrap();
    }
    let row = fpix.row(10);
    rp.compare_values(100.0, row.len() as f64, 0.0);
    rp.compare_values(0.0, row[0] as f64, 1e-6);
    rp.compare_values(5.0, row[50] as f64, 1e-5);
    rp.compare_values(9.9, row[99] as f64, 1e-5);

    assert!(rp.cleanup(), "fpix1_pixel_access regression test failed");
}

// ==========================================================================
// Test 3: Resolution
// C版: fpixGetResolution(), fpixSetResolution() に相当
// ==========================================================================
#[test]
fn fpix1_reg_resolution() {
    let mut rp = RegParams::new("fpix1_resolution");

    let mut fpix = FPix::new(100, 100).expect("FPix::new failed");

    // Default resolution is 0
    rp.compare_values(0.0, fpix.xres() as f64, 0.0);
    rp.compare_values(0.0, fpix.yres() as f64, 0.0);

    // Set resolution
    fpix.set_resolution(300, 600);
    rp.compare_values(300.0, fpix.xres() as f64, 0.0);
    rp.compare_values(600.0, fpix.yres() as f64, 0.0);

    let (xr, yr) = fpix.resolution();
    rp.compare_values(300.0, xr as f64, 0.0);
    rp.compare_values(600.0, yr as f64, 0.0);

    // Set individually
    fpix.set_xres(150);
    fpix.set_yres(200);
    rp.compare_values(150.0, fpix.xres() as f64, 0.0);
    rp.compare_values(200.0, fpix.yres() as f64, 0.0);

    assert!(rp.cleanup(), "fpix1_resolution regression test failed");
}

// ==========================================================================
// Test 4: Arithmetic operations
// C版: fpixLinearCombination(), fpixAddMultConstant() の一部に相当
// ==========================================================================
#[test]
fn fpix1_reg_arithmetic() {
    let mut rp = RegParams::new("fpix1_arithmetic");

    let w = 50u32;
    let h = 50u32;

    let fpix_a = FPix::new_with_value(w, h, 10.0).unwrap();
    let fpix_b = FPix::new_with_value(w, h, 3.0).unwrap();

    // --- Addition ---
    let result = fpix_a.add(&fpix_b).unwrap();
    rp.compare_values(13.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(13.0, result.get_pixel(25, 25).unwrap() as f64, 1e-6);

    // --- Subtraction ---
    let result = fpix_a.sub(&fpix_b).unwrap();
    rp.compare_values(7.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    // --- Multiplication ---
    let result = fpix_a.mul(&fpix_b).unwrap();
    rp.compare_values(30.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    // --- Division ---
    let result = fpix_a.div(&fpix_b).unwrap();
    rp.compare_values(10.0 / 3.0, result.get_pixel(0, 0).unwrap() as f64, 1e-5);

    // --- Operator overloading ---
    let result = (&fpix_a + &fpix_b).unwrap();
    rp.compare_values(13.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    let result = (&fpix_a - &fpix_b).unwrap();
    rp.compare_values(7.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    let result = (&fpix_a * &fpix_b).unwrap();
    rp.compare_values(30.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    let result = (&fpix_a / &fpix_b).unwrap();
    rp.compare_values(10.0 / 3.0, result.get_pixel(0, 0).unwrap() as f64, 1e-5);

    // --- Size mismatch should error ---
    let fpix_small = FPix::new(10, 10).unwrap();
    rp.compare_values(
        1.0,
        if fpix_a.add(&fpix_small).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if fpix_a.sub(&fpix_small).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if fpix_a.mul(&fpix_small).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if fpix_a.div(&fpix_small).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // --- Division by zero produces infinity ---
    let fpix_zero = FPix::new_with_value(w, h, 0.0).unwrap();
    let result = fpix_a.div(&fpix_zero).unwrap();
    let val = result.get_pixel(0, 0).unwrap();
    rp.compare_values(1.0, if val.is_infinite() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "fpix1_arithmetic regression test failed");
}

// ==========================================================================
// Test 5: Constant operations and linear combination
// C版: fpixAddMultConstant() に相当
// ==========================================================================
#[test]
fn fpix1_reg_constant_ops() {
    let mut rp = RegParams::new("fpix1_constant_ops");

    // --- add_constant ---
    let mut fpix = FPix::new_with_value(20, 20, 5.0).unwrap();
    fpix.add_constant(3.0);
    rp.compare_values(8.0, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(8.0, fpix.get_pixel(10, 10).unwrap() as f64, 1e-6);

    // --- mul_constant ---
    fpix.mul_constant(2.0);
    rp.compare_values(16.0, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);

    // --- C版: fpixAddMultConstant(fpixd, 0.0, 23.174) ---
    // This is equivalent to: add 0.0 first, then multiply by 23.174
    // In Rust: add_constant(0.0) + mul_constant(23.174)
    let mut fpix2 = FPix::new_with_value(20, 20, 10.0).unwrap();
    fpix2.add_constant(0.0);
    fpix2.mul_constant(23.174);
    rp.compare_values(231.74, fpix2.get_pixel(0, 0).unwrap() as f64, 1e-3);

    // --- linear_combination: result = multiplier * pixel + addend ---
    let fpix3 = FPix::new_with_value(20, 20, 2.0).unwrap();
    let result = fpix3.linear_combination(3.0, 1.0); // 3*2 + 1 = 7
    rp.compare_values(7.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(7.0, result.get_pixel(19, 19).unwrap() as f64, 1e-6);

    // --- linear_combination with negative multiplier ---
    let result = fpix3.linear_combination(-1.0, 10.0); // -1*2 + 10 = 8
    rp.compare_values(8.0, result.get_pixel(0, 0).unwrap() as f64, 1e-6);

    // --- set_all and clear ---
    let mut fpix4 = FPix::new(30, 30).unwrap();
    fpix4.set_all(42.0);
    rp.compare_values(42.0, fpix4.get_pixel(15, 15).unwrap() as f64, 1e-6);

    fpix4.clear();
    rp.compare_values(0.0, fpix4.get_pixel(15, 15).unwrap() as f64, 1e-6);

    assert!(rp.cleanup(), "fpix1_constant_ops regression test failed");
}

// ==========================================================================
// Test 6: Statistics (min, max, mean, sum)
// ==========================================================================
#[test]
fn fpix1_reg_statistics() {
    let mut rp = RegParams::new("fpix1_statistics");

    // Create FPix with known pattern
    let mut fpix = FPix::new(10, 10).unwrap();
    for y in 0..10u32 {
        for x in 0..10u32 {
            let val = (y * 10 + x) as f32;
            fpix.set_pixel(x, y, val).unwrap();
        }
    }
    // Values: 0, 1, 2, ..., 99

    // --- min ---
    let (min_val, min_x, min_y) = fpix.min().unwrap();
    rp.compare_values(0.0, min_val as f64, 1e-6);
    rp.compare_values(0.0, min_x as f64, 0.0);
    rp.compare_values(0.0, min_y as f64, 0.0);

    rp.compare_values(0.0, fpix.min_value().unwrap() as f64, 1e-6);

    // --- max ---
    let (max_val, max_x, max_y) = fpix.max().unwrap();
    rp.compare_values(99.0, max_val as f64, 1e-6);
    rp.compare_values(9.0, max_x as f64, 0.0);
    rp.compare_values(9.0, max_y as f64, 0.0);

    rp.compare_values(99.0, fpix.max_value().unwrap() as f64, 1e-6);

    // --- mean ---
    // Mean of 0..99 = 49.5
    rp.compare_values(49.5, fpix.mean().unwrap() as f64, 1e-4);

    // --- sum ---
    // Sum of 0..99 = 99*100/2 = 4950
    rp.compare_values(4950.0, fpix.sum() as f64, 1e-2);

    // --- Test with negative values ---
    let mut fpix2 = FPix::new_with_value(5, 5, 10.0).unwrap();
    fpix2.set_pixel(3, 3, -100.0).unwrap();
    fpix2.set_pixel(1, 4, 200.0).unwrap();

    let (neg_min, neg_min_x, neg_min_y) = fpix2.min().unwrap();
    rp.compare_values(-100.0, neg_min as f64, 1e-6);
    rp.compare_values(3.0, neg_min_x as f64, 0.0);
    rp.compare_values(3.0, neg_min_y as f64, 0.0);

    let (neg_max, neg_max_x, neg_max_y) = fpix2.max().unwrap();
    rp.compare_values(200.0, neg_max as f64, 1e-6);
    rp.compare_values(1.0, neg_max_x as f64, 0.0);
    rp.compare_values(4.0, neg_max_y as f64, 0.0);

    // Mean: (23 * 10.0 + (-100.0) + 200.0) / 25 = (230 - 100 + 200) / 25 = 330/25 = 13.2
    rp.compare_values(13.2, fpix2.mean().unwrap() as f64, 1e-4);

    assert!(rp.cleanup(), "fpix1_statistics regression test failed");
}

// ==========================================================================
// Test 7: Pix <-> FPix conversion (round-trip)
// C版: pixConvertToFPix(), fpixConvertToPix() に相当
// ==========================================================================
#[test]
fn fpix1_reg_pix_conversion() {
    let mut rp = RegParams::new("fpix1_pix_conversion");

    // --- Create 8-bit Pix with known values ---
    let pix8 = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
    let mut pix8_mut = pix8.to_mut();
    for y in 0..8u32 {
        for x in 0..8u32 {
            let val = (y * 32 + x * 4) as u32; // 0..252 range
            pix8_mut.set_pixel(x, y, val).unwrap();
        }
    }
    let pix8: Pix = pix8_mut.into();

    // Convert to FPix
    let fpix = FPix::from_pix(&pix8).unwrap();
    rp.compare_values(8.0, fpix.width() as f64, 0.0);
    rp.compare_values(8.0, fpix.height() as f64, 0.0);

    // Check pixel (0,0) = 0, pixel (1,0) = 4, pixel (0,1) = 32
    rp.compare_values(0.0, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(4.0, fpix.get_pixel(1, 0).unwrap() as f64, 1e-6);
    rp.compare_values(32.0, fpix.get_pixel(0, 1).unwrap() as f64, 1e-6);

    // Convert back to Pix
    let pix_back = fpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(8.0, pix_back.width() as f64, 0.0);
    rp.compare_values(8.0, pix_back.height() as f64, 0.0);
    assert_eq!(pix_back.depth(), PixelDepth::Bit8);

    // Round-trip should be exact for integer values in 8-bit range
    for y in 0..8u32 {
        for x in 0..8u32 {
            let orig = pix8.get_pixel(x, y).unwrap_or(0);
            let back = pix_back.get_pixel(x, y).unwrap_or(u32::MAX);
            if orig != back {
                rp.compare_values(orig as f64, back as f64, 0.0);
            }
        }
    }
    // If we got here without failures, register a pass
    rp.compare_values(1.0, 1.0, 0.0);

    // --- Test NegativeHandling ---
    let mut fpix_neg = FPix::new(4, 1).unwrap();
    fpix_neg.set_pixel(0, 0, -50.0).unwrap();
    fpix_neg.set_pixel(1, 0, 100.0).unwrap();
    fpix_neg.set_pixel(2, 0, -200.0).unwrap();
    fpix_neg.set_pixel(3, 0, 0.0).unwrap();

    // ClipToZero: negative -> 0
    let pix_clip = fpix_neg.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    let clip_data = pix_clip.data();
    let p0 = (clip_data[0] >> 24) & 0xff;
    let p1 = (clip_data[0] >> 16) & 0xff;
    let p2 = (clip_data[0] >> 8) & 0xff;
    let p3 = clip_data[0] & 0xff;
    rp.compare_values(0.0, p0 as f64, 0.0); // -50 clipped to 0
    rp.compare_values(100.0, p1 as f64, 1.0); // 100 + 0.5 -> 100
    rp.compare_values(0.0, p2 as f64, 0.0); // -200 clipped to 0
    rp.compare_values(0.0, p3 as f64, 0.0); // 0 + 0.5 -> 0

    // TakeAbsValue: negative -> absolute value
    let pix_abs = fpix_neg.to_pix(8, NegativeHandling::TakeAbsValue).unwrap();
    let abs_data = pix_abs.data();
    let a0 = (abs_data[0] >> 24) & 0xff;
    let a1 = (abs_data[0] >> 16) & 0xff;
    let a2 = (abs_data[0] >> 8) & 0xff;
    rp.compare_values(50.0, a0 as f64, 1.0); // |-50| + 0.5 -> 50
    rp.compare_values(100.0, a1 as f64, 1.0); // 100 + 0.5 -> 100
    rp.compare_values(200.0, a2 as f64, 1.0); // |-200| + 0.5 -> 200

    // --- Test auto-detect depth ---
    let fpix_low = FPix::new_with_value(10, 10, 200.0).unwrap();
    let pix_auto = fpix_low.to_pix(0, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(8.0, pix_auto.depth().bits() as f64, 0.0);

    let fpix_mid = FPix::new_with_value(10, 10, 1000.0).unwrap();
    let pix_auto = fpix_mid.to_pix(0, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(16.0, pix_auto.depth().bits() as f64, 0.0);

    let fpix_high = FPix::new_with_value(10, 10, 100000.0).unwrap();
    let pix_auto = fpix_high.to_pix(0, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(32.0, pix_auto.depth().bits() as f64, 0.0);

    // --- Invalid out_depth ---
    rp.compare_values(
        1.0,
        if fpix_low.to_pix(4, NegativeHandling::ClipToZero).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "fpix1_pix_conversion regression test failed");
}

// ==========================================================================
// Test 8: Resolution preservation through Pix conversion
// C版のfpix作成時のncomps引数はRust版ではresolution保持で代替
// ==========================================================================
#[test]
fn fpix1_reg_resolution_preservation() {
    let mut rp = RegParams::new("fpix1_res_preserve");

    // Create Pix with resolution
    let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    let mut pix_mut = pix.to_mut();
    pix_mut.set_xres(300);
    pix_mut.set_yres(600);
    let pix: Pix = pix_mut.into();

    // Convert to FPix - resolution should be preserved
    let fpix = FPix::from_pix(&pix).unwrap();
    rp.compare_values(300.0, fpix.xres() as f64, 0.0);
    rp.compare_values(600.0, fpix.yres() as f64, 0.0);

    // Convert back to Pix - resolution should still be preserved
    let pix_back = fpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(300.0, pix_back.xres() as f64, 0.0);
    rp.compare_values(600.0, pix_back.yres() as f64, 0.0);

    assert!(rp.cleanup(), "fpix1_res_preserve regression test failed");
}

// ==========================================================================
// Test 9: FPix clone independence
// ==========================================================================
#[test]
fn fpix1_reg_clone() {
    let mut rp = RegParams::new("fpix1_clone");

    let fpix1 = FPix::new_with_value(50, 50, 5.0).unwrap();
    let mut fpix2 = fpix1.clone();

    // Clones should have equal data
    let data_match = fpix1.data() == fpix2.data();
    rp.compare_values(1.0, if data_match { 1.0 } else { 0.0 }, 0.0);

    // But be independent (modifying one doesn't affect the other)
    fpix2.set_pixel(0, 0, 999.0).unwrap();
    rp.compare_values(5.0, fpix1.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(999.0, fpix2.get_pixel(0, 0).unwrap() as f64, 1e-6);

    assert!(rp.cleanup(), "fpix1_clone regression test failed");
}

// ==========================================================================
// Test 10: Pix conversion with real image
// C版: pixConvertToFPix(pixs, 3) に相当
// ==========================================================================
#[test]
fn fpix1_reg_real_image_conversion() {
    let mut rp = RegParams::new("fpix1_real_image");

    // Load test8.jpg (8-bit grayscale, used in C version)
    let pixs = match leptonica_test::load_test_image("test8.jpg") {
        Ok(pix) => pix,
        Err(e) => {
            eprintln!("Cannot load test8.jpg: {} -- skipping", e);
            assert!(rp.cleanup());
            return;
        }
    };

    let w = pixs.width();
    let h = pixs.height();
    eprintln!("  test8.jpg: {}x{}, depth={}", w, h, pixs.depth().bits());

    // Convert to FPix
    let fpix = FPix::from_pix(&pixs).unwrap();
    rp.compare_values(w as f64, fpix.width() as f64, 0.0);
    rp.compare_values(h as f64, fpix.height() as f64, 0.0);

    // All values should be in [0, 255] range for 8-bit source
    let min_val = fpix.min_value().unwrap();
    let max_val = fpix.max_value().unwrap();
    rp.compare_values(1.0, if min_val >= 0.0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if max_val <= 255.0 { 1.0 } else { 0.0 }, 0.0);

    eprintln!("  FPix value range: [{}, {}]", min_val, max_val);

    // Round-trip: FPix -> Pix should produce same image
    let pix_back = fpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
    rp.compare_values(w as f64, pix_back.width() as f64, 0.0);
    rp.compare_values(h as f64, pix_back.height() as f64, 0.0);

    // Compare pixel values - should be same for 8-bit int values
    let mut max_diff = 0u32;
    for y in 0..h.min(100) {
        for x in 0..w.min(100) {
            let orig = pixs.get_pixel(x, y).unwrap_or(0);
            let back = pix_back.get_pixel(x, y).unwrap_or(u32::MAX);
            let diff = if orig > back {
                orig - back
            } else {
                back - orig
            };
            if diff > max_diff {
                max_diff = diff;
            }
        }
    }
    eprintln!("  Round-trip max pixel difference: {}", max_diff);
    // Allow up to 1 difference due to rounding
    rp.compare_values(1.0, if max_diff <= 1 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "fpix1_real_image regression test failed");
}

// ==========================================================================
// Test 11: Arithmetic with real image (partial port of C tests 10-12)
// C版: fpixLinearCombination(), fpixAddMultConstant(), fpixDisplayMaxDynamicRange()
// ==========================================================================
#[test]
fn fpix1_reg_arithmetic_with_image() {
    let mut rp = RegParams::new("fpix1_arith_image");

    // Load test image
    let pixs = match leptonica_test::load_test_image("test8.jpg") {
        Ok(pix) => pix,
        Err(e) => {
            eprintln!("Cannot load test8.jpg: {} -- skipping", e);
            assert!(rp.cleanup());
            return;
        }
    };

    let fpixs = FPix::from_pix(&pixs).unwrap();
    let w = fpixs.width();
    let h = fpixs.height();

    // C版: fpixs3 = pixConvertToFPix(pixs3, 3) where pixs3 = rotate180(pixs)
    // Since we don't have rotate180 for FPix, create a manually rotated version
    let mut fpix_rot = FPix::new(w, h).unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = fpixs.get_pixel(x, y).unwrap();
            fpix_rot.set_pixel(w - 1 - x, h - 1 - y, val).unwrap();
        }
    }

    // C版: fpixd = fpixLinearCombination(NULL, fpixs, fpixs3, 20.0, 5.0)
    // This computes: fpixd[i] = 20.0 * fpixs[i] + 5.0 * fpixs3[i]
    // Rust: manual element-wise combination
    let mut fpixd = FPix::new(w, h).unwrap();
    for y in 0..h {
        for x in 0..w {
            let v1 = fpixs.get_pixel(x, y).unwrap();
            let v2 = fpix_rot.get_pixel(x, y).unwrap();
            fpixd.set_pixel(x, y, 20.0 * v1 + 5.0 * v2).unwrap();
        }
    }

    // C版: fpixAddMultConstant(fpixd, 0.0, 23.174)
    // adds 0.0 then multiplies by 23.174
    fpixd.add_constant(0.0);
    fpixd.mul_constant(23.174);

    // Result should have large values (original 0-255 * 20 * 23.174 ≈ 0 .. ~118000)
    let max_val = fpixd.max_value().unwrap();
    let min_val = fpixd.min_value().unwrap();
    eprintln!(
        "  After linear combination: min={:.1}, max={:.1}",
        min_val, max_val
    );
    rp.compare_values(1.0, if max_val > 10000.0 { 1.0 } else { 0.0 }, 0.0);

    // C版: pixd = fpixDisplayMaxDynamicRange(fpixd) -- Rust未実装のためスキップ
    // Instead, manually normalize to [0, 255] range and convert
    let range = max_val - min_val;
    if range > 0.0 {
        let mut fpix_norm = FPix::new(w, h).unwrap();
        for y in 0..h {
            for x in 0..w {
                let v = fpixd.get_pixel(x, y).unwrap();
                fpix_norm
                    .set_pixel(x, y, (v - min_val) / range * 255.0)
                    .unwrap();
            }
        }
        let pix_result = fpix_norm.to_pix(8, NegativeHandling::ClipToZero).unwrap();
        rp.compare_values(w as f64, pix_result.width() as f64, 0.0);
        rp.compare_values(h as f64, pix_result.height() as f64, 0.0);
    } else {
        rp.compare_values(1.0, 0.0, 0.0); // Should not happen
    }

    assert!(rp.cleanup(), "fpix1_arith_image regression test failed");
}

// ==========================================================================
// Test 12: Conversion from different bit depths
// C版: pixConvertToFPix() は 1, 2, 4, 8, 16, 32 bpp をサポート
// ==========================================================================
#[test]
fn fpix1_reg_multi_depth_conversion() {
    let mut rp = RegParams::new("fpix1_multi_depth");

    // --- 1-bit image ---
    let pix1 = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
    let mut pix1_mut = pix1.to_mut();
    // Set some pixels to 1 (bit pattern: set pixel (0,0) and (7,0))
    let data = pix1_mut.data_mut();
    data[0] = 0x81000000; // bits 31 and 24: pixels 0 and 7 of row 0
    let pix1: Pix = pix1_mut.into();

    let fpix1 = FPix::from_pix(&pix1).unwrap();
    rp.compare_values(1.0, fpix1.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(0.0, fpix1.get_pixel(1, 0).unwrap() as f64, 1e-6);
    rp.compare_values(1.0, fpix1.get_pixel(7, 0).unwrap() as f64, 1e-6);

    // --- 8-bit image ---
    let pix8 = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
    let mut pix8_mut = pix8.to_mut();
    let data = pix8_mut.data_mut();
    data[0] = 0x00_40_80_FF; // pixels: 0, 64, 128, 255
    let pix8: Pix = pix8_mut.into();

    let fpix8 = FPix::from_pix(&pix8).unwrap();
    rp.compare_values(0.0, fpix8.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(64.0, fpix8.get_pixel(1, 0).unwrap() as f64, 1e-6);
    rp.compare_values(128.0, fpix8.get_pixel(2, 0).unwrap() as f64, 1e-6);
    rp.compare_values(255.0, fpix8.get_pixel(3, 0).unwrap() as f64, 1e-6);

    // --- 16-bit image ---
    let pix16 = Pix::new(2, 1, PixelDepth::Bit16).unwrap();
    let mut pix16_mut = pix16.to_mut();
    let data = pix16_mut.data_mut();
    data[0] = 0x0100_FF00; // pixels: 256, 65280
    let pix16: Pix = pix16_mut.into();

    let fpix16 = FPix::from_pix(&pix16).unwrap();
    rp.compare_values(256.0, fpix16.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(65280.0, fpix16.get_pixel(1, 0).unwrap() as f64, 1e-6);

    assert!(rp.cleanup(), "fpix1_multi_depth regression test failed");
}

// ==========================================================================
// Test 13: Data access (row and slice)
// ==========================================================================
#[test]
fn fpix1_reg_data_access() {
    let mut rp = RegParams::new("fpix1_data_access");

    let mut fpix = FPix::new(5, 3).unwrap();

    // Set values in row 1
    for x in 0..5u32 {
        fpix.set_pixel(x, 1, (x + 1) as f32).unwrap();
    }

    // Check row access
    let row1 = fpix.row(1);
    rp.compare_values(5.0, row1.len() as f64, 0.0);
    rp.compare_values(1.0, row1[0] as f64, 1e-6);
    rp.compare_values(2.0, row1[1] as f64, 1e-6);
    rp.compare_values(3.0, row1[2] as f64, 1e-6);
    rp.compare_values(4.0, row1[3] as f64, 1e-6);
    rp.compare_values(5.0, row1[4] as f64, 1e-6);

    // Check mutable row access
    {
        let row0_mut = fpix.row_mut(0);
        row0_mut[0] = 10.0;
        row0_mut[4] = 50.0;
    }
    rp.compare_values(10.0, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);
    rp.compare_values(50.0, fpix.get_pixel(4, 0).unwrap() as f64, 1e-6);

    // Check full data access
    let data = fpix.data();
    rp.compare_values(15.0, data.len() as f64, 0.0); // 5 * 3 = 15
    rp.compare_values(10.0, data[0] as f64, 1e-6); // row 0, col 0

    // Check mutable data access
    fpix.data_mut()[0] = 99.0;
    rp.compare_values(99.0, fpix.get_pixel(0, 0).unwrap() as f64, 1e-6);

    assert!(rp.cleanup(), "fpix1_data_access regression test failed");
}

// ==========================================================================
// Ignored tests: C API functions not available in Rust
// ==========================================================================

#[test]
#[ignore = "C版: makeGaussianKernel(), kernelGetSum(), kernelDisplayInPix() -- Rust (leptonica-filter) 未実装のためスキップ (tests 0-3)"]
fn fpix1_reg_gaussian_kernel() {
    // C版: kel = makeGaussianKernel(5, 5, 3.0, 4.0)
    // C版: makeGaussianKernelSep(5, 5, 3.0, 4.0, &kelx, &kely)
    // C版: kernelGetSum(kel, &sum)
    // C版: kernelDisplayInPix(kel, 41, 2)
    // Rust版にはmakeGaussianKernel (非対称sigma)、kernelDisplayInPix未実装
}

#[test]
#[ignore = "C版: pixConvolve(), pixConvolveSep(), fpixConvolve(), fpixConvolveSep() -- Rust未実装のためスキップ (tests 4-9)"]
fn fpix1_reg_convolution() {
    // C版: pixs上の畳み込み + fpixs上の畳み込みの比較
    // Rust版にfpixConvolve, fpixConvolveSep 未実装
}

#[test]
#[ignore = "C版: fpixDisplayMaxDynamicRange() -- Rust未実装のためスキップ (tests 10-12)"]
fn fpix1_reg_display_max_dynamic_range() {
    // C版: fpixDisplayMaxDynamicRange(fpixd)
    // 最大ダイナミックレンジでの表示（ログスケール）
}

#[test]
#[ignore = "C版: l_setConvolveSampling(), pixConvolveRGB(), pixConvolveRGBSep() -- Rust未実装のためスキップ (tests 13-21)"]
fn fpix1_reg_sampled_convolution() {
    // C版: サンプリング付き畳み込み (5,5)
    // C版: RGB畳み込み
}

#[test]
#[ignore = "C版: dewarpCreate(), fpixAddContinuedBorder(), fpixAddSlopeBorder(), fpixRenderContours(), DPix -- Rust未実装のためスキップ (tests 22-26)"]
fn fpix1_reg_border_extension_and_dpix() {
    // C版: dewarp -> fullvdispar fpix の取得
    // C版: fpixAddContinuedBorder, fpixAddSlopeBorder
    // C版: fpixConvertToDPix, dpixConvertToFPix
    // C版: fpixRenderContours
    // C版: pixConvertToDPix, dpixConvertToPix
}

#[test]
#[ignore = "C版: fpixWrite(), fpixRead(), fpixAffinePta(), fpixProjectivePta(), fpixAutoRenderContours() -- Rust未実装のためスキップ (tests 27-29)"]
fn fpix1_reg_affine_projective() {
    // C版: fpixシリアライズI/O
    // C版: fpixアフィン変換
    // C版: fpix射影変換
    // C版: fpixAutoRenderContours
}

#[test]
#[ignore = "C版: fpixRasterop() -- Rust未実装のためスキップ"]
fn fpix1_reg_rasterop() {
    // C版: fpixRasterop(fpixs, 150, 125, 150, 100, fpixs2, 75, 100)
    // FPix間のraster操作
}

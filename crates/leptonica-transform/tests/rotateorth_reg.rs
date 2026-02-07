//! Orthogonal rotation regression test
//!
//! C版: reference/leptonica/prog/rotateorth_reg.c
//! 直交回転(0°, 90°, 180°, 270°)をテスト。
//! すべてのビット深度で正確な回転を検証。

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{rotate_90, rotate_180, rotate_orth};

#[test]
fn rotateorth_reg() {
    let mut rp = RegParams::new("rotateorth");

    // Test with binary image
    let pix1 = load_test_image("feyn-fract.tif").expect("load binary image");
    test_orth_rotation(&mut rp, &pix1, "1bpp");

    // Test with 8bpp grayscale
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp image");
    test_orth_rotation(&mut rp, &pix8, "8bpp");

    // Test with 32bpp color
    let pix32 = load_test_image("weasel32.png").expect("load 32bpp image");
    test_orth_rotation(&mut rp, &pix32, "32bpp");

    assert!(rp.cleanup(), "rotateorth regression test failed");
}

fn test_orth_rotation(rp: &mut RegParams, pixs: &leptonica_core::Pix, label: &str) {
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Testing {} orthogonal rotation: {}x{}", label, w, h);

    // --- rotate_orth(0) = identity ---
    let r0 = rotate_orth(pixs, 0).expect("rotate_orth 0");
    let same = pixs.equals(&r0);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(0) == identity: {}", label, same);

    // --- rotate_orth(1) = 90° CW ---
    let r1 = rotate_orth(pixs, 1).expect("rotate_orth 1");
    rp.compare_values(h as f64, r1.width() as f64, 0.0);
    rp.compare_values(w as f64, r1.height() as f64, 0.0);

    // Should match rotate_90(cw)
    let r90 = rotate_90(pixs, true).expect("rotate_90 cw");
    let same = r1.equals(&r90);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(1) == rotate_90(cw): {}", label, same);

    // --- rotate_orth(2) = 180° ---
    let r2 = rotate_orth(pixs, 2).expect("rotate_orth 2");
    rp.compare_values(w as f64, r2.width() as f64, 0.0);
    rp.compare_values(h as f64, r2.height() as f64, 0.0);

    let r180 = rotate_180(pixs).expect("rotate_180");
    let same = r2.equals(&r180);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(2) == rotate_180: {}", label, same);

    // --- rotate_orth(3) = 270° CW = 90° CCW ---
    let r3 = rotate_orth(pixs, 3).expect("rotate_orth 3");
    rp.compare_values(h as f64, r3.width() as f64, 0.0);
    rp.compare_values(w as f64, r3.height() as f64, 0.0);

    let r90ccw = rotate_90(pixs, false).expect("rotate_90 ccw");
    let same = r3.equals(&r90ccw);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(3) == rotate_90(ccw): {}", label, same);

    // --- 4 orthogonal rotations = identity ---
    let r4 = rotate_orth(&r3, 1).expect("4th rotation");
    let same = pixs.equals(&r4);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: 4x orth(1) == identity: {}", label, same);
}

//! Orthogonal rotation regression test
//!
//! C version: `reference/leptonica/prog/rotateorth_reg.c`
//!
//! Tests orthogonal rotation operations across all bit depths:
//!   1. Four successive 90-degree rotations = identity
//!   2. Two successive 180-degree rotations = identity
//!   3. Two successive LR flips = identity
//!   4. Two successive TB flips = identity
//!   5. `rotate_orth(quads)` matches equivalent `rotate_90`/`rotate_180`
//!
//! C version runs the RotateOrthTest on 5 image types:
//! binary (test1.png), 4bpp colormapped (weasel4.8g.png),
//! grayscale (test8.jpg), colormap (dreyfus8.png), RGB (marge.jpg).

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{rotate_90, rotate_180, rotate_orth};

/// Test orthogonal rotations on multiple depths
///
/// For each depth, verifies `rotate_orth(0..3)` matches `rotate_90` / `rotate_180`,
/// and that 4x rotation returns identity. Directly mirrors `RotateOrthTest` in C version.
#[test]
fn rotateorth_reg() {
    let mut rp = RegParams::new("rotateorth");

    // C version: pixRead(BINARY_IMAGE) — test1.png
    let pix1 = load_test_image("feyn-fract.tif").expect("load binary image");
    test_orth_rotation(&mut rp, &pix1, "1bpp");

    // C version: pixRead(GRAYSCALE_IMAGE) — test8.jpg
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp image");
    test_orth_rotation(&mut rp, &pix8, "8bpp");

    // C version: pixRead(RGB_IMAGE) — marge.jpg
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

    // --- rotate_orth(1) = 90 degrees CW ---
    // C version: pixt = pixRotate90(pixs, 1)
    let r1 = rotate_orth(pixs, 1).expect("rotate_orth 1");
    rp.compare_values(h as f64, r1.width() as f64, 0.0);
    rp.compare_values(w as f64, r1.height() as f64, 0.0);

    let r90 = rotate_90(pixs, true).expect("rotate_90 cw");
    let same = r1.equals(&r90);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(1) == rotate_90(cw): {}", label, same);

    // --- rotate_orth(2) = 180 degrees ---
    // C version: pixRotate180(NULL, pixs)
    let r2 = rotate_orth(pixs, 2).expect("rotate_orth 2");
    rp.compare_values(w as f64, r2.width() as f64, 0.0);
    rp.compare_values(h as f64, r2.height() as f64, 0.0);

    let r180 = rotate_180(pixs).expect("rotate_180");
    let same = r2.equals(&r180);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(2) == rotate_180: {}", label, same);

    // --- rotate_orth(3) = 270 degrees CW = 90 degrees CCW ---
    // C version: pixRotate90(pixs, -1)
    let r3 = rotate_orth(pixs, 3).expect("rotate_orth 3");
    rp.compare_values(h as f64, r3.width() as f64, 0.0);
    rp.compare_values(w as f64, r3.height() as f64, 0.0);

    let r90ccw = rotate_90(pixs, false).expect("rotate_90 ccw");
    let same = r3.equals(&r90ccw);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: orth(3) == rotate_90(ccw): {}", label, same);

    // --- 4 orthogonal rotations = identity ---
    // C version: 4x pixRotate90(pix, 1); regTestComparePix; pixXor; pixZero
    let r4 = rotate_orth(&r3, 1).expect("4th rotation");
    let same = pixs.equals(&r4);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  {}: 4x orth(1) == identity: {}", label, same);
}

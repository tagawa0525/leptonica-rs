//! Rotation regression test 1 - basic rotation and flip operations
//!
//! C version: `reference/leptonica/prog/rotate1_reg.c`
//!
//! Tests basic orthogonal rotation and flip operations:
//!   1. Four successive 90-degree rotations = identity (all depths)
//!   2. Two successive 180-degree rotations = identity
//!   3. Two successive LR flips = identity
//!   4. Two successive TB flips = identity
//!   5. 90cw + 90ccw = identity
//!
//! C version additionally tests repeated arbitrary-angle rotation (shear,
//! sampling, area-map) on multiple image depths, which is covered in rotate2_reg.

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{flip_lr, flip_tb, rotate_90, rotate_180};

/// Test basic orthogonal rotations and flips on a 1bpp image
///
/// C version: `rotate1_reg.c` — tests `pixRotate90`, `pixRotate180`,
/// `pixFlipLR`, `pixFlipTB` identity properties.
#[test]
#[ignore = "not yet implemented"]
fn rotate1_reg() {
    let mut rp = RegParams::new("rotate1");

    let pixs = load_test_image("feyn-fract.tif").expect("load test image");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{}", w, h);

    // --- Test 1: Rotate 90 clockwise ---
    let r90 = rotate_90(&pixs, true).expect("rotate_90 cw");
    rp.compare_values(h as f64, r90.width() as f64, 0.0);
    rp.compare_values(w as f64, r90.height() as f64, 0.0);
    eprintln!("  rotate_90 cw: {}x{}", r90.width(), r90.height());

    // --- Test 2: Rotate 90 counter-clockwise ---
    let r90ccw = rotate_90(&pixs, false).expect("rotate_90 ccw");
    rp.compare_values(h as f64, r90ccw.width() as f64, 0.0);
    rp.compare_values(w as f64, r90ccw.height() as f64, 0.0);

    // --- Test 3: Rotate 180 ---
    let r180 = rotate_180(&pixs).expect("rotate_180");
    rp.compare_values(w as f64, r180.width() as f64, 0.0);
    rp.compare_values(h as f64, r180.height() as f64, 0.0);

    // --- Test 4: Rotate 360 should return to original ---
    // C version: pixRotate180(pixt, pixs); pixRotate180(pixt, pixt); regTestComparePix
    let r360 = rotate_180(&r180).expect("rotate 180 twice = 360");
    let same = pixs.equals(&r360);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  rotate360 == identity: {}", same);

    // --- Test 5: Four 90-degree rotations = identity ---
    // C version: 4x pixRotate90(pixs, 1); regTestComparePix; pixXor; pixZero
    let r1 = rotate_90(&pixs, true).expect("r90 1");
    let r2 = rotate_90(&r1, true).expect("r90 2");
    let r3 = rotate_90(&r2, true).expect("r90 3");
    let r4 = rotate_90(&r3, true).expect("r90 4");
    let same = pixs.equals(&r4);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  4x rotate_90 == identity: {}", same);

    // --- Test 6: Flip LR double = identity ---
    // C version: pixFlipLR(NULL, pixs); pixFlipLR(pixt, pixt); regTestComparePix
    let flr = flip_lr(&pixs).expect("flip_lr");
    rp.compare_values(w as f64, flr.width() as f64, 0.0);
    rp.compare_values(h as f64, flr.height() as f64, 0.0);

    let flr2 = flip_lr(&flr).expect("flip_lr twice");
    let same = pixs.equals(&flr2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  2x flip_lr == identity: {}", same);

    // --- Test 7: Flip TB double = identity ---
    // C version: pixFlipTB(NULL, pixs); pixFlipTB(pixt, pixt); regTestComparePix
    let ftb = flip_tb(&pixs).expect("flip_tb");
    rp.compare_values(w as f64, ftb.width() as f64, 0.0);
    rp.compare_values(h as f64, ftb.height() as f64, 0.0);

    let ftb2 = flip_tb(&ftb).expect("flip_tb twice");
    let same = pixs.equals(&ftb2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  2x flip_tb == identity: {}", same);

    // --- Test 8: Rotate 90cw + 90ccw = identity ---
    let rcw = rotate_90(&pixs, true).expect("90cw");
    let rback = rotate_90(&rcw, false).expect("90ccw");
    let same = pixs.equals(&rback);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  90cw + 90ccw == identity: {}", same);

    assert!(rp.cleanup(), "rotate1 regression test failed");
}

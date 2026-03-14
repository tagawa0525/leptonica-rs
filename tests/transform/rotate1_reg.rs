//! Rotation regression test 1 - basic rotation and flip operations
//!
//! C version: `prog/rotate1_reg.c`
//!
//! Tests basic orthogonal rotation and flip operations:
//!   1. Four successive 90-degree rotations = identity (all depths)
//!   2. Two successive 180-degree rotations = identity
//!   3. Two successive LR flips = identity
//!   4. Two successive TB flips = identity
//!   5. 90cw + 90ccw = identity
//!
//! Expanded in Phase 5 to add arbitrary-angle rotation tests across
//! four methods (Shear, Sampling, AreaMap, AMCorner) and multiple depths.
//!
//! C version: tests repeated arbitrary-angle rotation on multiple depths.

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::transform::{
    RotateFill, RotateMethod, RotateOptions, flip_lr, flip_tb, rotate, rotate_90, rotate_180,
    rotate_am_color_corner, rotate_am_corner,
};

/// Test basic orthogonal rotations and flips on a 1bpp image
///
/// C version: `rotate1_reg.c` — tests `pixRotate90`, `pixRotate180`,
/// `pixFlipLR`, `pixFlipTB` identity properties.
#[test]
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
    rp.write_pix_and_check(&r90, ImageFormat::Tiff)
        .expect("write r90");
    eprintln!("  rotate_90 cw: {}x{}", r90.width(), r90.height());

    // --- Test 2: Rotate 90 counter-clockwise ---
    let r90ccw = rotate_90(&pixs, false).expect("rotate_90 ccw");
    rp.compare_values(h as f64, r90ccw.width() as f64, 0.0);
    rp.compare_values(w as f64, r90ccw.height() as f64, 0.0);

    // --- Test 3: Rotate 180 ---
    let r180 = rotate_180(&pixs).expect("rotate_180");
    rp.compare_values(w as f64, r180.width() as f64, 0.0);
    rp.compare_values(h as f64, r180.height() as f64, 0.0);
    rp.write_pix_and_check(&r180, ImageFormat::Tiff)
        .expect("write r180");

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
    rp.write_pix_and_check(&flr, ImageFormat::Tiff)
        .expect("write flr");

    let flr2 = flip_lr(&flr).expect("flip_lr twice");
    let same = pixs.equals(&flr2);
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  2x flip_lr == identity: {}", same);

    // --- Test 7: Flip TB double = identity ---
    // C version: pixFlipTB(NULL, pixs); pixFlipTB(pixt, pixt); regTestComparePix
    let ftb = flip_tb(&pixs).expect("flip_tb");
    rp.compare_values(w as f64, ftb.width() as f64, 0.0);
    rp.compare_values(h as f64, ftb.height() as f64, 0.0);
    rp.write_pix_and_check(&ftb, ImageFormat::Tiff)
        .expect("write ftb");

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

/// Test arbitrary-angle rotation using Shear method across image depths.
///
/// C: pixRotate with L_ROTATE_SHEAR on 1bpp, 8bpp images.
/// Performs 8 successive rotations at pi/12 each.
#[test]
fn rotate1_reg_shear_method() {
    let mut rp = RegParams::new("rotate1_shear");

    let angle = std::f32::consts::PI / 12.0; // 15 degrees

    // 1bpp: shear rotation works well for binary images
    let pix1 = load_test_image("feyn-fract.tif").expect("load 1bpp");
    assert_eq!(pix1.depth(), PixelDepth::Bit1);
    let opts_shear = RotateOptions {
        method: RotateMethod::Shear,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        expand: false,
    };
    let mut cur = pix1.clone();
    for _ in 0..8 {
        cur = rotate(&cur, angle, &opts_shear).expect("shear rotate 1bpp");
    }
    rp.write_pix_and_check(&cur, ImageFormat::Tiff)
        .expect("write shear 1bpp result");

    // 8bpp grayscale
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp");
    assert_eq!(pix8.depth(), PixelDepth::Bit8);
    let mut cur8 = pix8.clone();
    for _ in 0..8 {
        cur8 = rotate(&cur8, angle, &opts_shear).expect("shear rotate 8bpp");
    }
    rp.write_pix_and_check(&cur8, ImageFormat::Png)
        .expect("write shear 8bpp result");

    // 32bpp RGB
    let pix32 = load_test_image("marge.jpg").expect("load 32bpp");
    let pix32 = if pix32.depth() != PixelDepth::Bit32 {
        pix32.convert_to_32().expect("convert to 32bpp")
    } else {
        pix32
    };
    let mut cur32 = pix32.clone();
    for _ in 0..8 {
        cur32 = rotate(&cur32, angle, &opts_shear).expect("shear rotate 32bpp");
    }
    rp.write_pix_and_check(&cur32, ImageFormat::Tiff)
        .expect("write shear 32bpp result");

    assert!(rp.cleanup(), "rotate1 shear_method test failed");
}

/// Test arbitrary-angle rotation using Sampling method across image depths.
///
/// C: pixRotate with L_ROTATE_SAMPLING on 1bpp, 8bpp, 32bpp images.
/// Performs 8 successive rotations at pi/12 each.
#[test]
fn rotate1_reg_sampling_method() {
    let mut rp = RegParams::new("rotate1_sampling");

    let angle = std::f32::consts::PI / 12.0;
    let opts = RotateOptions {
        method: RotateMethod::Sampling,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        expand: false,
    };

    // 1bpp
    let pix1 = load_test_image("feyn-fract.tif").expect("load 1bpp");
    let mut cur = pix1;
    for _ in 0..8 {
        cur = rotate(&cur, angle, &opts).expect("sampling rotate 1bpp");
    }
    rp.write_pix_and_check(&cur, ImageFormat::Tiff)
        .expect("write sampling 1bpp");

    // 8bpp grayscale
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp");
    let mut cur8 = pix8;
    for _ in 0..8 {
        cur8 = rotate(&cur8, angle, &opts).expect("sampling rotate 8bpp");
    }
    rp.write_pix_and_check(&cur8, ImageFormat::Png)
        .expect("write sampling 8bpp");

    // 32bpp RGB
    let pix32 = load_test_image("marge.jpg").expect("load 32bpp");
    let pix32 = if pix32.depth() != PixelDepth::Bit32 {
        pix32.convert_to_32().expect("convert to 32bpp")
    } else {
        pix32
    };
    let mut cur32 = pix32;
    for _ in 0..8 {
        cur32 = rotate(&cur32, angle, &opts).expect("sampling rotate 32bpp");
    }
    rp.write_pix_and_check(&cur32, ImageFormat::Tiff)
        .expect("write sampling 32bpp");

    assert!(rp.cleanup(), "rotate1 sampling_method test failed");
}

/// Test arbitrary-angle rotation using AreaMap method across image depths.
///
/// C: pixRotate with L_ROTATE_AREA_MAP on 8bpp, 32bpp images.
/// Performs 8 successive rotations at pi/12 each.
#[test]
fn rotate1_reg_areamap_method() {
    let mut rp = RegParams::new("rotate1_areamap");

    let angle = std::f32::consts::PI / 12.0;
    let opts = RotateOptions {
        method: RotateMethod::AreaMap,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        expand: false,
    };

    // 8bpp grayscale
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp");
    let mut cur8 = pix8;
    for _ in 0..8 {
        cur8 = rotate(&cur8, angle, &opts).expect("areamap rotate 8bpp");
    }
    rp.write_pix_and_check(&cur8, ImageFormat::Png)
        .expect("write areamap 8bpp");

    // 32bpp RGB
    let pix32 = load_test_image("marge.jpg").expect("load 32bpp");
    let pix32 = if pix32.depth() != PixelDepth::Bit32 {
        pix32.convert_to_32().expect("convert to 32bpp")
    } else {
        pix32
    };
    let mut cur32 = pix32;
    for _ in 0..8 {
        cur32 = rotate(&cur32, angle, &opts).expect("areamap rotate 32bpp");
    }
    rp.write_pix_and_check(&cur32, ImageFormat::Tiff)
        .expect("write areamap 32bpp");

    assert!(rp.cleanup(), "rotate1 areamap_method test failed");
}

/// Test rotate_am_corner and rotate_am_color_corner methods.
///
/// C: pixRotateAMCorner (8bpp), pixRotateAMColorFast (32bpp).
/// Performs 8 successive rotations at pi/12 each.
#[test]
fn rotate1_reg_am_corner() {
    let mut rp = RegParams::new("rotate1_amcorner");

    let angle = std::f32::consts::PI / 12.0;

    // 8bpp: rotate_am_corner
    let pix8 = load_test_image("dreyfus8.png").expect("load 8bpp");
    let mut cur8 = pix8;
    for _ in 0..8 {
        cur8 = rotate_am_corner(&cur8, angle, RotateFill::White).expect("am_corner 8bpp");
    }
    rp.write_pix_and_check(&cur8, ImageFormat::Png)
        .expect("write am_corner 8bpp");

    // 32bpp: rotate_am_color_corner (fast color rotation)
    let pix32 = load_test_image("marge.jpg").expect("load 32bpp");
    let pix32 = if pix32.depth() != PixelDepth::Bit32 {
        pix32.convert_to_32().expect("convert to 32bpp")
    } else {
        pix32
    };
    let mut cur32 = pix32;
    for _ in 0..8 {
        cur32 = rotate_am_color_corner(&cur32, angle, RotateFill::White)
            .expect("am_color_corner 32bpp");
    }
    rp.write_pix_and_check(&cur32, ImageFormat::Tiff)
        .expect("write am_color_corner 32bpp");

    assert!(rp.cleanup(), "rotate1 am_corner test failed");
}

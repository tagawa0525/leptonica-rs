//! Scale regression test
//!
//! C version: `reference/leptonica/prog/scale_reg.c`
//!
//! Tests various scaling operations:
//!   1. Scale up by 2x — dimensions double
//!   2. Scale down by 0.5x — dimensions halve
//!   3. Scale to specific target size
//!   4. Scale by sampling (nearest-neighbor)
//!   5. Scale by 1.0 preserves dimensions
//!   6. Anisotropic scaling (different x/y factors)
//!   7. Scale with different methods (Linear, Sampling)
//!   8. Scale on binary (1bpp) image
//!
//! C version tests `pixScale` on 10 images of varying depth/colormap,
//! and also tests `pixScaleToGray*`, `pixScaleSmoothToSize`, etc.

use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::{ScaleMethod, scale, scale_by_sampling, scale_to_size};

/// Test scaling operations on grayscale and binary images
///
/// C version: tests `pixScale` at factors [2.3, 1.5, 1.1, 0.6, 0.3] on each
/// of 10 image types (1bpp, 2bpp, 4bpp, 8bpp, 16bpp, 32bpp with/without cmap).
#[test]
#[ignore = "not yet implemented"]
fn scale_reg() {
    let mut rp = RegParams::new("scale");

    let pixs = load_test_image("dreyfus8.png").expect("load dreyfus8.png");
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{} d={}", w, h, pixs.depth().bits());

    // --- Test 1: Scale up 2x ---
    // C version: pixc = pixScale(pixs, 2.25, 2.25)
    let up2 = scale(&pixs, 2.0, 2.0, ScaleMethod::Linear).expect("scale 2x");
    rp.compare_values((w * 2) as f64, up2.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, up2.height() as f64, 1.0);
    eprintln!("  scale 2x: {}x{}", up2.width(), up2.height());

    // --- Test 2: Scale down 0.5x ---
    // C version: pixc = pixScale(pixs, 0.65, 0.65) etc.
    let down2 = scale(&pixs, 0.5, 0.5, ScaleMethod::Linear).expect("scale 0.5x");
    rp.compare_values((w / 2) as f64, down2.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, down2.height() as f64, 1.0);
    eprintln!("  scale 0.5x: {}x{}", down2.width(), down2.height());

    // --- Test 3: Scale to specific size ---
    let target_w = 200u32;
    let target_h = 150u32;
    let sized = scale_to_size(&pixs, target_w, target_h).expect("scale_to_size");
    rp.compare_values(target_w as f64, sized.width() as f64, 0.0);
    rp.compare_values(target_h as f64, sized.height() as f64, 0.0);
    eprintln!(
        "  scale_to_size(200,150): {}x{}",
        sized.width(),
        sized.height()
    );

    // --- Test 4: Scale by sampling ---
    // C version: pixScaleBySampling used internally for 1bpp upscaling
    let sampled = scale_by_sampling(&pixs, 2.0, 2.0).expect("scale_by_sampling 2x");
    rp.compare_values((w * 2) as f64, sampled.width() as f64, 1.0);
    rp.compare_values((h * 2) as f64, sampled.height() as f64, 1.0);

    // --- Test 5: Scale 1.0 should preserve dimensions ---
    let s1 = scale(&pixs, 1.0, 1.0, ScaleMethod::Linear).expect("scale 1x");
    rp.compare_values(w as f64, s1.width() as f64, 0.0);
    rp.compare_values(h as f64, s1.height() as f64, 0.0);

    // --- Test 6: Anisotropic scaling ---
    let aniso = scale(&pixs, 2.0, 0.5, ScaleMethod::Linear).expect("aniso scale");
    rp.compare_values((w * 2) as f64, aniso.width() as f64, 1.0);
    rp.compare_values((h / 2) as f64, aniso.height() as f64, 1.0);
    eprintln!(
        "  aniso scale(2.0, 0.5): {}x{}",
        aniso.width(),
        aniso.height()
    );

    // --- Test 7: Scale with different methods ---
    for method in [ScaleMethod::Linear, ScaleMethod::Sampling] {
        let s = scale(&pixs, 1.5, 1.5, method).expect("scale method");
        rp.compare_values(
            1.0,
            if s.width() > 0 && s.height() > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!("  scale {:?} 1.5x: {}x{}", method, s.width(), s.height());
    }

    // --- Test 8: Scale with binary image ---
    // C version: pixs = pixRead("feyn-fract.tif"); pixc = pixScale(pixs, 0.32, 0.32)
    let pixb = load_test_image("feyn-fract.tif").expect("load binary");
    let sb = scale(&pixb, 2.0, 2.0, ScaleMethod::Sampling).expect("scale binary");
    rp.compare_values((pixb.width() * 2) as f64, sb.width() as f64, 1.0);
    rp.compare_values((pixb.height() * 2) as f64, sb.height() as f64, 1.0);

    assert!(rp.cleanup(), "scale regression test failed");
}

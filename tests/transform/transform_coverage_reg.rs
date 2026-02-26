//! Coverage tests for transform module functions
//!
//! Tests for 16 new transform functions:
//! - pixEmbedForRotation, pixRotateBinaryNice (rotate.rs)
//! - pixScaleToSizeRel, pixScaleBySamplingToSize, pixScaleSmoothToSize,
//!   pixScaleAreaMap2, pixScaleAreaMapToSize, pixScaleBinaryWithShift,
//!   pixScaleGrayMinMax2, pixScaleGrayRank2, pixScaleWithAlpha (scale.rs)
//! - ptaTranslate, ptaScale, boxaTranslate, boxaScale, boxaRotate (affinecompose via core)

use crate::common::load_test_image;
use leptonica::core::box_::Boxa;
use leptonica::core::pta::Pta;
use leptonica::core::{Pix, PixelDepth};
use leptonica::transform::{GrayMinMaxMode, RotateFill};

// ============================================================================
// rotate.rs tests
// ============================================================================

/// Test embed_for_rotation: embed a small image in a larger one for rotation
#[test]
fn test_embed_for_rotation_basic() {
    use leptonica::transform::rotate::embed_for_rotation;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();
    let angle: f32 = 0.3; // ~17 degrees

    let embedded = embed_for_rotation(&pix, angle, RotateFill::White).expect("embed_for_rotation");
    // Embedded image should be larger than original to fit rotation
    assert!(embedded.width() >= w);
    assert!(embedded.height() >= h);
    assert_eq!(embedded.depth(), pix.depth());
}

/// Test embed_for_rotation: tiny angle returns clone
#[test]
fn test_embed_for_rotation_tiny_angle() {
    use leptonica::transform::rotate::embed_for_rotation;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let embedded = embed_for_rotation(&pix, 0.0001, RotateFill::White).expect("tiny angle");
    // Tiny angle should return same dimensions
    assert_eq!(embedded.width(), pix.width());
    assert_eq!(embedded.height(), pix.height());
}

/// Test rotate_binary_nice: rotate a 1bpp image with anti-aliased edges
#[test]
fn test_rotate_binary_nice() {
    use leptonica::transform::rotate::rotate_binary_nice;
    let pix = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let angle: f32 = 0.1; // ~5.7 degrees
    let rotated = rotate_binary_nice(&pix, angle, RotateFill::White).expect("rotate_binary_nice");
    // Result should be 1bpp
    assert_eq!(rotated.depth(), PixelDepth::Bit1);
    // Dimensions should be similar (not expanded)
    assert!(rotated.width() > 0);
    assert!(rotated.height() > 0);
}

/// Test rotate_binary_nice: validates 1bpp input
#[test]
fn test_rotate_binary_nice_rejects_non_binary() {
    use leptonica::transform::rotate::rotate_binary_nice;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    assert_ne!(pix.depth(), PixelDepth::Bit1);
    assert!(rotate_binary_nice(&pix, 0.1, RotateFill::White).is_err());
}

// ============================================================================
// scale.rs tests
// ============================================================================

/// Test scale_to_size_rel: relative size specification
#[test]
fn test_scale_to_size_rel_basic() {
    use leptonica::transform::scale::scale_to_size_rel;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    // Add 50 to width, 30 to height
    let scaled = scale_to_size_rel(&pix, 50, 30).expect("scale_to_size_rel");
    assert_eq!(scaled.width(), w + 50);
    assert_eq!(scaled.height(), h + 30);
}

/// Test scale_to_size_rel: zero deltas return copy
#[test]
fn test_scale_to_size_rel_zero_delta() {
    use leptonica::transform::scale::scale_to_size_rel;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let scaled = scale_to_size_rel(&pix, 0, 0).expect("zero delta");
    assert_eq!(scaled.width(), pix.width());
    assert_eq!(scaled.height(), pix.height());
}

/// Test scale_to_size_rel: negative delta reduces size
#[test]
fn test_scale_to_size_rel_negative() {
    use leptonica::transform::scale::scale_to_size_rel;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();
    let dw = -(w as i32 / 2);
    let dh = -(h as i32 / 2);
    let scaled = scale_to_size_rel(&pix, dw, dh).expect("negative delta");
    assert_eq!(scaled.width(), (w as i32 + dw) as u32);
    assert_eq!(scaled.height(), (h as i32 + dh) as u32);
}

/// Test scale_to_size_rel: delta reducing to zero returns error
#[test]
fn test_scale_to_size_rel_zero_result() {
    use leptonica::transform::scale::scale_to_size_rel;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let result = scale_to_size_rel(&pix, -(pix.width() as i32), 0);
    assert!(result.is_err());
}

/// Test scale_by_sampling_to_size: target width only
#[test]
fn test_scale_by_sampling_to_size_width_only() {
    use leptonica::transform::scale::scale_by_sampling_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();
    let target_w = 200u32;

    let scaled = scale_by_sampling_to_size(&pix, target_w, 0).expect("width only");
    assert_eq!(scaled.width(), target_w);
    // Aspect ratio preserved
    let expected_h = ((target_w as f32 / w as f32) * h as f32).round() as u32;
    let diff = (scaled.height() as i32 - expected_h as i32).unsigned_abs();
    assert!(
        diff <= 1,
        "height should be ~{}, got {}",
        expected_h,
        scaled.height()
    );
}

/// Test scale_by_sampling_to_size: target height only
#[test]
fn test_scale_by_sampling_to_size_height_only() {
    use leptonica::transform::scale::scale_by_sampling_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let target_h = 100u32;

    let scaled = scale_by_sampling_to_size(&pix, 0, target_h).expect("height only");
    assert_eq!(scaled.height(), target_h);
}

/// Test scale_by_sampling_to_size: both zero returns error
#[test]
fn test_scale_by_sampling_to_size_both_zero() {
    use leptonica::transform::scale::scale_by_sampling_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    assert!(scale_by_sampling_to_size(&pix, 0, 0).is_err());
}

/// Test scale_smooth_to_size: target size with smooth downscaling
#[test]
fn test_scale_smooth_to_size_basic() {
    use leptonica::transform::scale::scale_smooth_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let target_w = 100u32;
    let target_h = 80u32;

    let scaled = scale_smooth_to_size(&pix, target_w, target_h).expect("smooth to size");
    assert_eq!(scaled.width(), target_w);
    assert_eq!(scaled.height(), target_h);
}

/// Test scale_smooth_to_size: width only
#[test]
fn test_scale_smooth_to_size_width_only() {
    use leptonica::transform::scale::scale_smooth_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let scaled = scale_smooth_to_size(&pix, 150, 0).expect("smooth to width");
    assert_eq!(scaled.width(), 150);
}

/// Test scale_area_map_2: fast 2x downscale
#[test]
fn test_scale_area_map_2_gray() {
    use leptonica::transform::scale::scale_area_map_2;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let w = pix.width();
    let h = pix.height();

    let scaled = scale_area_map_2(&pix).expect("area_map_2");
    assert_eq!(scaled.width(), w / 2);
    assert_eq!(scaled.height(), h / 2);
    assert_eq!(scaled.depth(), pix.depth());
}

/// Test scale_area_map_2: 32bpp color
#[test]
fn test_scale_area_map_2_color() {
    use leptonica::transform::scale::scale_area_map_2;
    let pix = load_test_image("test24.jpg").expect("load test24.jpg");
    let depth = pix.depth();
    let w = pix.width();
    let h = pix.height();

    let scaled = scale_area_map_2(&pix).expect("area_map_2 color");
    assert_eq!(scaled.width(), w / 2);
    assert_eq!(scaled.height(), h / 2);
    assert_eq!(scaled.depth(), depth);
}

/// Test scale_area_map_2: rejects 1bpp
#[test]
fn test_scale_area_map_2_rejects_1bpp() {
    use leptonica::transform::scale::scale_area_map_2;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    assert!(scale_area_map_2(&pix).is_err());
}

/// Test scale_area_map_to_size: target dimensions
#[test]
fn test_scale_area_map_to_size_basic() {
    use leptonica::transform::scale::scale_area_map_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let scaled = scale_area_map_to_size(&pix, 100, 80).expect("area_map_to_size");
    assert_eq!(scaled.width(), 100);
    assert_eq!(scaled.height(), 80);
}

/// Test scale_area_map_to_size: height only (aspect ratio)
#[test]
fn test_scale_area_map_to_size_height_only() {
    use leptonica::transform::scale::scale_area_map_to_size;
    let pix = load_test_image("test8.jpg").expect("load test8.jpg");
    let scaled = scale_area_map_to_size(&pix, 0, 100).expect("area_map_to_size height");
    assert_eq!(scaled.height(), 100);
}

/// Test scale_binary_with_shift: scale 1bpp with sub-pixel shift
#[test]
fn test_scale_binary_with_shift() {
    use leptonica::transform::scale::scale_binary_with_shift;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    assert_eq!(pix.depth(), PixelDepth::Bit1);
    let w = pix.width();
    let h = pix.height();

    // Scale down with shift=0.5
    let scaled = scale_binary_with_shift(&pix, 0.5, 0.5, 0.5, 0.5).expect("binary_with_shift");
    let expected_w = (0.5 * w as f32 + 0.5) as u32;
    let expected_h = (0.5 * h as f32 + 0.5) as u32;
    assert_eq!(scaled.width(), expected_w);
    assert_eq!(scaled.height(), expected_h);
    assert_eq!(scaled.depth(), PixelDepth::Bit1);
}

/// Test scale_binary_with_shift: shift=0.0 also works
#[test]
fn test_scale_binary_with_shift_zero() {
    use leptonica::transform::scale::scale_binary_with_shift;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    let scaled = scale_binary_with_shift(&pix, 2.0, 2.0, 0.0, 0.0).expect("binary_with_shift 2x");
    assert_eq!(scaled.depth(), PixelDepth::Bit1);
    assert!(scaled.width() > pix.width());
}

/// Test scale_binary_with_shift: invalid shift rejected
#[test]
fn test_scale_binary_with_shift_invalid_shift() {
    use leptonica::transform::scale::scale_binary_with_shift;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    assert!(scale_binary_with_shift(&pix, 1.0, 1.0, 0.3, 0.0).is_err());
}

/// Test scale_binary_with_shift: identity returns copy
#[test]
fn test_scale_binary_with_shift_identity() {
    use leptonica::transform::scale::scale_binary_with_shift;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    let scaled = scale_binary_with_shift(&pix, 1.0, 1.0, 0.0, 0.0).expect("identity");
    assert_eq!(scaled.width(), pix.width());
    assert_eq!(scaled.height(), pix.height());
}

/// Test scale_binary_with_shift: rejects non-1bpp
#[test]
fn test_scale_binary_with_shift_rejects_non_binary() {
    use leptonica::transform::scale::scale_binary_with_shift;
    let pix = load_test_image("test8.jpg").expect("load gray");
    assert!(scale_binary_with_shift(&pix, 1.0, 1.0, 0.0, 0.0).is_err());
}

/// Test scale_gray_min_max_2: min mode
#[test]
fn test_scale_gray_min_max_2_min() {
    use leptonica::transform::scale::scale_gray_min_max_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let w = pix.width();
    let h = pix.height();

    let scaled = scale_gray_min_max_2(&pix, GrayMinMaxMode::Min).expect("min_max_2 min");
    assert_eq!(scaled.width(), w / 2);
    assert_eq!(scaled.height(), h / 2);
    assert_eq!(scaled.depth(), PixelDepth::Bit8);
}

/// Test scale_gray_min_max_2: max mode
#[test]
fn test_scale_gray_min_max_2_max() {
    use leptonica::transform::scale::scale_gray_min_max_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let scaled = scale_gray_min_max_2(&pix, GrayMinMaxMode::Max).expect("min_max_2 max");
    assert_eq!(scaled.depth(), PixelDepth::Bit8);
    assert_eq!(scaled.width(), pix.width() / 2);
}

/// Test scale_gray_min_max_2: maxdiff mode
#[test]
fn test_scale_gray_min_max_2_maxdiff() {
    use leptonica::transform::scale::scale_gray_min_max_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let scaled = scale_gray_min_max_2(&pix, GrayMinMaxMode::MaxDiff).expect("min_max_2 maxdiff");
    assert_eq!(scaled.depth(), PixelDepth::Bit8);
}

/// Test scale_gray_min_max_2: rejects non-8bpp
#[test]
fn test_scale_gray_min_max_2_rejects_non_gray() {
    use leptonica::transform::scale::scale_gray_min_max_2;
    let pix = load_test_image("feyn-fract.tif").expect("load binary");
    assert!(scale_gray_min_max_2(&pix, GrayMinMaxMode::Min).is_err());
}

/// Test scale_gray_rank_2: rank 1 (darkest)
#[test]
fn test_scale_gray_rank_2_rank1() {
    use leptonica::transform::scale::scale_gray_rank_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let w = pix.width();
    let h = pix.height();

    let scaled = scale_gray_rank_2(&pix, 1).expect("rank_2 rank1");
    assert_eq!(scaled.width(), w / 2);
    assert_eq!(scaled.height(), h / 2);
    assert_eq!(scaled.depth(), PixelDepth::Bit8);
}

/// Test scale_gray_rank_2: rank 2 and 3 (middle ranks)
#[test]
fn test_scale_gray_rank_2_middle() {
    use leptonica::transform::scale::scale_gray_rank_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let r2 = scale_gray_rank_2(&pix, 2).expect("rank 2");
    let r3 = scale_gray_rank_2(&pix, 3).expect("rank 3");
    assert_eq!(r2.width(), pix.width() / 2);
    assert_eq!(r3.width(), pix.width() / 2);
}

/// Test scale_gray_rank_2: rank 4 (lightest)
#[test]
fn test_scale_gray_rank_2_rank4() {
    use leptonica::transform::scale::scale_gray_rank_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    let scaled = scale_gray_rank_2(&pix, 4).expect("rank 4");
    assert_eq!(scaled.depth(), PixelDepth::Bit8);
}

/// Test scale_gray_rank_2: invalid rank
#[test]
fn test_scale_gray_rank_2_invalid_rank() {
    use leptonica::transform::scale::scale_gray_rank_2;
    let pix = load_test_image("test8.jpg").expect("load gray");
    assert!(scale_gray_rank_2(&pix, 0).is_err());
    assert!(scale_gray_rank_2(&pix, 5).is_err());
}

/// Test scale_with_alpha: scale a 32bpp image preserving alpha
#[test]
fn test_scale_with_alpha_basic() {
    use leptonica::transform::scale::scale_with_alpha;
    // Create a 32bpp image
    let pix = load_test_image("test24.jpg").expect("load color");
    let w = pix.width();
    let h = pix.height();

    let scaled = scale_with_alpha(&pix, 0.5, 0.5, None, 1.0).expect("scale_with_alpha");
    let expected_w = (w as f32 * 0.5).round() as u32;
    let expected_h = (h as f32 * 0.5).round() as u32;
    let diff_w = (scaled.width() as i32 - expected_w as i32).unsigned_abs();
    let diff_h = (scaled.height() as i32 - expected_h as i32).unsigned_abs();
    assert!(diff_w <= 1);
    assert!(diff_h <= 1);
    assert_eq!(scaled.depth(), PixelDepth::Bit32);
}

/// Test scale_with_alpha: with explicit alpha mask
#[test]
fn test_scale_with_alpha_with_mask() {
    use leptonica::transform::scale::scale_with_alpha;
    let pix = load_test_image("test24.jpg").expect("load color");
    let w = pix.width();
    let h = pix.height();

    // Create an 8bpp alpha mask
    let alpha_pix = Pix::new(w, h, PixelDepth::Bit8).expect("create alpha");
    let mut alpha_mut = alpha_pix.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            alpha_mut.set_pixel_unchecked(x, y, 200);
        }
    }
    let alpha: Pix = alpha_mut.into();

    let scaled = scale_with_alpha(&pix, 2.0, 2.0, Some(&alpha), 1.0).expect("with alpha mask");
    assert_eq!(scaled.depth(), PixelDepth::Bit32);
}

/// Test scale_with_alpha: fractional alpha
#[test]
fn test_scale_with_alpha_fractional() {
    use leptonica::transform::scale::scale_with_alpha;
    let pix = load_test_image("test24.jpg").expect("load color");
    let scaled = scale_with_alpha(&pix, 0.5, 0.5, None, 0.5).expect("fractional alpha");
    assert_eq!(scaled.depth(), PixelDepth::Bit32);
}

// ============================================================================
// Pta transform tests (ptaTranslate, ptaScale via core)
// ============================================================================

/// Test Pta translated_by (ptaTranslate)
#[test]
fn test_pta_translate() {
    use leptonica::transform::pta_translate;
    let mut pta = Pta::new();
    pta.push(10.0, 20.0);
    pta.push(30.0, 40.0);

    let result = pta_translate(&pta, 5.0, -3.0);
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0), Some((15.0, 17.0)));
    assert_eq!(result.get(1), Some((35.0, 37.0)));
}

/// Test Pta translated_by with empty pta
#[test]
fn test_pta_translate_empty() {
    use leptonica::transform::pta_translate;
    let pta = Pta::new();
    let result = pta_translate(&pta, 5.0, 3.0);
    assert_eq!(result.len(), 0);
}

/// Test Pta scaled_by (ptaScale)
#[test]
fn test_pta_scale() {
    use leptonica::transform::pta_scale;
    let mut pta = Pta::new();
    pta.push(10.0, 20.0);
    pta.push(30.0, 40.0);

    let result = pta_scale(&pta, 2.0, 0.5);
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0), Some((20.0, 10.0)));
    assert_eq!(result.get(1), Some((60.0, 20.0)));
}

// ============================================================================
// Boxa transform tests (boxaTranslate, boxaScale, boxaRotate via core)
// ============================================================================

/// Test Boxa translate (boxaTranslate)
#[test]
fn test_boxa_translate() {
    use leptonica::core::box_::Box as LBox;
    use leptonica::transform::boxa_translate;
    let mut boxa = Boxa::new();
    boxa.push(LBox::new(10, 20, 30, 40).unwrap());
    boxa.push(LBox::new(50, 60, 70, 80).unwrap());

    let result = boxa_translate(&boxa, 5.0, -10.0);
    assert_eq!(result.len(), 2);
    let b0 = result.get(0).unwrap();
    assert_eq!(b0.x, 15);
    assert_eq!(b0.y, 10);
    assert_eq!(b0.w, 30);
    assert_eq!(b0.h, 40);
}

/// Test Boxa scale (boxaScale)
#[test]
fn test_boxa_scale() {
    use leptonica::core::box_::Box as LBox;
    use leptonica::transform::boxa_scale;
    let mut boxa = Boxa::new();
    boxa.push(LBox::new(10, 20, 30, 40).unwrap());

    let result = boxa_scale(&boxa, 2.0, 0.5);
    assert_eq!(result.len(), 1);
    let b0 = result.get(0).unwrap();
    assert_eq!(b0.x, 20);
    assert_eq!(b0.y, 10);
    assert_eq!(b0.w, 60);
    assert_eq!(b0.h, 20);
}

/// Test Boxa rotate (boxaRotate)
#[test]
fn test_boxa_rotate() {
    use leptonica::core::box_::Box as LBox;
    use leptonica::transform::boxa_rotate;
    let mut boxa = Boxa::new();
    boxa.push(LBox::new(10, 0, 10, 10).unwrap());

    // Rotate 90 degrees around origin
    let angle = std::f32::consts::FRAC_PI_2;
    let result = boxa_rotate(&boxa, 0.0, 0.0, angle);
    assert_eq!(result.len(), 1);
    let b0 = result.get(0).unwrap();
    // After 90° rotation of box (10,0)-(20,10), corners map to roughly (0,10)-(10,20)
    // The bounding box should capture the rotated rectangle
    assert!(b0.w > 0);
    assert!(b0.h > 0);
}

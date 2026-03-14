//! Crop regression test
//!
//! Tests pixel profile plotting and rectangle clipping with borders.
//! The C version uses pixReversalProfile, pixAverageIntensityProfile,
//! numaOpen, numaLowPassIntervals, and numaThresholdEdges for automatic
//! crop boundary detection.
//!
//! Partial migration: reversal profile and Numa edge operations are not
//! available. Tests clip_rectangle_with_border which is the direct
//! cropping primitive.
//!
//! # See also
//!
//! C Leptonica: `prog/crop_reg.c`

use crate::common::RegParams;
use leptonica::Box as LeptBox;
use leptonica::io::ImageFormat;

/// Test clip_rectangle_with_border fully contained (C check 6).
///
/// Clips a rectangle that is fully within the image with an added border.
#[test]
fn crop_reg_clip_with_border_contained() {
    let mut rp = RegParams::new("crop_border_contained");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix8 = pix.convert_to_8().expect("convert_to_8");

    // Create a box fully within the image
    let bx = LeptBox::new(125, 50, 180, 230).expect("create box");
    let (clipped, result_box) = pix8
        .clip_rectangle_with_border(&bx, 30)
        .expect("clip_with_border");

    // Result should be wider/taller than the requested box by up to 2*border
    rp.compare_values(1.0, if clipped.width() >= 180 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if clipped.height() >= 230 { 1.0 } else { 0.0 }, 0.0);
    rp.write_pix_and_check(&clipped, ImageFormat::Png)
        .expect("write clipped crop_border_contained");

    // Result box should indicate the location of the original box within the clipped image
    rp.compare_values(1.0, if result_box.w > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if result_box.h > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "crop clip with border contained test failed");
}

/// Test clip_rectangle_with_border at image edge (C check 7).
///
/// Clips a rectangle that extends near the edge, so full border is not possible.
#[test]
fn crop_reg_clip_with_border_edge() {
    let mut rp = RegParams::new("crop_border_edge");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix8 = pix.convert_to_8().expect("convert_to_8");

    // Box near top edge: border will be clipped
    let bx = LeptBox::new(125, 10, 180, 270).expect("create box");
    let (clipped, _result_box) = pix8
        .clip_rectangle_with_border(&bx, 30)
        .expect("clip_with_border edge");

    // Should still produce valid output
    rp.compare_values(1.0, if clipped.width() >= 180 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if clipped.height() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "crop clip with border edge test failed");
}

/// Test basic clip_rectangle for simple cropping.
///
/// Additional test beyond the C checks, verifying pixel correspondence.
#[test]
fn crop_reg_basic_clip() {
    let mut rp = RegParams::new("crop_basic_clip");

    let pix = crate::common::load_test_image("karen8.jpg").expect("load karen8.jpg");

    // Clip a centered region
    let cw = 100;
    let ch = 80;
    let cx = pix.width() / 4;
    let cy = pix.height() / 4;
    let clipped = pix.clip_rectangle(cx, cy, cw, ch).expect("clip_rectangle");
    rp.compare_values(cw as f64, clipped.width() as f64, 0.0);
    rp.compare_values(ch as f64, clipped.height() as f64, 0.0);
    rp.write_pix_and_check(&clipped, ImageFormat::Png)
        .expect("write clipped crop_basic_clip");

    // Pixel at (0,0) of clip should match (cx, cy) of original
    let p_orig = pix.get_pixel(cx, cy).expect("get_pixel original");
    let p_clip = clipped.get_pixel(0, 0).expect("get_pixel clipped");
    rp.compare_values(p_orig as f64, p_clip as f64, 0.0);

    assert!(rp.cleanup(), "crop basic clip test failed");
}

/// Test reversal profile and intensity profile (C checks 0-5).
///
/// Requires pixReversalProfile, numaOpen, numaLowPassIntervals,
/// numaThresholdEdges, and test images lyra.005.jpg, lyra.036.jpg, 1555.007.jpg.
#[test]
#[ignore = "not yet implemented: pixReversalProfile and Numa edge detection not available"]
fn crop_reg_profile_analysis() {
    // C version:
    // 1. pixReversalProfile() for edge counting in vertical/horizontal scans
    // 2. numaOpen() for morphological smoothing of profiles
    // 3. numaLowPassIntervals() for finding low-activity regions
    // 4. numaThresholdEdges() for detecting transitions
    // 5. Compute left/right crop boundaries from profiles
}

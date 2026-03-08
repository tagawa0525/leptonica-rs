//! Speckle removal regression test
//!
//! Tests region-based speckle removal using connected component analysis
//! and morphological operations.
//!
//! Expanded in Phase 5 to implement the full speckle removal pipeline:
//! background_norm_flex → gamma_trc_masked → threshold → HMT → dilate → subtract
//!
//! C version: `reference/leptonica/prog/speckle_reg.c`
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/speckle_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::color::threshold_to_binary;
use leptonica::filter::{FlexNormOptions, background_norm_flex, gamma_trc_masked};
use leptonica::io::ImageFormat;
use leptonica::morph::{Sel, dilate, hit_miss_transform};
use leptonica::region::{
    ConnectivityType, SizeSelectRelation, SizeSelectType, clear_border, find_connected_components,
    pix_count_components, pix_select_by_size,
};

/// Test clear_border on a binary image (border noise removal).
///
/// Clears all foreground pixels connected to the border.
#[test]
fn speckle_reg_clear_border() {
    let mut rp = RegParams::new("speckle_border");

    // Use feyn.tif as a binary test image (already 1bpp)
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let region = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    let w = region.width();
    let h = region.height();

    // clear_border removes foreground pixels connected to the border
    let cleared = clear_border(&region, ConnectivityType::FourWay).expect("clear_border 4-way");
    rp.compare_values(w as f64, cleared.width() as f64, 0.0);
    rp.compare_values(h as f64, cleared.height() as f64, 0.0);
    assert_eq!(cleared.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&cleared, ImageFormat::Tiff)
        .expect("write cleared speckle_border");

    // 8-way connectivity clears more border pixels
    let cleared8 = clear_border(&region, ConnectivityType::EightWay).expect("clear_border 8-way");
    rp.compare_values(w as f64, cleared8.width() as f64, 0.0);

    assert!(rp.cleanup(), "speckle clear_border test failed");
}

/// Test pix_count_components and find_connected_components on a binary image.
///
/// Counts and finds connected components in feyn.tif (already 1bpp).
#[test]
fn speckle_reg_count_components() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("speckle_count");

    // feyn.tif is already 1bpp — no threshold needed
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let binary = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    assert_eq!(binary.depth(), PixelDepth::Bit1);

    // Count connected components
    let count =
        pix_count_components(&binary, ConnectivityType::EightWay).expect("pix_count_components");
    rp.compare_values(1.0, if count > 0 { 1.0 } else { 0.0 }, 0.0);

    // Find connected components with bounding boxes
    let components = find_connected_components(&binary, ConnectivityType::EightWay)
        .expect("find_connected_components");
    rp.compare_values(count as f64, components.len() as f64, 0.0);

    assert!(rp.cleanup(), "speckle count_components test failed");
}

/// Test pix_select_by_size to remove small speckle components (C: speckle removal).
///
/// Removes connected components smaller than a given size threshold to
/// simulate speckle noise removal.
#[test]
fn speckle_reg_select_by_size() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("speckle_size");

    // feyn.tif is already 1bpp
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let binary = pix.clip_rectangle(383, 338, 400, 300).expect("clip region");
    let w = binary.width();
    let h = binary.height();

    // Count components before filtering
    let count_before =
        pix_count_components(&binary, ConnectivityType::FourWay).expect("count before");

    // Remove components smaller than 3x3 (speckle removal)
    // C: sel1 removes components up to 2x2; we use size threshold 3x3
    let filtered = pix_select_by_size(
        &binary,
        3,
        3,
        ConnectivityType::FourWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size");
    rp.compare_values(w as f64, filtered.width() as f64, 0.0);
    rp.compare_values(h as f64, filtered.height() as f64, 0.0);
    assert_eq!(filtered.depth(), PixelDepth::Bit1);
    rp.write_pix_and_check(&filtered, ImageFormat::Tiff)
        .expect("write filtered speckle_size");

    // After filtering, there should be fewer or equal components
    let count_after =
        pix_count_components(&filtered, ConnectivityType::FourWay).expect("count after");
    rp.compare_values(
        1.0,
        if count_after <= count_before {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "speckle select_by_size test failed");
}

/// Test full speckle removal pipeline.
///
/// C: pixBackgroundNormFlex → pixGammaTRCMasked → pixThresholdToBinary
///    → pixHMT(sel_speckle2) → pixDilate(2x2) → pixSubtract (2x2 speckle removal)
///    → pixHMT(sel_speckle3) → pixDilate(3x3) → pixSubtract (3x3 speckle removal)
///
/// Uses w91frag.jpg (grayscale document fragment with speckle noise).
#[test]
fn speckle_reg_full_pipeline() {
    let mut rp = RegParams::new("speckle_pipeline");

    // Load grayscale document image with speckle noise
    let pixs = crate::common::load_test_image("w91frag.jpg").expect("load w91frag.jpg");
    let pixs = if pixs.depth() != PixelDepth::Bit8 {
        pixs.convert_to_8().expect("convert to 8bpp")
    } else {
        pixs
    };
    rp.write_pix_and_check(&pixs, ImageFormat::Jpeg)
        .expect("write original w91frag");

    // Step 1: Normalize background (C: pixBackgroundNormFlex with tile=7, smooth=1)
    let opts = FlexNormOptions {
        tile_width: 7,
        tile_height: 7,
        smooth_x: 1,
        smooth_y: 1,
        delta: 0, // delta=10 not yet supported in Rust; use 0
    };
    let pix1 = background_norm_flex(&pixs, &opts).expect("background_norm_flex");
    rp.write_pix_and_check(&pix1, ImageFormat::Jpeg)
        .expect("write background normalized");

    // Step 2: Remove background and enhance contrast (C: pixGammaTRCMasked with 1.0, 100, 175)
    let pix2 = gamma_trc_masked(&pix1, None, 1.0, 100, 175).expect("gamma_trc_masked");
    rp.write_pix_and_check(&pix2, ImageFormat::Jpeg)
        .expect("write gamma corrected");

    // Step 3: Binarize (C: pixThresholdToBinary(pix2, 180))
    let pix3 = threshold_to_binary(&pix2, 180).expect("threshold_to_binary");
    rp.write_pix_and_check(&pix3, ImageFormat::Png)
        .expect("write binarized");

    // Step 4: Remove 2x2 speckle noise via HMT + dilate + subtract
    // SEL: 4x4 with inner 2x2 DON'T CARE surrounded by MISS
    // C selstr2: "oooo" / "oC o" / "o  o" / "oooo" (4x4, origin at (1,1))
    let sel1 = Sel::from_string("oooo\no  o\no  o\noooo", 1, 1).expect("speckle2 SEL");
    let pix4 = hit_miss_transform(&pix3, &sel1).expect("HMT speckle2");

    // Dilate the detected speckle positions with a 2x2 brick
    let sel2 = Sel::create_brick(2, 2).expect("2x2 brick");
    let pix5 = dilate(&pix4, &sel2).expect("dilate speckle2");
    rp.write_pix_and_check(&pix5, ImageFormat::Png)
        .expect("write speckle2 dilated");

    // Subtract speckle mask from binarized image
    let pix6 = pix3.subtract(&pix5).expect("subtract speckle2");
    rp.write_pix_and_check(&pix6, ImageFormat::Png)
        .expect("write speckle2 removed");

    // Step 5: Remove 3x3 speckle noise via HMT + dilate + subtract
    // SEL: 5x5 with inner 3x3 DON'T CARE surrounded by MISS
    // C selstr3: "ooooo" / "oC  o" / "o   o" / "o   o" / "ooooo" (5x5, origin at (1,1))
    let sel3 = Sel::from_string("ooooo\no   o\no   o\no   o\nooooo", 1, 1).expect("speckle3 SEL");
    let pix7 = hit_miss_transform(&pix3, &sel3).expect("HMT speckle3");

    let sel4 = Sel::create_brick(3, 3).expect("3x3 brick");
    let pix8 = dilate(&pix7, &sel4).expect("dilate speckle3");
    rp.write_pix_and_check(&pix8, ImageFormat::Png)
        .expect("write speckle3 dilated");

    let pix9 = pix3.subtract(&pix8).expect("subtract speckle3");
    rp.write_pix_and_check(&pix9, ImageFormat::Png)
        .expect("write speckle3 removed");

    assert!(rp.cleanup(), "speckle full_pipeline test failed");
}

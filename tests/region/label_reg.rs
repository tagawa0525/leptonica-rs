//! Labeling regression test
//!
//! C reference: prog/label_reg.c
//!
//! Verifies:
//! 1. 4-connected and 8-connected labeling produce correct dimensions
//! 2. Component counting works for both connectivity types
//! 3. 4-connected count >= 8-connected count
//! 4. Component properties (area, bounds) are valid
//! 5. feyn-fract.tif has a large number of components

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::core::pix::{Convert16To8Type, Convert32To16Type};
use leptonica::io::ImageFormat;
use leptonica::region::{
    ConnectivityType, component_area_transform, find_connected_components,
    label_connected_components, pix_get_sorted_neighbor_values, pix_loc_to_color_transform,
};

#[test]
fn label_reg() {
    let mut rp = RegParams::new("label");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let w = pixs.width();
    let h = pixs.height();
    if crate::common::is_display_mode() {
        let labeled8 =
            label_connected_components(&pixs, ConnectivityType::EightWay).expect("label 8");
        rp.compare_values(w as f64, labeled8.width() as f64, 0.0);
        rp.compare_values(h as f64, labeled8.height() as f64, 0.0);
        assert!(rp.cleanup(), "label regression test failed");
        return;
    }

    eprintln!("Image: {}x{}", w, h);

    // --- Test 1: 4-connected labeling ---
    eprintln!("=== 4-connected labeling ===");
    let labeled4 =
        label_connected_components(&pixs, ConnectivityType::FourWay).expect("label 4-connected");
    rp.compare_values(w as f64, labeled4.width() as f64, 0.0);
    rp.compare_values(h as f64, labeled4.height() as f64, 0.0);

    // --- Test 2: 8-connected labeling ---
    eprintln!("=== 8-connected labeling ===");
    let labeled8 =
        label_connected_components(&pixs, ConnectivityType::EightWay).expect("label 8-connected");
    rp.compare_values(w as f64, labeled8.width() as f64, 0.0);
    rp.compare_values(h as f64, labeled8.height() as f64, 0.0);

    // --- Test 3: Component counting ---
    eprintln!("=== Component counting ===");
    let comps4 =
        find_connected_components(&pixs, ConnectivityType::FourWay).expect("find components 4-way");
    let comps8 = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find components 8-way");

    let n4 = comps4.len();
    let n8 = comps8.len();
    eprintln!("  4-connected components: {}", n4);
    eprintln!("  8-connected components: {}", n8);

    rp.compare_values(1.0, if n4 > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if n8 > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if n4 >= n8 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 4: Component properties ---
    eprintln!("=== Component properties ===");
    for (i, comp) in comps8.iter().take(5).enumerate() {
        let area = comp.pixel_count;
        let bounds = &comp.bounds;
        rp.compare_values(1.0, if area > 0 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(
            1.0,
            if bounds.w > 0 && bounds.h > 0 {
                1.0
            } else {
                0.0
            },
            0.0,
        );
        eprintln!(
            "  comp[{}]: area={}, bounds=({},{},{},{})",
            i, area, bounds.x, bounds.y, bounds.w, bounds.h
        );
    }

    // --- Test 5: feyn-fract.tif has many components ---
    let pixf = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let comps_fract = find_connected_components(&pixf, ConnectivityType::EightWay)
        .expect("find components feyn-fract");
    eprintln!("  feyn-fract 8-way components: {}", comps_fract.len());
    rp.compare_values(1.0, if comps_fract.len() > 100 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 6: component_area_transform (C check 4) ---
    // C: pix2 = pixConnCompAreaTransform(feyn-fract, 8);
    //    pix3 = pixConvert32To8(pix2, L_LS_TWO_BYTES, L_CLIP_TO_FF);
    let labeled_f =
        label_connected_components(&pixf, ConnectivityType::EightWay).expect("label feyn-fract");
    let area32 = component_area_transform(&labeled_f).expect("component_area_transform");
    let area8 = area32
        .convert_32_to_8(Convert32To16Type::LsTwoBytes, Convert16To8Type::ClipToFf)
        .expect("convert area to 8bpp");
    rp.compare_values(pixf.width() as f64, area8.width() as f64, 0.0);
    rp.compare_values(pixf.height() as f64, area8.height() as f64, 0.0);
    rp.write_pix_and_check(&area8, ImageFormat::Png)
        .expect("check: area transform");

    // --- Test 7: pix_loc_to_color_transform (C check 6) ---
    // C: pix5 = pixLocToColorTransform(pixRead("form1.tif"));
    let form1 = load_test_image("form1.tif").expect("load form1.tif");
    let color_img = pix_loc_to_color_transform(&form1).expect("pix_loc_to_color_transform");
    rp.compare_values(form1.width() as f64, color_img.width() as f64, 0.0);
    rp.compare_values(form1.height() as f64, color_img.height() as f64, 0.0);
    rp.write_pix_and_check(&color_img, ImageFormat::Png)
        .expect("check: loc-to-color transform");

    assert!(rp.cleanup(), "label regression test failed");
}

/// Test pix_get_sorted_neighbor_values on a labeled image.
#[test]
fn label_reg_sorted_neighbors() {
    if crate::common::is_display_mode() {
        return;
    }

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let labeled =
        label_connected_components(&pixs, ConnectivityType::EightWay).expect("label components");

    // Find a pixel with neighbors (somewhere in the middle)
    let x = labeled.width() / 2;
    let y = labeled.height() / 2;

    let neighbors = pix_get_sorted_neighbor_values(&labeled, x, y, ConnectivityType::EightWay)
        .expect("get sorted neighbors");

    // Values should be sorted
    for i in 1..neighbors.len() {
        assert!(neighbors[i] >= neighbors[i - 1], "neighbors not sorted");
    }

    // No zero values (background excluded)
    for &v in &neighbors {
        assert_ne!(v, 0, "background value should be excluded");
    }
}

/// Location-to-color transform under 4-fold symmetry, translation, small
/// rotation, and a different form, mirroring C checks 7-27 exactly
/// (same input form1.tif / form2.tif, same transforms), so the six outputs
/// are C-comparable at pixel level (plan 902 PR 6).
#[test]
fn label_reg_color_transform_series() {
    use leptonica::core::pix::rop::InColor;
    use leptonica::transform::{ShearFill, rotate_orth, rotate_shear_center_ip};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("label_color");

    let form1 = load_test_image("form1.tif").expect("load form1.tif");

    // C checks 7 / 11 / 15: pixLocToColorTransform(pixRotateOrth(form1, q))
    for quads in [1u32, 2, 3] {
        let rotated = rotate_orth(&form1, quads).expect("rotate_orth");
        let color = pix_loc_to_color_transform(&rotated).expect("loc_to_color rotated");
        rp.write_pix_and_check(&color, ImageFormat::Png)
            .expect("check: loc-to-color rotated");
    }

    // C check 19: pixTranslate(pix1, pix1, 10, 10, L_BRING_IN_WHITE)
    let translated = form1.translate(10, 10, InColor::White);
    let color = pix_loc_to_color_transform(&translated).expect("loc_to_color translated");
    rp.write_pix_and_check(&color, ImageFormat::Png)
        .expect("check: loc-to-color translated");

    // C check 23: pixRotateShearCenterIP(pix1, 0.1, L_BRING_IN_WHITE)
    let mut sheared = form1.deep_clone().try_into_mut().expect("into mut");
    rotate_shear_center_ip(&mut sheared, 0.1, ShearFill::White).expect("rotate_shear_center_ip");
    let sheared: leptonica::Pix = sheared.into();
    let color = pix_loc_to_color_transform(&sheared).expect("loc_to_color sheared");
    rp.write_pix_and_check(&color, ImageFormat::Png)
        .expect("check: loc-to-color sheared");

    // C check 27: pixLocToColorTransform(pixRead("form2.tif"))
    let form2 = load_test_image("form2.tif").expect("load form2.tif");
    let color = pix_loc_to_color_transform(&form2).expect("loc_to_color form2");
    rp.write_pix_and_check(&color, ImageFormat::Png)
        .expect("check: loc-to-color form2");

    assert!(rp.cleanup(), "label color transform series failed");
}

/// pix_loc_to_color_transform must reproduce C pixLocToColorTransform
/// exactly. Expected value hand-computed from the C algorithm
/// (`pixLocToColorTransform()` in upstream pixlabel.c):
///
/// - r/g: 255/(dim/2) * |coord - dim/2|, truncated to int
/// - b: component area, taken as (area & 0xffff) then clipped to 255
/// - channels composed like C pixCreateRGBImage, i.e. **alpha byte = 0**
///   (same convention as pixConvert8To32, cf. PR #405)
#[test]
#[ignore = "not yet implemented"]
fn label_reg_loc_to_color_matches_c() {
    // 4x4 with a single fg pixel at (1,1): w2 = h2 = 2, inv = 127.5,
    // rval = gval = (127.5 * 1) as int = 127, bval = area = 1.
    let pix = {
        let p = leptonica::Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut pm = p.try_into_mut().unwrap();
        pm.set_pixel(1, 1, 1).unwrap();
        let p: leptonica::Pix = pm.into();
        p
    };
    let out = pix_loc_to_color_transform(&pix).expect("loc_to_color");
    assert_eq!(out.get_pixel(1, 1).unwrap(), 0x7F7F_0100);
    // bg pixels stay 0x00000000
    assert_eq!(out.get_pixel(0, 0).unwrap(), 0);
}

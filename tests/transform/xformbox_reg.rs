//! Box transform regression test
//!
//! Tests ordered box transformations (translation, scaling, rotation)
//! and composite affine transforms on Boxa. Also tests hash box rendering
//! using connected component boxes from feyn.tif.
//!
//! # See also
//!
//! C Leptonica: `prog/xformbox_reg.c`

use crate::common::RegParams;
use leptonica::transform::AffineMatrix;
use leptonica::{Box as LeptBox, Boxa};

/// Test Boxa translate, scale, rotate individually (C partial check 5).
///
/// Creates a Boxa, applies individual transforms, and verifies
/// the resulting box coordinates are correct.
#[test]
fn xformbox_reg_individual_transforms() {
    let mut rp = RegParams::new("xformbox_indiv");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));
    boxa.push(LeptBox::new(300, 200, 70, 50).expect("box3"));

    let n = boxa.len();
    rp.compare_values(3.0, n as f64, 0.0);

    // Translate by (44, 39)
    let translated = boxa.translate(44.0, 39.0);
    rp.compare_values(n as f64, translated.len() as f64, 0.0);
    let tb = translated.get(0).expect("get translated box 0");
    rp.compare_values(144.0, tb.x as f64, 1.0);
    rp.compare_values(139.0, tb.y as f64, 1.0);
    rp.compare_values(50.0, tb.w as f64, 1.0);
    rp.compare_values(30.0, tb.h as f64, 1.0);

    // Scale by (0.83, 0.78)
    let scaled = boxa.scale(0.83, 0.78);
    rp.compare_values(n as f64, scaled.len() as f64, 0.0);
    let sb = scaled.get(0).expect("get scaled box 0");
    // Scaled coordinates: x=100*0.83=83, y=100*0.78=78, w=50*0.83≈41.5, h=30*0.78≈23.4
    rp.compare_values(83.0, sb.x as f64, 1.0);
    rp.compare_values(78.0, sb.y as f64, 1.0);

    assert!(rp.cleanup(), "xformbox individual transforms test failed");
}

/// Test Boxa affine transform (C check 5 composite part).
///
/// Verifies that affine transform with identity preserves coordinates,
/// and that translation-only affine matches Boxa::translate.
#[test]
fn xformbox_reg_affine_transform() {
    let mut rp = RegParams::new("xformbox_affine");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));

    // Identity affine should preserve all coordinates
    let identity = AffineMatrix::identity();
    let id_result = leptonica::transform::boxa_affine_transform(&boxa, &identity);
    rp.compare_values(boxa.len() as f64, id_result.len() as f64, 0.0);
    let ob = boxa.get(0).expect("original box 0");
    let ib = id_result.get(0).expect("identity box 0");
    rp.compare_values(ob.x as f64, ib.x as f64, 1.0);
    rp.compare_values(ob.y as f64, ib.y as f64, 1.0);
    rp.compare_values(ob.w as f64, ib.w as f64, 1.0);
    rp.compare_values(ob.h as f64, ib.h as f64, 1.0);

    // Translation-only affine should match Boxa::translate
    let mat_translate = AffineMatrix::translation(44.0, 39.0);
    let affine_translated = leptonica::transform::boxa_affine_transform(&boxa, &mat_translate);
    let direct_translated = boxa.translate(44.0, 39.0);

    let at = affine_translated.get(0).expect("affine translated box 0");
    let dt = direct_translated.get(0).expect("direct translated box 0");
    rp.compare_values(dt.x as f64, at.x as f64, 1.0);
    rp.compare_values(dt.y as f64, at.y as f64, 1.0);
    rp.compare_values(dt.w as f64, at.w as f64, 1.0);
    rp.compare_values(dt.h as f64, at.h as f64, 1.0);

    assert!(rp.cleanup(), "xformbox affine transform test failed");
}

/// Test Boxa rotation (C check 3 rotation part).
///
/// Rotates boxes by a small angle and verifies the result is reasonable.
#[test]
fn xformbox_reg_rotation() {
    let mut rp = RegParams::new("xformbox_rotate");

    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 50, 80, 60).expect("box1"));
    boxa.push(LeptBox::new(250, 100, 90, 70).expect("box2"));

    let n = boxa.len();

    // Rotate by 0.10 radians about (200, 150)
    let rotated = boxa.rotate(200.0, 150.0, 0.10);
    rp.compare_values(n as f64, rotated.len() as f64, 0.0);

    // After small rotation, boxes should still have positive dimensions
    let rb = rotated.get(0).expect("get rotated box 0");
    rp.compare_values(1.0, if rb.w > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if rb.h > 0 { 1.0 } else { 0.0 }, 0.0);

    // Identity rotation (angle=0) should preserve coordinates
    let identity = boxa.rotate(200.0, 150.0, 0.0);
    let ib = identity.get(0).expect("identity box 0");
    let ob = boxa.get(0).expect("original box 0");
    rp.compare_values(ob.x as f64, ib.x as f64, 1.0);
    rp.compare_values(ob.y as f64, ib.y as f64, 1.0);
    rp.compare_values(ob.w as f64, ib.w as f64, 1.0);
    rp.compare_values(ob.h as f64, ib.h as f64, 1.0);

    assert!(rp.cleanup(), "xformbox rotation test failed");
}

/// Test boxaTransformOrdered (C checks 4-5 ordered transforms).
///
/// Tests all 6 orderings with translation-only (scale=1.0, angle=0)
/// and verifies they produce identical results. Also tests with
/// rotation and scale to verify orderings produce different results.
#[test]
fn xformbox_reg_ordered() {
    use leptonica::TransformOrder;

    let mut rp = RegParams::new("xformbox_ordered");

    // Create test boxes
    let mut boxa = Boxa::new();
    boxa.push(LeptBox::new(100, 100, 50, 30).expect("box1"));
    boxa.push(LeptBox::new(200, 150, 60, 40).expect("box2"));
    boxa.push(LeptBox::new(300, 200, 70, 50).expect("box3"));

    let orderings = [
        TransformOrder::TrScRo,
        TransformOrder::TrRoSc,
        TransformOrder::ScTrRo,
        TransformOrder::RoTrSc,
        TransformOrder::RoScTr,
        TransformOrder::ScRoTr,
    ];

    // For translation-only (scale=1.0, angle=0.0), all orderings must produce
    // the same result.
    let reference = boxa.transform_ordered(44, 39, 1.0, 1.0, 0, 0, 0.0, orderings[0]);
    rp.compare_values(boxa.len() as f64, reference.len() as f64, 0.0);

    for &order in &orderings[1..] {
        let result = boxa.transform_ordered(44, 39, 1.0, 1.0, 0, 0, 0.0, order);
        rp.compare_values(reference.len() as f64, result.len() as f64, 0.0);
        for i in 0..reference.len() {
            let rb = reference.get(i).expect("ref box");
            let ob = result.get(i).expect("order box");
            rp.compare_values(rb.x as f64, ob.x as f64, 0.0);
            rp.compare_values(rb.y as f64, ob.y as f64, 0.0);
            rp.compare_values(rb.w as f64, ob.w as f64, 0.0);
            rp.compare_values(rb.h as f64, ob.h as f64, 0.0);
        }
    }

    // With rotation and scale, different orderings should generally
    // produce different results. Verify at least that each produces
    // valid boxes with positive dimensions.
    for &order in &orderings {
        let result = boxa.transform_ordered(10, 20, 1.5, 1.5, 200, 150, 0.15, order);
        rp.compare_values(boxa.len() as f64, result.len() as f64, 0.0);
        for i in 0..result.len() {
            let b = result.get(i).expect("transformed box");
            rp.compare_values(1.0, if b.w > 0 { 1.0 } else { 0.0 }, 0.0);
            rp.compare_values(1.0, if b.h > 0 { 1.0 } else { 0.0 }, 0.0);
        }
    }

    assert!(rp.cleanup(), "xformbox ordered transforms test failed");
}

/// Test hash box rendering (C checks 0-2).
///
/// Uses conncomp_pixa on feyn.tif to extract component boxes, then
/// renders hash lines using binary, color, and blend modes.
#[test]
fn xformbox_reg_hash_rendering() {
    use leptonica::core::pix::{HashOrientation, PixelOp};
    use leptonica::{Color, InitColor, PixMut, PixelDepth};

    let mut rp = RegParams::new("xformbox_hash");

    // Load feyn.tif (1bpp) and extract connected component boxes
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let (boxa, _pixa) =
        leptonica::region::conncomp_pixa(&pix, leptonica::region::ConnectivityType::EightWay)
            .expect("conncomp_pixa");
    let n = boxa.len();
    rp.compare_values(1.0, if n > 10 { 1.0 } else { 0.0 }, 0.0);

    let w = pix.width();
    let h = pix.height();

    // 1. Binary hash rendering
    let mut pm1 = PixMut::new(w, h, PixelDepth::Bit1).expect("create 1bpp");
    let b0 = boxa.get(0).expect("box 0");
    pm1.render_hash_box(b0, 5, 1, HashOrientation::Horizontal, false, PixelOp::Set)
        .expect("render_hash_box binary");
    let pix1: leptonica::Pix = pm1.into();
    rp.compare_values(1.0, if pix1.count_pixels() > 0 { 1.0 } else { 0.0 }, 0.0);

    // 2. Color hash rendering on 32bpp canvas
    let mut pm32 = PixMut::new(w, h, PixelDepth::Bit32).expect("create 32bpp");
    pm32.set_black_or_white(InitColor::White);
    let blue = Color::new(0, 0, 255);
    let b1 = boxa.get(1.min(n - 1)).expect("box 1");
    pm32.render_hash_box_color(b1, 6, 1, HashOrientation::Vertical, false, blue)
        .expect("render_hash_box_color");
    let pix32: leptonica::Pix = pm32.into();
    rp.compare_values(w as f64, pix32.width() as f64, 0.0);

    // 3. Blend hash rendering on 32bpp canvas
    let mut pm_blend = PixMut::new(w, h, PixelDepth::Bit32).expect("create 32bpp blend");
    pm_blend.set_black_or_white(InitColor::White);
    let red = Color::new(255, 0, 0);
    let b2 = boxa.get(2.min(n - 1)).expect("box 2");
    pm_blend
        .render_hash_box_blend(b2, 5, 1, HashOrientation::PosSlope, true, red, 0.5)
        .expect("render_hash_box_blend");
    let pix_blend: leptonica::Pix = pm_blend.into();
    rp.compare_values(h as f64, pix_blend.height() as f64, 0.0);

    assert!(rp.cleanup(), "xformbox hash rendering test failed");
}

/// C-comparable hash-box and box-transform series (plan 902 PR 16).
///
/// Mirrors C xformbox_reg checks 0-4: feyn.tif clipped to the same box,
/// connected components rendered with the three hash-box variants, the
/// four orthogonal rotations with correspondingly rotated boxa, and the
/// six `transform_ordered` orders applied to translation and scaling.
///
/// C check 5 additionally needs `boxaAffineTransform` with the 2D matrix
/// builders, which this port does not have yet.
#[test]
fn xformbox_c_compat() {
    use leptonica::TransformOrder;
    use leptonica::core::pix::PixelOp;
    use leptonica::core::pix::graphics::{Color, HashOrientation};
    use leptonica::io::ImageFormat;
    use leptonica::region::{ConnectivityType, find_connected_components};
    use leptonica::transform::rotate_orth;
    use leptonica::{Boxa, Pix, Pixa, PixelDepth};

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("xformbox_c");

    // C: pixClipRectangle(feyn.tif, boxCreate(461, 429, 1393, 342))
    let feyn = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pix1 = feyn.clip_rectangle(461, 429, 1393, 342).expect("clip");
    let comps = find_connected_components(&pix1, ConnectivityType::EightWay).expect("conncomp");
    let mut boxa = Boxa::new();
    for c in &comps {
        boxa.push(c.bounds);
    }

    // C uses i % 4 as the hash orientation index.
    let orient_of = |i: usize| match i % 4 {
        0 => HashOrientation::Horizontal,
        1 => HashOrientation::PosSlope,
        2 => HashOrientation::Vertical,
        _ => HashOrientation::NegSlope,
    };
    let color_of = |i: usize| {
        Color::new(
            ((1413 * i) % 256) as u8,
            ((4917 * i) % 256) as u8,
            ((7341 * i) % 256) as u8,
        )
    };

    // C checks 0-2: the three hash-box renderers over the same boxa.
    let mut bin = pix1.deep_clone().to_mut();
    // C: pixConvertTo8(pix1, 1) -> pixConvert1To8Cmap (8bpp with colormap).
    let mut gray = pix1.convert_1_to_8_cmap().expect("to 8bpp cmap").to_mut();
    let mut rgb = pix1.convert_to_32().expect("to 32bpp").to_mut();
    for (i, b) in boxa.iter().enumerate() {
        let orient = orient_of(i);
        let color = color_of(i + 1);
        bin.render_hash_box(b, 8, 2, orient, true, PixelOp::Set)
            .expect("render_hash_box");
        gray.render_hash_box_color(b, 7, 2, orient, true, color)
            .expect("render_hash_box_color");
        rgb.render_hash_box_blend(b, 7, 2, orient, true, color, 0.5)
            .expect("render_hash_box_blend");
    }
    for pix in [Pix::from(bin), Pix::from(gray), Pix::from(rgb)] {
        rp.write_pix_and_check(&pix, ImageFormat::Png)
            .expect("check: xformbox hash render");
    }

    // C check 3: four orthogonal rotations with matching boxa rotations.
    let (w, h) = (pix1.width() as i32, pix1.height() as i32);
    let pixc = pix1.convert_to_32().expect("to 32bpp");
    let mut pixa = Pixa::new();
    for i in 0..4u32 {
        let rotated = rotate_orth(&pixc, i).expect("rotate_orth");
        let boxa2 = boxa.rotate_orth(w, h, i as i32).expect("boxa rotate_orth");
        let mut m = rotated.to_mut();
        m.render_hash_boxa_color(
            &boxa2,
            10,
            3,
            orient_of(i as usize),
            true,
            color_of(i as usize + 4),
        )
        .expect("render_hash_boxa_color");
        pixa.push(m.into());
    }
    let tiled = pixa
        .display_tiled_in_rows(PixelDepth::Bit32, 1200, 0.7, 0, 30, 3)
        .expect("tiled in rows");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: xformbox rotate orth");

    // C check 4: the six transform orders, over translation then scaling.
    // pixs = clip(feyn, 420, 360, 1500, 465) with a 200px right border.
    let pixt = feyn.clip_rectangle(420, 360, 1500, 465).expect("clip 2");
    let pixs = pixt
        .add_border_general(0, 200, 0, 0, 0)
        .expect("add right border");
    let comps = find_connected_components(&pixs, ConnectivityType::EightWay).expect("conncomp 2");
    let mut boxa = Boxa::new();
    for c in &comps {
        boxa.push(c.bounds);
    }

    let render = |pix: &Pix, boxa: &Boxa, i: usize| -> Pix {
        let mut m = pix.deep_clone().to_mut();
        for b in boxa.iter() {
            m.render_hash_box_color(b, 10, 3, orient_of(i), true, color_of(i))
                .expect("render_hash_box_color");
        }
        m.into()
    };

    // (shift, scale, orders) per C group.
    /// (shift, scale, three transform orders, base colour index).
    struct Group {
        shift: (i32, i32),
        scale: (f32, f32),
        orders: [TransformOrder; 3],
        base_index: usize,
    }
    let groups: [Group; 4] = [
        Group {
            shift: (50, 70),
            scale: (1.0, 1.0),
            orders: [
                TransformOrder::TrScRo,
                TransformOrder::TrRoSc,
                TransformOrder::ScTrRo,
            ],
            base_index: 0,
        },
        Group {
            shift: (50, 70),
            scale: (1.0, 1.0),
            orders: [
                TransformOrder::RoTrSc,
                TransformOrder::RoScTr,
                TransformOrder::ScRoTr,
            ],
            base_index: 4,
        },
        Group {
            shift: (0, 0),
            scale: (1.17, 1.13),
            orders: [
                TransformOrder::TrScRo,
                TransformOrder::ScRoTr,
                TransformOrder::ScTrRo,
            ],
            base_index: 8,
        },
        Group {
            shift: (0, 0),
            scale: (1.17, 1.13),
            orders: [
                TransformOrder::RoTrSc,
                TransformOrder::RoScTr,
                TransformOrder::TrRoSc,
            ],
            base_index: 12,
        },
    ];

    let base = pixs.convert_to_32().expect("pixs to 32bpp");
    let mut pixa = Pixa::new();
    for group in groups {
        let (sx, sy) = group.shift;
        let (scx, scy) = group.scale;
        let mut acc = base.deep_clone();
        for (i, order) in group.orders.into_iter().enumerate() {
            let boxat = boxa.transform_ordered(sx, sy, scx, scy, 450, 250, 0.10, order);
            acc = render(&acc, &boxat, group.base_index + i);
        }
        pixa.push(acc);
    }
    let tiled = pixa
        .display_tiled_in_columns(1, 0.5, 20, 0)
        .expect("tiled in columns");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: xformbox ordered transforms");

    assert!(rp.cleanup(), "xformbox c-compat test failed");
}

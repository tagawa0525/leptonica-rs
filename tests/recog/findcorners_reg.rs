//! Find-corners regression test
//!
//! Uses checkerboard corner detection as the Rust mapping.
//!
//! # See also
//!
//! C Leptonica: `prog/findcorners_reg.c`

use crate::common::RegParams;
use leptonica::io::ImageFormat;
use leptonica::region::find_checkerboard_corners;
use leptonica::{Pix, PixelDepth};

fn make_checkerboard(size: u32, cell: u32) -> Pix {
    let pix = Pix::new(size, size, PixelDepth::Bit1).expect("create checkerboard");
    let mut pm = pix.try_into_mut().expect("mutable checkerboard");

    let cells = size / cell;
    for cy in 0..cells {
        for cx in 0..cells {
            if (cx + cy) % 2 == 0 {
                for y in (cy * cell)..((cy + 1) * cell).min(size) {
                    for x in (cx * cell)..((cx + 1) * cell).min(size) {
                        pm.set_pixel_unchecked(x, y, 1);
                    }
                }
            }
        }
    }
    pm.into()
}

#[test]
fn findcorners_reg() {
    let mut rp = RegParams::new("findcorners");

    let pix = make_checkerboard(72, 12);
    let (corner_pix, pta) = find_checkerboard_corners(&pix, 7, 1, 2).expect("find corners");
    rp.compare_values(pix.width() as f64, corner_pix.width() as f64, 0.0);
    rp.compare_values(pix.height() as f64, corner_pix.height() as f64, 0.0);
    rp.compare_values(1.0, if pta.len() <= 400 { 1.0 } else { 0.0 }, 0.0);

    rp.write_pix_and_check(&corner_pix, ImageFormat::Tiff)
        .expect("write corner_pix findcorners");

    let empty = Pix::new(50, 50, PixelDepth::Bit1).expect("empty image");
    let (_empty_pix, empty_pta) =
        find_checkerboard_corners(&empty, 7, 1, 2).expect("empty corners");
    rp.compare_values(0.0, empty_pta.len() as f64, 0.0);

    rp.compare_values(
        1.0,
        if find_checkerboard_corners(&pix, 3, 1, 2).is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "findcorners regression test failed");
}

/// C-compat: `prog/findcorners_reg.c`, all 12 outputs.
///
/// `tickets.tif` is lossless and C writes every output as G4 TIFF, so the
/// whole program is bit-exactly comparable. Each ticket is located by
/// morphology, deskewed with `pixFindSkew`, then re-located and clipped, so
/// this exercises the skew search on real page data.
#[test]
fn findcorners_c_compat() {
    use leptonica::core::{Boxa, RopOp, SizeRelation, SizeSelectType};
    use leptonica::morph::{
        MorphOpType, Sel, dilate, morph_sequence, sela_add_hit_miss, union_of_morph_ops,
    };
    use leptonica::recog::SkewDetectOptions;
    use leptonica::recog::skew::find_skew;
    use leptonica::region::{ConnectivityType, find_connected_components, pix_select_by_size};
    use leptonica::transform::{
        RotateEmbed, RotateFill, RotateMethod, RotateOptions, ScaleMethod, rotate, scale,
    };

    if crate::common::is_display_mode() {
        return;
    }

    // C: sel_cross, 13x13, origin at the 'X' (row 6, col 6).
    const SEL_CROSS: &str = "     xxx     \n     xxx     \n     xxx     \n     xxx     \n     xxx     \nxxxxxxxxxxxxx\nxxxxxxXxxxxxx\nxxxxxxxxxxxxx\n     xxx     \n     xxx     \n     xxx     \n     xxx     \n     xxx     ";

    // C: LocateBarcodes(pixs, ppixd, 0)
    fn locate_barcodes(pixs: &Pix, want_pixd: bool) -> (Boxa, Option<Pix>) {
        let pix1 = scale(pixs, 0.5, 0.5, ScaleMethod::Auto).expect("scale 0.5");
        let pix2 = morph_sequence(&pix1, "o1.5 + c15.1 + o10.15 + c20.20").expect("morph sequence");
        let comps =
            find_connected_components(&pix2, ConnectivityType::EightWay).expect("conn comp");
        let boxa1: Boxa = comps.iter().map(|c| c.bounds).collect();
        // C: boxaSelectBySize(boxa1, 300, 0, L_SELECT_WIDTH, L_SELECT_IF_GT)
        let boxa2 = boxa1.select_by_size(300, 0, SizeSelectType::Width, SizeRelation::GreaterThan);
        let boxad = boxa2.transform(0, 0, 2.0, 2.0);

        let pixd = want_pixd.then(|| {
            // C: pixSelectBySize(pix2, 300, 0, 8, L_SELECT_WIDTH, L_SELECT_IF_GT)
            let pix3 = pix_select_by_size(
                &pix2,
                300,
                0,
                ConnectivityType::EightWay,
                SizeSelectType::Width,
                SizeRelation::GreaterThan,
            )
            .expect("select by size");
            scale(&pix3, 2.0, 2.0, ScaleMethod::Auto).expect("scale 2.0")
        });
        (boxad, pixd)
    }

    let mut rp = RegParams::new("findcorners_c");

    let pixs = crate::common::load_test_image("tickets.tif").expect("load tickets.tif");
    let (boxa, pixd) = locate_barcodes(&pixs, true);
    let pixd = pixd.expect("pixd");
    rp.write_pix_and_check(&pixd, ImageFormat::TiffG4)
        .expect("check: located barcodes");

    // C uses the literal 3.14159265, not M_PI.
    #[allow(clippy::approx_constant, clippy::excessive_precision)]
    let deg2rad = 3.14159265_f32 / 180.0;
    let sampling = RotateOptions {
        method: RotateMethod::Sampling,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        embed: RotateEmbed::None,
    };

    for i in 0..boxa.len() {
        let box1 = *boxa.get(i).expect("barcode box");
        let box2 = box1.adjust_sides(-266, 346, -1560, 182).expect("adjust 1");
        let pix1 = pixs
            .clip_rectangle(box2.x as u32, box2.y as u32, box2.w as u32, box2.h as u32)
            .expect("clip 1");
        let angle = find_skew(&pix1, &SkewDetectOptions::default())
            .expect("find skew")
            .angle;
        let pix2 = rotate(&pix1, deg2rad * angle, &sampling).expect("deskew");
        let (boxa2, _) = locate_barcodes(&pix2, false);
        let box3 = *boxa2.get(0).expect("relocated box");
        let box4 = box3.adjust_sides(-141, 221, -1535, 157).expect("adjust 2");
        let pix3 = pix2
            .clip_rectangle(box4.x as u32, box4.y as u32, box4.w as u32, box4.h as u32)
            .expect("clip 2");
        rp.write_pix_and_check(&pix3, ImageFormat::TiffG4)
            .expect("check: deskewed ticket");
    }

    let pix1 = scale(&pixd, 0.5, 0.5, ScaleMethod::Auto).expect("scale half");
    rp.write_pix_and_check(&pix1, ImageFormat::TiffG4)
        .expect("check: halved");

    // C: GetCornerSela picks the four corner hit-miss sels by name.
    let all = sela_add_hit_miss();
    let sela: Vec<Sel> = ["sel_ulc", "sel_urc", "sel_llc", "sel_lrc"]
        .iter()
        .map(|name| {
            all.iter()
                .find(|s| s.name() == Some(name))
                .expect("corner sel")
                .clone()
        })
        .collect();
    let pix2 = union_of_morph_ops(&pix1, &sela, MorphOpType::HitMiss).expect("union of HMT");
    let sel = Sel::from_string(SEL_CROSS, 6, 6).expect("sel cross");
    let pix3 = dilate(&pix2, &sel).expect("dilate cross");
    let pix3 = pix3.rop(&pix1, RopOp::Xor).expect("xor");
    rp.write_pix_and_check(&pix3, ImageFormat::TiffG4)
        .expect("check: corners marked");

    assert!(rp.cleanup(), "findcorners C-compat test failed");
}

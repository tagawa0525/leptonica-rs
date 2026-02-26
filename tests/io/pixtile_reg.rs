//! Pixel tiling regression test
//!
//! Tests the PixTiling structure for tile-based image processing.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixtile_reg.c`

use crate::common::RegParams;
use leptonica::core::pixtiling::PixTiling;

/// Test basic image clipping as partial substitute for tiling.
#[test]
fn pixtile_reg_basic_clip() {
    let mut rp = RegParams::new("pixtile_clip");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let w = pix.width();
    let h = pix.height();

    let tile_w = w / 2;
    let tile_h = h / 2;
    let tile = pix.clip_rectangle(0, 0, tile_w, tile_h).expect("clip");
    rp.compare_values(tile_w as f64, tile.width() as f64, 0.0);
    rp.compare_values(tile_h as f64, tile.height() as f64, 0.0);

    let p_orig = pix.get_pixel(0, 0).expect("get_pixel origin");
    let p_tile = tile.get_pixel(0, 0).expect("get_pixel tile origin");
    rp.compare_values(p_orig as f64, p_tile as f64, 0.0);

    let mid_x = tile_w.min(tile.width()) - 1;
    let mid_y = tile_h.min(tile.height()) - 1;
    let p_orig_mid = pix.get_pixel(mid_x, mid_y).expect("get_pixel mid");
    let p_tile_mid = tile.get_pixel(mid_x, mid_y).expect("get_pixel tile mid");
    rp.compare_values(p_orig_mid as f64, p_tile_mid as f64, 0.0);

    let tile_br = pix
        .clip_rectangle(tile_w, tile_h, w - tile_w, h - tile_h)
        .expect("clip br");
    rp.compare_values((w - tile_w) as f64, tile_br.width() as f64, 0.0);
    rp.compare_values((h - tile_h) as f64, tile_br.height() as f64, 0.0);

    let p_orig_br = pix.get_pixel(tile_w, tile_h).expect("get_pixel br");
    let p_tile_br = tile_br.get_pixel(0, 0).expect("get_pixel tile br");
    rp.compare_values(p_orig_br as f64, p_tile_br as f64, 0.0);

    assert!(rp.cleanup(), "pixtile basic clip test failed");
}

/// Test PixTiling create and get_count/get_size.
#[test]
#[ignore = "not yet implemented"]
fn pixtile_reg_create_and_query() {
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    let tiling = PixTiling::create(&pix, 3, 4, 0, 0, 0, 0).unwrap();
    let (nx, ny) = tiling.get_count();
    assert_eq!(nx, 3);
    assert_eq!(ny, 4);

    let (tw, th) = tiling.get_size();
    assert!(tw > 0);
    assert!(th > 0);
}

/// Test PixTiling round-trip: extract tiles and paint back.
#[test]
#[ignore = "not yet implemented"]
fn pixtile_reg_tiling_roundtrip() {
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let w = pix.width();
    let h = pix.height();

    let tiling = PixTiling::create(&pix, 2, 3, 0, 0, 0, 0).unwrap();
    let (nx, ny) = tiling.get_count();

    let mut dst = leptonica::Pix::new(w, h, pix.depth()).unwrap().to_mut();

    for j in 0..ny {
        for i in 0..nx {
            let tile = tiling.get_tile(i, j).unwrap();
            tiling.paint_tile(&mut dst, i, j, &tile).unwrap();
        }
    }

    // Verify reconstruction
    let dst_pix: leptonica::Pix = dst.into();
    assert_eq!(dst_pix.width(), w);
    assert_eq!(dst_pix.height(), h);
}

/// Test PixTiling with overlap.
#[test]
#[ignore = "not yet implemented"]
fn pixtile_reg_overlap() {
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    let tiling = PixTiling::create(&pix, 3, 3, 0, 0, 10, 10).unwrap();
    let (nx, ny) = tiling.get_count();
    assert_eq!(nx, 3);
    assert_eq!(ny, 3);

    // Each tile should be larger than non-overlapping tile
    let tile = tiling.get_tile(1, 1).unwrap();
    let (tw, _th) = tiling.get_size();
    assert!(tile.width() >= tw);
}

/// Test PixTiling by tile size.
#[test]
#[ignore = "not yet implemented"]
fn pixtile_reg_by_size() {
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");

    let tiling = PixTiling::create(&pix, 0, 0, 200, 300, 0, 0).unwrap();
    let (nx, ny) = tiling.get_count();
    assert!(nx > 0);
    assert!(ny > 0);
}

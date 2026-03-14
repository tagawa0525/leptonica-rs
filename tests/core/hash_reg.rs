//! Hash regression test
//!
//! Covers hash-based color counting and hash-line rendering.
//!
//! # See also
//!
//! C Leptonica: `prog/hash_reg.c`

use crate::common::RegParams;
use leptonica::core::pix::HashOrientation;
use leptonica::core::pixel;
use leptonica::io::ImageFormat;
use leptonica::{Box, Pix, PixelDepth, PixelOp};

// ============================================================================
// C-equivalent regression test skeletons — L_ASET / L_HASHMAP operations
// ============================================================================

/// Sarray dedup via ordered set (C checks: asetCreateFromSarray).
///
/// C: sa3 = sarrayRemoveDupsByAset(sa1)
#[test]
#[ignore = "L_ASET (ordered set) not implemented"]
fn hash_reg_sarray_dedup_aset() {}

/// Pta dedup via ordered set (C checks: l_asetCreateFromPta).
///
/// C: pta3 = ptaRemoveDupsByAset(pta)
#[test]
#[ignore = "L_ASET (ordered set) not implemented"]
fn hash_reg_pta_dedup_aset() {}

/// Dna dedup via ordered set (C checks: l_dnaRemoveDupsByAset).
///
/// C: da3 = l_dnaRemoveDupsByAset(da1)
#[test]
#[ignore = "L_ASET (ordered set) not implemented"]
fn hash_reg_dna_dedup_aset() {}

/// L_HASHMAP-based Sarray dedup (C checks: sarrayRemoveDupsByHmap).
#[test]
#[ignore = "L_HASHMAP not implemented"]
fn hash_reg_sarray_dedup_hmap() {}

/// L_HASHMAP-based color intersection (C checks: pixCountColors with hmap).
#[test]
#[ignore = "L_HASHMAP not implemented"]
fn hash_reg_color_intersection_hmap() {}

#[test]
fn hash_reg() {
    let mut rp = RegParams::new("hash");

    // Hash-based color counting
    let pix = Pix::new(16, 12, PixelDepth::Bit32).expect("create 32bpp");
    let mut pm = pix.to_mut();
    pm.set_pixel(0, 0, pixel::compose_rgb(255, 0, 0))
        .expect("set red");
    pm.set_pixel(1, 0, pixel::compose_rgb(0, 255, 0))
        .expect("set green");
    let pix: Pix = pm.into();

    // default black + red + green
    let ncolors = pix.count_rgb_colors_by_hash().expect("count by hash");
    rp.compare_values(3.0, ncolors as f64, 0.0);

    // Also test count_rgb_colors (factor-based)
    let ncolors2 = pix.count_rgb_colors(1).expect("count_rgb_colors factor=1");
    rp.compare_values(3.0, ncolors2 as f64, 0.0);

    // Hash with more colors
    let big_pix = Pix::new(100, 100, PixelDepth::Bit32).expect("create big 32bpp");
    let mut bpm = big_pix.to_mut();
    for y in 0..100u32 {
        for x in 0..100u32 {
            let val = pixel::compose_rgb((x * 2 % 256) as u8, (y * 2 % 256) as u8, 0);
            bpm.set_pixel_unchecked(x, y, val);
        }
    }
    let big_pix: Pix = bpm.into();
    let big_count = big_pix
        .count_rgb_colors_by_hash()
        .expect("count many colors");
    rp.compare_values(1.0, if big_count > 100 { 1.0 } else { 0.0 }, 0.0);

    // C also tests Sarray/Pta/Dna hashing via L_ASET (not available in Rust)

    // Hash-line rendering — horizontal
    let dest = Pix::new(40, 20, PixelDepth::Bit1).expect("create 1bpp");
    let mut dm = dest.to_mut();
    let b = Box::new(5, 3, 24, 12).expect("box");
    dm.render_hash_box(&b, 4, 1, HashOrientation::Horizontal, false, PixelOp::Set)
        .expect("render_hash_box horizontal");
    let rendered: Pix = dm.into();

    rp.compare_values(40.0, rendered.width() as f64, 0.0);
    let on = rendered.count_pixels();
    rp.compare_values(1.0, if on > 0 { 1.0 } else { 0.0 }, 0.0);

    rp.write_pix_and_check(&rendered, ImageFormat::Tiff)
        .expect("check: hash horizontal");

    // Hash-line rendering — vertical
    let dest2 = Pix::new(40, 20, PixelDepth::Bit1).expect("create 1bpp v");
    let mut dm2 = dest2.to_mut();
    dm2.render_hash_box(&b, 4, 1, HashOrientation::Vertical, false, PixelOp::Set)
        .expect("render_hash_box vertical");
    let rendered2: Pix = dm2.into();
    let on2 = rendered2.count_pixels();
    rp.compare_values(1.0, if on2 > 0 { 1.0 } else { 0.0 }, 0.0);
    rp.write_pix_and_check(&rendered2, ImageFormat::Tiff)
        .expect("check: hash vertical");

    assert!(rp.cleanup(), "hash regression test failed");
}

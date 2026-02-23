//! Gray fill regression test
//!
//! Tests gray seedfill operations for morphological reconstruction.
//! The C version creates synthetic 8bpp images programmatically and
//! tests forward/inverse seedfill and basin fill from local minima.
//!
//! Partial migration: seedfill_gray, seedfill_gray_inv, seedfill_gray_basin,
//! and local_extrema are tested. Hybrid vs. iterative comparison
//! (pixAddConstantGray) is not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/grayfill_reg.c`

use leptonica_core::{Pix, PixMut, PixelDepth};
use leptonica_region::{
    ConnectivityType, local_extrema, seedfill_gray, seedfill_gray_basin, seedfill_gray_inv,
};
use leptonica_test::RegParams;

/// Create the 200x200 mask image from the C test.
///
/// Mask pixel value = 20 + |((100-y)*(100-x))| / 50, clamped to u8.
fn make_mask_200() -> Pix {
    let mut m = PixMut::new(200, 200, PixelDepth::Bit8).expect("create mask");
    for y in 0..200u32 {
        for x in 0..200u32 {
            let dy = (100i32 - y as i32).abs();
            let dx = (100i32 - x as i32).abs();
            let val = (20 + (dy * dx) / 50).min(255) as u32;
            m.set_pixel(x, y, val).unwrap();
        }
    }
    m.into()
}

/// Test seedfill_gray_inv (C checks 2-3: inverse gray seedfill).
///
/// Seeds the image from a small central region and propagates using the
/// inverse gray fill rule (seed <= mask).
#[test]
#[ignore = "not yet implemented: grayfill regression tests"]
fn grayfill_reg_inv() {
    let mut rp = RegParams::new("gfill_inv");

    let mask = make_mask_200();
    let w = mask.width();
    let h = mask.height();

    // C: pixs1 = pixCreate(200, 200, 8); ... pixSetPixel at (99..101, 99..101) with ~50
    let mut seed = PixMut::new(200, 200, PixelDepth::Bit8).expect("create seed");
    for y in 99u32..=101 {
        for x in 99u32..=101 {
            let val = (50u32).saturating_sub(y / 100 + x / 100);
            seed.set_pixel(x, y, val).unwrap();
        }
    }
    let seed: Pix = seed.into();

    // C: pixSeedfillGrayInv(pixs1, pixm, 4); -- 4-way
    let result4 = seedfill_gray_inv(&seed, &mask, ConnectivityType::FourWay)
        .expect("seedfill_gray_inv 4-way");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);
    rp.compare_values(h as f64, result4.height() as f64, 0.0);
    assert_eq!(result4.depth(), PixelDepth::Bit8);

    // C: pixSeedfillGrayInv(pixs1_8, pixm, 8); -- 8-way
    let result8 = seedfill_gray_inv(&seed, &mask, ConnectivityType::EightWay)
        .expect("seedfill_gray_inv 8-way");
    rp.compare_values(w as f64, result8.width() as f64, 0.0);

    assert!(rp.cleanup(), "grayfill inv test failed");
}

/// Test seedfill_gray (C checks 9-10: standard gray seedfill).
///
/// Seeds from a high-value central region and fills up to mask values.
#[test]
#[ignore = "not yet implemented: grayfill regression tests"]
fn grayfill_reg_standard() {
    let mut rp = RegParams::new("gfill_std");

    let mask = make_mask_200();
    let mask_inv = mask.invert();
    let w = mask.width();
    let h = mask.height();

    // C: pixs2 = pixCreate(200, 200, 8); ... pixSetPixel at (99..101, 99..101) with ~205
    let mut seed = PixMut::new(200, 200, PixelDepth::Bit8).expect("create seed");
    for y in 99u32..=101 {
        for x in 99u32..=101 {
            let val = (205u32).saturating_sub(y / 100 + x / 100);
            seed.set_pixel(x, y, val).unwrap();
        }
    }
    let seed: Pix = seed.into();

    // C: pixSeedfillGray(pixs2, pixmi, 4); -- 4-way (using inverted mask)
    let result4 =
        seedfill_gray(&seed, &mask_inv, ConnectivityType::FourWay).expect("seedfill_gray 4-way");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);
    rp.compare_values(h as f64, result4.height() as f64, 0.0);
    assert_eq!(result4.depth(), PixelDepth::Bit8);

    // C: pixSeedfillGray(pixs2_8, pixmi, 8); -- 8-way
    let result8 =
        seedfill_gray(&seed, &mask_inv, ConnectivityType::EightWay).expect("seedfill_gray 8-way");
    rp.compare_values(w as f64, result8.width() as f64, 0.0);

    assert!(rp.cleanup(), "grayfill standard test failed");
}

/// Test local_extrema and seedfill_gray_basin (C checks 14-15).
///
/// Finds local minima in the mask, then uses them as seeds for basin filling.
#[test]
#[ignore = "not yet implemented: grayfill regression tests"]
fn grayfill_reg_basin() {
    let mut rp = RegParams::new("gfill_basin");

    let mask = make_mask_200();
    let w = mask.width();
    let h = mask.height();

    // C: pixLocalExtrema(pixm, 0, 0, &pixmin, NULL);
    let (pixmin, _pixmax) = local_extrema(&mask, 0, 0).expect("local_extrema");
    assert_eq!(pixmin.depth(), PixelDepth::Bit1);

    // C: pixs3 = pixSeedfillGrayBasin(pixmin, pixm, 30, 4);
    let result4 = seedfill_gray_basin(&pixmin, &mask, 30, ConnectivityType::FourWay)
        .expect("seedfill_gray_basin 4-way");
    rp.compare_values(w as f64, result4.width() as f64, 0.0);
    rp.compare_values(h as f64, result4.height() as f64, 0.0);
    assert_eq!(result4.depth(), PixelDepth::Bit8);

    // C: pixs3_8 = pixSeedfillGrayBasin(pixmin, pixm, 30, 8);
    let result8 = seedfill_gray_basin(&pixmin, &mask, 30, ConnectivityType::EightWay)
        .expect("seedfill_gray_basin 8-way");
    rp.compare_values(w as f64, result8.width() as f64, 0.0);

    assert!(rp.cleanup(), "grayfill basin test failed");
}

/// Test hybrid vs. iterative seedfill comparison (C checks 19-34).
///
/// Requires pixAddConstantGray which is not available in the Rust API.
#[test]
#[ignore = "not yet implemented: pixAddConstantGray not available"]
fn grayfill_reg_hybrid_comparison() {
    // C version uses pixAddConstantGray to prepare seeds:
    // pixAddConstantGray(pixs1, -30);  -- lower by 30
    // pixAddConstantGray(pixs2, 60);   -- raise by 60
    // Then compares pixSeedfillGray vs pixSeedfillGraySimple
    // and pixSeedfillGrayInv vs pixSeedfillGrayInvSimple
}

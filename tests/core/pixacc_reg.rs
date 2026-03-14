//! Pixel accumulator regression test
//!
//! Tests PixAcc creation, arithmetic operations, and finalization.
//!
//! # See also
//!
//! C Leptonica: `prog/pixacc_reg.c` (not a separate
//! regression test in C, but functionality from pixacc.c)

use leptonica::core::pixacc::PixAcc;
use leptonica::{Pix, PixelDepth};

/// Test PixAcc creation and basic finalization.
#[test]
fn pixacc_create() {
    let acc = PixAcc::create(10, 10, false).unwrap();
    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.height(), 10);
    // Empty accumulator should give all zeros
    assert_eq!(result.get_pixel(0, 0), Some(0));
}

/// Test PixAcc creation from Pix.
#[test]
fn pixacc_create_from_pix() {
    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_pixel(5, 5, 128).unwrap();
    let pix: Pix = pix.into();

    let acc = PixAcc::create_from_pix(&pix, false).unwrap();
    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(5, 5), Some(128));
    assert_eq!(result.get_pixel(0, 0), Some(0));
}

/// Test PixAcc add and subtract.
#[test]
fn pixacc_add_subtract() {
    let mut acc = PixAcc::create(10, 10, false).unwrap();

    let mut pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix1.set_pixel(0, 0, 100).unwrap();
    let pix1: Pix = pix1.into();

    acc.add(&pix1).unwrap();
    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(0, 0), Some(100));

    // Add again
    acc.add(&pix1).unwrap();
    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(0, 0), Some(200));

    // Subtract
    acc.subtract(&pix1).unwrap();
    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(0, 0), Some(100));
}

/// Test PixAcc with negative flag.
#[test]
fn pixacc_negative_flag() {
    let acc = PixAcc::create(10, 10, true).unwrap();
    assert!(acc.offset() > 0);

    let result = acc.finish(PixelDepth::Bit8).unwrap();
    // With offset, empty accumulator should still give zero after finalization
    assert_eq!(result.get_pixel(0, 0), Some(0));
}

/// Test PixAcc mult_const.
#[test]
fn pixacc_mult_const() {
    let mut acc = PixAcc::create(10, 10, false).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_pixel(0, 0, 50).unwrap();
    let pix: Pix = pix.into();

    acc.add(&pix).unwrap();
    acc.mult_const(2.0).unwrap();

    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(0, 0), Some(100));
}

/// Test PixAcc mult_const_accumulate.
#[test]
fn pixacc_mult_const_accumulate() {
    let mut acc = PixAcc::create(10, 10, false).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_pixel(0, 0, 40).unwrap();
    let pix: Pix = pix.into();

    acc.mult_const_accumulate(&pix, 3.0).unwrap();

    let result = acc.finish(PixelDepth::Bit8).unwrap();
    assert_eq!(result.get_pixel(0, 0), Some(120));
}

/// Test PixAcc get_pix returns current state.
#[test]
fn pixacc_get_pix() {
    let mut acc = PixAcc::create(10, 10, false).unwrap();

    let mut pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap().to_mut();
    pix.set_pixel(3, 3, 77).unwrap();
    let pix: Pix = pix.into();

    acc.add(&pix).unwrap();

    let current = acc.get_pix();
    assert_eq!(current.get_pixel(3, 3), Some(77));
    assert_eq!(current.get_pixel(0, 0), Some(0));
}

/// Test PixAcc size mismatch error.
#[test]
fn pixacc_size_mismatch() {
    let mut acc = PixAcc::create(10, 10, false).unwrap();
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    assert!(acc.add(&pix).is_err());
}

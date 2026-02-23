//! Low-level pixel access regression test
//!
//! Tests low-level pixel get/set operations at 1, 2, 4, 8, 16, and 32 bpp
//! using both high-level (get_pixel/set_pixel) and low-level
//! (get_data_bit, get_data_dibit, etc.) access functions.
//!
//! Verifies that reading back a written pixel value returns the original value.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/lowaccess_reg.c`

use leptonica_core::pix::{
    get_data_bit, get_data_byte, get_data_dibit, get_data_qbit, get_data_two_bytes, set_data_bit,
    set_data_byte, set_data_dibit, set_data_qbit, set_data_two_bytes,
};
use leptonica_core::{Pix, PixelDepth};
use leptonica_test::RegParams;

/// Test 1bpp get/set pixel via high-level API (C checks 0-3).
///
/// Creates a 1bpp image, sets pixels, reads them back, and verifies consistency.
#[test]
fn lowaccess_reg_1bpp() {
    let mut rp = RegParams::new("lowaccess_1bpp");

    let pix = leptonica_test::load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    assert_eq!(pix.depth(), PixelDepth::Bit1);

    let w = pix.width();
    let h = pix.height();

    // Count pixels via get_pixel
    let mut count_high = 0u64;
    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel(x, y).unwrap_or(0) != 0 {
                count_high += 1;
            }
        }
    }

    // Must match count_pixels
    let count_api = pix.count_pixels();
    rp.compare_values(count_api as f64, count_high as f64, 0.0);

    // Test set/get round-trip on a mutable copy
    let pix2 = Pix::new(16, 4, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix2.try_into_mut().expect("into_mut");
    for x in (0..16u32).step_by(2) {
        pm.set_pixel(x, 0, 1).expect("set pixel");
    }
    let pix3: Pix = pm.into();
    let mut alternating = 0u32;
    for x in (0..16u32).step_by(2) {
        alternating += pix3.get_pixel(x, 0).unwrap_or(0);
    }
    rp.compare_values(8.0, alternating as f64, 0.0);

    assert!(rp.cleanup(), "lowaccess 1bpp test failed");
}

/// Test low-level 1bpp access functions: get_data_bit, set_data_bit (C checks 4-7).
#[test]
fn lowaccess_reg_1bpp_lowlevel() {
    let mut rp = RegParams::new("lowaccess_1bpp_ll");

    let pix = Pix::new(32, 4, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set alternating bits via low-level API
    {
        let line = pm.row_data_mut(0);
        for x in (0..32u32).step_by(2) {
            set_data_bit(line, x, 1);
        }
    }

    // Convert to Pix for immutable read-back
    let pix_out: Pix = pm.into();

    // Verify via low-level get
    let set_count: u32 = {
        let line = pix_out.row_data(0);
        (0..32u32).step_by(2).map(|x| get_data_bit(line, x)).sum()
    };
    rp.compare_values(16.0, set_count as f64, 0.0);

    // Verify unset bits are 0
    let unset_count: u32 = {
        let line = pix_out.row_data(0);
        (1..32u32).step_by(2).map(|x| get_data_bit(line, x)).sum()
    };
    rp.compare_values(0.0, unset_count as f64, 0.0);

    assert!(rp.cleanup(), "lowaccess 1bpp low-level test failed");
}

/// Test 2bpp low-level access (C checks 8-11).
#[test]
fn lowaccess_reg_2bpp() {
    let mut rp = RegParams::new("lowaccess_2bpp");

    let pix = Pix::new(32, 4, PixelDepth::Bit2).expect("new 2bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set 4 different pixel values: 0, 1, 2, 3 in a cycle
    {
        let line = pm.row_data_mut(0);
        for x in 0..32u32 {
            set_data_dibit(line, x, x % 4);
        }
    }

    // Convert to Pix for immutable read-back
    let pix_out: Pix = pm.into();

    // Read back and verify
    {
        let line = pix_out.row_data(0);
        let mut errors = 0u32;
        for x in 0..32u32 {
            let val = get_data_dibit(line, x);
            if val != x % 4 {
                errors += 1;
            }
        }
        rp.compare_values(0.0, errors as f64, 0.0);
    }

    assert!(rp.cleanup(), "lowaccess 2bpp test failed");
}

/// Test 4bpp low-level access (C checks 12-15).
#[test]
fn lowaccess_reg_4bpp() {
    let mut rp = RegParams::new("lowaccess_4bpp");

    let pix = Pix::new(32, 4, PixelDepth::Bit4).expect("new 4bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set 16 different pixel values: 0-15 in a cycle
    {
        let line = pm.row_data_mut(0);
        for x in 0..32u32 {
            set_data_qbit(line, x, x % 16);
        }
    }

    // Convert to Pix for immutable read-back
    let pix_out: Pix = pm.into();

    // Read back and verify
    {
        let line = pix_out.row_data(0);
        let mut errors = 0u32;
        for x in 0..32u32 {
            let val = get_data_qbit(line, x);
            if val != x % 16 {
                errors += 1;
            }
        }
        rp.compare_values(0.0, errors as f64, 0.0);
    }

    assert!(rp.cleanup(), "lowaccess 4bpp test failed");
}

/// Test 8bpp low-level access (C checks 16-19).
#[test]
fn lowaccess_reg_8bpp() {
    let mut rp = RegParams::new("lowaccess_8bpp");

    let pix = Pix::new(64, 4, PixelDepth::Bit8).expect("new 8bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set byte values 0-255 in cycle
    {
        let line = pm.row_data_mut(0);
        for x in 0..64u32 {
            set_data_byte(line, x, x % 256);
        }
    }

    // Convert to Pix for immutable read-back
    let pix_out: Pix = pm.into();

    // Read back and verify
    {
        let line = pix_out.row_data(0);
        let mut errors = 0u32;
        for x in 0..64u32 {
            let val = get_data_byte(line, x);
            if val != x % 256 {
                errors += 1;
            }
        }
        rp.compare_values(0.0, errors as f64, 0.0);
    }

    assert!(rp.cleanup(), "lowaccess 8bpp test failed");
}

/// Test 16bpp low-level access (C checks 20-23).
#[test]
fn lowaccess_reg_16bpp() {
    let mut rp = RegParams::new("lowaccess_16bpp");

    let pix = Pix::new(16, 4, PixelDepth::Bit16).expect("new 16bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set two-byte values
    {
        let line = pm.row_data_mut(0);
        for x in 0..16u32 {
            set_data_two_bytes(line, x, x * 1000);
        }
    }

    // Convert to Pix for immutable read-back
    let pix_out: Pix = pm.into();

    // Read back and verify
    {
        let line = pix_out.row_data(0);
        let mut errors = 0u32;
        for x in 0..16u32 {
            let val = get_data_two_bytes(line, x);
            if val != x * 1000 {
                errors += 1;
            }
        }
        rp.compare_values(0.0, errors as f64, 0.0);
    }

    assert!(rp.cleanup(), "lowaccess 16bpp test failed");
}

/// Test 32bpp pixel access (C checks 24-27).
///
/// Uses high-level get_pixel/set_pixel for 32bpp (RGBA) images.
#[test]
fn lowaccess_reg_32bpp() {
    let mut rp = RegParams::new("lowaccess_32bpp");

    let pix = Pix::new(16, 4, PixelDepth::Bit32).expect("new 32bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");

    // Set RGBA values
    for x in 0..16u32 {
        let val = 0xFF000000 | (x * 16) << 16 | (x * 8) << 8 | x;
        pm.set_pixel(x, 0, val).expect("set pixel");
    }

    let pix_out: Pix = pm.into();

    // Read back and verify
    let mut errors = 0u32;
    for x in 0..16u32 {
        let expected = 0xFF000000 | (x * 16) << 16 | (x * 8) << 8 | x;
        let actual = pix_out.get_pixel(x, 0).unwrap_or(u32::MAX);
        if actual != expected {
            errors += 1;
        }
    }
    rp.compare_values(0.0, errors as f64, 0.0);

    assert!(rp.cleanup(), "lowaccess 32bpp test failed");
}

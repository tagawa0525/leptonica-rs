//! Pix allocation regression test
//!
//! Covers allocation, mutable ownership, and copy-on-write behavior.
//!
//! # See also
//!
//! C Leptonica: `prog/pixalloc_reg.c`

use crate::common::RegParams;
use leptonica::{Pix, PixelDepth};

#[test]
fn pixalloc_reg() {
    let mut rp = RegParams::new("pixalloc");

    let pix = Pix::new(24, 16, PixelDepth::Bit8).expect("create pix");
    rp.compare_values(24.0, pix.width() as f64, 0.0);
    rp.compare_values(16.0, pix.height() as f64, 0.0);

    // Shared reference cannot be turned into unique mutable ownership.
    rp.compare_values(
        1.0,
        if pix.clone().try_into_mut().is_err() {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let mut pm = pix.try_into_mut().expect("unique mutable pix");
    pm.set_pixel(3, 4, 123).expect("set pixel");
    let pix_unique: Pix = pm.into();
    rp.compare_values(
        123.0,
        pix_unique.get_pixel(3, 4).expect("get pixel") as f64,
        0.0,
    );

    // to_mut() creates a copy; source remains unchanged.
    let pix_src = Pix::new(10, 10, PixelDepth::Bit8).expect("create src");
    let mut pm_copy = pix_src.to_mut();
    pm_copy.set_pixel(1, 1, 200).expect("set copied pixel");
    let pix_copy: Pix = pm_copy.into();

    rp.compare_values(0.0, pix_src.get_pixel(1, 1).expect("src pixel") as f64, 0.0);
    rp.compare_values(
        200.0,
        pix_copy.get_pixel(1, 1).expect("copy pixel") as f64,
        0.0,
    );

    assert!(rp.cleanup(), "pixalloc regression test failed");
}

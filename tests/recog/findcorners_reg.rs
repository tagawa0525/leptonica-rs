//! Find-corners regression test
//!
//! Uses checkerboard corner detection as the Rust mapping.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/findcorners_reg.c`

use crate::common::RegParams;
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

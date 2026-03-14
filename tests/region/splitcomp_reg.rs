//! Split-component regression test
//!
//! Covers splitting binary images into connected components.
//!
//! # See also
//!
//! C Leptonica: `prog/splitcomp_reg.c`

use crate::common::RegParams;
use leptonica::region::conncomp::count_conn_comp;
use leptonica::region::{ConnectivityType, conncomp_pixa};
use leptonica::{Pix, PixelDepth};

fn make_two_components() -> Pix {
    let pix = Pix::new(40, 20, PixelDepth::Bit1).expect("create image");
    let mut pm = pix.try_into_mut().expect("mutable image");

    for y in 3..10u32 {
        for x in 3..12u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    for y in 8..17u32 {
        for x in 24..36u32 {
            pm.set_pixel_unchecked(x, y, 1);
        }
    }
    pm.into()
}

#[test]
fn splitcomp_reg() {
    let mut rp = RegParams::new("splitcomp");

    let pix = make_two_components();
    let count = count_conn_comp(&pix, ConnectivityType::FourWay).expect("count_conn_comp");
    rp.compare_values(2.0, count as f64, 0.0);

    let (boxa, pixa) = conncomp_pixa(&pix, ConnectivityType::FourWay).expect("conncomp_pixa");
    rp.compare_values(2.0, boxa.len() as f64, 0.0);
    rp.compare_values(2.0, pixa.len() as f64, 0.0);

    let b0 = boxa.get(0).expect("box0");
    let b1 = boxa.get(1).expect("box1");
    rp.compare_values(
        1.0,
        if b0.x != b1.x || b0.y != b1.y {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    assert!(rp.cleanup(), "splitcomp regression test failed");
}

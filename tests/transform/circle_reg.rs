//! Circle regression test
//!
//! Tests extraction of digits embedded within circular regions using
//! erosion and connected-component counting. The C version reads a
//! Pixa archive of pre-rendered circles, uses seedfill to isolate
//! circle interiors, and counts components at each erosion step.
//!
//! Not yet migrated: the test data file circles.pa (Pixa archive) is
//! not present in the test data directory.
//!
//! # See also
//!
//! C Leptonica: `prog/circle_reg.c`

use leptonica::morph::erode_brick;
use leptonica::region::{ConnectivityType, conncomp::count_conn_comp};
use leptonica::{Pix, PixelDepth};

/// Circle-like extraction smoke test for benchmark mapping.
#[test]
fn circle_reg_smoke() {
    let pix = Pix::new(64, 64, PixelDepth::Bit1).expect("create image");
    let mut pm = pix.try_into_mut().expect("mutable image");

    for y in 0..64u32 {
        for x in 0..64u32 {
            let dx = x as i32 - 32;
            let dy = y as i32 - 32;
            if dx * dx + dy * dy <= 20 * 20 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
    }
    let pix = pm.into();

    let eroded = erode_brick(&pix, 3, 3).expect("erode_brick");
    let n = count_conn_comp(&eroded, ConnectivityType::EightWay).expect("count_conn_comp");
    assert!(n >= 1);
}

/// Test circle extraction using erosion and connected components (C checks 0-1).
///
/// Requires circles.pa Pixa archive which is not present in test data.
#[test]
#[ignore = "not yet implemented: circles.pa test data not available"]
fn circle_reg_extract_circles() {
    // C version:
    // 1. pixaRead("circles.pa") to load pre-rendered circle images
    // 2. pixInvert + pixCreateTemplate + pixSetOrClearBorder + pixSeedfillBinary
    //    to isolate circle interior
    // 3. pixAnd + pixCountConnComp to count components in and around circles
    // 4. pixErodeBrick to find boundary transitions
    // 5. regTestCompareValues() for component counts at erosion thresholds
}

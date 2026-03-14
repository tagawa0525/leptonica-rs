//! Gray morphology 2 regression test
//!
//! C version: prog/graymorph2_reg.c
//! Tests gray morphological operations with 3x1, 1x3, and 3x3 sizes.
//!
//! C版は pixDilateGray3 vs pixDilateGray 等の等価比較（12 compare_pix）。
//! Rust版は _3 最適化バリアントがないため、一般実装結果のgolden化 +
//! open/close/tophat/hdome の追加テスト。
//!
//! C checkpoint mapping (12 total):
//!   0-2:  compare_pix dilateGray3 vs dilateGray (3x1, 1x3, 3x3)
//!   3-5:  compare_pix erodeGray3 vs erodeGray (3x1, 1x3, 3x3)
//!   6-8:  compare_pix openGray3 vs openGray (3x1, 1x3, 3x3)
//!   9-11: compare_pix closeGray3 vs closeGray (3x1, 1x3, 3x3)
//!
//! Rust追加:
//!   write_pix_and_check: dilate/erode/open/close 各サイズ結果

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::morph::{close_gray, dilate_gray, erode_gray, open_gray};

/// Test dilate_gray and erode_gray with golden checks (C checks 0-5 equivalent).
#[test]
fn graymorph2_reg_dilate_erode() {
    let mut rp = RegParams::new("gmorph2_dilate_erode");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);

    let orig_mean = pix.average_in_rect(None).expect("average_in_rect") as f64;

    let sizes: &[(u32, u32)] = &[(3, 1), (1, 3), (3, 3)];
    for &(hsize, vsize) in sizes {
        // Dilation
        let dilated = dilate_gray(&pix, hsize, vsize).expect("dilate_gray");
        let dil_mean = dilated.average_in_rect(None).expect("average dilated") as f64;
        rp.compare_values(1.0, if dil_mean >= orig_mean { 1.0 } else { 0.0 }, 0.0);
        rp.write_pix_and_check(&dilated, ImageFormat::Jpeg)
            .expect("write dilate_gray result");

        // Erosion
        let eroded = erode_gray(&pix, hsize, vsize).expect("erode_gray");
        let ero_mean = eroded.average_in_rect(None).expect("average eroded") as f64;
        rp.compare_values(1.0, if ero_mean <= orig_mean { 1.0 } else { 0.0 }, 0.0);
        rp.write_pix_and_check(&eroded, ImageFormat::Jpeg)
            .expect("write erode_gray result");
    }

    assert!(rp.cleanup(), "graymorph2 dilate_erode test failed");
}

/// Test open_gray and close_gray with golden checks (C checks 6-11 equivalent).
#[test]
fn graymorph2_reg_open_close() {
    let mut rp = RegParams::new("gmorph2_open_close");

    let pix = crate::common::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);

    let orig_mean = pix.average_in_rect(None).expect("average_in_rect") as f64;

    let sizes: &[(u32, u32)] = &[(3, 1), (1, 3), (3, 3)];
    for &(hsize, vsize) in sizes {
        // Opening: anti-extensive (mean <= original)
        let opened = open_gray(&pix, hsize, vsize).expect("open_gray");
        let open_mean = opened.average_in_rect(None).expect("average opened") as f64;
        rp.compare_values(1.0, if open_mean <= orig_mean { 1.0 } else { 0.0 }, 0.0);
        rp.write_pix_and_check(&opened, ImageFormat::Jpeg)
            .expect("write open_gray result");

        // Closing: extensive (mean >= original)
        let closed = close_gray(&pix, hsize, vsize).expect("close_gray");
        let close_mean = closed.average_in_rect(None).expect("average closed") as f64;
        rp.compare_values(1.0, if close_mean >= orig_mean { 1.0 } else { 0.0 }, 0.0);
        rp.write_pix_and_check(&closed, ImageFormat::Jpeg)
            .expect("write close_gray result");
    }

    assert!(rp.cleanup(), "graymorph2 open_close test failed");
}

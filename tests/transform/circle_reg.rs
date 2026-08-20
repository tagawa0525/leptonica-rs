//! Circle regression test
//!
//! Tests extraction of digits embedded within circular regions using
//! erosion and connected-component counting. The C version reads a
//! Pixa archive of pre-rendered circles, uses seedfill to isolate
//! circle interiors, and counts components at each erosion step.
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

/// C-compat: `prog/circle_reg.c`, all 13 outputs.
///
/// `circles.pa` is a serialized pixa of lossless PNGs and C writes every
/// output as PNG, so the whole program is bit-exactly comparable. For each
/// circle it fills the outside, erodes the disk step by step, and picks the
/// erosion where the intersection with the source stops fragmenting.
#[test]
fn circle_c_compat() {
    use crate::common::RegParams;
    use leptonica::core::Pixa;
    use leptonica::core::pix::InitColor;
    use leptonica::io::ImageFormat;
    use leptonica::morph::erode_brick;
    use leptonica::region::{ConnectivityType, count_conn_comp, seedfill_binary_restricted};
    use leptonica::{Pix, PixelDepth};

    if crate::common::is_display_mode() {
        return;
    }

    const NUM_ERODES: usize = 8;

    let mut rp = RegParams::new("circle_c");

    let path = crate::common::test_data_path("circles.pa");
    let pixas = Pixa::read_from_file(&path).expect("read circles.pa");
    let mut pixa2 = Pixa::new();

    for k in 0..pixas.len() {
        let mut pixa1 = Pixa::new();
        let pixs = pixas.get(k).expect("circle pix").clone();
        pixa1.push(pixs.clone());

        // C: fill in from the border of the inverted image, then invert back,
        // which leaves the filled disk that encloses the drawing.
        let pixsi = pixs.invert();
        let pixc = Pix::new(pixs.width(), pixs.height(), PixelDepth::Bit1).expect("template");
        let pixc = {
            let mut m = pixc.try_into_mut().expect("into mut");
            m.set_or_clear_border(1, 1, 1, 1, InitColor::Black);
            Pix::from(m)
        };
        let pixc = seedfill_binary_restricted(&pixc, &pixsi, ConnectivityType::FourWay, 0, 0)
            .expect("seedfill");
        let mut pixc = pixc.invert();
        let pixoc = pixc.clone(); // the original circle

        pixa1.push(pixoc.clone());

        let mut counts = Vec::with_capacity(NUM_ERODES);
        let pix1 = pixs.and(&pixc).expect("and");
        counts.push(count_conn_comp(&pix1, ConnectivityType::EightWay).expect("count"));
        pixa1.push(pix1);

        for _ in 1..NUM_ERODES {
            pixc = erode_brick(&pixc, 3, 3).expect("erode");
            let pix1 = pixs.and(&pixc).expect("and");
            counts.push(count_conn_comp(&pix1, ConnectivityType::EightWay).expect("count"));
            pixa1.push(pix1);
        }

        // C: the most fragmented erosion, then the first erosion past it that
        // is back to the fewest components.
        let mut maxval = 0;
        let mut maxloc = 0;
        for (i, &count) in counts.iter().enumerate().take(NUM_ERODES).skip(1) {
            if count > maxval {
                maxval = count;
                maxloc = i;
            }
        }
        // When the peak is the last erosion, C's two search loops never run
        // and `i` keeps the loop-exit value `num_erodes`, giving a final
        // erosion one step past everything measured. Reproduce that rather
        // than clamping.
        let minval = counts[maxloc + 1..NUM_ERODES]
            .iter()
            .copied()
            .min()
            .unwrap_or(1000);
        let i = (maxloc + 1..NUM_ERODES)
            .find(|&i| counts[i] == minval)
            .unwrap_or(NUM_ERODES);

        let size = 2 * i as u32 + 1;
        let pix1 = erode_brick(&pixoc, size, size).expect("final erode");
        let pix2 = pixs.and(&pix1).expect("final and");
        pixa1.push(pix2);

        let pix3 = pixa1
            .display_tiled_in_columns(11, 1.0, 10, 2)
            .expect("tile circle");
        rp.write_pix_and_check(&pix3, ImageFormat::Png)
            .expect("check: circle erosion series");
        pixa2.push(pix3);
    }

    let pix1 = pixa2
        .display_tiled_in_columns(1, 1.0, 10, 0)
        .expect("tile all circles");
    rp.write_pix_and_check(&pix1, ImageFormat::Png)
        .expect("check: circle summary");

    assert!(rp.cleanup(), "circle C-compat test failed");
}

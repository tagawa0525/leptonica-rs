//! Pixa selection regression tests (plan 106 / C pixafunc1.c)
//!
//! Covers select_range, select_with_indicator, select_with_string,
//! select_by_num_conn_comp / area_fraction / perim_size_ratio /
//! perim_to_area_ratio / width_height_ratio, plus the Pix-level wrappers
//! `pix_select_by_*` and `pix_add_with_indicator` / `pix_remove_with_indicator`.

use leptonica::core::pixa::{
    ThresholdSelect, pix_add_with_indicator, pix_remove_with_indicator,
    pix_select_by_area_fraction, pix_select_by_perim_size_ratio, pix_select_by_perim_to_area_ratio,
    pix_select_by_width_height_ratio,
};
use leptonica::region::ConnectivityType;
use leptonica::{Pix, Pixa, PixelDepth};

fn make_filled_1bpp(w: u32, h: u32, fg_ratio: f32) -> Pix {
    let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
    let mut m = pix.try_into_mut().unwrap();
    let total = (w * h) as f32;
    let target = (total * fg_ratio) as u32;
    let mut set = 0u32;
    for y in 0..h {
        for x in 0..w {
            if set >= target {
                break;
            }
            m.set_pixel(x, y, 1).unwrap();
            set += 1;
        }
    }
    m.into()
}

fn make_empty_1bpp(w: u32, h: u32) -> Pix {
    Pix::new(w, h, PixelDepth::Bit1).unwrap()
}

#[test]
fn select_range_basic() {
    let mut pa = Pixa::new();
    for _ in 0..5 {
        pa.push(make_empty_1bpp(8, 8));
    }
    let r = pa.select_range(1, Some(3));
    assert_eq!(r.pix_slice().len(), 3);
}

#[test]
fn select_range_open_end() {
    let mut pa = Pixa::new();
    for _ in 0..4 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let r = pa.select_range(2, None);
    assert_eq!(r.pix_slice().len(), 2);
}

#[test]
fn select_range_out_of_bounds_returns_empty() {
    let mut pa = Pixa::new();
    pa.push(make_empty_1bpp(4, 4));
    let r = pa.select_range(10, None);
    assert!(r.pix_slice().is_empty());
}

#[test]
fn select_range_last_lt_first_returns_empty() {
    let mut pa = Pixa::new();
    for _ in 0..4 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let r = pa.select_range(2, Some(1));
    assert!(r.pix_slice().is_empty());
}

#[test]
fn select_range_usize_max_last_saturates() {
    let mut pa = Pixa::new();
    for _ in 0..3 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let r = pa.select_range(0, Some(usize::MAX));
    assert_eq!(r.pix_slice().len(), 3);
}

#[test]
fn select_with_indicator_filters_correctly() {
    let mut pa = Pixa::new();
    for _ in 0..3 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let (filtered, changed) = pa.select_with_indicator(&[true, false, true]).unwrap();
    assert_eq!(filtered.pix_slice().len(), 2);
    assert!(changed);
}

#[test]
fn select_with_indicator_all_kept_not_changed() {
    let mut pa = Pixa::new();
    for _ in 0..2 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let (filtered, changed) = pa.select_with_indicator(&[true, true]).unwrap();
    assert_eq!(filtered.pix_slice().len(), 2);
    assert!(!changed);
}

#[test]
fn select_with_indicator_length_mismatch_errors() {
    let mut pa = Pixa::new();
    pa.push(make_empty_1bpp(4, 4));
    let r = pa.select_with_indicator(&[true, false]);
    assert!(r.is_err());
}

#[test]
fn select_with_string_parses_zero_one() {
    let mut pa = Pixa::new();
    for _ in 0..4 {
        pa.push(make_empty_1bpp(4, 4));
    }
    let (filtered, changed) = pa.select_with_string("1010").unwrap();
    assert_eq!(filtered.pix_slice().len(), 2);
    assert!(changed);
}

#[test]
fn select_with_string_length_mismatch_errors() {
    let mut pa = Pixa::new();
    pa.push(make_empty_1bpp(4, 4));
    assert!(pa.select_with_string("10").is_err());
}

#[test]
fn select_by_area_fraction_greater_than() {
    let mut pa = Pixa::new();
    pa.push(make_filled_1bpp(10, 10, 0.1)); // 10%
    pa.push(make_filled_1bpp(10, 10, 0.5)); // 50%
    pa.push(make_filled_1bpp(10, 10, 0.9)); // 90%
    let (out, changed) = pa
        .select_by_area_fraction(0.4, ThresholdSelect::GreaterThan)
        .unwrap();
    assert_eq!(out.pix_slice().len(), 2); // 0.5 and 0.9 kept
    assert!(changed);
}

#[test]
fn select_by_area_fraction_less_than() {
    let mut pa = Pixa::new();
    pa.push(make_filled_1bpp(8, 8, 0.2));
    pa.push(make_filled_1bpp(8, 8, 0.8));
    let (out, _) = pa
        .select_by_area_fraction(0.5, ThresholdSelect::LessThan)
        .unwrap();
    assert_eq!(out.pix_slice().len(), 1);
}

#[test]
fn select_by_width_height_ratio() {
    let mut pa = Pixa::new();
    pa.push(make_empty_1bpp(20, 10)); // ratio 2.0
    pa.push(make_empty_1bpp(10, 10)); // ratio 1.0
    pa.push(make_empty_1bpp(5, 10)); // ratio 0.5
    let (out, changed) = pa
        .select_by_width_height_ratio(1.0, ThresholdSelect::GreaterThan)
        .unwrap();
    assert_eq!(out.pix_slice().len(), 1); // only 20x10
    assert!(changed);
}

#[test]
fn select_by_num_conn_comp_one_component() {
    // Single 1bpp image with two separate components → pick those with count in [2, 2]
    let pa1 = {
        let p = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let mut m = p.try_into_mut().unwrap();
        m.set_pixel(1, 1, 1).unwrap();
        m.set_pixel(10, 10, 1).unwrap();
        let p: Pix = m.into();
        p
    };
    let pa_zero = make_empty_1bpp(8, 8);
    let mut pa = Pixa::new();
    pa.push(pa1);
    pa.push(pa_zero);
    let (out, _) = pa
        .select_by_num_conn_comp(2, 2, ConnectivityType::EightWay)
        .unwrap();
    // First image has 2 components, second has 0 → only first kept.
    assert_eq!(out.pix_slice().len(), 1);
}

#[test]
fn select_by_num_conn_comp_invalid_range_errors() {
    let mut pa = Pixa::new();
    pa.push(make_empty_1bpp(4, 4));
    let r = pa.select_by_num_conn_comp(5, 2, ConnectivityType::EightWay);
    assert!(r.is_err());
}

#[test]
fn select_by_perim_to_area_ratio_runs() {
    let mut pa = Pixa::new();
    pa.push(make_filled_1bpp(10, 10, 0.5));
    let (out, _) = pa
        .select_by_perim_to_area_ratio(2.0, ThresholdSelect::LessThan)
        .unwrap();
    // ratio is small (large blob), so it should be kept under threshold 2.0
    assert_eq!(out.pix_slice().len(), 1);
}

#[test]
fn pix_select_by_area_fraction_wrapper() {
    let p = make_filled_1bpp(16, 16, 0.5);
    let r = pix_select_by_area_fraction(
        &p,
        0.001,
        ConnectivityType::EightWay,
        ThresholdSelect::GreaterThan,
    )
    .unwrap();
    assert_eq!(r.width(), 16);
    assert_eq!(r.height(), 16);
}

#[test]
fn pix_select_by_width_height_ratio_wrapper() {
    let p = make_filled_1bpp(16, 8, 1.0);
    let r = pix_select_by_width_height_ratio(
        &p,
        0.5,
        ConnectivityType::EightWay,
        ThresholdSelect::GreaterThan,
    )
    .unwrap();
    assert_eq!(r.width(), 16);
}

#[test]
fn pix_select_by_perim_size_ratio_wrapper() {
    let p = make_filled_1bpp(16, 16, 0.5);
    let r = pix_select_by_perim_size_ratio(
        &p,
        0.0,
        ConnectivityType::EightWay,
        ThresholdSelect::GreaterOrEqual,
    )
    .unwrap();
    assert_eq!(r.width(), 16);
}

#[test]
fn pix_select_by_perim_to_area_ratio_wrapper() {
    let p = make_filled_1bpp(16, 16, 1.0);
    let r = pix_select_by_perim_to_area_ratio(
        &p,
        10.0,
        ConnectivityType::EightWay,
        ThresholdSelect::LessThan,
    )
    .unwrap();
    assert_eq!(r.width(), 16);
}

#[test]
fn pix_add_with_indicator_runs() {
    let pixs = make_filled_1bpp(16, 16, 0.3);
    let dst = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
    let mut dst_mut = dst.try_into_mut().unwrap();
    // Compute number of components to size indicator correctly
    let count = leptonica::region::count_conn_comp(&pixs, ConnectivityType::EightWay).unwrap();
    let indicator: Vec<bool> = (0..count as usize).map(|_| true).collect();
    pix_add_with_indicator(&pixs, &mut dst_mut, &indicator).unwrap();
    let out: Pix = dst_mut.into();
    assert!(out.count_pixels() > 0);
}

#[test]
fn pix_remove_with_indicator_clears() {
    let pixs = make_filled_1bpp(16, 16, 0.5);
    let count = leptonica::region::count_conn_comp(&pixs, ConnectivityType::EightWay).unwrap();
    // Build a fresh destination buffer with the same fg pattern as pixs.
    let dst = make_filled_1bpp(16, 16, 0.5);
    let mut dst_mut = dst.try_into_mut().unwrap();
    let indicator: Vec<bool> = (0..count as usize).map(|_| true).collect();
    pix_remove_with_indicator(&pixs, &mut dst_mut, &indicator).unwrap();
    let out: Pix = dst_mut.into();
    assert_eq!(out.count_pixels(), 0);
}

#[test]
fn pix_select_rejects_non_1bpp() {
    let p = Pix::new(16, 16, PixelDepth::Bit8).unwrap();
    let r = pix_select_by_area_fraction(
        &p,
        0.5,
        ConnectivityType::EightWay,
        ThresholdSelect::GreaterThan,
    );
    assert!(r.is_err());
}

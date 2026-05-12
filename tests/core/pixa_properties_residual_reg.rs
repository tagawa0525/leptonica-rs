//! Regression tests for plan 121 (Pixa property residual 3 関数).

use leptonica::core::numa::Numaa;
use leptonica::core::pixa::{SizeIndicatorAxis, ThresholdSelect};
use leptonica::{Numa, Pix, Pixa, PixelDepth};

fn make_pixa(items: &[(u32, u32)]) -> Pixa {
    let mut pa = Pixa::with_capacity(items.len());
    for &(w, h) in items {
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        pa.push(pix);
    }
    pa
}

// -- make_size_indicator -----------------------------------------------

#[test]
fn size_indicator_width_greater_than() {
    let pa = make_pixa(&[(5, 5), (15, 5), (10, 5)]);
    let na = pa.make_size_indicator(
        10,
        0,
        SizeIndicatorAxis::Width,
        ThresholdSelect::GreaterThan,
    );
    assert_eq!(na.get(0).unwrap(), 0.0);
    assert_eq!(na.get(1).unwrap(), 1.0);
    assert_eq!(na.get(2).unwrap(), 0.0); // equal, not greater
}

#[test]
fn size_indicator_height_lte() {
    let pa = make_pixa(&[(5, 5), (15, 10), (10, 3)]);
    let na = pa.make_size_indicator(
        0,
        5,
        SizeIndicatorAxis::Height,
        ThresholdSelect::LessOrEqual,
    );
    assert_eq!(na.get(0).unwrap(), 1.0);
    assert_eq!(na.get(1).unwrap(), 0.0);
    assert_eq!(na.get(2).unwrap(), 1.0);
}

#[test]
fn size_indicator_if_both() {
    // Pix matches only when both w > 10 AND h > 5.
    let pa = make_pixa(&[(15, 10), (5, 10), (15, 3), (20, 8)]);
    let na = pa.make_size_indicator(
        10,
        5,
        SizeIndicatorAxis::IfBoth,
        ThresholdSelect::GreaterThan,
    );
    assert_eq!(na.get(0).unwrap(), 1.0);
    assert_eq!(na.get(1).unwrap(), 0.0);
    assert_eq!(na.get(2).unwrap(), 0.0);
    assert_eq!(na.get(3).unwrap(), 1.0);
}

#[test]
fn size_indicator_if_either() {
    let pa = make_pixa(&[(15, 4), (5, 10), (5, 4)]);
    let na = pa.make_size_indicator(
        10,
        5,
        SizeIndicatorAxis::IfEither,
        ThresholdSelect::GreaterThan,
    );
    assert_eq!(na.get(0).unwrap(), 1.0);
    assert_eq!(na.get(1).unwrap(), 1.0);
    assert_eq!(na.get(2).unwrap(), 0.0);
}

// -- sort_2d_by_index --------------------------------------------------

#[test]
fn sort_2d_by_index_basic_split() {
    let pa = make_pixa(&[(4, 4), (5, 5), (6, 6)]);
    let mut naa = Numaa::new();
    naa.push(Numa::from_vec(vec![0.0, 2.0]));
    naa.push(Numa::from_vec(vec![1.0]));
    let paa = pa.sort_2d_by_index(&naa).unwrap();
    assert_eq!(paa.len(), 2);
    assert_eq!(paa.get(0).unwrap().pix_slice().len(), 2);
    assert_eq!(paa.get(1).unwrap().pix_slice().len(), 1);
    // First inner Pixa: indices 0 and 2.
    assert_eq!(paa.get(0).unwrap().pix_slice()[0].width(), 4);
    assert_eq!(paa.get(0).unwrap().pix_slice()[1].width(), 6);
    // Second inner Pixa: index 1.
    assert_eq!(paa.get(1).unwrap().pix_slice()[0].width(), 5);
}

#[test]
fn sort_2d_by_index_count_mismatch_errors() {
    let pa = make_pixa(&[(4, 4), (5, 5)]);
    let mut naa = Numaa::new();
    naa.push(Numa::from_vec(vec![0.0, 1.0, 2.0])); // 3 indices for 2 entries
    assert!(pa.sort_2d_by_index(&naa).is_err());
}

#[test]
fn sort_2d_by_index_out_of_range_errors() {
    let pa = make_pixa(&[(4, 4), (5, 5)]);
    let mut naa = Numaa::new();
    naa.push(Numa::from_vec(vec![0.0, 5.0])); // 5 out of range
    assert!(pa.sort_2d_by_index(&naa).is_err());
}

// -- constrained_select ------------------------------------------------

#[test]
fn constrained_select_basic() {
    let pa = make_pixa(&[(4, 4), (5, 5), (6, 6), (7, 7), (8, 8)]);
    let out = pa.constrained_select(0, 4, 3, false).unwrap();
    // 3 indices evenly spread across [0..=4]: 0, 2, 4.
    assert_eq!(out.pix_slice().len(), 3);
    assert_eq!(out.pix_slice()[0].width(), 4);
    assert_eq!(out.pix_slice()[1].width(), 6);
    assert_eq!(out.pix_slice()[2].width(), 8);
}

#[test]
fn constrained_select_negative_last_means_full_range() {
    let pa = make_pixa(&[(4, 4), (5, 5), (6, 6)]);
    let out = pa.constrained_select(0, -1, 3, false).unwrap();
    assert_eq!(out.pix_slice().len(), 3);
}

#[test]
fn constrained_select_invalid_errors() {
    let pa = make_pixa(&[(4, 4), (5, 5)]);
    // first > last
    assert!(pa.constrained_select(3, 1, 2, false).is_err());
    // nmax < 1
    assert!(pa.constrained_select(0, 1, 0, false).is_err());
}

#[test]
fn constrained_select_empty_pixa_returns_empty() {
    let pa = Pixa::new();
    let out = pa.constrained_select(0, -1, 5, false).unwrap();
    assert_eq!(out.pix_slice().len(), 0);
}

#[test]
fn constrained_select_deep_clones_output() {
    let pa = make_pixa(&[(4, 4), (5, 5)]);
    let out = pa.constrained_select(0, -1, 2, false).unwrap();
    // Source Pix and output Pix must not share the same Arc-backed
    // pixel buffer (each is independently mutable).
    assert_eq!(pa.pix_slice()[0].ref_count(), 1);
    assert_eq!(out.pix_slice()[0].ref_count(), 1);
}

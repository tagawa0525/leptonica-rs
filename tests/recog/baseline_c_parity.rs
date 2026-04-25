//! C-parity test for `find_baselines`.
//!
//! Reference values derived from C leptonica master (commit f7082ecd) by
//! running `scripts/verify_findbaselines.c` against the same input images.
//! These tests assert exact y-coordinate equivalence with C, except for
//! `keystone.png` where Rust currently lacks the second-stage "short
//! baseline removal" filter (`pixFindBaselinesGen` connected-component pass).
//!
//! See also `tests/recog/baseline_reg.rs` (count-only regression tests).
use crate::common::load_test_image;
use leptonica::recog::baseline::{BaselineOptions, find_baselines};

/// C version y-coordinates for `baseline1.png` with default minw=80.
/// Source: scripts/verify_findbaselines.c → /tmp/c_baseline_short_textblock.txt
const C_SHORT_TEXTBLOCK: &[i32] = &[1874, 1914];

/// C version y-coordinates for `baseline2.tif` with minw=30.
const C_SHORT_LINES: &[i32] = &[
    564, 614, 664, 714, 763, 813, 863, 913, 963, 1012, 1062, 1112, 1162, 1212, 1262, 1311, 1361,
    1411, 1461, 1511, 1560, 1610, 1660, 1710, 1760, 1810, 1859, 1909, 1959,
];

/// C version y-coordinates for `baseline3.tif` with minw=30.
const C_MORE_SHORT_LINES: &[i32] = &[
    177, 255, 300, 340, 384, 426, 473, 518, 562, 604, 650, 694, 740, 779, 825, 869, 914, 955, 999,
    1044, 1088, 1134, 1178, 1221, 1267, 1309, 1355, 1397, 1442, 1484, 1527, 1572, 1616, 1660, 1703,
    1747, 1791, 1834, 1878, 1922,
];

/// C version y-coordinates for `keystone.png` with default minw=80
/// (after pixFindBaselinesGen short-baseline filter).
const C_KEYSTONE: &[i32] = &[
    63, 166, 247, 308, 394, 471, 543, 621, 684, 759, 832, 919, 997, 1064, 1147, 1228, 1364, 1431,
    1525, 1712,
];

/// y-coordinates that C's pixFindBaselinesGen filters out as "short
/// baselines" but Rust currently retains (second-stage filter unimplemented).
const KEYSTONE_RUST_EXTRA: &[i32] = &[1277, 1488, 1574, 1651];

fn baselines(img: &str, minw: u32) -> Vec<i32> {
    let pix = load_test_image(img).unwrap_or_else(|e| panic!("load {}: {}", img, e));
    let opts = BaselineOptions::default().with_min_block_width(minw);
    find_baselines(&pix, &opts)
        .unwrap_or_else(|e| panic!("find_baselines: {}", e))
        .baselines
}

#[test]
fn c_parity_short_textblock() {
    let rust = baselines("baseline1.png", 80);
    assert_eq!(
        rust, C_SHORT_TEXTBLOCK,
        "Rust must match C version y-coords for baseline1.png"
    );
}

#[test]
fn c_parity_short_lines() {
    let rust = baselines("baseline2.tif", 30);
    assert_eq!(
        rust, C_SHORT_LINES,
        "Rust must match C version y-coords for baseline2.tif"
    );
}

#[test]
fn c_parity_more_short_lines() {
    let rust = baselines("baseline3.tif", 30);
    assert_eq!(
        rust, C_MORE_SHORT_LINES,
        "Rust must match C version y-coords for baseline3.tif"
    );
}

/// keystone.png: Rust returns C's set plus 4 short baselines that C's
/// second-stage `pixFindBaselinesGen` filter removes. Once that filter
/// lands in Rust this test should match `C_KEYSTONE` exactly.
#[test]
fn c_parity_keystone_with_known_divergence() {
    let rust = baselines("keystone.png", 80);

    let mut expected: Vec<i32> = C_KEYSTONE
        .iter()
        .chain(KEYSTONE_RUST_EXTRA)
        .copied()
        .collect();
    expected.sort();
    let mut actual = rust.clone();
    actual.sort();

    assert_eq!(
        actual, expected,
        "Rust keystone baselines must equal C set ∪ known-extra short baselines.\n\
         If this fails, either find_baselines diverged further or short-baseline\n\
         filter was implemented (then update C_KEYSTONE expectation)."
    );
}

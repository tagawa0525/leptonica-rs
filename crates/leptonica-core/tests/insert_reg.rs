//! Insert/remove regression test
//!
//! Tests removal and insertion operations in Numa, Boxa, and Pixa.
//! Verifies that remove+insert cycles preserve the original data.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/insert_reg.c`

use leptonica_core::{Box, Boxa, Numa};
use leptonica_test::RegParams;

/// Test Numa insert/remove cycle preserves data (C test checks 0–2)
#[test]
#[ignore = "not yet implemented"]
fn insert_reg_numa() {
    let mut rp = RegParams::new("insert_numa");

    let pi: f64 = std::f64::consts::PI;
    let mut na1 = Numa::new();
    for i in 0..500 {
        let angle = 0.02293 * i as f64 * pi;
        let val = angle.sin() as f32;
        na1.push(val);
    }
    let data1 = na1.write_to_bytes().expect("serialize na1");

    // Remove and re-insert each element in order
    let mut na2 = na1.clone();
    let n = na2.len();
    for i in 0..n {
        let val = na2[i];
        na2.remove(i).expect("remove");
        na2.insert(i, val).expect("insert");
    }
    let data2 = na2.write_to_bytes().expect("serialize na2");

    // Serialized bytes should be identical after remove+insert cycle
    let _ = rp.compare_strings(&data1, &data2);

    assert!(rp.cleanup(), "insert_reg numa test failed");
}

/// Test Boxa insert/remove cycle preserves data (C test checks 3–5)
#[test]
#[ignore = "not yet implemented"]
fn insert_reg_boxa() {
    let mut rp = RegParams::new("insert_boxa");

    // Create a representative boxa (C version extracts from feyn.tif conncomp,
    // but we create equivalent test data directly)
    let mut boxa1 = Boxa::new();
    for i in 0..50i32 {
        let x = (i * 23) % 500;
        let y = (i * 37) % 300;
        let w = 10 + (i * 7) % 40;
        let h = 5 + (i * 11) % 30;
        boxa1.push(Box::new_unchecked(x, y, w, h));
    }
    let data1 = boxa1.write_to_bytes().expect("serialize boxa1");

    // Remove and re-insert each box in order
    let mut boxa2 = boxa1.clone();
    let n = boxa2.len();
    for i in 0..n {
        let b = boxa2.remove(i).expect("remove box");
        boxa2.insert(i, b).expect("insert box");
    }
    let data2 = boxa2.write_to_bytes().expect("serialize boxa2");

    // Serialized bytes should be identical after remove+insert cycle
    let _ = rp.compare_strings(&data1, &data2);

    assert!(rp.cleanup(), "insert_reg boxa test failed");
}

/// Test Pixa insert/remove cycle preserves data (C test checks 6–11)
///
/// Requires connected component extraction from leptonica-region,
/// so this test covers only the core Pixa operations available.
#[test]
#[ignore = "not yet implemented: requires leptonica-region conncomp"]
fn insert_reg_pixa() {
    // The C version tests pixaRemovePixAndSave + pixaInsertPix
    // and rotation cycles (move last→first, first→last).
    // This requires pixConnComp from leptonica-region which is not
    // available from leptonica-core. A separate test in leptonica-region
    // can cover the full pixa insert/remove cycle.
}

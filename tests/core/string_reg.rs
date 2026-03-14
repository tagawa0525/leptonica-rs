//! String regression test
//!
//! Covers `Sarray`/`Sarraya` operations and serialization.
//!
//! # See also
//!
//! C Leptonica: `prog/string_reg.c`

use crate::common::RegParams;
use leptonica::{Sarray, Sarraya};

// ============================================================================
// C-equivalent regression test skeletons — missing string operations
// ============================================================================

/// Sarray substring search (C: sarrayFindStringByHash).
#[test]
#[ignore = "sarrayFindStringByHash not implemented"]
fn string_reg_find_by_hash() {}

/// Sarray replace substring (C: sarrayReplaceString).
#[test]
#[ignore = "string search/replace via hash not implemented"]
fn string_reg_replace_string() {}

/// Binary sequence operations (C: arrayFindSequence).
#[test]
#[ignore = "arrayFindSequence not implemented"]
fn string_reg_binary_sequence() {}

/// Sarray file I/O round-trip (C: sarrayWrite/sarrayRead).
#[test]
#[ignore = "sarrayWrite/sarrayRead file I/O not implemented"]
fn string_reg_file_io() {}

#[test]
fn string_reg() {
    let mut rp = RegParams::new("string");

    let mut sa = Sarray::from_words("alpha beta gamma");
    sa.insert(1, "inserted").expect("insert");
    rp.compare_values(4.0, sa.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sa.get(1) == Some("inserted") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let bytes = sa.write_to_bytes().expect("write_to_bytes");
    let sa2 = Sarray::read_from_bytes(&bytes).expect("read_from_bytes");
    rp.compare_values(sa.len() as f64, sa2.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sa2.join(" ") == "alpha inserted beta gamma" {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    let mut saa = Sarraya::new();
    saa.push(sa2.clone());
    saa.push(Sarray::from_str_slice(&["delta", "epsilon"]));
    saa.add_string(1, "zeta").expect("add_string");

    let flat = saa.flatten();
    rp.compare_values(7.0, flat.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if flat.get(6) == Some("zeta") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Additional string operations
    // Remove operation
    sa.remove(1).expect("remove");
    rp.compare_values(3.0, sa.len() as f64, 0.0);
    rp.compare_values(1.0, if sa.get(0) == Some("alpha") { 1.0 } else { 0.0 }, 0.0);

    // Sort
    let sorted = sa.sorted();
    rp.compare_values(3.0, sorted.len() as f64, 0.0);
    rp.compare_values(
        1.0,
        if sorted.get(0) == Some("alpha") {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Words from lines
    let multi = Sarray::from_words("one two three\nfour five");
    rp.compare_values(5.0, multi.len() as f64, 0.0);

    // Sarraya operations
    rp.compare_values(2.0, saa.len() as f64, 0.0);
    let flat2 = saa.flatten();
    rp.compare_values(7.0, flat2.len() as f64, 0.0);

    assert!(rp.cleanup(), "string regression test failed");
}

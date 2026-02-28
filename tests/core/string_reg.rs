//! String regression test
//!
//! Covers `Sarray`/`Sarraya` operations and serialization.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/string_reg.c`

use crate::common::RegParams;
use leptonica::{Sarray, Sarraya};

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

    assert!(rp.cleanup(), "string regression test failed");
}

//! DNA-style numeric array regression test
//!
//! Uses `Numa`/`Numaa` dynamic arrays and serialization as the Rust mapping.
//!
//! # See also
//!
//! C Leptonica: `prog/dna_reg.c`

use crate::common::RegParams;
use leptonica::{Numa, Numaa, SortOrder};

#[test]
fn dna_reg() {
    let mut rp = RegParams::new("dna");

    let mut na = Numa::from_slice(&[1.0, 3.0, 5.0]);
    na.insert(1, 2.0).expect("insert");
    na.add_sorted(4.0).expect("add_sorted");

    rp.compare_values(5.0, na.len() as f64, 0.0);
    rp.compare_values(4.0, na.get(3).expect("value") as f64, 0.0);

    let nad = na.sorted(SortOrder::Decreasing);
    rp.compare_values(5.0, nad.get(0).expect("max") as f64, 0.0);

    let bytes = na.write_to_bytes().expect("write_to_bytes");
    let na2 = Numa::read_from_bytes(&bytes).expect("read_from_bytes");
    rp.compare_values(na.len() as f64, na2.len() as f64, 0.0);
    rp.compare_values(
        na.median().expect("median") as f64,
        na2.median().expect("median2") as f64,
        0.001,
    );

    let mut naa = Numaa::new();
    naa.push(na);
    naa.push(Numa::from_slice(&[10.0, 20.0]));

    let data = naa.write_to_bytes().expect("numaa write_to_bytes");
    let naa2 = Numaa::read_from_bytes(&data).expect("numaa read_from_bytes");
    rp.compare_values(2.0, naa2.len() as f64, 0.0);
    rp.compare_values(7.0, naa2.total_count() as f64, 0.0);

    assert!(rp.cleanup(), "dna regression test failed");
}

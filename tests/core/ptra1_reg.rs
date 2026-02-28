//! Pointer-array regression test (part 1)
//!
//! Uses `Ptaa` operations as the Rust mapping for pointer-array behavior.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/ptra1_reg.c`

use crate::common::RegParams;
use leptonica::{Pta, Ptaa};

#[test]
fn ptra1_reg() {
    let mut rp = RegParams::new("ptra1");

    let mut ptaa = Ptaa::new();

    let mut pa0 = Pta::new();
    pa0.push(1.0, 1.0);
    pa0.push(2.0, 2.0);
    ptaa.push(pa0);

    let mut pa1 = Pta::new();
    pa1.push(10.0, 10.0);
    ptaa.push(pa1);

    ptaa.add_pt(1, 20.0, 20.0).expect("add_pt");
    rp.compare_values(2.0, ptaa.len() as f64, 0.0);
    rp.compare_values(4.0, ptaa.total_points() as f64, 0.0);

    let repl = Pta::from_vecs(vec![5.0, 6.0], vec![7.0, 8.0]).expect("from_vecs");
    ptaa.replace(0, repl).expect("replace");
    let (x, y) = ptaa.get(0).expect("pta").get(1).expect("point");
    rp.compare_values(6.0, x as f64, 0.0);
    rp.compare_values(8.0, y as f64, 0.0);

    assert!(rp.cleanup(), "ptra1 regression test failed");
}

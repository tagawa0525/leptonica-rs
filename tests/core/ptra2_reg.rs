//! Pointer-array regression test (part 2)
//!
//! Additional `Ptaa` operations: init, flatten, pop, clear.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/ptra2_reg.c`

use crate::common::RegParams;
use leptonica::{Pta, Ptaa};

#[test]
fn ptra2_reg() {
    let mut rp = RegParams::new("ptra2");

    let mut ptaa = Ptaa::with_capacity(3);
    ptaa.push(Pta::new());
    ptaa.push(Pta::new());
    ptaa.push(Pta::new());

    let mut seed = Pta::new();
    seed.push(3.0, 4.0);
    ptaa.init_full(&seed);

    rp.compare_values(3.0, ptaa.len() as f64, 0.0);
    rp.compare_values(3.0, ptaa.total_points() as f64, 0.0);

    ptaa.add_pt(2, 30.0, 40.0).expect("add_pt");
    let flat = ptaa.flatten();
    rp.compare_values(4.0, flat.len() as f64, 0.0);

    let popped = ptaa.pop().expect("pop");
    rp.compare_values(2.0, ptaa.len() as f64, 0.0);
    rp.compare_values(2.0, popped.len() as f64, 0.0);

    ptaa.clear();
    rp.compare_values(0.0, ptaa.len() as f64, 0.0);

    assert!(rp.cleanup(), "ptra2 regression test failed");
}

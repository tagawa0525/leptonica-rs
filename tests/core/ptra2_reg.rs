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

    // Verify point values after init_full
    let (x, y) = ptaa.get(0).expect("pta0").get(0).expect("pt0");
    rp.compare_values(3.0, x as f64, 0.0);
    rp.compare_values(4.0, y as f64, 0.0);

    // Verify flatten with larger dataset
    let mut ptaa2 = Ptaa::with_capacity(5);
    for i in 0..5u32 {
        let mut pa = Pta::new();
        for j in 0..3u32 {
            pa.push(i as f32 * 10.0 + j as f32, i as f32 * 100.0 + j as f32);
        }
        ptaa2.push(pa);
    }
    let flat2 = ptaa2.flatten();
    rp.compare_values(15.0, flat2.len() as f64, 0.0);

    ptaa.clear();
    rp.compare_values(0.0, ptaa.len() as f64, 0.0);

    assert!(rp.cleanup(), "ptra2 regression test failed");
}

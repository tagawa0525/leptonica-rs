//! Heap-style regression test
//!
//! Uses sorted-insert and ordering operations on `Numa`.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/heap_reg.c`

use crate::common::RegParams;
use leptonica::{Numa, SortOrder};

#[test]
fn heap_reg() {
    let mut rp = RegParams::new("heap");

    let mut na = Numa::from_slice(&[2.0, 4.0, 6.0, 8.0]);
    na.add_sorted(5.0).expect("add_sorted 5");
    na.add_sorted(1.0).expect("add_sorted 1");

    rp.compare_values(6.0, na.len() as f64, 0.0);
    rp.compare_values(1.0, na.get(0).expect("first") as f64, 0.0);
    rp.compare_values(5.0, na.get(3).expect("mid") as f64, 0.0);

    let med = na.median().expect("median");
    rp.compare_values(1.0, if (4.0..=5.0).contains(&med) { 1.0 } else { 0.0 }, 0.0);

    na.sort(SortOrder::Decreasing);
    rp.compare_values(8.0, na.get(0).expect("max") as f64, 0.0);
    rp.compare_values(1.0, na.get(na.len() - 1).expect("min") as f64, 0.0);

    assert!(rp.cleanup(), "heap regression test failed");
}

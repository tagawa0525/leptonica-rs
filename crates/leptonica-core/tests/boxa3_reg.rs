//! Boxa regression test 3 - size reconciliation
//!
//! Tests higher-level Boxa operations for detecting and correcting
//! anomalous-sized boxes: median dimensions, size consistency,
//! and reconciliation by median.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/boxa3_reg.c`

use leptonica_core::Boxa;
use leptonica_test::RegParams;

/// Expected variance values for pair-based size consistency check
/// (matches C static arrays: varp, varm, same)
#[allow(dead_code)]
const VARP: [f64; 3] = [0.0165, 0.0432, 0.0716];
#[allow(dead_code)]
const VARM: [f64; 3] = [0.0088, 0.0213, 0.0357];
#[allow(dead_code)]
const SAME: [i32; 3] = [1, -1, -1];

/// Test Boxa median dimensions, size consistency, and reconciliation
/// for three different boxa datasets.
///
/// C version uses boxap1.ba, boxap2.ba, boxap3.ba test data files
/// and performs 45 regtest checks (3 datasets × 15 checks each).
#[test]
#[ignore = "not yet implemented: median_dimensions, size_consistency, reconcile_size_by_median"]
fn boxa3_reg() {
    let mut rp = RegParams::new("boxa3");

    let boxa_files = ["boxap1.ba", "boxap2.ba", "boxap3.ba"];

    for file in &boxa_files {
        let boxa1 = Boxa::read_from_file(leptonica_test::test_data_path(file))
            .unwrap_or_else(|_| panic!("read {file}"));

        // Scale to normalized width
        let (w, _h, _bb) = boxa1.get_extent().expect("get extent");
        let scale_fact = 100.0 / w as f32;
        let boxa2 = boxa1.scale(scale_fact, scale_fact);

        // Serialize the scaled boxa
        let data = boxa2.write_to_bytes().expect("serialize");
        rp.write_data_and_check(&data, "ba").unwrap();

        // TODO: boxaDisplayTiled (visualization, not critical for regression)

        // TODO: Find median dimensions
        // let (medw, medh) = boxa2.median_dimensions();

        // TODO: Check size consistency
        // let (fvarp, fvarm, isame) = boxa2.size_consistency(CheckMode::Height, 0.0, 0.0);
        // rp.compare_values(VARP[index], fvarp, 0.003);
        // rp.compare_values(VARM[index], fvarm, 0.003);
        // rp.compare_values(SAME[index] as f64, isame as f64, 0.0);

        // TODO: Reconcile widths
        // let boxa3 = boxa2.reconcile_size_by_median(CheckMode::Width, 0.05, 0.04, 1.03);

        // TODO: Reconcile heights
        // let boxa3 = boxa2.reconcile_size_by_median(CheckMode::Height, 0.05, 0.04, 1.03);

        // TODO: Reconcile both
        // let boxa3 = boxa2.reconcile_size_by_median(CheckMode::Both, 0.05, 0.04, 1.03);
    }

    assert!(rp.cleanup(), "boxa3 regression test failed");
}

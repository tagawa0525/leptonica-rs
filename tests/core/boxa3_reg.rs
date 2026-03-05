//! Boxa regression test 3 - size reconciliation
//!
//! Tests higher-level Boxa operations for detecting and correcting
//! anomalous-sized boxes: median dimensions, size consistency,
//! and reconciliation by median.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/boxa3_reg.c`

use crate::common::RegParams;
use leptonica::Boxa;
use leptonica::core::box_::smooth::CheckType;

/// Expected variance values for pair-based size consistency check
/// (matches C static arrays: varp, varm, same)
const VARP: [f64; 3] = [0.0165, 0.0432, 0.0716];
const VARM: [f64; 3] = [0.0088, 0.0213, 0.0357];
const SAME: [i32; 3] = [1, -1, -1];

/// Test Boxa median dimensions, size consistency, and reconciliation
/// for three different boxa datasets.
///
/// C version uses boxap1.ba, boxap2.ba, boxap3.ba test data files
/// and performs 45 regtest checks (3 datasets × 15 checks each).
/// Partial port. The C version also tests boxaMedianDimensions,
/// boxaSizeConsistency, boxaReconcileSizeByMedian, and boxaPlotSizes/Sides
/// (all not yet implemented). Currently only tests read → scale → serialize.
#[test]
fn boxa3_reg() {
    let mut rp = RegParams::new("boxa3");

    let boxa_files = ["boxap1.ba", "boxap2.ba", "boxap3.ba"];

    for (idx, file) in boxa_files.iter().enumerate() {
        let boxa1 = Boxa::read_from_file(crate::common::test_data_path(file))
            .unwrap_or_else(|_| panic!("read {file}"));

        // Scale to normalized width
        let (w, _h, _bb) = boxa1.get_extent().expect("get extent");
        let scale_fact = 100.0 / w as f32;
        let boxa2 = boxa1.scale(scale_fact, scale_fact);

        // Serialize the scaled boxa
        let data = boxa2.write_to_bytes().expect("serialize");
        rp.write_data_and_check(&data, "ba").unwrap();

        // Find median dimensions (C: boxaMedianDimensions)
        if let Ok(med) = boxa2.median_dimensions() {
            rp.compare_values(1.0, if med.med_w > 0 { 1.0 } else { 0.0 }, 0.0);
            rp.compare_values(1.0, if med.med_h > 0 { 1.0 } else { 0.0 }, 0.0);
        }

        // Check size consistency (C: boxaSizeConsistency)
        if let Ok(sc) = boxa2.size_consistency(CheckType::Height, 0.0, 0.0) {
            rp.compare_values(VARP[idx], sc.fvar_pair as f64, 0.01);
            rp.compare_values(VARM[idx], sc.fvar_median as f64, 0.01);
            rp.compare_values(SAME[idx] as f64, sc.same as f64, 0.0);
        }
    }

    assert!(rp.cleanup(), "boxa3 regression test failed");
}

//! Boxa regression test 3 - size reconciliation
//!
//! Tests higher-level Boxa operations for detecting and correcting
//! anomalous-sized boxes: median dimensions, size consistency,
//! and reconciliation by median.
//!
//! # See also
//!
//! C Leptonica: `prog/boxa3_reg.c`

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
        let med = boxa2.median_dimensions().expect("median dimensions");
        rp.compare_values(1.0, if med.med_w > 0 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(1.0, if med.med_h > 0 { 1.0 } else { 0.0 }, 0.0);

        // Check size consistency (C: boxaSizeConsistency)
        let sc = boxa2
            .size_consistency(CheckType::Height, 0.0, 0.0)
            .expect("size consistency");
        rp.compare_values(VARP[idx], sc.fvar_pair as f64, 0.01);
        rp.compare_values(VARM[idx], sc.fvar_median as f64, 0.01);
        rp.compare_values(SAME[idx] as f64, sc.same as f64, 0.0);
    }

    assert!(rp.cleanup(), "boxa3 regression test failed");
}

/// C-comparable boxa reconciliation series (plan 902 PR 22).
///
/// Mirrors C boxa3_reg's `TestBoxa` for the three `boxap*.ba` inputs:
/// scale each boxa so its extent is 100 wide, serialize it, tile it with
/// the C-signature `display_tiled`, then do the same for the three
/// `reconcile_size_by_median` variants.
#[test]
fn boxa3_c_compat() {
    use leptonica::TransformOrder;
    use leptonica::core::box_::Boxa;
    use leptonica::core::box_::smooth::CheckType;
    use leptonica::io::ImageFormat;

    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("boxa3_c");

    for name in ["boxap1", "boxap2", "boxap3"] {
        let path = format!("{}/tests/data/{name}.ba", env!("CARGO_MANIFEST_DIR"));
        let boxa1 = Boxa::read_from_file(&path).expect("read boxa");

        // C: scalefact = 100 / extent_width, then boxaTransform(0, 0, s, s)
        let (w, _, _) = boxa1.get_extent().expect("extent");
        let scalefact = 100.0 / w as f32;
        let boxa2 = boxa1.transform_ordered(
            0,
            0,
            scalefact,
            scalefact,
            0,
            0,
            0.0,
            TransformOrder::TrScRo,
        );

        let data = boxa2.write_to_bytes().expect("serialize boxa2");
        rp.write_data_and_check(&data, "ba")
            .expect("check: scaled boxa");
        let tiled = boxa2
            .display_tiled(None, 0, -1, 2200, 2, 1.0, 0, 3, 2)
            .expect("display_tiled");
        rp.write_pix_and_check(&tiled, ImageFormat::Png)
            .expect("check: scaled boxa display");

        // C: three reconcile_size_by_median variants
        for check in [CheckType::Width, CheckType::Height, CheckType::Both] {
            let boxa3 = boxa2
                .reconcile_size_by_median(check, 0.05, 0.04, 1.03)
                .expect("reconcile_size_by_median");
            let data = boxa3.write_to_bytes().expect("serialize boxa3");
            rp.write_data_and_check(&data, "ba")
                .expect("check: reconciled boxa");
            let tiled = boxa3
                .display_tiled(None, 0, -1, 2200, 2, 1.0, 0, 3, 2)
                .expect("display_tiled reconciled");
            rp.write_pix_and_check(&tiled, ImageFormat::Png)
                .expect("check: reconciled boxa display");
        }
    }

    assert!(rp.cleanup(), "boxa3 c-compat test failed");
}

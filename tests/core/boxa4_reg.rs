//! Boxa regression test 4 - smoothing and display
//!
//! Tests Boxa smoothing operations (median sequence smoothing),
//! reconciliation by median for all sides, split even/odd operations,
//! and Boxaa transpose.
//!
//! # See also
//!
//! C Leptonica: `prog/boxa4_reg.c`

use crate::common::RegParams;
use leptonica::Boxa;

/// Test Boxa smoothing, reconciliation, split/merge, and Boxaa transpose.
///
/// C version uses boxa1.ba through boxa5.ba and showboxes.pac/showboxes1.baa
/// test data files and performs 14 regtest checks.
/// Partial port. The C version also tests boxaSmoothSequenceMedian,
/// boxaReconcileAllByMedian, boxaReconcileSidesByMedian, boxaaTranspose,
/// and boxaPlotSides (all not yet implemented).
/// Currently only tests split_even_odd + merge_even_odd roundtrip.
#[test]
fn boxa4_reg() {
    let mut rp = RegParams::new("boxa4");

    // --- Test split even/odd + reconcile sides (C check 9) ---
    let boxa5_path = crate::common::test_data_path("boxa5.ba");
    assert!(
        std::path::Path::new(&boxa5_path).exists(),
        "test fixture boxa5.ba not found at {boxa5_path}"
    );
    let boxa1 = Boxa::read_from_file(&boxa5_path).expect("read boxa5.ba");
    let (boxa1e, boxa1o) = boxa1.split_even_odd(false);
    // split_even_odd should partition correctly
    rp.compare_values(
        boxa1.len() as f64,
        (boxa1e.len() + boxa1o.len()) as f64,
        0.0,
    );
    // Merge back and verify content equality (not just length)
    let merged = Boxa::merge_even_odd(&boxa1e, &boxa1o, false).expect("merge");
    rp.compare_values(boxa1.len() as f64, merged.len() as f64, 0.0);
    for i in 0..boxa1.len() {
        assert_eq!(
            boxa1.get(i),
            merged.get(i),
            "box at index {i} differs after split/merge roundtrip"
        );
    }

    assert!(rp.cleanup(), "boxa4 regression test failed");
}

// ============================================================================
// C-equivalent regression test skeletons
// ============================================================================

/// boxaSmoothSequenceMedian with L_USE_CAPPED_MAX (C checks 0-1).
#[test]
#[ignore = "boxaSmoothSequenceMedian visualization not available"]
fn boxa4_reg_smooth_capped_max() {}

/// boxaSmoothSequenceMedian with L_SUB_ON_LOC_DIFF / L_SUB_ON_SIZE_DIFF (C checks 2-4).
#[test]
#[ignore = "boxaPlotSides visualization not available"]
fn boxa4_reg_smooth_loc_size_diff() {}

/// boxaReconcileAllByMedian with L_ADJUST_LEFT_AND_RIGHT (C checks 5-6).
#[test]
#[ignore = "boxaReconcileAllByMedian visualization not available"]
fn boxa4_reg_reconcile_all_lr() {}

/// boxaReconcileAllByMedian with L_ADJUST_SKIP (C checks 7-8).
#[test]
#[ignore = "boxaReconcileAllByMedian L_ADJUST_SKIP visualization not available"]
fn boxa4_reg_reconcile_all_skip() {}

/// boxaSmoothSequenceMedian with L_USE_CAPPED_MIN (C check 10).
#[test]
#[ignore = "boxaSmoothSequenceMedian visualization not available"]
fn boxa4_reg_smooth_capped_min() {}

/// boxaaTranspose reversibility and pixaDisplayBoxaa (C checks 11-13).
#[test]
#[ignore = "boxaaTranspose not implemented"]
fn boxa4_reg_boxaa_transpose() {}

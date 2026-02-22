//! Boxa regression test 4 - smoothing and display
//!
//! Tests Boxa smoothing operations (median sequence smoothing),
//! reconciliation by median for all sides, split even/odd operations,
//! and Boxaa transpose.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/boxa4_reg.c`

use leptonica_core::Boxa;
use leptonica_test::RegParams;

/// Test Boxa smoothing, reconciliation, split/merge, and Boxaa transpose.
///
/// C version uses boxa1.ba through boxa5.ba and showboxes.pac/showboxes1.baa
/// test data files and performs 14 regtest checks.
#[test]
#[ignore = "not yet implemented: smooth_sequence_median, reconcile_all_by_median, reconcile_sides_by_median, Boxaa::transpose"]
fn boxa4_reg() {
    let mut rp = RegParams::new("boxa4");

    // --- Test smoothing with capped max (C checks 0-1) ---
    // boxa1.ba: fairly clean boxa
    // boxa2.ba: unsmoothed and noisy boxa
    // TODO: boxaSmoothSequenceMedian(boxa1, 10, L_USE_CAPPED_MAX, 50, 0, 0)
    // TODO: boxaSmoothSequenceMedian(boxa2, 10, L_USE_CAPPED_MAX, 50, 0, 0)

    // --- Test smoothing with loc/size diff (C checks 2-4) ---
    // TODO: boxaSmoothSequenceMedian(boxa2, 10, L_SUB_ON_LOC_DIFF, 80, 20, 1)
    // TODO: boxaSmoothSequenceMedian(boxa2, 10, L_SUB_ON_SIZE_DIFF, 80, 20, 1)
    // TODO: boxaPlotSides (visualization)

    // --- Test reconcile all by median (C checks 5-8) ---
    // TODO: boxaReconcileAllByMedian(boxa5, L_ADJUST_LEFT_AND_RIGHT, L_ADJUST_TOP_AND_BOT, 50, 0)
    // TODO: boxaReconcileAllByMedian(boxa5, L_ADJUST_SKIP, L_ADJUST_TOP_AND_BOT, 50, 0)

    // --- Test split even/odd + reconcile sides (C check 9) ---
    let boxa5_path = leptonica_test::test_data_path("boxa5.ba");
    if std::path::Path::new(&boxa5_path).exists() {
        let boxa1 = Boxa::read_from_file(&boxa5_path).expect("read boxa5.ba");
        let (boxa1e, boxa1o) = boxa1.split_even_odd(false);
        // split_even_odd should partition correctly
        rp.compare_values(
            boxa1.len() as f64,
            (boxa1e.len() + boxa1o.len()) as f64,
            0.0,
        );
        // Merge back
        let merged = Boxa::merge_even_odd(&boxa1e, &boxa1o, false).expect("merge");
        rp.compare_values(boxa1.len() as f64, merged.len() as f64, 0.0);
    }

    // --- Test smoothing with capped min (C check 10) ---
    // TODO: boxaSmoothSequenceMedian(boxa3, 10, L_USE_CAPPED_MIN, 20, 0, 1)

    // --- Test Boxaa transpose reversibility (C checks 11-13) ---
    // TODO: boxaaTranspose + boxaEqual

    assert!(rp.cleanup(), "boxa4 regression test failed");
}

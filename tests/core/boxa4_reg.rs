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

/// boxaaTranspose reversibility (C checks 11-13).
///
/// The C version verifies that two consecutive transposes restore the original
/// Boxaa. We mirror that and additionally check the explicit shape inversion
/// and error cases (empty Boxaa, non-uniform inner Boxa lengths).
#[test]
fn boxa4_reg_boxaa_transpose() {
    use leptonica::Box;

    // Build a 3 x 4 Boxaa where the (i, j) entry is encoded into x/y so that
    // we can check exact equality after the transpose.
    let mut baas = leptonica::Boxaa::with_capacity(3);
    for outer in 0..3i32 {
        let mut boxa = leptonica::Boxa::with_capacity(4);
        for inner in 0..4i32 {
            boxa.push(Box::new(outer, inner, 1, 1).unwrap());
        }
        baas.push(boxa);
    }

    // Transpose: result should be 4 x 3 with (i, j) holding the original (j, i).
    let baad = baas.transpose().expect("transpose should succeed");
    assert_eq!(baad.len(), 4, "outer count after transpose");
    for i in 0..4 {
        let row = baad.get(i).expect("row i exists");
        assert_eq!(row.len(), 3, "inner count after transpose");
        for j in 0..3 {
            let b = row.get(j).copied().expect("box (i, j) exists");
            assert_eq!(b.x, j as i32, "x at ({i}, {j})");
            assert_eq!(b.y, i as i32, "y at ({i}, {j})");
        }
    }

    // Round-trip: transpose twice gives back the original.
    let round = baad.transpose().expect("second transpose should succeed");
    assert_eq!(round.len(), baas.len());
    for outer in 0..baas.len() {
        let original = baas.get(outer).unwrap();
        let recovered = round.get(outer).unwrap();
        assert_eq!(original.len(), recovered.len(), "row {outer} length");
        for inner in 0..original.len() {
            assert_eq!(
                original.get(inner).copied(),
                recovered.get(inner).copied(),
                "box at ({outer}, {inner}) after roundtrip",
            );
        }
    }

    // Empty Boxaa is rejected.
    assert!(
        leptonica::Boxaa::new().transpose().is_err(),
        "empty Boxaa should error",
    );

    // Non-uniform inner sizes are rejected.
    let mut ragged = leptonica::Boxaa::new();
    let mut a = leptonica::Boxa::new();
    a.push(Box::new(0, 0, 1, 1).unwrap());
    a.push(Box::new(1, 0, 1, 1).unwrap());
    ragged.push(a);
    let mut b = leptonica::Boxa::new();
    b.push(Box::new(0, 1, 1, 1).unwrap());
    ragged.push(b);
    assert!(ragged.transpose().is_err(), "ragged Boxaa should error",);
}

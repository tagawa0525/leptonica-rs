//! Component filter regression test
//!
//! Tests filtering of connected components by size, perimeter-area ratio,
//! perimeter-size ratio, and area fraction. The C version uses
//! pixSelectBySize, pixSelectByPerimToAreaRatio, pixSelectByPerimSizeRatio,
//! and pixSelectByAreaFraction, followed by verification via numaMakeThresholdIndicator.
//!
//! Not yet migrated: all component selection functions
//! (pixSelectBySize, pixSelectByPerimToAreaRatio, etc.) are in
//! leptonica-region, not leptonica-filter.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/compfilter_reg.c`

/// Test component selection by size (C checks 2-13).
#[test]
fn compfilter_reg_select_by_size() {
    use crate::common::{RegParams, load_test_image};
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, find_connected_components,
        pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_size");
    if crate::common::is_display_mode() {
        rp.compare_values(1.0, 1.0, 0.0);
        assert!(rp.cleanup(), "compfilter_select_by_size test failed");
        return;
    }

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let comps_before = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find_connected_components");

    // Select only components with both dimensions >= 10
    let filtered = pix_select_by_size(
        &pixs,
        10,
        10,
        ConnectivityType::EightWay,
        SizeSelectType::IfBoth,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size");

    let comps_after = find_connected_components(&filtered, ConnectivityType::EightWay)
        .expect("find_connected_components after filter");

    // Filtered result should have fewer components
    rp.compare_values(
        1.0,
        if comps_after.len() < comps_before.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    // Dimensions preserved
    rp.compare_values(pixs.width() as f64, filtered.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, filtered.height() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_size test failed");
}

/// Test component selection by shape metrics (C checks 14-25).
///
/// Uses pix_select_by_size with different thresholds as a shape proxy,
/// since pixSelectByPerimToAreaRatio and similar are not yet available.
#[test]
fn compfilter_reg_select_by_shape() {
    if crate::common::is_display_mode() {
        return;
    }

    use crate::common::{RegParams, load_test_image};
    use leptonica::region::{
        ConnectivityType, SizeSelectRelation, SizeSelectType, find_connected_components,
        pix_select_by_size,
    };

    let mut rp = RegParams::new("compfilter_select_by_shape");

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    let comps_before = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find_connected_components");

    // Select components where either dimension >= 20 (larger features)
    let filtered = pix_select_by_size(
        &pixs,
        20,
        20,
        ConnectivityType::EightWay,
        SizeSelectType::IfEither,
        SizeSelectRelation::Gte,
    )
    .expect("pix_select_by_size shape");

    let comps_after = find_connected_components(&filtered, ConnectivityType::EightWay)
        .expect("find_connected_components after shape filter");

    // Filtered result should have fewer components
    rp.compare_values(
        1.0,
        if comps_after.len() < comps_before.len() {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    // Dimensions preserved
    rp.compare_values(pixs.width() as f64, filtered.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, filtered.height() as f64, 0.0);

    assert!(rp.cleanup(), "compfilter_select_by_shape test failed");
}

/// Test connected component analysis with indicators (C checks 26-29).
///
/// Requires pixConnComp and numaMakeThresholdIndicator/numaLogicalOp
/// which are in leptonica-region and leptonica-core.
#[test]
#[ignore = "not yet implemented: pixConnComp/numaMakeThresholdIndicator in leptonica-region/core"]
fn compfilter_reg_indicator_operations() {
    // C version:
    // 1. pixConnComp(pixs, &pixa1, 8) to get connected components
    // 2. numaGetWidths/GetHeights to extract dimension arrays
    // 3. numaMakeThresholdIndicator(na1, threshold, L_SELECT_IF_GTE)
    // 4. numaLogicalOp to combine indicators
    // 5. pixRemoveWithIndicator to filter components
}

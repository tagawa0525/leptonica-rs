//! Test Numa sort, interpolation, and utility functions
//!
//! # See also
//!
//! C Leptonica: `numafunc1.c`
//! - numaSortAutoSelect, numaSortIndexAutoSelect, numaGetSortIndex
//! - numaSortByIndex, numaIsSorted
//! - numaInterpolateEqxVal, numaInterpolateArbxVal
//! - numaClipToInterval, numaMakeThresholdIndicator
//! - numaGetNonzeroRange, numaGetCountRelativeToZero, numaSubsample

use leptonica_core::{
    CountRelativeToZero, InterpolationType, Numa, SortOrder, ThresholdComparison,
};

// ============================================================================
// numaSortAutoSelect / sort_auto_select
// ============================================================================

#[test]
fn test_sort_auto_select_increasing() {
    let na = Numa::from_vec(vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0]);
    let sorted = na.sort_auto_select(SortOrder::Increasing);
    assert_eq!(sorted.as_slice(), &[1.0, 2.0, 3.0, 5.0, 8.0, 9.0]);
}

#[test]
fn test_sort_auto_select_decreasing() {
    let na = Numa::from_vec(vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0]);
    let sorted = na.sort_auto_select(SortOrder::Decreasing);
    assert_eq!(sorted.as_slice(), &[9.0, 8.0, 5.0, 3.0, 2.0, 1.0]);
}

#[test]
fn test_sort_auto_select_empty() {
    let na = Numa::new();
    let sorted = na.sort_auto_select(SortOrder::Increasing);
    assert!(sorted.is_empty());
}

// ============================================================================
// numaSortIndexAutoSelect / sort_index_auto_select
// ============================================================================

#[test]
fn test_sort_index_auto_select_basic() {
    let na = Numa::from_vec(vec![30.0, 10.0, 20.0]);
    let indices = na.sort_index_auto_select(SortOrder::Increasing);
    // Sorted order: 10(1), 20(2), 30(0) → indices = [1, 2, 0]
    assert_eq!(indices.as_slice(), &[1.0, 2.0, 0.0]);
}

#[test]
fn test_sort_index_auto_select_decreasing() {
    let na = Numa::from_vec(vec![30.0, 10.0, 20.0]);
    let indices = na.sort_index_auto_select(SortOrder::Decreasing);
    // Sorted order: 30(0), 20(2), 10(1) → indices = [0, 2, 1]
    assert_eq!(indices.as_slice(), &[0.0, 2.0, 1.0]);
}

// ============================================================================
// numaGetSortIndex / sort_index
// ============================================================================

#[test]
fn test_sort_index_increasing() {
    let na = Numa::from_vec(vec![50.0, 10.0, 40.0, 20.0, 30.0]);
    let indices = na.sort_index(SortOrder::Increasing);
    // Sorted: 10(1), 20(3), 30(4), 40(2), 50(0) → indices = [1, 3, 4, 2, 0]
    assert_eq!(indices.as_slice(), &[1.0, 3.0, 4.0, 2.0, 0.0]);
}

#[test]
fn test_sort_index_stable() {
    // Duplicate values: indices should preserve relative order (stable sort)
    let na = Numa::from_vec(vec![2.0, 1.0, 2.0, 1.0]);
    let indices = na.sort_index(SortOrder::Increasing);
    // Sorted: 1(1), 1(3), 2(0), 2(2) → indices = [1, 3, 0, 2]
    assert_eq!(indices.as_slice(), &[1.0, 3.0, 0.0, 2.0]);
}

// ============================================================================
// numaSortByIndex / sort_by_index
// ============================================================================

#[test]
fn test_sort_by_index_basic() {
    let na = Numa::from_vec(vec![50.0, 10.0, 40.0, 20.0, 30.0]);
    let indices = Numa::from_vec(vec![1.0, 3.0, 4.0, 2.0, 0.0]);
    let result = na.sort_by_index(&indices).unwrap();
    assert_eq!(result.as_slice(), &[10.0, 20.0, 30.0, 40.0, 50.0]);
}

#[test]
fn test_sort_by_index_roundtrip() {
    let na = Numa::from_vec(vec![5.0, 2.0, 8.0, 1.0, 9.0]);
    let indices = na.sort_index(SortOrder::Increasing);
    let sorted = na.sort_by_index(&indices).unwrap();
    assert_eq!(sorted.as_slice(), &[1.0, 2.0, 5.0, 8.0, 9.0]);
}

#[test]
fn test_sort_by_index_empty() {
    let na = Numa::new();
    let indices = Numa::new();
    let result = na.sort_by_index(&indices).unwrap();
    assert!(result.is_empty());
}

// ============================================================================
// numaIsSorted / is_sorted
// ============================================================================

#[test]
fn test_is_sorted_increasing() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert!(na.is_sorted(SortOrder::Increasing));
    assert!(!na.is_sorted(SortOrder::Decreasing));
}

#[test]
fn test_is_sorted_decreasing() {
    let na = Numa::from_vec(vec![5.0, 4.0, 3.0, 2.0, 1.0]);
    assert!(na.is_sorted(SortOrder::Decreasing));
    assert!(!na.is_sorted(SortOrder::Increasing));
}

#[test]
fn test_is_sorted_equal() {
    // All-equal should be sorted in both directions
    let na = Numa::from_vec(vec![3.0, 3.0, 3.0]);
    assert!(na.is_sorted(SortOrder::Increasing));
    assert!(na.is_sorted(SortOrder::Decreasing));
}

#[test]
fn test_is_sorted_single() {
    let na = Numa::from_vec(vec![42.0]);
    assert!(na.is_sorted(SortOrder::Increasing));
    assert!(na.is_sorted(SortOrder::Decreasing));
}

#[test]
fn test_is_sorted_empty() {
    let na = Numa::new();
    assert!(na.is_sorted(SortOrder::Increasing));
}

// ============================================================================
// numaInterpolateEqxVal / interpolate_eqx_val
// ============================================================================

#[test]
fn test_interpolate_eqx_val_linear() {
    // y = [0, 10, 20, 30] with startx=0, delx=1
    let mut na = Numa::from_vec(vec![0.0, 10.0, 20.0, 30.0]);
    na.set_parameters(0.0, 1.0);
    let val = na
        .interpolate_eqx_val(InterpolationType::Linear, 1.5)
        .unwrap();
    assert!((val - 15.0).abs() < 0.01, "expected 15.0, got {val}");
}

#[test]
fn test_interpolate_eqx_val_at_knot() {
    let mut na = Numa::from_vec(vec![0.0, 10.0, 20.0, 30.0]);
    na.set_parameters(0.0, 1.0);
    let val = na
        .interpolate_eqx_val(InterpolationType::Linear, 2.0)
        .unwrap();
    assert!((val - 20.0).abs() < 0.01, "expected 20.0, got {val}");
}

#[test]
fn test_interpolate_eqx_val_quadratic() {
    // y = [0, 1, 4, 9, 16] (x^2) with startx=0, delx=1
    let mut na = Numa::from_vec(vec![0.0, 1.0, 4.0, 9.0, 16.0]);
    na.set_parameters(0.0, 1.0);
    let val = na
        .interpolate_eqx_val(InterpolationType::Quadratic, 1.5)
        .unwrap();
    // Quadratic interp of x^2 at x=1.5 should be close to 2.25
    assert!((val - 2.25).abs() < 0.5, "expected near 2.25, got {val}");
}

#[test]
fn test_interpolate_eqx_val_out_of_range() {
    let mut na = Numa::from_vec(vec![0.0, 10.0, 20.0]);
    na.set_parameters(0.0, 1.0);
    assert!(
        na.interpolate_eqx_val(InterpolationType::Linear, -1.0)
            .is_err()
    );
    assert!(
        na.interpolate_eqx_val(InterpolationType::Linear, 3.0)
            .is_err()
    );
}

// ============================================================================
// numaInterpolateArbxVal / interpolate_arbx_val
// ============================================================================

#[test]
fn test_interpolate_arbx_val_linear() {
    let nax = Numa::from_vec(vec![0.0, 2.0, 5.0, 10.0]);
    let nay = Numa::from_vec(vec![0.0, 20.0, 50.0, 100.0]);
    let val = nax
        .interpolate_arbx_val(InterpolationType::Linear, &nay, 1.0)
        .unwrap();
    // Between (0,0) and (2,20): at x=1 → y=10
    assert!((val - 10.0).abs() < 0.01, "expected 10.0, got {val}");
}

#[test]
fn test_interpolate_arbx_val_at_knot() {
    let nax = Numa::from_vec(vec![0.0, 2.0, 5.0, 10.0]);
    let nay = Numa::from_vec(vec![0.0, 20.0, 50.0, 100.0]);
    let val = nax
        .interpolate_arbx_val(InterpolationType::Linear, &nay, 5.0)
        .unwrap();
    assert!((val - 50.0).abs() < 0.01, "expected 50.0, got {val}");
}

#[test]
fn test_interpolate_arbx_val_out_of_range() {
    let nax = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let nay = Numa::from_vec(vec![10.0, 20.0, 30.0]);
    assert!(
        nax.interpolate_arbx_val(InterpolationType::Linear, &nay, 0.0)
            .is_err()
    );
    assert!(
        nax.interpolate_arbx_val(InterpolationType::Linear, &nay, 4.0)
            .is_err()
    );
}

#[test]
fn test_interpolate_arbx_val_length_mismatch() {
    let nax = Numa::from_vec(vec![0.0, 1.0]);
    let nay = Numa::from_vec(vec![0.0, 10.0, 20.0]);
    assert!(
        nax.interpolate_arbx_val(InterpolationType::Linear, &nay, 0.5)
            .is_err()
    );
}

// ============================================================================
// numaClipToInterval / clip_to_interval
// ============================================================================

#[test]
fn test_clip_to_interval_basic() {
    let na = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    let clipped = na.clip_to_interval(1, 3).unwrap();
    assert_eq!(clipped.as_slice(), &[20.0, 30.0, 40.0]);
}

#[test]
fn test_clip_to_interval_full() {
    let na = Numa::from_vec(vec![10.0, 20.0, 30.0]);
    let clipped = na.clip_to_interval(0, 2).unwrap();
    assert_eq!(clipped.as_slice(), &[10.0, 20.0, 30.0]);
}

#[test]
fn test_clip_to_interval_clamped() {
    let na = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    // last > len-1 should be clamped
    let clipped = na.clip_to_interval(3, 100).unwrap();
    assert_eq!(clipped.as_slice(), &[40.0, 50.0]);
}

#[test]
fn test_clip_to_interval_empty() {
    let na = Numa::new();
    assert!(na.clip_to_interval(0, 5).is_err());
}

// ============================================================================
// numaMakeThresholdIndicator / make_threshold_indicator
// ============================================================================

#[test]
fn test_threshold_indicator_less_than() {
    let na = Numa::from_vec(vec![1.0, 5.0, 3.0, 7.0, 2.0]);
    let ind = na.make_threshold_indicator(4.0, ThresholdComparison::LessThan);
    assert_eq!(ind.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0]);
}

#[test]
fn test_threshold_indicator_greater_than() {
    let na = Numa::from_vec(vec![1.0, 5.0, 3.0, 7.0, 2.0]);
    let ind = na.make_threshold_indicator(4.0, ThresholdComparison::GreaterThan);
    assert_eq!(ind.as_slice(), &[0.0, 1.0, 0.0, 1.0, 0.0]);
}

#[test]
fn test_threshold_indicator_less_than_or_equal() {
    let na = Numa::from_vec(vec![1.0, 4.0, 3.0, 7.0, 4.0]);
    let ind = na.make_threshold_indicator(4.0, ThresholdComparison::LessThanOrEqual);
    assert_eq!(ind.as_slice(), &[1.0, 1.0, 1.0, 0.0, 1.0]);
}

#[test]
fn test_threshold_indicator_greater_than_or_equal() {
    let na = Numa::from_vec(vec![1.0, 4.0, 3.0, 7.0, 4.0]);
    let ind = na.make_threshold_indicator(4.0, ThresholdComparison::GreaterThanOrEqual);
    assert_eq!(ind.as_slice(), &[0.0, 1.0, 0.0, 1.0, 1.0]);
}

#[test]
fn test_threshold_indicator_empty() {
    let na = Numa::new();
    let ind = na.make_threshold_indicator(5.0, ThresholdComparison::LessThan);
    assert!(ind.is_empty());
}

// ============================================================================
// numaGetNonzeroRange / get_nonzero_range
// ============================================================================

#[test]
fn test_nonzero_range_basic() {
    let na = Numa::from_vec(vec![0.0, 0.0, 3.0, 5.0, 0.0, 7.0, 0.0, 0.0]);
    let (first, last) = na.get_nonzero_range(0.001).unwrap().unwrap();
    assert_eq!(first, 2);
    assert_eq!(last, 5);
}

#[test]
fn test_nonzero_range_all_zero() {
    let na = Numa::from_vec(vec![0.0, 0.0, 0.0]);
    let result = na.get_nonzero_range(0.001).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_nonzero_range_all_nonzero() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let (first, last) = na.get_nonzero_range(0.001).unwrap().unwrap();
    assert_eq!(first, 0);
    assert_eq!(last, 2);
}

#[test]
fn test_nonzero_range_empty() {
    let na = Numa::new();
    assert!(na.get_nonzero_range(0.001).is_err());
}

// ============================================================================
// numaGetCountRelativeToZero / get_count_relative_to_zero
// ============================================================================

#[test]
fn test_count_less_than_zero() {
    let na = Numa::from_vec(vec![-3.0, -1.0, 0.0, 2.0, 5.0]);
    let count = na
        .get_count_relative_to_zero(CountRelativeToZero::LessThan)
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_count_equal_to_zero() {
    let na = Numa::from_vec(vec![-3.0, 0.0, 0.0, 2.0, 0.0]);
    let count = na
        .get_count_relative_to_zero(CountRelativeToZero::EqualTo)
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_count_greater_than_zero() {
    let na = Numa::from_vec(vec![-3.0, -1.0, 0.0, 2.0, 5.0]);
    let count = na
        .get_count_relative_to_zero(CountRelativeToZero::GreaterThan)
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_count_relative_to_zero_empty() {
    let na = Numa::new();
    assert!(
        na.get_count_relative_to_zero(CountRelativeToZero::LessThan)
            .is_err()
    );
}

// ============================================================================
// numaSubsample / subsample
// ============================================================================

#[test]
fn test_subsample_every_other() {
    let na = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    let sub = na.subsample(2).unwrap();
    assert_eq!(sub.as_slice(), &[10.0, 30.0, 50.0]);
}

#[test]
fn test_subsample_every_third() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    let sub = na.subsample(3).unwrap();
    assert_eq!(sub.as_slice(), &[1.0, 4.0, 7.0]);
}

#[test]
fn test_subsample_factor_one() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let sub = na.subsample(1).unwrap();
    assert_eq!(sub.as_slice(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_subsample_factor_larger_than_len() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    let sub = na.subsample(10).unwrap();
    assert_eq!(sub.as_slice(), &[1.0]);
}

#[test]
fn test_subsample_zero_factor() {
    let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    assert!(na.subsample(0).is_err());
}

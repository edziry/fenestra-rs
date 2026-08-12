use super::*;

#[test]
fn scalar_constants_and_domain_are_exact() {
    assert_eq!(SpatialScalarV2::FRACTIONAL_BITS, 16);
    assert_eq!(SpatialScalarV2::SCALE, SCALE);
    assert_eq!(SpatialScalarV2::MIN_RAW, -140_737_488_289_792);
    assert_eq!(SpatialScalarV2::MAX_RAW, 140_737_488_289_792);

    assert!(scalar(SpatialScalarV2::MIN_RAW).is_in_domain());
    assert!(scalar(SpatialScalarV2::MAX_RAW).is_in_domain());
    assert!(!scalar(SpatialScalarV2::MIN_RAW - 1).is_in_domain());
    assert!(!scalar(SpatialScalarV2::MAX_RAW + 1).is_in_domain());
    assert_eq!(scalar(i64::MIN).raw(), i64::MIN);
    assert_eq!(scalar(i64::MAX).raw(), i64::MAX);
}

#[test]
fn integer_conversion_reaches_both_domain_edges() {
    assert_eq!(
        SpatialScalarV2::checked_from_i32(i32::MAX),
        Some(scalar(SpatialScalarV2::MAX_RAW))
    );
    assert_eq!(SpatialScalarV2::checked_from_i32(i32::MIN), None);
    assert_eq!(
        SpatialScalarV2::checked_from_i32(i32::MIN + 1),
        Some(scalar(SpatialScalarV2::MIN_RAW))
    );
}

#[test]
fn checked_add_subtract_and_negate_preserve_exact_edges() {
    let minimum = scalar(SpatialScalarV2::MIN_RAW);
    let maximum = scalar(SpatialScalarV2::MAX_RAW);
    let tick = scalar(1);

    assert_eq!(
        maximum.checked_add(scalar(-1)),
        Some(scalar(maximum.raw() - 1))
    );
    assert_eq!(maximum.checked_add(tick), None);
    assert_eq!(minimum.checked_sub(tick), None);
    assert_eq!(minimum.checked_neg(), Some(maximum));
    assert_eq!(maximum.checked_neg(), Some(minimum));
    assert_eq!(maximum.checked_sub(minimum), None);
}

#[test]
fn checked_multiply_and_divide_round_once() {
    let tick = scalar(1);
    let half = scalar(SCALE / 2);
    let two = integer(2);

    assert_eq!(tick.checked_mul(half), Some(tick));
    assert_eq!(scalar(-1).checked_mul(half), Some(scalar(-1)));
    assert_eq!(tick.checked_div(two), Some(tick));
    assert_eq!(scalar(-1).checked_div(two), Some(scalar(-1)));
    assert_eq!(tick.checked_div(integer(-2)), Some(scalar(-1)));
    assert_eq!(scalar(-1).checked_div(integer(-2)), Some(tick));

    let maximum = scalar(SpatialScalarV2::MAX_RAW);
    assert_eq!(maximum.checked_mul(integer(1)), Some(maximum));
    assert_eq!(maximum.checked_mul(scalar(SCALE + 1)), None);
    assert_eq!(maximum.checked_div(scalar(SCALE / 2)), None);
    assert_eq!(maximum.checked_div(scalar(0)), None);
}

#[test]
fn checked_scalar_operations_reject_raw_out_of_domain_operands() {
    let invalid_low = scalar(SpatialScalarV2::MIN_RAW - 1);
    let invalid_high = scalar(SpatialScalarV2::MAX_RAW + 1);
    let zero = scalar(0);
    let one = integer(1);

    assert_eq!(invalid_low.checked_add(one), None);
    assert_eq!(one.checked_add(invalid_low), None);
    assert_eq!(invalid_high.checked_sub(one), None);
    assert_eq!(one.checked_sub(invalid_high), None);
    assert_eq!(invalid_low.checked_neg(), None);
    assert_eq!(zero.checked_mul(invalid_high), None);
    assert_eq!(invalid_high.checked_mul(zero), None);
    assert_eq!(zero.checked_div(invalid_high), None);
    assert_eq!(invalid_high.checked_div(integer(2)), None);
}

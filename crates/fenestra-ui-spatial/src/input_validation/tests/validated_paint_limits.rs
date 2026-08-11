use super::check_paint_item_limit;
use super::validated_paint_support::{expect_limit, expect_valid, fill, fixture, limits, validate};
use crate::coverage::SpatialFillRuleV2;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

fn paint(owner: u32, ordinal: u32) -> crate::paint::SpatialPaintV2 {
    fill(
        owner,
        ordinal,
        owner - 1,
        0,
        None,
        SpatialFillRuleV2::NonZero,
    )
}

#[test]
fn the_per_node_count_resets_for_each_owner_and_equality_succeeds() {
    let fixture = fixture(vec![paint(1, 0), paint(1, 1), paint(2, 0), paint(2, 1)]);
    expect_valid(validate(&fixture, limits(2)));
}

#[test]
fn the_first_candidate_above_the_per_node_limit_reports_the_full_count() {
    let fixture = fixture(vec![
        paint(1, 0),
        paint(1, 1),
        paint(2, 0),
        paint(2, 1),
        paint(2, 2),
    ]);
    expect_limit(validate(&fixture, limits(2)), 4, 3, 2);
}

#[test]
fn the_candidate_limit_precedes_variant_validation_on_the_same_record() {
    let fixture = fixture(vec![
        paint(1, 0),
        fill(
            1,
            1,
            u32::MAX,
            u32::MAX,
            Some(u32::MAX),
            SpatialFillRuleV2::EvenOdd,
        ),
    ]);
    expect_limit(validate(&fixture, limits(1)), 1, 2, 1);
}

#[test]
fn the_registered_maximum_is_enforced_but_does_not_cap_the_caller() {
    const REGISTERED: usize = 64;
    assert_eq!(
        REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PaintItemsPerNode),
        REGISTERED
    );
    let paints = (0..=REGISTERED)
        .map(|ordinal| paint(1, ordinal as u32))
        .collect::<Vec<_>>();
    let fixture = fixture(paints);

    expect_limit(
        validate(&fixture, limits(REGISTERED)),
        REGISTERED as u32,
        65,
        64,
    );
    expect_valid(validate(&fixture, limits(REGISTERED + 1)));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn the_limit_helper_preserves_counts_above_u32_without_an_extra_ceiling() {
    let observed = usize::try_from(u32::MAX as u128 + 1).expect("the test requires 64-bit usize");
    expect_valid(check_paint_item_limit(u32::MAX, observed, limits(observed)));
    expect_limit(
        check_paint_item_limit(u32::MAX, observed, limits(u32::MAX as usize)),
        u32::MAX,
        u32::MAX as u128 + 1,
        u32::MAX as u128,
    );
}

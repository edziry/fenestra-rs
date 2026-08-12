use super::check_hit_item_limit;
use super::validated_hit_support::{expect_limit, expect_valid, fill, fixture, limits, validate};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

fn hit(owner: u32, ordinal: u32) -> crate::content_item::SpatialHitV2 {
    fill(
        owner,
        ordinal,
        owner - 1,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )
}

#[test]
fn count_resets_per_owner_and_equality_succeeds() {
    let fixture = fixture(vec![hit(1, 0), hit(1, 1), hit(2, 0), hit(2, 1)]);
    expect_valid(validate(&fixture, limits(2)));
}

#[test]
fn first_candidate_above_the_limit_reports_the_current_record_and_full_count() {
    let fixture = fixture(vec![hit(1, 0), hit(1, 1), hit(2, 0), hit(2, 1), hit(2, 2)]);
    expect_limit(validate(&fixture, limits(2)), 4, 3, 2);
}

#[test]
fn candidate_limit_precedes_coverage_validation_on_the_same_record() {
    let fixture = fixture(vec![
        hit(1, 0),
        fill(
            1,
            1,
            u32::MAX,
            Some(u32::MAX),
            SpatialFillRuleV2::EvenOdd,
            SpatialInputPolicyV2::Ignore,
        ),
    ]);
    expect_limit(validate(&fixture, limits(1)), 1, 2, 1);
}

#[test]
fn registered_maximum_is_enforced_without_capping_a_custom_caller() {
    const REGISTERED: usize = 64;
    assert_eq!(
        REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::HitItemsPerNode),
        REGISTERED
    );
    let hits = (0..=REGISTERED)
        .map(|ordinal| hit(1, ordinal as u32))
        .collect::<Vec<_>>();
    let fixture = fixture(hits);

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
fn limit_helper_preserves_counts_and_callers_above_u32() {
    let observed = usize::try_from(u32::MAX as u128 + 1).expect("the test requires 64-bit usize");
    expect_valid(check_hit_item_limit(u32::MAX, observed, limits(observed)));
    expect_limit(
        check_hit_item_limit(u32::MAX, observed, limits(u32::MAX as usize)),
        u32::MAX,
        u32::MAX as u128 + 1,
        u32::MAX as u128,
    );
}

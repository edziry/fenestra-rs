use super::check_clip_depth;
use super::validated_clip_support::{
    clip, expect_depth, expect_valid, limits, root_clip, standard_fixture, validate,
};
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

#[test]
fn a_root_clip_has_depth_one_with_an_inclusive_limit() {
    let fixture = standard_fixture(vec![root_clip(0, 1, 0)]);

    expect_depth(validate(&fixture, limits(0)), 0, 1, 0);
    expect_valid(validate(&fixture, limits(1)));
}

#[test]
fn chain_depth_uses_the_complete_parent_chain_and_exact_crossing() {
    let fixture = standard_fixture(chain(3));

    expect_depth(validate(&fixture, limits(2)), 2, 3, 2);
    expect_valid(validate(&fixture, limits(3)));
}

#[test]
fn depth_follows_parent_branches_instead_of_the_clip_ordinal() {
    let branch = vec![
        root_clip(0, 1, 0),
        clip(1, 1, Some(0), 0, SpatialFillRuleV2::NonZero),
        clip(2, 1, Some(0), 0, SpatialFillRuleV2::EvenOdd),
    ];
    let fixture = standard_fixture(branch.clone());
    expect_valid(validate(&fixture, limits(2)));

    let mut deeper = branch;
    deeper.push(clip(3, 1, Some(2), 0, SpatialFillRuleV2::NonZero));
    let fixture = standard_fixture(deeper);
    expect_depth(validate(&fixture, limits(2)), 3, 3, 2);
}

#[test]
fn caller_depth_limits_are_not_capped_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::ClipDepth);
    assert_eq!(registered, 32);
    let fixture = standard_fixture(chain(33));

    expect_depth(validate(&fixture, limits(registered)), 32, 33, 32);
    expect_valid(validate(&fixture, limits(registered + 1)));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn depth_helper_preserves_caller_maxima_and_evidence_above_u32() {
    let above_u32 = u32::MAX as usize + 1;

    expect_valid(check_clip_depth(u32::MAX, above_u32, limits(above_u32)));
    expect_depth(
        check_clip_depth(u32::MAX, above_u32, limits(u32::MAX as usize)),
        u32::MAX,
        u128::from(u32::MAX) + 1,
        u128::from(u32::MAX),
    );
}

fn chain(count: u32) -> Vec<SpatialClipV2> {
    (0..count)
        .map(|index| {
            clip(
                index,
                1,
                index.checked_sub(1),
                0,
                SpatialFillRuleV2::NonZero,
            )
        })
        .collect()
}

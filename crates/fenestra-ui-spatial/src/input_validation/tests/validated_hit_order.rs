use super::validated_hit_support::{
    expect_order, expect_reference, expect_valid, fill, fixture, limits, validate,
};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialHitFieldV2;

fn valid(owner: u32, ordinal: u32) -> crate::content_item::SpatialHitV2 {
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
fn sentinel_absent_and_maximum_owners_are_invalid_references() {
    for owner in [0, 5, u32::MAX] {
        let fixture = fixture(vec![fill(
            owner,
            99,
            0,
            Some(u32::MAX),
            SpatialFillRuleV2::EvenOdd,
            SpatialInputPolicyV2::Ignore,
        )]);
        expect_reference(
            validate(&fixture, limits(0)),
            SpatialContentReferenceV2::Owner,
            0,
            SpatialHitFieldV2::Owner,
        );
    }
}

#[test]
fn owner_reference_precedes_owner_order_and_local_ordinal() {
    let fixture = fixture(vec![
        valid(1, 0),
        valid(2, 0),
        fill(
            0,
            99,
            0,
            None,
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Accept,
        ),
    ]);
    expect_reference(
        validate(&fixture, limits(usize::MAX)),
        SpatialContentReferenceV2::Owner,
        2,
        SpatialHitFieldV2::Owner,
    );
}

#[test]
fn decreasing_and_reopened_owners_fail_at_owner_before_a_bad_ordinal() {
    for hits in [
        vec![valid(2, 0), valid(1, u32::MAX)],
        vec![valid(1, 0), valid(2, 0), valid(1, u32::MAX)],
    ] {
        let index = (hits.len() - 1) as u32;
        let fixture = fixture(hits);
        expect_order(
            validate(&fixture, limits(usize::MAX)),
            index,
            SpatialHitFieldV2::Owner,
        );
    }
}

#[test]
fn first_duplicate_gapped_and_maximum_local_ordinals_are_rejected() {
    let first = fixture(vec![valid(1, 1)]);
    expect_order(
        validate(&first, limits(0)),
        0,
        SpatialHitFieldV2::ItemOrdinal,
    );

    for ordinal in [0, 2, u32::MAX] {
        let fixture = fixture(vec![valid(1, 0), valid(1, ordinal)]);
        expect_order(
            validate(&fixture, limits(usize::MAX)),
            1,
            SpatialHitFieldV2::ItemOrdinal,
        );
    }
}

#[test]
fn local_ordinal_precedes_the_candidate_limit() {
    let fixture = fixture(vec![valid(1, 1)]);
    expect_order(
        validate(&fixture, limits(0)),
        0,
        SpatialHitFieldV2::ItemOrdinal,
    );
}

#[test]
fn owners_may_start_late_and_skip_nodes_with_independent_local_orders() {
    let fixture = fixture(vec![valid(2, 0), valid(4, 0)]);
    expect_valid(validate(&fixture, limits(1)));
}

use super::validated_semantic_support::{
    expect_order, expect_reference, expect_valid, fixture, limits, no_item_limits, semantic,
    validate,
};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialSemanticFieldV2;

fn valid(owner: u32, ordinal: u32) -> crate::content_item::SpatialSemanticGeometryV2 {
    semantic(owner, ordinal, owner - 1, SpatialFillRuleV2::NonZero, None)
}

#[test]
fn sentinel_absent_and_maximum_owners_are_invalid_references() {
    for owner in [0, 5, u32::MAX] {
        let fixture = fixture(vec![semantic(
            owner,
            u32::MAX,
            u32::MAX,
            SpatialFillRuleV2::EvenOdd,
            Some(u32::MAX),
        )]);
        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Owner,
            0,
            SpatialSemanticFieldV2::Owner,
        );
    }
}

#[test]
fn owner_reference_precedes_owner_order_and_local_ordinal() {
    let fixture = fixture(vec![
        valid(1, 0),
        valid(2, 0),
        semantic(
            0,
            u32::MAX,
            u32::MAX,
            SpatialFillRuleV2::EvenOdd,
            Some(u32::MAX),
        ),
    ]);
    expect_reference(
        validate(&fixture, limits()),
        SpatialContentReferenceV2::Owner,
        2,
        SpatialSemanticFieldV2::Owner,
    );
}

#[test]
fn decreasing_and_reopened_owners_fail_at_owner_before_a_bad_ordinal() {
    let poisoned = |owner| {
        semantic(
            owner,
            u32::MAX,
            u32::MAX,
            SpatialFillRuleV2::EvenOdd,
            Some(u32::MAX),
        )
    };
    for semantics in [
        vec![valid(2, 0), poisoned(1)],
        vec![valid(1, 0), valid(2, 0), poisoned(1)],
    ] {
        let index = (semantics.len() - 1) as u32;
        let fixture = fixture(semantics);
        expect_order(
            validate(&fixture, limits()),
            index,
            SpatialSemanticFieldV2::Owner,
        );
    }
}

#[test]
fn first_duplicate_gapped_and_maximum_local_ordinals_are_rejected() {
    let poisoned = |ordinal| {
        semantic(
            1,
            ordinal,
            u32::MAX,
            SpatialFillRuleV2::EvenOdd,
            Some(u32::MAX),
        )
    };
    let first = fixture(vec![poisoned(1)]);
    expect_order(
        validate(&first, limits()),
        0,
        SpatialSemanticFieldV2::ItemOrdinal,
    );

    for ordinal in [0, 2, u32::MAX] {
        let fixture = fixture(vec![valid(1, 0), poisoned(ordinal)]);
        expect_order(
            validate(&fixture, limits()),
            1,
            SpatialSemanticFieldV2::ItemOrdinal,
        );
    }
}

#[test]
fn owners_may_start_late_and_skip_nodes_with_independent_local_orders() {
    let fixture = fixture(vec![valid(2, 0), valid(4, 0)]);
    expect_valid(validate(&fixture, limits()));
}

#[test]
fn semantic_owner_counts_have_no_derived_or_registered_per_node_limit() {
    let semantics = (0..257)
        .map(|ordinal| valid(1, ordinal))
        .collect::<Vec<_>>();
    let fixture = fixture(semantics);
    expect_valid(validate(&fixture, no_item_limits(257)));
}

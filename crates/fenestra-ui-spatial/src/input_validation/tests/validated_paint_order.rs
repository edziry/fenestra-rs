use super::validated_paint_support::{
    expect_order, expect_reference, expect_valid, fill, fixture, limits, validate,
};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialPaintFieldV2;

fn valid(owner: u32, ordinal: u32) -> crate::paint::SpatialPaintV2 {
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
fn sentinel_absent_and_maximum_owners_are_invalid_references() {
    for owner in [0, 5, u32::MAX] {
        let fixture = fixture(vec![fill(owner, 9, 0, 0, None, SpatialFillRuleV2::NonZero)]);
        expect_reference(
            validate(&fixture, limits(0)),
            SpatialContentReferenceV2::Owner,
            0,
            SpatialPaintFieldV2::Owner,
        );
    }
}

#[test]
fn owner_reference_precedes_owner_order_and_local_ordinal() {
    let fixture = fixture(vec![
        valid(1, 0),
        valid(2, 0),
        fill(0, 99, 0, 0, None, SpatialFillRuleV2::NonZero),
    ]);
    expect_reference(
        validate(&fixture, limits(usize::MAX)),
        SpatialContentReferenceV2::Owner,
        2,
        SpatialPaintFieldV2::Owner,
    );
}

#[test]
fn decreasing_and_reopened_owners_are_rejected_at_the_current_record() {
    for paints in [
        vec![valid(2, 0), valid(1, 0)],
        vec![valid(1, 0), valid(2, 0), valid(1, 1)],
    ] {
        let index = (paints.len() - 1) as u32;
        let fixture = fixture(paints);
        expect_order(
            validate(&fixture, limits(usize::MAX)),
            index,
            SpatialPaintFieldV2::Owner,
        );
    }
}

#[test]
fn owner_order_precedes_a_bad_local_ordinal_on_the_same_record() {
    let fixture = fixture(vec![valid(2, 0), valid(1, u32::MAX)]);
    expect_order(
        validate(&fixture, limits(usize::MAX)),
        1,
        SpatialPaintFieldV2::Owner,
    );
}

#[test]
fn the_first_owner_local_ordinal_must_be_zero() {
    let fixture = fixture(vec![valid(1, 1)]);
    expect_order(
        validate(&fixture, limits(0)),
        0,
        SpatialPaintFieldV2::ItemOrdinal,
    );
}

#[test]
fn duplicate_and_gapped_local_ordinals_are_rejected() {
    for ordinal in [0, 2, u32::MAX] {
        let fixture = fixture(vec![valid(1, 0), valid(1, ordinal)]);
        expect_order(
            validate(&fixture, limits(usize::MAX)),
            1,
            SpatialPaintFieldV2::ItemOrdinal,
        );
    }
}

#[test]
fn local_ordinal_is_checked_before_the_candidate_limit() {
    let fixture = fixture(vec![valid(1, 1)]);
    expect_order(
        validate(&fixture, limits(0)),
        0,
        SpatialPaintFieldV2::ItemOrdinal,
    );
}

#[test]
fn owners_may_start_late_and_skip_nodes_while_each_local_order_starts_at_zero() {
    let fixture = fixture(vec![valid(2, 0), valid(4, 0)]);
    expect_valid(validate(&fixture, limits(1)));
}

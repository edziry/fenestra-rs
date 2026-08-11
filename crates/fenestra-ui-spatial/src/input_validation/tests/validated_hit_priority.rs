use super::local_transform_support::VIEWPORT;
use super::validated_hit_support::{
    expect_reference, fill, fixture, fixture_with_paints, limits, stroke, validate,
};
use super::validated_paint_support::{destination, image_paint, source};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2};
use crate::model::SpatialScalarV2;

fn late_clip(owner: u32, ordinal: u32) -> crate::content_item::SpatialHitV2 {
    fill(
        owner,
        ordinal,
        owner - 1,
        Some(99),
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Ignore,
    )
}

#[test]
fn a_late_hit_zero_failure_beats_later_header_shape_and_stroke_stages() {
    let cases = [
        (
            late_clip(1, 0),
            fill(
                0,
                99,
                0,
                None,
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            ),
            usize::MAX,
        ),
        (
            late_clip(2, 0),
            fill(
                1,
                0,
                0,
                None,
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            ),
            usize::MAX,
        ),
        (
            late_clip(1, 0),
            fill(
                1,
                2,
                0,
                None,
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            ),
            usize::MAX,
        ),
        (
            late_clip(1, 0),
            fill(
                1,
                1,
                0,
                None,
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            ),
            1,
        ),
        (
            late_clip(1, 0),
            fill(
                1,
                1,
                u32::MAX,
                None,
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            ),
            usize::MAX,
        ),
        (
            late_clip(1, 0),
            stroke(
                1,
                1,
                0,
                SpatialScalarV2::MAX_RAW + 1,
                None,
                SpatialInputPolicyV2::Accept,
            ),
            usize::MAX,
        ),
    ];
    for (first, second, maximum) in cases {
        let fixture = fixture(vec![first, second]);
        expect_reference(
            validate(&fixture, limits(maximum)),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialHitFieldV2::Clip,
        );
    }
}

#[test]
fn stroke_and_fill_records_are_not_batched_by_coverage_kind() {
    let stroke_first = fixture(vec![
        stroke(1, 0, 0, 1, Some(99), SpatialInputPolicyV2::Ignore),
        fill(
            1,
            1,
            u32::MAX,
            None,
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Accept,
        ),
    ]);
    expect_reference(
        validate(&stroke_first, limits(2)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialHitFieldV2::Clip,
    );

    let fill_first = fixture(vec![
        fill(
            1,
            0,
            u32::MAX,
            None,
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Accept,
        ),
        stroke(1, 1, 0, 1, Some(99), SpatialInputPolicyV2::Ignore),
    ]);
    expect_reference(
        validate(&fill_first, limits(2)),
        SpatialContentReferenceV2::Shape,
        0,
        SpatialHitFieldV2::Shape,
    );
}

#[test]
fn every_paint_record_precedes_the_first_hit_record() {
    let paints = vec![super::validated_paint_support::fill(
        1,
        0,
        0,
        0,
        Some(99),
        SpatialFillRuleV2::NonZero,
    )];
    let hits = vec![fill(
        0,
        99,
        u32::MAX,
        Some(u32::MAX),
        SpatialFillRuleV2::EvenOdd,
        SpatialInputPolicyV2::Ignore,
    )];
    let fixture = fixture_with_paints(paints, hits);
    super::validated_paint_support::expect_reference(
        prepare_validated_hit_items!(&fixture, VIEWPORT, limits(0)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialPaintFieldV2::Clip,
    );
}

#[test]
fn the_last_hit_field_precedes_an_earlier_deferred_image_far_edge() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let paints = vec![image_paint(
        1,
        0,
        0,
        source(0, 0, 1, 1),
        destination(maximum, maximum, 1, 1),
        None,
    )];
    let hits = vec![fill(
        1,
        0,
        0,
        Some(99),
        SpatialFillRuleV2::EvenOdd,
        SpatialInputPolicyV2::Ignore,
    )];
    let fixture = fixture_with_paints(paints, hits);
    expect_reference(
        validate(&fixture, limits(1)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialHitFieldV2::Clip,
    );
}

#[test]
fn paint_and_hit_owner_local_ordinals_are_independent_tables() {
    let paints = vec![super::validated_paint_support::fill(
        1,
        0,
        0,
        0,
        None,
        SpatialFillRuleV2::NonZero,
    )];
    let hits = vec![fill(
        1,
        0,
        0,
        None,
        SpatialFillRuleV2::EvenOdd,
        SpatialInputPolicyV2::Accept,
    )];
    let fixture = fixture_with_paints(paints, hits);
    super::validated_hit_support::expect_valid(validate(&fixture, limits(1)));
}

use super::local_transform_support::VIEWPORT;
use super::validated_hit_support::{fill as hit_fill, limits as hit_limits};
use super::validated_paint_support::{destination, image_paint, source};
use super::validated_semantic_support::{
    expect_reference, fixture, fixture_with_items, limits, semantic, validate,
};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2, SpatialSemanticFieldV2};
use crate::model::SpatialScalarV2;

fn late_clip(owner: u32, ordinal: u32) -> crate::content_item::SpatialSemanticGeometryV2 {
    semantic(
        owner,
        ordinal,
        owner - 1,
        SpatialFillRuleV2::NonZero,
        Some(99),
    )
}

#[test]
fn a_late_semantic_clip_beats_every_stage_of_the_next_record() {
    let cases = [
        (
            late_clip(1, 0),
            semantic(0, u32::MAX, 0, SpatialFillRuleV2::EvenOdd, None),
        ),
        (
            late_clip(2, 0),
            semantic(1, u32::MAX, 0, SpatialFillRuleV2::EvenOdd, None),
        ),
        (
            late_clip(1, 0),
            semantic(1, 2, 0, SpatialFillRuleV2::EvenOdd, None),
        ),
        (
            late_clip(1, 0),
            semantic(1, 1, u32::MAX, SpatialFillRuleV2::EvenOdd, None),
        ),
    ];
    for (first, second) in cases {
        let fixture = fixture(vec![first, second]);
        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialSemanticFieldV2::Clip,
        );
    }
}

#[test]
fn semantic_shapes_and_clips_are_not_batched_by_field() {
    let shape_first = fixture(vec![
        semantic(1, 0, u32::MAX, SpatialFillRuleV2::NonZero, None),
        late_clip(1, 1),
    ]);
    expect_reference(
        validate(&shape_first, limits()),
        SpatialContentReferenceV2::Shape,
        0,
        SpatialSemanticFieldV2::Shape,
    );

    let clip_first = fixture(vec![
        late_clip(1, 0),
        semantic(1, 1, u32::MAX, SpatialFillRuleV2::NonZero, None),
    ]);
    expect_reference(
        validate(&clip_first, limits()),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialSemanticFieldV2::Clip,
    );
}

#[test]
fn paint_then_hit_then_semantic_tables_finish_in_that_order() {
    let paints = vec![super::validated_paint_support::fill(
        1,
        0,
        0,
        0,
        Some(99),
        SpatialFillRuleV2::NonZero,
    )];
    let hits = vec![hit_fill(
        0,
        u32::MAX,
        u32::MAX,
        Some(u32::MAX),
        SpatialFillRuleV2::EvenOdd,
        SpatialInputPolicyV2::Ignore,
    )];
    let semantics = vec![semantic(
        0,
        u32::MAX,
        u32::MAX,
        SpatialFillRuleV2::EvenOdd,
        Some(u32::MAX),
    )];
    let fixture = fixture_with_items(paints, hits, semantics);
    super::validated_paint_support::expect_reference(
        prepare_validated_semantic_items!(&fixture, VIEWPORT, limits()),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialPaintFieldV2::Clip,
    );

    let hits = vec![hit_fill(
        1,
        0,
        0,
        Some(99),
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )];
    let semantics = vec![semantic(
        0,
        u32::MAX,
        u32::MAX,
        SpatialFillRuleV2::EvenOdd,
        Some(u32::MAX),
    )];
    let fixture = fixture_with_items(Vec::new(), hits, semantics);
    super::validated_hit_support::expect_reference(
        prepare_validated_semantic_items!(&fixture, VIEWPORT, hit_limits(1)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialHitFieldV2::Clip,
    );
}

#[test]
fn the_last_semantic_field_precedes_an_earlier_deferred_image_far_edge() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let paints = vec![image_paint(
        1,
        0,
        0,
        source(0, 0, 1, 1),
        destination(maximum, maximum, 1, 1),
        None,
    )];
    let hits = vec![hit_fill(
        1,
        0,
        0,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )];
    let semantics = vec![late_clip(1, 0)];
    let fixture = fixture_with_items(paints, hits, semantics);
    expect_reference(
        validate(&fixture, limits()),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialSemanticFieldV2::Clip,
    );
}

use super::validated_semantic_support::{
    expect_clip, expect_reference, expect_valid, fixture, limits, semantic, validate,
};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialSemanticFieldV2;

#[test]
fn missing_and_wrong_owner_shapes_are_invalid_references_at_the_target_record() {
    for shape in [1, 4, u32::MAX] {
        let fixture = fixture(vec![
            semantic(1, 0, 0, SpatialFillRuleV2::NonZero, None),
            semantic(1, 1, shape, SpatialFillRuleV2::EvenOdd, Some(u32::MAX)),
        ]);
        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Shape,
            1,
            SpatialSemanticFieldV2::Shape,
        );
    }
}

#[test]
fn shape_validation_precedes_the_terminal_clip_on_the_same_record() {
    let fixture = fixture(vec![semantic(
        1,
        0,
        u32::MAX,
        SpatialFillRuleV2::EvenOdd,
        Some(u32::MAX),
    )]);
    expect_reference(
        validate(&fixture, limits()),
        SpatialContentReferenceV2::Shape,
        0,
        SpatialSemanticFieldV2::Shape,
    );
}

#[test]
fn future_shapes_and_none_same_owner_and_ancestor_clips_are_valid() {
    let fixture = fixture(vec![
        semantic(1, 0, 0, SpatialFillRuleV2::NonZero, None),
        semantic(1, 1, 0, SpatialFillRuleV2::EvenOdd, Some(0)),
        semantic(3, 0, 2, SpatialFillRuleV2::NonZero, Some(2)),
        semantic(3, 1, 2, SpatialFillRuleV2::EvenOdd, Some(1)),
        semantic(3, 2, 2, SpatialFillRuleV2::NonZero, Some(0)),
        semantic(4, 0, 3, SpatialFillRuleV2::EvenOdd, Some(3)),
    ]);
    expect_valid(validate(&fixture, limits()));
}

#[test]
fn absent_and_maximum_terminal_clips_are_invalid_references() {
    for clip in [4, u32::MAX] {
        let fixture = fixture(vec![
            semantic(1, 0, 0, SpatialFillRuleV2::NonZero, None),
            semantic(1, 1, 0, SpatialFillRuleV2::EvenOdd, Some(clip)),
        ]);
        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Clip,
            1,
            SpatialSemanticFieldV2::Clip,
        );
    }
}

#[test]
fn unrelated_descendant_and_lower_unrelated_clips_are_rejected() {
    for (owner, shape, clip) in [(3, 2, 3), (1, 0, 1), (4, 3, 0)] {
        let fixture = fixture(vec![semantic(
            owner,
            0,
            shape,
            SpatialFillRuleV2::NonZero,
            Some(clip),
        )]);
        expect_clip(validate(&fixture, limits()), 0);
    }
}

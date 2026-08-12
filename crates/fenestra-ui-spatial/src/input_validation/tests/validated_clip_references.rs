use super::validated_clip_support::{
    clip, clip_location, expect_clip, expect_reference, expect_valid, limits, root_clip,
    standard_fixture, validate,
};
use crate::content_diagnostic::{SpatialClipErrorV2, SpatialContentReferenceV2};
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialClipFieldV2;

#[test]
fn root_sentinel_and_missing_owners_precede_parent_and_shape_faults() {
    for owner in [0, 3, u32::MAX] {
        let fixture = standard_fixture(vec![clip(
            0,
            owner,
            Some(u32::MAX),
            u32::MAX,
            SpatialFillRuleV2::NonZero,
        )]);

        expect_reference(
            validate(&fixture, limits(0)),
            SpatialContentReferenceV2::Owner,
            clip_location(0, SpatialClipFieldV2::Owner),
        );
    }
}

#[test]
fn missing_parents_precede_shapes_and_are_not_called_forward() {
    for parent in [1, u32::MAX] {
        let fixture = standard_fixture(vec![clip(
            0,
            1,
            Some(parent),
            u32::MAX,
            SpatialFillRuleV2::NonZero,
        )]);

        expect_reference(
            validate(&fixture, limits(0)),
            SpatialContentReferenceV2::Clip,
            clip_location(0, SpatialClipFieldV2::Parent),
        );
    }
}

#[test]
fn self_and_existing_future_parents_are_forward() {
    let self_parent = standard_fixture(vec![clip(
        0,
        1,
        Some(0),
        u32::MAX,
        SpatialFillRuleV2::NonZero,
    )]);
    expect_clip(
        validate(&self_parent, limits(0)),
        SpatialClipErrorV2::ForwardParent,
        clip_location(0, SpatialClipFieldV2::Parent),
    );

    let future = standard_fixture(vec![
        clip(0, 1, Some(1), u32::MAX, SpatialFillRuleV2::NonZero),
        root_clip(1, 1, 0),
    ]);
    expect_clip(
        validate(&future, limits(0)),
        SpatialClipErrorV2::ForwardParent,
        clip_location(0, SpatialClipFieldV2::Parent),
    );
}

#[test]
fn missing_shapes_use_shape_after_parent_validation() {
    for shape in [2, u32::MAX] {
        let fixture = standard_fixture(vec![root_clip(0, 1, shape)]);

        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Shape,
            clip_location(0, SpatialClipFieldV2::Shape),
        );
    }
}

#[test]
fn a_future_shape_key_is_a_valid_reference() {
    let fixture = standard_fixture(vec![root_clip(0, 2, 1)]);
    expect_valid(validate(&fixture, limits(1)));
}

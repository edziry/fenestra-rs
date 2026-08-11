use super::validated_clip_ancestry::ancestry_fixture;
use super::validated_clip_support::{
    clip, clip_location, expect_clip, expect_depth, expect_reference, limits, root_clip,
    standard_fixture, validate,
};
use super::validated_image_support::{expect_image, image, image_location};
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialImageErrorV2,
};
use crate::coverage::SpatialFillRuleV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::item_field::SpatialClipFieldV2;

#[test]
fn shape_owner_mismatch_precedes_parent_owner_ancestry() {
    let fixture = ancestry_fixture(vec![
        root_clip(0, 3, 2),
        clip(1, 2, Some(0), 0, SpatialFillRuleV2::NonZero),
    ]);

    expect_clip(
        validate(&fixture, limits(2)),
        SpatialClipErrorV2::ShapeOwnerMismatch,
        clip_location(1, SpatialClipFieldV2::Shape),
    );
}

#[test]
fn parent_owner_ancestry_precedes_clip_depth() {
    let fixture = ancestry_fixture(vec![
        root_clip(0, 3, 2),
        clip(1, 2, Some(0), 1, SpatialFillRuleV2::NonZero),
    ]);

    expect_clip(
        validate(&fixture, limits(1)),
        SpatialClipErrorV2::OwnerNotAncestor,
        clip_location(1, SpatialClipFieldV2::Parent),
    );
}

#[test]
fn record_major_order_beats_global_owner_or_depth_passes() {
    let early_depth = standard_fixture(vec![root_clip(0, 1, 0), root_clip(1, 0, 0)]);
    expect_depth(validate(&early_depth, limits(0)), 0, 1, 0);

    let early_owner = standard_fixture(vec![root_clip(0, 0, 0), root_clip(1, 1, 0)]);
    expect_reference(
        validate(&early_owner, limits(0)),
        SpatialContentReferenceV2::Owner,
        clip_location(0, SpatialClipFieldV2::Owner),
    );
}

#[test]
fn an_earlier_depth_failure_beats_every_later_intermediate_stage() {
    let cases = [
        standard_fixture(vec![
            root_clip(0, 1, 0),
            clip(1, 2, Some(2), 1, SpatialFillRuleV2::NonZero),
        ]),
        standard_fixture(vec![root_clip(0, 1, 0), root_clip(1, 2, 2)]),
        standard_fixture(vec![root_clip(0, 1, 0), root_clip(1, 2, 0)]),
        ancestry_fixture(vec![
            root_clip(0, 1, 0),
            clip(1, 5, Some(0), 4, SpatialFillRuleV2::NonZero),
        ]),
    ];

    for fixture in cases {
        expect_depth(validate(&fixture, limits(0)), 0, 1, 0);
    }
}

#[test]
fn validated_image_failures_precede_clip_keys_and_records() {
    let fixture = standard_fixture(vec![root_clip(u32::MAX, 0, u32::MAX)])
        .with_images(vec![image(0, 0, 0, u32::MAX, Vec::new())]);

    expect_image(
        validate(&fixture, limits(0)),
        SpatialImageErrorV2::ZeroExtent,
        image_location(0, SpatialImageFieldV2::Width),
    );
}

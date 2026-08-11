use super::local_transform_support::{free_node, identity, root};
use super::validated_clip_support::{
    clip, clip_location, expect_clip, expect_valid, fixture_with, limits, root_clip,
    standard_fixture, validate,
};
use super::validated_shape_support::rect;
use crate::content_diagnostic::SpatialClipErrorV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialClipFieldV2;

#[test]
fn a_clip_shape_must_have_the_same_owner() {
    let fixture = standard_fixture(vec![root_clip(0, 1, 1)]);

    expect_clip(
        validate(&fixture, limits(1)),
        SpatialClipErrorV2::ShapeOwnerMismatch,
        clip_location(0, SpatialClipFieldV2::Shape),
    );
}

#[test]
fn same_owner_and_spatial_ancestor_parent_owners_are_valid() {
    let fixture = ancestry_fixture(vec![
        root_clip(0, 1, 0),
        clip(1, 1, Some(0), 0, SpatialFillRuleV2::EvenOdd),
        clip(2, 2, Some(0), 1, SpatialFillRuleV2::NonZero),
        clip(3, 3, Some(0), 2, SpatialFillRuleV2::EvenOdd),
    ]);

    expect_valid(validate(&fixture, limits(2)));
}

#[test]
fn a_lower_key_and_shallower_unrelated_owner_is_not_an_ancestor() {
    let fixture = ancestry_fixture(vec![
        root_clip(0, 1, 0),
        clip(1, 5, Some(0), 4, SpatialFillRuleV2::NonZero),
    ]);

    expect_clip(
        validate(&fixture, limits(2)),
        SpatialClipErrorV2::OwnerNotAncestor,
        clip_location(1, SpatialClipFieldV2::Parent),
    );
}

#[test]
fn a_parent_clip_owned_by_a_descendant_is_not_an_ancestor() {
    let fixture = ancestry_fixture(vec![
        root_clip(0, 2, 1),
        clip(1, 1, Some(0), 0, SpatialFillRuleV2::NonZero),
    ]);

    expect_clip(
        validate(&fixture, limits(2)),
        SpatialClipErrorV2::OwnerNotAncestor,
        clip_location(1, SpatialClipFieldV2::Parent),
    );
}

pub(super) fn ancestry_fixture(
    clips: Vec<crate::coverage::SpatialClipV2>,
) -> super::fixture::RawInputFixture {
    let transform = identity();
    fixture_with(
        vec![
            root(),
            free_node(1, 0, 10, 10, transform),
            free_node(2, 1, 10, 10, transform),
            free_node(3, 2, 10, 10, transform),
            free_node(4, 0, 10, 10, transform),
            free_node(5, 4, 10, 10, transform),
        ],
        vec![rect(0, 1), rect(1, 2), rect(2, 3), rect(3, 4), rect(4, 5)],
        clips,
    )
}

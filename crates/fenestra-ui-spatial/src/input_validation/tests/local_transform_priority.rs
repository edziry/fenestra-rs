use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};

use super::layout_preflight_support::expect_input;
use super::local_transform_support::{
    Placement, VIEWPORT, container, expect_transform, identity_values, input, limits, node, root,
    root_with_container, set_field, transform, validate,
};
use crate::error::{SpatialContainerErrorKindV2, SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::model::SpatialScalarV2;
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::vocabulary::{SpatialNodeFieldV2, SpatialTransformScalarFieldV2};

#[test]
fn every_scalar_on_one_node_precedes_its_singular_determinant() {
    let mut values = [1, 1, 1, 1, 0, 0, 0, 0];
    set_field(
        &mut values,
        SpatialTransformScalarFieldV2::TransformOriginY,
        SpatialScalarV2::MAX_RAW + 1,
    );
    let fixture = input(vec![
        root(),
        node(Placement::Layout, 1, 0, transform(values)),
    ]);

    expect_transform(
        validate(&fixture, VIEWPORT, limits(1, 2, 2)),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(
            SpatialTransformScalarFieldV2::TransformOriginY,
        ),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::TransformOriginY,
        },
    );
}

#[test]
fn transform_validation_is_complete_record_major() {
    let mut first_origin = identity_values();
    set_field(
        &mut first_origin,
        SpatialTransformScalarFieldV2::TransformOriginY,
        SpatialScalarV2::MAX_RAW + 1,
    );
    let mut second_a = identity_values();
    set_field(
        &mut second_a,
        SpatialTransformScalarFieldV2::AffineA,
        SpatialScalarV2::MIN_RAW - 1,
    );
    let scalar_fixture = input(vec![
        root(),
        node(Placement::Layout, 1, 0, transform(first_origin)),
        node(Placement::Free, 2, 0, transform(second_a)),
    ]);
    expect_transform(
        validate(&scalar_fixture, VIEWPORT, limits(1, 2, 2)),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(
            SpatialTransformScalarFieldV2::TransformOriginY,
        ),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::TransformOriginY,
        },
    );

    let singular_fixture = input(vec![
        root(),
        node(Placement::Layout, 1, 0, transform([1, 1, 1, 1, 0, 0, 0, 0])),
        node(Placement::Free, 2, 0, transform(second_a)),
    ]);
    expect_transform(
        validate(&singular_fixture, VIEWPORT, limits(1, 2, 2)),
        SpatialTransformErrorKindV2::SingularTransform,
        SpatialErrorLocationV2::Node { index: 1 },
    );
}

#[test]
fn layout_preflight_failure_precedes_local_transform_validation() {
    let root_container = container(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), -1);
    let mut invalid = identity_values();
    set_field(
        &mut invalid,
        SpatialTransformScalarFieldV2::AffineA,
        SpatialScalarV2::MAX_RAW + 1,
    );
    let fixture = input(vec![
        root_with_container(root_container),
        node(Placement::Layout, 1, 0, transform(invalid)),
    ]);

    expect_input(
        prepare_local_transforms!(&fixture, VIEWPORT, limits(1, 2, 2)).map(|_| ()),
        SpatialInputErrorKindV2::InvalidContainer(SpatialContainerErrorKindV2::NegativeGap),
        SpatialErrorLocationV2::NodeField {
            index: 0,
            field: SpatialNodeFieldV2::Gap,
        },
    );
}

#[test]
fn root_has_structural_identity_and_no_authored_transform() {
    let fixture = input(vec![root()]);
    super::local_transform_support::expect_valid(validate(&fixture, VIEWPORT, limits(0, 0, 0)));
}

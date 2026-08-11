use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutDimensionV1, LayoutExtentV1, LayoutPaddingSideV1,
};

use super::layout_preflight_support::{
    VIEWPORT, container, dimension, expect_input, fixed, free, input, invalid_container, layout,
    limits, negative_constraint, padding, preflight, root, valid_container,
};
use crate::error::{
    SpatialContainerErrorKindV2, SpatialErrorLocationV2, SpatialInputErrorKindV2,
    SpatialLayoutDimensionErrorKindV2,
};
use crate::topology::SpatialContainerV2;
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn every_negative_constraint_maps_to_its_authored_member_field() {
    let valid = fixed(10);
    let cases = [
        (
            dimension(-1, 0, 0),
            valid,
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Minimum,
            SpatialNodeFieldV2::LayoutWidthMinimum,
        ),
        (
            dimension(0, -1, 0),
            valid,
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Preferred,
            SpatialNodeFieldV2::LayoutWidthPreferred,
        ),
        (
            dimension(0, 0, -1),
            valid,
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Maximum,
            SpatialNodeFieldV2::LayoutWidthMaximum,
        ),
        (
            valid,
            dimension(-1, 0, 0),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Minimum,
            SpatialNodeFieldV2::LayoutHeightMinimum,
        ),
        (
            valid,
            dimension(0, -1, 0),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Preferred,
            SpatialNodeFieldV2::LayoutHeightPreferred,
        ),
        (
            valid,
            dimension(0, 0, -1),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Maximum,
            SpatialNodeFieldV2::LayoutHeightMaximum,
        ),
    ];

    for (width, height, extent, field, node_field) in cases {
        expect_input(
            mapped_member_result(width, height, valid_container()),
            negative_constraint(extent, field),
            SpatialErrorLocationV2::NodeField {
                index: 6,
                field: node_field,
            },
        );
    }
}

#[test]
fn both_inverted_dimensions_map_to_the_owning_member() {
    let valid = fixed(10);
    for (width, height, extent) in [
        (dimension(2, 1, 1), valid, LayoutExtentV1::Width),
        (valid, dimension(2, 1, 1), LayoutExtentV1::Height),
    ] {
        expect_input(
            mapped_member_result(width, height, valid_container()),
            SpatialInputErrorKindV2::InvalidLayoutDimensions(
                SpatialLayoutDimensionErrorKindV2::InvertedConstraint(extent),
            ),
            SpatialErrorLocationV2::Node { index: 6 },
        );
    }
}

#[test]
fn every_negative_padding_side_maps_to_its_authored_member_field() {
    let cases = [
        (
            padding(-1, 0, 0, 0),
            LayoutPaddingSideV1::Left,
            SpatialNodeFieldV2::PaddingLeft,
        ),
        (
            padding(0, -1, 0, 0),
            LayoutPaddingSideV1::Right,
            SpatialNodeFieldV2::PaddingRight,
        ),
        (
            padding(0, 0, -1, 0),
            LayoutPaddingSideV1::Top,
            SpatialNodeFieldV2::PaddingTop,
        ),
        (
            padding(0, 0, 0, -1),
            LayoutPaddingSideV1::Bottom,
            SpatialNodeFieldV2::PaddingBottom,
        ),
    ];

    for (node_padding, side, field) in cases {
        expect_input(
            mapped_member_result(fixed(10), fixed(10), container(node_padding, 0)),
            invalid_container(SpatialContainerErrorKindV2::NegativePadding(side)),
            SpatialErrorLocationV2::NodeField { index: 6, field },
        );
    }
}

#[test]
fn padding_fit_and_gap_map_to_the_exact_member_and_location_shape() {
    let cases = [
        (
            container(padding(6, 5, 0, 0), 0),
            SpatialContainerErrorKindV2::PaddingExceedsExtent(LayoutExtentV1::Width),
            SpatialErrorLocationV2::Node { index: 6 },
        ),
        (
            container(padding(0, 0, 6, 5), 0),
            SpatialContainerErrorKindV2::PaddingExceedsExtent(LayoutExtentV1::Height),
            SpatialErrorLocationV2::Node { index: 6 },
        ),
        (
            container(padding(0, 0, 0, 0), -1),
            SpatialContainerErrorKindV2::NegativeGap,
            SpatialErrorLocationV2::NodeField {
                index: 6,
                field: SpatialNodeFieldV2::Gap,
            },
        ),
    ];

    for (node_container, kind, location) in cases {
        expect_input(
            mapped_member_result(fixed(10), fixed(10), node_container),
            invalid_container(kind),
            location,
        );
    }
}

#[test]
fn singleton_and_island_host_records_map_to_their_spatial_owners() {
    let root_singleton = input(vec![root(container(padding(0, 0, 0, 0), -1))]);
    expect_input(
        preflight(&root_singleton, VIEWPORT, limits(0, 0, 0)),
        invalid_container(SpatialContainerErrorKindV2::NegativeGap),
        SpatialErrorLocationV2::NodeField {
            index: 0,
            field: SpatialNodeFieldV2::Gap,
        },
    );

    let free_singleton = input(vec![
        root(valid_container()),
        free(1, 0, 10, 10, container(padding(0, -1, 0, 0), 0)),
    ]);
    expect_input(
        preflight(&free_singleton, VIEWPORT, limits(0, 0, 0)),
        invalid_container(SpatialContainerErrorKindV2::NegativePadding(
            LayoutPaddingSideV1::Right,
        )),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::PaddingRight,
        },
    );

    let island_host = input(vec![
        root(valid_container()),
        free(1, 0, 10, 10, container(padding(0, 0, -1, 0), 0)),
        layout(2, 1, fixed(10), fixed(10), valid_container()),
    ]);
    expect_input(
        preflight(&island_host, VIEWPORT, limits(1, 2, 2)),
        invalid_container(SpatialContainerErrorKindV2::NegativePadding(
            LayoutPaddingSideV1::Top,
        )),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::PaddingTop,
        },
    );
}

#[test]
fn synthetic_host_extents_come_from_each_root_or_free_owner() {
    let width_fit = invalid_container(SpatialContainerErrorKindV2::PaddingExceedsExtent(
        LayoutExtentV1::Width,
    ));
    let height_fit = invalid_container(SpatialContainerErrorKindV2::PaddingExceedsExtent(
        LayoutExtentV1::Height,
    ));
    let too_wide = container(padding(6, 5, 0, 0), 0);
    let too_tall = container(padding(0, 0, 6, 5), 0);

    let root_singleton = input(vec![root(too_wide)]);
    expect_input(
        preflight(
            &root_singleton,
            crate::model::SpatialViewportV2::new(10, 20),
            limits(0, 0, 0),
        ),
        width_fit,
        SpatialErrorLocationV2::Node { index: 0 },
    );

    let root_island = input(vec![
        root(too_tall),
        layout(1, 0, fixed(10), fixed(10), valid_container()),
    ]);
    expect_input(
        preflight(
            &root_island,
            crate::model::SpatialViewportV2::new(20, 10),
            limits(1, 2, 2),
        ),
        height_fit,
        SpatialErrorLocationV2::Node { index: 0 },
    );

    let free_singleton = input(vec![root(valid_container()), free(1, 0, 10, 20, too_wide)]);
    expect_input(
        preflight(&free_singleton, VIEWPORT, limits(0, 0, 0)),
        width_fit,
        SpatialErrorLocationV2::Node { index: 1 },
    );

    let free_island = input(vec![
        root(valid_container()),
        free(1, 0, 20, 10, too_tall),
        layout(2, 1, fixed(10), fixed(10), valid_container()),
    ]);
    expect_input(
        preflight(&free_island, VIEWPORT, limits(1, 2, 2)),
        height_fit,
        SpatialErrorLocationV2::Node { index: 1 },
    );
}

fn mapped_member_result(
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    target_container: SpatialContainerV2,
) -> Result<(), crate::resolve_error::SpatialResolveErrorV2> {
    let valid = valid_container();
    let fixture = input(vec![
        root(valid),
        free(1, 0, 10, 10, valid),
        free(2, 0, 10, 10, valid),
        layout(3, 2, fixed(10), fixed(10), valid),
        free(4, 3, 10, 10, valid),
        layout(5, 4, fixed(10), fixed(10), valid),
        layout(6, 3, width, height, target_container),
    ]);
    preflight(&fixture, VIEWPORT, limits(2, 3, 5))
}

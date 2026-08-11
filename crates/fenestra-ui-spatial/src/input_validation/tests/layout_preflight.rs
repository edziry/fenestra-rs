use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutConstraintFieldV1, LayoutExtentV1, LayoutPaddingSideV1,
};

use super::island_support::expect_limit;
use super::layout_preflight_support::{
    VIEWPORT, container, container_on, dimension, expect_input, expect_valid, fixed, free, input,
    invalid_container, layout, limits, negative_constraint, padding, preflight, root,
    valid_container,
};
use crate::error::{SpatialContainerErrorKindV2, SpatialErrorLocationV2};
use crate::limits::SpatialLimitKindV2;
use crate::model::SpatialViewportV2;
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn successful_preflight_retains_only_island_record_remaps() {
    let valid = valid_container();
    let fixture = input(vec![
        root(valid),
        free(1, 0, 10, 10, valid),
        free(2, 0, 10, 10, valid),
        layout(3, 2, fixed(10), fixed(10), valid),
        free(4, 3, 10, 10, valid),
        layout(5, 4, fixed(10), fixed(10), valid),
        layout(6, 3, fixed(10), fixed(10), valid),
    ]);

    let proof = super::island_support::expect_plan(prepare_layout_preflight!(
        &fixture,
        VIEWPORT,
        limits(2, 3, 5)
    ));
    assert_eq!(
        proof.prepared_island_facts(),
        vec![(0, vec![2, 3, 6]), (1, vec![4, 5])]
    );
}

#[test]
fn layout_internal_priority_restarts_for_each_complete_item() {
    let member_constraint = input(vec![
        root(container(padding(-1, 0, 0, 0), 0)),
        layout(1, 0, dimension(-1, 0, 0), fixed(10), valid_container()),
    ]);
    expect_input(
        preflight(&member_constraint, VIEWPORT, limits(1, 2, 2)),
        negative_constraint(LayoutExtentV1::Width, LayoutConstraintFieldV1::Minimum),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::LayoutWidthMinimum,
        },
    );

    let member_padding = input(vec![
        root(container(padding(11, 10, 0, 0), 0)),
        layout(
            1,
            0,
            fixed(10),
            fixed(10),
            container(padding(0, -1, 0, 0), 0),
        ),
    ]);
    expect_input(
        preflight(&member_padding, VIEWPORT, limits(1, 2, 2)),
        invalid_container(SpatialContainerErrorKindV2::NegativePadding(
            LayoutPaddingSideV1::Right,
        )),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::PaddingRight,
        },
    );

    let member_padding_fit = input(vec![
        root(container(padding(0, 0, 0, 0), -1)),
        layout(
            1,
            0,
            fixed(10),
            fixed(10),
            container(padding(6, 5, 0, 0), 0),
        ),
    ]);
    expect_input(
        preflight(&member_padding_fit, VIEWPORT, limits(1, 2, 2)),
        invalid_container(SpatialContainerErrorKindV2::PaddingExceedsExtent(
            LayoutExtentV1::Width,
        )),
        SpatialErrorLocationV2::Node { index: 1 },
    );
}

#[test]
fn constraints_keep_layout_record_order_within_one_island() {
    let fixture = input(vec![
        root(valid_container()),
        layout(1, 0, fixed(10), dimension(0, -1, 0), valid_container()),
        layout(2, 0, dimension(-1, 0, 0), fixed(10), valid_container()),
    ]);

    expect_input(
        preflight(&fixture, VIEWPORT, limits(1, 3, 3)),
        negative_constraint(LayoutExtentV1::Height, LayoutConstraintFieldV1::Preferred),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::LayoutHeightPreferred,
        },
    );
}

#[test]
fn earlier_singleton_finishes_before_a_later_island() {
    let fixture = input(vec![
        root(valid_container()),
        free(1, 0, 10, 10, container(padding(0, 0, 0, 0), -1)),
        layout(2, 0, dimension(-1, 0, 0), fixed(10), valid_container()),
    ]);

    expect_input(
        preflight(&fixture, VIEWPORT, limits(1, 2, 2)),
        invalid_container(SpatialContainerErrorKindV2::NegativeGap),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::Gap,
        },
    );
}

#[test]
fn earlier_island_finishes_before_a_later_singleton() {
    let fixture = input(vec![
        root(valid_container()),
        layout(
            1,
            0,
            fixed(10),
            fixed(10),
            container(padding(0, 0, 0, 0), -1),
        ),
        free(2, 0, 10, 10, container(padding(-1, 0, 0, 0), 0)),
    ]);

    expect_input(
        preflight(&fixture, VIEWPORT, limits(1, 2, 2)),
        invalid_container(SpatialContainerErrorKindV2::NegativeGap),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::Gap,
        },
    );
}

#[test]
fn stable_island_item_order_precedes_member_record_order() {
    let valid = valid_container();
    let fixture = input(vec![
        root(valid),
        free(1, 0, 10, 10, valid),
        free(2, 0, 10, 10, valid),
        layout(3, 2, fixed(10), fixed(10), valid),
        free(4, 3, 10, 10, valid),
        layout(5, 4, dimension(-1, 0, 0), fixed(10), valid),
        layout(
            6,
            3,
            fixed(10),
            fixed(10),
            container(padding(0, 0, 0, 0), -1),
        ),
    ]);

    expect_input(
        preflight(&fixture, VIEWPORT, limits(2, 3, 5)),
        invalid_container(SpatialContainerErrorKindV2::NegativeGap),
        SpatialErrorLocationV2::NodeField {
            index: 6,
            field: SpatialNodeFieldV2::Gap,
        },
    );
}

#[test]
fn island_limits_precede_layout_preflight() {
    let fixture = input(vec![
        root(valid_container()),
        layout(1, 0, dimension(-1, 0, 0), fixed(10), valid_container()),
    ]);

    expect_limit(
        prepare_layout_preflight!(&fixture, VIEWPORT, limits(0, 0, 0)),
        SpatialLimitKindV2::Islands,
        SpatialErrorLocationV2::Input,
        1,
        0,
    );
}

#[test]
fn each_layout_limit_uses_the_complete_item_record_count() {
    let mut nodes = vec![root(valid_container())];
    for key in 1..=17 {
        nodes.push(layout(key, key - 1, fixed(0), fixed(0), valid_container()));
    }
    for key in 18..=33 {
        nodes.push(layout(key, 0, fixed(0), fixed(0), valid_container()));
    }
    let fixture = input(nodes);

    expect_valid(preflight(
        &fixture,
        SpatialViewportV2::new(0, 0),
        limits(1, 34, 34),
    ));
}

#[test]
fn preparation_stops_before_layout_engine_arithmetic() {
    let fixture = input(vec![
        root(container_on(LayoutAxisV1::Row, padding(1, 0, 0, 0), 0)),
        layout(1, 0, fixed(i32::MAX), fixed(0), valid_container()),
    ]);

    expect_valid(preflight(
        &fixture,
        SpatialViewportV2::new(i32::MAX, 0),
        limits(1, 2, 2),
    ));
}

#[test]
fn valid_singletons_and_islands_accept_deferred_transforms_and_targets() {
    let valid = valid_container();
    let fixture = input(vec![
        root(valid),
        free(1, 0, 10, 10, valid),
        layout(2, 0, fixed(10), fixed(10), valid),
        free(3, 2, 10, 10, valid),
        layout(4, 3, fixed(10), fixed(10), valid),
    ]);

    expect_valid(preflight(&fixture, VIEWPORT, limits(2, 2, 4)));
}

mod support;

use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutErrorLocationV1, LayoutExtentV1, LayoutInputErrorKindV1,
    LayoutPaddingSideV1, LayoutViewportV1,
};

use support::{
    GENEROUS_LIMITS, VIEWPORT, assert_invalid, assert_valid, dimension, input_node, node_with,
    padding,
};

#[test]
fn viewport_zero_extents_are_valid_and_negatives_follow_axis_order() {
    let nodes = [valid_root()];
    for viewport in [
        LayoutViewportV1::new(0, 9),
        LayoutViewportV1::new(7, 0),
        LayoutViewportV1::new(0, 0),
    ] {
        assert_valid(&nodes, viewport, GENEROUS_LIMITS);
    }

    assert_invalid(
        &nodes,
        LayoutViewportV1::new(-1, 9),
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Width),
        LayoutErrorLocationV1::Viewport,
    );
    assert_invalid(
        &nodes,
        LayoutViewportV1::new(7, -1),
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Height),
        LayoutErrorLocationV1::Viewport,
    );
    assert_invalid(
        &nodes,
        LayoutViewportV1::new(-1, -1),
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Width),
        LayoutErrorLocationV1::Viewport,
    );
}

#[test]
fn every_negative_constraint_field_is_typed() {
    let cases = [
        (
            dimension(-1, 0, 10),
            valid_dimension(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Minimum,
        ),
        (
            dimension(0, -1, 10),
            valid_dimension(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Preferred,
        ),
        (
            dimension(0, 0, -1),
            valid_dimension(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Maximum,
        ),
        (
            valid_dimension(),
            dimension(-1, 0, 10),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Minimum,
        ),
        (
            valid_dimension(),
            dimension(0, -1, 10),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Preferred,
        ),
        (
            valid_dimension(),
            dimension(0, 0, -1),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Maximum,
        ),
    ];

    for (width, height, extent, field) in cases {
        assert_invalid(
            &[node_with(0, None, width, height, padding(0, 0, 0, 0), 0)],
            VIEWPORT,
            GENEROUS_LIMITS,
            LayoutInputErrorKindV1::NegativeConstraint { extent, field },
            input_node(0),
        );
    }
}

#[test]
fn inverted_constraints_are_typed_per_extent() {
    assert_invalid(
        &[root_with_dimensions(dimension(8, 8, 7), valid_dimension())],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::InvertedConstraint(LayoutExtentV1::Width),
        input_node(0),
    );
    assert_invalid(
        &[root_with_dimensions(valid_dimension(), dimension(8, 8, 7))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::InvertedConstraint(LayoutExtentV1::Height),
        input_node(0),
    );
}

#[test]
fn preferred_values_outside_valid_min_max_are_accepted() {
    let below = root_with_dimensions(dimension(10, 3, 20), dimension(7, 2, 15));
    let above = root_with_dimensions(dimension(10, 30, 20), dimension(7, 25, 15));

    assert_valid(&[below], VIEWPORT, GENEROUS_LIMITS);
    assert_valid(&[above], VIEWPORT, GENEROUS_LIMITS);
}

#[test]
fn every_negative_padding_side_is_typed() {
    let cases = [
        (padding(-1, 0, 0, 0), LayoutPaddingSideV1::Left),
        (padding(0, -1, 0, 0), LayoutPaddingSideV1::Right),
        (padding(0, 0, -1, 0), LayoutPaddingSideV1::Top),
        (padding(0, 0, 0, -1), LayoutPaddingSideV1::Bottom),
    ];

    for (padding, side) in cases {
        assert_invalid(
            &[node_with(
                0,
                None,
                valid_dimension(),
                valid_dimension(),
                padding,
                0,
            )],
            VIEWPORT,
            GENEROUS_LIMITS,
            LayoutInputErrorKindV1::NegativePadding(side),
            input_node(0),
        );
    }
}

#[test]
fn padding_fit_uses_resolved_extents_and_widened_sums() {
    assert_invalid(
        &[root_with_padding(padding(11, 10, 0, 0))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Width),
        input_node(0),
    );
    assert_invalid(
        &[root_with_padding(padding(0, 0, 6, 5))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Height),
        input_node(0),
    );
    assert_invalid(
        &[root_with_padding(padding(i32::MAX, i32::MAX, 0, 0))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Width),
        input_node(0),
    );

    assert_valid(
        &[root_with_padding(padding(7, 13, 4, 6))],
        VIEWPORT,
        GENEROUS_LIMITS,
    );
}

#[test]
fn padding_fit_uses_the_clamped_preference_not_a_raw_constraint_field() {
    assert_valid(
        &[root_with_width_and_padding(
            dimension(10, 3, 20),
            padding(10, 0, 0, 0),
        )],
        VIEWPORT,
        GENEROUS_LIMITS,
    );
    assert_invalid(
        &[root_with_width_and_padding(
            dimension(10, 30, 20),
            padding(21, 0, 0, 0),
        )],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Width),
        input_node(0),
    );
}

#[test]
fn widened_padding_equality_at_i32_max_is_valid() {
    assert_valid(
        &[root_with_width_and_padding(
            dimension(0, i32::MAX, i32::MAX),
            padding(i32::MAX, 0, 0, 0),
        )],
        VIEWPORT,
        GENEROUS_LIMITS,
    );
}

#[test]
fn negative_gap_is_rejected_and_full_nonnegative_domain_is_admitted() {
    assert_invalid(
        &[node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(0, 0, 0, 0),
            -1,
        )],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeGap,
        input_node(0),
    );

    let maximum = node_with(
        0,
        None,
        dimension(0, i32::MAX, i32::MAX),
        dimension(0, i32::MAX, i32::MAX),
        padding(0, 0, 0, 0),
        i32::MAX,
    );
    assert_valid(&[maximum], VIEWPORT, GENEROUS_LIMITS);
}

const fn valid_dimension() -> fenestra_ui_layout::prototype::LayoutDimensionV1 {
    dimension(0, 10, 20)
}

const fn valid_root() -> fenestra_ui_layout::prototype::LayoutNodeV1 {
    root_with_dimensions(valid_dimension(), valid_dimension())
}

const fn root_with_dimensions(
    width: fenestra_ui_layout::prototype::LayoutDimensionV1,
    height: fenestra_ui_layout::prototype::LayoutDimensionV1,
) -> fenestra_ui_layout::prototype::LayoutNodeV1 {
    node_with(0, None, width, height, padding(0, 0, 0, 0), 0)
}

const fn root_with_padding(
    padding: fenestra_ui_layout::prototype::LayoutPaddingV1,
) -> fenestra_ui_layout::prototype::LayoutNodeV1 {
    node_with(
        0,
        None,
        dimension(0, 20, 20),
        dimension(0, 10, 10),
        padding,
        0,
    )
}

const fn root_with_width_and_padding(
    width: fenestra_ui_layout::prototype::LayoutDimensionV1,
    padding: fenestra_ui_layout::prototype::LayoutPaddingV1,
) -> fenestra_ui_layout::prototype::LayoutNodeV1 {
    node_with(0, None, width, dimension(0, 10, 10), padding, 0)
}

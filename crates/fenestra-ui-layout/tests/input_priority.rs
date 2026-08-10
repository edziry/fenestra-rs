mod support;

use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutErrorLocationV1, LayoutExtentV1, LayoutInputErrorKindV1,
    LayoutLimitKindV1, LayoutLimitsV1, LayoutPaddingSideV1, LayoutViewportV1,
};

use support::{
    GENEROUS_LIMITS, VIEWPORT, assert_invalid, dimension, input_node, node, node_with, padding,
    root,
};

#[test]
fn failure_vocabulary_places_width_inversion_before_height_fields() {
    use LayoutConstraintFieldV1::{Maximum, Minimum, Preferred};
    use LayoutExtentV1::{Height, Width};
    use LayoutInputErrorKindV1::{InvertedConstraint, NegativeConstraint};

    assert_eq!(
        &LayoutInputErrorKindV1::ALL[12..20],
        &[
            NegativeConstraint {
                extent: Width,
                field: Minimum,
            },
            NegativeConstraint {
                extent: Width,
                field: Preferred,
            },
            NegativeConstraint {
                extent: Width,
                field: Maximum,
            },
            InvertedConstraint(Width),
            NegativeConstraint {
                extent: Height,
                field: Minimum,
            },
            NegativeConstraint {
                extent: Height,
                field: Preferred,
            },
            NegativeConstraint {
                extent: Height,
                field: Maximum,
            },
            InvertedConstraint(Height),
        ]
    );
}

#[test]
fn node_count_precedes_root_shape_and_fields() {
    let invalid = node_with(
        9,
        Some(9),
        dimension(-1, -1, -1),
        dimension(-1, -1, -1),
        padding(-1, -1, -1, -1),
        -1,
    );
    assert_invalid(
        &[invalid],
        LayoutViewportV1::new(-1, -1),
        LayoutLimitsV1::new(0, 0, 0),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Nodes),
        LayoutErrorLocationV1::Input,
    );
}

#[test]
fn topology_phases_precede_depth_even_for_later_nodes() {
    let invalid_preorder = [root(), node(1, Some(0)), node(2, Some(0)), node(3, Some(1))];
    assert_invalid(
        &invalid_preorder,
        VIEWPORT,
        LayoutLimitsV1::new(4, 1, 1),
        LayoutInputErrorKindV1::InvalidPreorder,
        input_node(3),
    );

    let missing_before_forward = [root(), node(1, None), node(2, Some(2))];
    assert_invalid(
        &missing_before_forward,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::MissingParent,
        input_node(1),
    );
}

#[test]
fn depth_precedes_children_and_children_precede_viewport() {
    let depth_and_children = [root(), node(1, Some(0)), node(2, Some(1)), node(3, Some(0))];
    assert_invalid(
        &depth_and_children,
        VIEWPORT,
        LayoutLimitsV1::new(4, 2, 1),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
        input_node(2),
    );

    let too_many_children = [root(), node(1, Some(0)), node(2, Some(0))];
    assert_invalid(
        &too_many_children,
        LayoutViewportV1::new(-1, -1),
        LayoutLimitsV1::new(3, 3, 1),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
        input_node(0),
    );
}

#[test]
fn viewport_width_precedes_height_and_all_constraints() {
    let bad_fields = [node_with(
        0,
        None,
        dimension(-1, -1, -1),
        dimension(-1, -1, -1),
        padding(0, 0, 0, 0),
        0,
    )];
    assert_invalid(
        &bad_fields,
        LayoutViewportV1::new(-1, -1),
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Width),
        LayoutErrorLocationV1::Viewport,
    );
    assert_invalid(
        &bad_fields,
        LayoutViewportV1::new(1, -1),
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Height),
        LayoutErrorLocationV1::Viewport,
    );
}

#[test]
fn constraint_priority_is_node_then_width_then_field() {
    let width_inversion_and_height_negative = [node_with(
        0,
        None,
        dimension(8, 8, 7),
        dimension(-1, -1, -1),
        padding(0, 0, 0, 0),
        0,
    )];
    assert_invalid(
        &width_inversion_and_height_negative,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::InvertedConstraint(LayoutExtentV1::Width),
        input_node(0),
    );

    let root_height_and_child_width = [
        node_with(
            0,
            None,
            valid_dimension(),
            dimension(0, -1, 10),
            padding(0, 0, 0, 0),
            0,
        ),
        node_with(
            1,
            Some(0),
            dimension(-1, 0, 10),
            valid_dimension(),
            padding(0, 0, 0, 0),
            0,
        ),
    ];
    assert_invalid(
        &root_height_and_child_width,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeConstraint {
            extent: LayoutExtentV1::Height,
            field: LayoutConstraintFieldV1::Preferred,
        },
        input_node(0),
    );

    for (width, expected) in [
        (dimension(-3, -2, -1), LayoutConstraintFieldV1::Minimum),
        (dimension(0, -2, -1), LayoutConstraintFieldV1::Preferred),
        (dimension(1, 2, -1), LayoutConstraintFieldV1::Maximum),
    ] {
        assert_invalid(
            &[node_with(
                0,
                None,
                width,
                valid_dimension(),
                padding(0, 0, 0, 0),
                0,
            )],
            VIEWPORT,
            GENEROUS_LIMITS,
            LayoutInputErrorKindV1::NegativeConstraint {
                extent: LayoutExtentV1::Width,
                field: expected,
            },
            input_node(0),
        );
    }
}

#[test]
fn all_constraints_precede_all_padding_checks() {
    let nodes = [
        node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(-1, 0, 0, 0),
            0,
        ),
        node_with(
            1,
            Some(0),
            dimension(0, 10, 20),
            dimension(0, -1, 20),
            padding(0, 0, 0, 0),
            0,
        ),
    ];
    assert_invalid(
        &nodes,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeConstraint {
            extent: LayoutExtentV1::Height,
            field: LayoutConstraintFieldV1::Preferred,
        },
        input_node(1),
    );
}

#[test]
fn padding_priority_is_negative_then_node_and_side_then_fit() {
    let negative_child_after_bad_root_fit = [
        node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(11, 10, 0, 0),
            0,
        ),
        node_with(
            1,
            Some(0),
            valid_dimension(),
            valid_dimension(),
            padding(0, 0, 0, -1),
            0,
        ),
    ];
    assert_invalid(
        &negative_child_after_bad_root_fit,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativePadding(LayoutPaddingSideV1::Bottom),
        input_node(1),
    );

    let root_right_before_child_left = [
        node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(0, -1, 0, 0),
            0,
        ),
        node_with(
            1,
            Some(0),
            valid_dimension(),
            valid_dimension(),
            padding(-1, 0, 0, 0),
            0,
        ),
    ];
    assert_invalid(
        &root_right_before_child_left,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativePadding(LayoutPaddingSideV1::Right),
        input_node(0),
    );

    assert_invalid(
        &[node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(-1, -1, -1, -1),
            0,
        )],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativePadding(LayoutPaddingSideV1::Left),
        input_node(0),
    );
}

#[test]
fn padding_fit_is_node_then_axis_and_precedes_every_gap() {
    let root_vertical_before_child_horizontal = [
        node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(0, 0, 11, 10),
            -1,
        ),
        node_with(
            1,
            Some(0),
            valid_dimension(),
            valid_dimension(),
            padding(11, 10, 0, 0),
            -1,
        ),
    ];
    assert_invalid(
        &root_vertical_before_child_horizontal,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Height),
        input_node(0),
    );

    let both_axes = [node_with(
        0,
        None,
        valid_dimension(),
        valid_dimension(),
        padding(11, 10, 11, 10),
        -1,
    )];
    assert_invalid(
        &both_axes,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Width),
        input_node(0),
    );
}

#[test]
fn negative_gaps_follow_node_order() {
    let nodes = [
        node_with(
            0,
            None,
            valid_dimension(),
            valid_dimension(),
            padding(0, 0, 0, 0),
            -1,
        ),
        node_with(
            1,
            Some(0),
            valid_dimension(),
            valid_dimension(),
            padding(0, 0, 0, 0),
            -2,
        ),
    ];
    assert_invalid(
        &nodes,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NegativeGap,
        input_node(0),
    );
}

const fn valid_dimension() -> fenestra_ui_layout::prototype::LayoutDimensionV1 {
    dimension(0, 10, 20)
}

use fenestra_ui_layout::prototype::{
    LayoutArithmeticOperationV1::{FarEdge, GapAdvance},
    LayoutAxisV1::{Column, Row},
    LayoutExtentV1::{Height, Width},
};

use super::support::{
    LIMITS, assert_reference_arithmetic_error, assert_reference_case, fixed, fixed_node,
    input_node, node, padding, record, viewport,
};

#[test]
fn child_far_edge_exactly_at_i32_max_is_valid() {
    let nodes = [
        fixed_node(0, None, Row, i32::MAX, i32::MAX),
        fixed_node(1, Some(0), Column, i32::MAX, i32::MAX),
    ];
    let expected = [
        record(0, 0, 0, i32::MAX, i32::MAX),
        record(1, 0, 0, i32::MAX, i32::MAX),
    ];

    assert_reference_case(
        "exact-child-far-edge",
        viewport(0, 0),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn child_far_edge_one_over_reports_width_and_child() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(i32::MAX),
            fixed(i32::MAX),
            padding(i32::MAX, 0, i32::MAX, 0),
            0,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
    ];

    assert_reference_arithmetic_error(
        "child-far-edge-one-over",
        viewport(0, 0),
        &nodes,
        FarEdge,
        Width,
        input_node(1),
    );
}

#[test]
fn child_far_edge_one_over_reports_height_and_child() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(1),
            fixed(i32::MAX),
            padding(0, 0, i32::MAX, 0),
            0,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
    ];

    assert_reference_arithmetic_error(
        "child-far-edge-height-one-over",
        viewport(0, 0),
        &nodes,
        FarEdge,
        Height,
        input_node(1),
    );
}

#[test]
fn two_zero_width_children_accept_exact_maximum_gap_advance() {
    let nodes = [
        node(
            0,
            None,
            Row,
            fixed(i32::MAX),
            fixed(0),
            padding(0, 0, 0, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, i32::MAX, 0),
        record(1, 0, 0, 0, 0),
        record(2, i32::MAX, 0, 0, 0),
    ];

    assert_reference_case(
        "exact-gap-advance",
        viewport(0, 0),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn two_zero_height_children_accept_exact_maximum_gap_advance() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(0),
            fixed(i32::MAX),
            padding(0, 0, 0, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, 0, i32::MAX),
        record(1, 0, 0, 0, 0),
        record(2, 0, i32::MAX, 0, 0),
    ];

    assert_reference_case(
        "exact-column-gap-advance",
        viewport(0, 0),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn non_final_gap_advance_one_over_reports_owning_parent() {
    let nodes = [
        node(
            0,
            None,
            Row,
            fixed(i32::MAX),
            fixed(1),
            padding(0, 0, 0, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
        fixed_node(2, Some(0), Column, 0, 0),
    ];

    assert_reference_arithmetic_error(
        "gap-advance-one-over",
        viewport(0, 0),
        &nodes,
        GapAdvance,
        Width,
        input_node(0),
    );
}

#[test]
fn column_gap_advance_one_over_reports_height_and_parent() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(1),
            fixed(i32::MAX),
            padding(0, 0, 0, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
        fixed_node(2, Some(0), Column, 0, 0),
    ];

    assert_reference_arithmetic_error(
        "gap-advance-height-one-over",
        viewport(0, 0),
        &nodes,
        GapAdvance,
        Height,
        input_node(0),
    );
}

#[test]
fn child_far_height_precedes_the_following_row_gap_width() {
    let nodes = [
        node(
            0,
            None,
            Row,
            fixed(i32::MAX),
            fixed(i32::MAX),
            padding(0, 0, i32::MAX, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
        fixed_node(2, Some(0), Column, 0, 0),
    ];

    assert_reference_arithmetic_error(
        "far-height-before-gap-width",
        viewport(0, 0),
        &nodes,
        FarEdge,
        Height,
        input_node(1),
    );
}

#[test]
fn parent_major_traversal_beats_an_earlier_descendant_failure() {
    let nodes = [
        fixed_node(0, None, Row, i32::MAX, 1),
        node(
            1,
            Some(0),
            Row,
            fixed(i32::MAX),
            fixed(1),
            padding(1, 0, 0, 0),
            0,
        ),
        fixed_node(2, Some(1), Column, i32::MAX, 1),
        fixed_node(3, Some(0), Column, 1, 1),
    ];

    assert_reference_arithmetic_error(
        "parent-major-arithmetic",
        viewport(0, 0),
        &nodes,
        FarEdge,
        Width,
        input_node(3),
    );
}

#[test]
fn leaf_padding_does_not_move_the_following_sibling() {
    let nodes = [
        fixed_node(0, None, Row, 20, 10),
        node(
            1,
            Some(0),
            Column,
            fixed(10),
            fixed(5),
            padding(4, 6, 2, 3),
            0,
        ),
        fixed_node(2, Some(0), Column, 3, 4),
    ];
    let expected = [
        record(0, 0, 0, 20, 10),
        record(1, 0, 0, 10, 5),
        record(2, 10, 0, 3, 4),
    ];

    assert_reference_case("leaf-padding", viewport(0, 0), &nodes, LIMITS, &expected);
}

#[test]
fn maximum_gap_is_not_applied_after_an_only_child() {
    let nodes = [
        node(
            0,
            None,
            Row,
            fixed(1),
            fixed(1),
            padding(0, 0, 0, 0),
            i32::MAX,
        ),
        fixed_node(1, Some(0), Column, 1, 1),
    ];
    let expected = [record(0, 0, 0, 1, 1), record(1, 0, 0, 1, 1)];

    assert_reference_case(
        "single-child-no-gap",
        viewport(0, 0),
        &nodes,
        LIMITS,
        &expected,
    );
}

use fenestra_ui_layout::prototype::LayoutAxisV1::{Column, Row};

use super::support::{
    LIMITS, assert_reference_case, dimension, fixed, fixed_node, node, padding, record, viewport,
};

#[test]
fn single_fixed_root_ignores_available_space() {
    let nodes = [fixed_node(0, None, Column, 31, 19)];
    let expected = [record(0, 0, 0, 31, 19)];

    assert_reference_case(
        "single-fixed-root",
        viewport(7, 5),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn column_places_two_children_in_authored_order() {
    let nodes = [
        fixed_node(0, None, Column, 80, 60),
        fixed_node(1, Some(0), Column, 20, 11),
        fixed_node(2, Some(0), Column, 30, 13),
    ];
    let expected = [
        record(0, 0, 0, 80, 60),
        record(1, 0, 0, 20, 11),
        record(2, 0, 11, 30, 13),
    ];

    assert_reference_case("column-two", viewport(100, 100), &nodes, LIMITS, &expected);
}

#[test]
fn row_places_two_children_in_authored_order() {
    let nodes = [
        fixed_node(0, None, Row, 70, 50),
        fixed_node(1, Some(0), Column, 17, 21),
        fixed_node(2, Some(0), Column, 13, 9),
    ];
    let expected = [
        record(0, 0, 0, 70, 50),
        record(1, 0, 0, 17, 21),
        record(2, 17, 0, 13, 9),
    ];

    assert_reference_case("row-two", viewport(100, 100), &nodes, LIMITS, &expected);
}

#[test]
fn nested_row_inside_column_uses_absolute_origins() {
    let nodes = [
        fixed_node(0, None, Column, 80, 70),
        fixed_node(1, Some(0), Column, 15, 7),
        fixed_node(2, Some(0), Row, 50, 30),
        fixed_node(3, Some(2), Column, 11, 5),
        fixed_node(4, Some(2), Column, 13, 9),
        fixed_node(5, Some(0), Column, 8, 6),
    ];
    let expected = [
        record(0, 0, 0, 80, 70),
        record(1, 0, 0, 15, 7),
        record(2, 0, 7, 50, 30),
        record(3, 0, 7, 11, 5),
        record(4, 11, 7, 13, 9),
        record(5, 0, 37, 8, 6),
    ];

    assert_reference_case(
        "nested-row-in-column",
        viewport(100, 100),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn asymmetric_padding_moves_children_only_by_left_and_top() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(100),
            fixed(70),
            padding(7, 11, 5, 13),
            0,
        ),
        fixed_node(1, Some(0), Column, 20, 10),
        fixed_node(2, Some(0), Column, 30, 15),
    ];
    let expected = [
        record(0, 0, 0, 100, 70),
        record(1, 7, 5, 20, 10),
        record(2, 7, 15, 30, 15),
    ];

    assert_reference_case(
        "asymmetric-padding",
        viewport(100, 100),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn column_gap_is_inserted_only_between_three_children() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(80),
            fixed(60),
            padding(0, 0, 0, 0),
            4,
        ),
        fixed_node(1, Some(0), Column, 12, 9),
        fixed_node(2, Some(0), Column, 13, 5),
        fixed_node(3, Some(0), Column, 14, 7),
    ];
    let expected = [
        record(0, 0, 0, 80, 60),
        record(1, 0, 0, 12, 9),
        record(2, 0, 13, 13, 5),
        record(3, 0, 22, 14, 7),
    ];

    assert_reference_case(
        "column-gap-three",
        viewport(100, 100),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn row_gap_is_inserted_only_between_three_children() {
    let nodes = [
        node(0, None, Row, fixed(80), fixed(30), padding(0, 0, 0, 0), 4),
        fixed_node(1, Some(0), Column, 9, 12),
        fixed_node(2, Some(0), Column, 5, 13),
        fixed_node(3, Some(0), Column, 7, 14),
    ];
    let expected = [
        record(0, 0, 0, 80, 30),
        record(1, 0, 0, 9, 12),
        record(2, 13, 0, 5, 13),
        record(3, 22, 0, 7, 14),
    ];

    assert_reference_case(
        "row-gap-three",
        viewport(100, 100),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn preferred_extents_below_minimum_are_clamped() {
    let nodes = [
        fixed_node(0, None, Row, 80, 40),
        node(
            1,
            Some(0),
            Column,
            dimension(10, 3, 20),
            dimension(7, 2, 15),
            padding(0, 0, 0, 0),
            0,
        ),
        fixed_node(2, Some(0), Column, 4, 5),
    ];
    let expected = [
        record(0, 0, 0, 80, 40),
        record(1, 0, 0, 10, 7),
        record(2, 10, 0, 4, 5),
    ];

    assert_reference_case("clamp-below", viewport(100, 100), &nodes, LIMITS, &expected);
}

#[test]
fn preferred_extents_above_maximum_are_clamped() {
    let nodes = [
        fixed_node(0, None, Column, 50, 100),
        node(
            1,
            Some(0),
            Column,
            dimension(10, 30, 20),
            dimension(7, 25, 15),
            padding(0, 0, 0, 0),
            0,
        ),
        fixed_node(2, Some(0), Column, 5, 4),
    ];
    let expected = [
        record(0, 0, 0, 50, 100),
        record(1, 0, 0, 20, 15),
        record(2, 0, 15, 5, 4),
    ];

    assert_reference_case("clamp-above", viewport(100, 100), &nodes, LIMITS, &expected);
}

#[test]
fn mixed_constraints_padding_and_gap_resolve_together() {
    let nodes = [
        node(
            0,
            None,
            Column,
            dimension(50, 60, 70),
            dimension(40, 50, 60),
            padding(6, 7, 4, 5),
            3,
        ),
        node(
            1,
            Some(0),
            Column,
            dimension(12, 8, 20),
            dimension(5, 14, 10),
            padding(0, 0, 0, 0),
            0,
        ),
        node(
            2,
            Some(0),
            Column,
            dimension(5, 18, 16),
            dimension(6, 3, 12),
            padding(0, 0, 0, 0),
            0,
        ),
    ];
    let expected = [
        record(0, 0, 0, 60, 50),
        record(1, 6, 4, 12, 10),
        record(2, 6, 17, 16, 6),
    ];

    assert_reference_case(
        "mixed-constraints-padding",
        viewport(100, 100),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn main_axis_overflow_is_retained_without_resizing() {
    let nodes = [
        fixed_node(0, None, Column, 20, 20),
        fixed_node(1, Some(0), Column, 8, 25),
        fixed_node(2, Some(0), Column, 5, 4),
    ];
    let expected = [
        record(0, 0, 0, 20, 20),
        record(1, 0, 0, 8, 25),
        record(2, 0, 25, 5, 4),
    ];

    assert_reference_case(
        "main-axis-overflow",
        viewport(20, 20),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn cross_axis_overflow_is_retained_beyond_content_area() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(20),
            fixed(30),
            padding(3, 4, 2, 1),
            0,
        ),
        fixed_node(1, Some(0), Column, 25, 6),
        fixed_node(2, Some(0), Column, 10, 5),
    ];
    let expected = [
        record(0, 0, 0, 20, 30),
        record(1, 3, 2, 25, 6),
        record(2, 3, 8, 10, 5),
    ];

    assert_reference_case(
        "cross-axis-overflow",
        viewport(20, 30),
        &nodes,
        LIMITS,
        &expected,
    );
}

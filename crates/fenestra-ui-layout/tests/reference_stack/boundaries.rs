use fenestra_ui_layout::prototype::LayoutAxisV1::{Column, Row};

use super::support::{
    LIMITS, assert_reference_case, dimension, fixed, fixed_node, node, padding, record, viewport,
};

#[test]
fn padding_can_exactly_consume_both_resolved_extents() {
    let nodes = [
        node(0, None, Column, fixed(10), fixed(8), padding(4, 6, 8, 0), 0),
        fixed_node(1, Some(0), Column, 3, 2),
    ];
    let expected = [record(0, 0, 0, 10, 8), record(1, 4, 8, 3, 2)];

    assert_reference_case(
        "padding-equal-box",
        viewport(10, 8),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn zero_width_child_does_not_advance_a_row_cursor() {
    let nodes = [
        fixed_node(0, None, Row, 30, 20),
        fixed_node(1, Some(0), Column, 7, 5),
        fixed_node(2, Some(0), Column, 0, 6),
        fixed_node(3, Some(0), Column, 5, 4),
    ];
    let expected = [
        record(0, 0, 0, 30, 20),
        record(1, 0, 0, 7, 5),
        record(2, 7, 0, 0, 6),
        record(3, 7, 0, 5, 4),
    ];

    assert_reference_case(
        "zero-width-child",
        viewport(30, 20),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn zero_height_child_still_has_gaps_on_both_sides() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(30),
            fixed(30),
            padding(0, 0, 0, 0),
            3,
        ),
        fixed_node(1, Some(0), Column, 7, 5),
        fixed_node(2, Some(0), Column, 8, 0),
        fixed_node(3, Some(0), Column, 9, 4),
    ];
    let expected = [
        record(0, 0, 0, 30, 30),
        record(1, 0, 0, 7, 5),
        record(2, 0, 8, 8, 0),
        record(3, 0, 11, 9, 4),
    ];

    assert_reference_case(
        "zero-height-gap",
        viewport(30, 30),
        &nodes,
        LIMITS,
        &expected,
    );
}

fn assert_zero_viewport_case(name: &str, width: i32, height: i32) {
    let nodes = [
        fixed_node(0, None, Column, 12, 9),
        fixed_node(1, Some(0), Column, 4, 3),
    ];
    let expected = [record(0, 0, 0, 12, 9), record(1, 0, 0, 4, 3)];

    assert_reference_case(name, viewport(width, height), &nodes, LIMITS, &expected);
}

#[test]
fn zero_width_viewport_keeps_fixed_bounds() {
    assert_zero_viewport_case("zero-width-viewport", 0, 5);
}

#[test]
fn zero_height_viewport_keeps_fixed_bounds() {
    assert_zero_viewport_case("zero-height-viewport", 7, 0);
}

#[test]
fn zero_by_zero_viewport_keeps_fixed_bounds() {
    assert_zero_viewport_case("zero-by-zero-viewport", 0, 0);
}

#[test]
fn large_integer_padding_and_gap_remain_exact() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(4096),
            fixed(4096),
            padding(4096, 0, 4096, 0),
            4096,
        ),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, 4096, 4096),
        record(1, 4096, 4096, 0, 0),
        record(2, 4096, 8192, 0, 0),
    ];

    assert_reference_case(
        "large-integer-padding-gap",
        viewport(4096, 4096),
        &nodes,
        LIMITS,
        &expected,
    );
}

#[test]
fn exact_node_ceiling_has_independent_child_headroom() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(1), Column, 0, 0),
        fixed_node(3, Some(1), Column, 0, 0),
        fixed_node(4, Some(1), Column, 0, 0),
        fixed_node(5, Some(0), Column, 0, 0),
        fixed_node(6, Some(5), Column, 0, 0),
        fixed_node(7, Some(5), Column, 0, 0),
        fixed_node(8, Some(5), Column, 0, 0),
        fixed_node(9, Some(0), Column, 0, 0),
        fixed_node(10, Some(9), Column, 0, 0),
        fixed_node(11, Some(9), Column, 0, 0),
        fixed_node(12, Some(9), Column, 0, 0),
        fixed_node(13, Some(0), Column, 0, 0),
        fixed_node(14, Some(13), Column, 0, 0),
        fixed_node(15, Some(13), Column, 0, 0),
        fixed_node(16, Some(13), Column, 0, 0),
        fixed_node(17, Some(0), Column, 0, 0),
        fixed_node(18, Some(17), Column, 0, 0),
        fixed_node(19, Some(17), Column, 0, 0),
        fixed_node(20, Some(17), Column, 0, 0),
        fixed_node(21, Some(0), Column, 0, 0),
        fixed_node(22, Some(21), Column, 0, 0),
        fixed_node(23, Some(21), Column, 0, 0),
        fixed_node(24, Some(21), Column, 0, 0),
        fixed_node(25, Some(0), Column, 0, 0),
        fixed_node(26, Some(25), Column, 0, 0),
        fixed_node(27, Some(25), Column, 0, 0),
        fixed_node(28, Some(25), Column, 0, 0),
        fixed_node(29, Some(0), Column, 0, 0),
        fixed_node(30, Some(29), Column, 0, 0),
        fixed_node(31, Some(29), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, 0, 0),
        record(1, 0, 0, 0, 0),
        record(2, 0, 0, 0, 0),
        record(3, 0, 0, 0, 0),
        record(4, 0, 0, 0, 0),
        record(5, 0, 0, 0, 0),
        record(6, 0, 0, 0, 0),
        record(7, 0, 0, 0, 0),
        record(8, 0, 0, 0, 0),
        record(9, 0, 0, 0, 0),
        record(10, 0, 0, 0, 0),
        record(11, 0, 0, 0, 0),
        record(12, 0, 0, 0, 0),
        record(13, 0, 0, 0, 0),
        record(14, 0, 0, 0, 0),
        record(15, 0, 0, 0, 0),
        record(16, 0, 0, 0, 0),
        record(17, 0, 0, 0, 0),
        record(18, 0, 0, 0, 0),
        record(19, 0, 0, 0, 0),
        record(20, 0, 0, 0, 0),
        record(21, 0, 0, 0, 0),
        record(22, 0, 0, 0, 0),
        record(23, 0, 0, 0, 0),
        record(24, 0, 0, 0, 0),
        record(25, 0, 0, 0, 0),
        record(26, 0, 0, 0, 0),
        record(27, 0, 0, 0, 0),
        record(28, 0, 0, 0, 0),
        record(29, 0, 0, 0, 0),
        record(30, 0, 0, 0, 0),
        record(31, 0, 0, 0, 0),
    ];

    assert_reference_case("node-ceiling", viewport(0, 0), &nodes, LIMITS, &expected);
}

#[test]
fn exact_child_ceiling_places_all_sixteen_children() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
        fixed_node(3, Some(0), Column, 0, 0),
        fixed_node(4, Some(0), Column, 0, 0),
        fixed_node(5, Some(0), Column, 0, 0),
        fixed_node(6, Some(0), Column, 0, 0),
        fixed_node(7, Some(0), Column, 0, 0),
        fixed_node(8, Some(0), Column, 0, 0),
        fixed_node(9, Some(0), Column, 0, 0),
        fixed_node(10, Some(0), Column, 0, 0),
        fixed_node(11, Some(0), Column, 0, 0),
        fixed_node(12, Some(0), Column, 0, 0),
        fixed_node(13, Some(0), Column, 0, 0),
        fixed_node(14, Some(0), Column, 0, 0),
        fixed_node(15, Some(0), Column, 0, 0),
        fixed_node(16, Some(0), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, 0, 0),
        record(1, 0, 0, 0, 0),
        record(2, 0, 0, 0, 0),
        record(3, 0, 0, 0, 0),
        record(4, 0, 0, 0, 0),
        record(5, 0, 0, 0, 0),
        record(6, 0, 0, 0, 0),
        record(7, 0, 0, 0, 0),
        record(8, 0, 0, 0, 0),
        record(9, 0, 0, 0, 0),
        record(10, 0, 0, 0, 0),
        record(11, 0, 0, 0, 0),
        record(12, 0, 0, 0, 0),
        record(13, 0, 0, 0, 0),
        record(14, 0, 0, 0, 0),
        record(15, 0, 0, 0, 0),
        record(16, 0, 0, 0, 0),
    ];

    assert_reference_case("child-ceiling", viewport(0, 0), &nodes, LIMITS, &expected);
}

#[test]
fn exact_depth_ceiling_keeps_absolute_zero_origins() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(1), Column, 0, 0),
        fixed_node(3, Some(2), Column, 0, 0),
        fixed_node(4, Some(3), Column, 0, 0),
        fixed_node(5, Some(4), Column, 0, 0),
        fixed_node(6, Some(5), Column, 0, 0),
        fixed_node(7, Some(6), Column, 0, 0),
    ];
    let expected = [
        record(0, 0, 0, 0, 0),
        record(1, 0, 0, 0, 0),
        record(2, 0, 0, 0, 0),
        record(3, 0, 0, 0, 0),
        record(4, 0, 0, 0, 0),
        record(5, 0, 0, 0, 0),
        record(6, 0, 0, 0, 0),
        record(7, 0, 0, 0, 0),
    ];

    assert_reference_case("depth-ceiling", viewport(0, 0), &nodes, LIMITS, &expected);
}

#[test]
fn registered_runtime_fixture_matches_materialized_vertical_stack() {
    let nodes = [
        node(
            0,
            None,
            Column,
            dimension(0, 100, 100),
            dimension(0, 80, 80),
            padding(0, 0, 0, 0),
            0,
        ),
        node(
            1,
            Some(0),
            Column,
            dimension(0, 80, 100),
            dimension(0, 50, 50),
            padding(0, 0, 0, 0),
            0,
        ),
        node(
            2,
            Some(1),
            Column,
            dimension(0, 30, 80),
            dimension(0, 10, 10),
            padding(0, 0, 0, 0),
            0,
        ),
        node(
            3,
            Some(1),
            Column,
            dimension(0, 40, 80),
            dimension(0, 12, 12),
            padding(0, 0, 0, 0),
            0,
        ),
        node(
            4,
            Some(1),
            Column,
            dimension(0, 40, 80),
            dimension(0, 12, 12),
            padding(0, 0, 0, 0),
            0,
        ),
    ];
    let expected = [
        record(0, 0, 0, 100, 80),
        record(1, 0, 0, 80, 50),
        record(2, 0, 0, 30, 10),
        record(3, 0, 10, 40, 12),
        record(4, 0, 22, 40, 12),
    ];

    assert_reference_case(
        "registered-runtime-fixture",
        viewport(120, 90),
        &nodes,
        LIMITS,
        &expected,
    );
}

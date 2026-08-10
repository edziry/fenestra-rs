use fenestra_ui_layout::prototype::{
    LayoutAxisV1::Column, LayoutConstraintFieldV1, LayoutErrorLocationV1, LayoutExtentV1,
    LayoutPaddingSideV1,
};

use crate::candidate::{
    CandidateProfileErrorFieldV1 as Field, CandidateProfileErrorKindV1 as Kind,
    validate_candidate_input_v1,
};

use super::support::{assert_profile_error, dimension, fixed, node, padding, viewport};

#[test]
fn every_candidate_input_scalar_accepts_the_inclusive_ceiling() {
    let nodes = [node(
        0,
        None,
        Column,
        dimension(4096, 4096, 4096),
        dimension(4096, 4096, 4096),
        padding(4096, 4096, 4096, 4096),
        4096,
    )];

    validate_candidate_input_v1(viewport(4096, 4096), &nodes)
        .expect("every candidate input scalar at 4096 must be admitted");
}

#[test]
fn viewport_fields_precede_every_node_field() {
    let nodes = [node(
        0,
        None,
        Column,
        dimension(4097, 4097, 4097),
        fixed(0),
        padding(0, 0, 0, 0),
        4097,
    )];

    assert_profile_error(
        validate_candidate_input_v1(viewport(4097, 4097), &nodes),
        Kind::CoordinateLimit,
        Field::Viewport(LayoutExtentV1::Width),
        LayoutErrorLocationV1::Viewport,
    );
    assert_profile_error(
        validate_candidate_input_v1(viewport(0, 4097), &nodes),
        Kind::CoordinateLimit,
        Field::Viewport(LayoutExtentV1::Height),
        LayoutErrorLocationV1::Viewport,
    );
}

#[test]
fn node_fields_follow_progressive_suffix_priority_through_gap() {
    let over = 4097;
    let all_over = dimension(over, over, over);
    let zero = fixed(0);
    let cases = [
        (
            all_over,
            all_over,
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Width,
                field: LayoutConstraintFieldV1::Minimum,
            },
        ),
        (
            dimension(0, over, over),
            all_over,
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Width,
                field: LayoutConstraintFieldV1::Preferred,
            },
        ),
        (
            dimension(0, 0, over),
            all_over,
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Width,
                field: LayoutConstraintFieldV1::Maximum,
            },
        ),
        (
            zero,
            all_over,
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Height,
                field: LayoutConstraintFieldV1::Minimum,
            },
        ),
        (
            zero,
            dimension(0, over, over),
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Height,
                field: LayoutConstraintFieldV1::Preferred,
            },
        ),
        (
            zero,
            dimension(0, 0, over),
            padding(over, over, over, over),
            over,
            Field::Constraint {
                extent: LayoutExtentV1::Height,
                field: LayoutConstraintFieldV1::Maximum,
            },
        ),
        (
            zero,
            zero,
            padding(over, over, over, over),
            over,
            Field::Padding(LayoutPaddingSideV1::Left),
        ),
        (
            zero,
            zero,
            padding(0, over, over, over),
            over,
            Field::Padding(LayoutPaddingSideV1::Right),
        ),
        (
            zero,
            zero,
            padding(0, 0, over, over),
            over,
            Field::Padding(LayoutPaddingSideV1::Top),
        ),
        (
            zero,
            zero,
            padding(0, 0, 0, over),
            over,
            Field::Padding(LayoutPaddingSideV1::Bottom),
        ),
        (zero, zero, padding(0, 0, 0, 0), over, Field::Gap),
    ];

    for (width, height, node_padding, gap, field) in cases {
        let nodes = [node(0, None, Column, width, height, node_padding, gap)];
        assert_profile_error(
            validate_candidate_input_v1(viewport(0, 0), &nodes),
            Kind::CoordinateLimit,
            field,
            LayoutErrorLocationV1::InputNode { index: 0 },
        );
    }
}

#[test]
fn node_major_order_places_root_gap_before_child_constraints() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(0),
            fixed(0),
            padding(0, 0, 0, 0),
            4097,
        ),
        node(
            1,
            Some(0),
            Column,
            dimension(4097, 4097, 4097),
            fixed(0),
            padding(0, 0, 0, 0),
            0,
        ),
    ];

    assert_profile_error(
        validate_candidate_input_v1(viewport(0, 0), &nodes),
        Kind::CoordinateLimit,
        Field::Gap,
        LayoutErrorLocationV1::InputNode { index: 0 },
    );
}

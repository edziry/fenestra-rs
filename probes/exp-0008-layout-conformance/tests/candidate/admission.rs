use fenestra_ui_exp_0008_layout_conformance::prototype::{
    TaffyStackEngineV1, registered_layout_corpus_v1,
};
use fenestra_ui_layout::prototype::{
    LayoutAxisV1::Column, LayoutErrorLocationV1, LayoutInputV1, REGISTERED_LAYOUT_LIMITS_V1,
    compute_layout_v1,
};

use super::support::{
    assert_candidate_rejection, dimension, fixed, fixed_node, node, padding, viewport,
};

#[test]
fn candidate_scalar_ceiling_is_inclusive_for_constraints_padding_gap_and_viewport() {
    let case = registered_layout_corpus_v1()
        .into_iter()
        .find(|case| case.name() == "large-integer-padding-gap")
        .expect("the exact candidate-ceiling case must be registered");

    let output = compute_layout_v1(
        &TaffyStackEngineV1::new(),
        LayoutInputV1::new(case.viewport(), case.nodes()),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect("every candidate scalar at 4096 must be admitted");
    assert_eq!(output.records(), case.expected_records());
}

#[test]
fn right_and_bottom_padding_at_the_candidate_ceiling_are_admitted() {
    let nodes = [
        node(
            0,
            None,
            Column,
            fixed(4096),
            fixed(4096),
            padding(0, 4096, 0, 4096),
            0,
        ),
        fixed_node(1, Some(0), Column, 0, 0),
    ];
    let output = compute_layout_v1(
        &TaffyStackEngineV1::new(),
        LayoutInputV1::new(viewport(4096, 4096), &nodes),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect("right and bottom padding at 4096 must be admitted");

    assert_eq!(output.records()[1].bounds().x(), 0);
    assert_eq!(output.records()[1].bounds().y(), 0);
}

#[test]
fn viewport_one_over_is_core_valid_but_candidate_rejected() {
    let engine = TaffyStackEngineV1::new();
    let nodes = [fixed_node(0, None, Column, 1, 1)];

    assert_candidate_rejection(
        "viewport-width-one-over",
        &engine,
        viewport(4097, 0),
        &nodes,
        LayoutErrorLocationV1::Viewport,
    );
    assert_candidate_rejection(
        "viewport-height-one-over",
        &engine,
        viewport(0, 4097),
        &nodes,
        LayoutErrorLocationV1::Viewport,
    );
}

#[test]
fn every_constraint_field_one_over_is_core_valid_but_candidate_rejected() {
    let engine = TaffyStackEngineV1::new();
    let zero = fixed(0);
    let cases = [
        ("width-minimum", dimension(4097, 4097, 4097), zero),
        ("width-preferred", dimension(0, 4097, 4097), zero),
        ("width-maximum", dimension(0, 0, 4097), zero),
        ("height-minimum", zero, dimension(4097, 4097, 4097)),
        ("height-preferred", zero, dimension(0, 4097, 4097)),
        ("height-maximum", zero, dimension(0, 0, 4097)),
    ];

    for (name, width, height) in cases {
        let nodes = [node(0, None, Column, width, height, padding(0, 0, 0, 0), 0)];
        assert_candidate_rejection(
            name,
            &engine,
            viewport(0, 0),
            &nodes,
            LayoutErrorLocationV1::InputNode { index: 0 },
        );
    }
}

#[test]
fn gap_one_over_is_core_valid_but_candidate_rejected() {
    let engine = TaffyStackEngineV1::new();
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
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
    ];

    assert_candidate_rejection(
        "gap-one-over",
        &engine,
        viewport(0, 0),
        &nodes,
        LayoutErrorLocationV1::InputNode { index: 0 },
    );
}

#[test]
fn admission_priority_is_viewport_then_node_major_fields() {
    let engine = TaffyStackEngineV1::new();
    let node_over = [node(
        0,
        None,
        Column,
        dimension(0, 0, 4097),
        fixed(0),
        padding(0, 0, 0, 0),
        0,
    )];
    assert_candidate_rejection(
        "viewport-before-node",
        &engine,
        viewport(4097, 0),
        &node_over,
        LayoutErrorLocationV1::Viewport,
    );

    let root_gap_before_child_constraint = [
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
            dimension(0, 0, 4097),
            fixed(0),
            padding(0, 0, 0, 0),
            0,
        ),
    ];
    assert_candidate_rejection(
        "root-gap-before-child-constraint",
        &engine,
        viewport(0, 0),
        &root_gap_before_child_constraint,
        LayoutErrorLocationV1::InputNode { index: 0 },
    );
}

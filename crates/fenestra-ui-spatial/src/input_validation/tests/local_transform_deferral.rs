use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};

use super::local_transform_support::{
    VIEWPORT, container, expect_valid, fixed, free_node, identity, input, layout_node, limits,
    root, root_with_container, transform, valid_container, validate,
};
use crate::model::SpatialViewportV2;

#[test]
fn successful_transform_stage_retains_the_prepared_islands() {
    let identity = identity();
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, identity),
        free_node(2, 0, 10, 10, identity),
        layout_node(3, 2, fixed(10), fixed(10), identity),
        free_node(4, 3, 10, 10, identity),
        layout_node(5, 4, fixed(10), fixed(10), identity),
        layout_node(6, 3, fixed(10), fixed(10), identity),
    ]);

    let proof = expect_valid(prepare_local_transforms!(
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
fn local_validation_does_not_execute_prepared_layout() {
    let root_container = container(LayoutAxisV1::Row, LayoutPaddingV1::new(1, 0, 0, 0), 0);
    let fixture = input(vec![
        root_with_container(root_container),
        layout_node(1, 0, fixed(i32::MAX), fixed(0), identity()),
    ]);

    expect_valid(validate(
        &fixture,
        SpatialViewportV2::new(i32::MAX, 0),
        limits(1, 2, 2),
    ));
}

#[test]
fn dependency_targets_and_content_grammar_remain_deferred() {
    let fixture = input(vec![root(), free_node(1, 0, 10, 10, identity())]);

    expect_valid(validate(&fixture, VIEWPORT, limits(0, 0, 0)));
}

#[test]
fn local_invertibility_does_not_validate_future_composition() {
    let tiny = transform([1, 0, 0, 1, 0, 0, 0, 0]);
    assert_eq!(tiny.affine().determinant_raw(), 1);
    let composed = match tiny.affine().checked_compose(tiny.affine()) {
        Ok(composed) => composed,
        Err(_) => panic!("tiny local transforms should compose within the scalar domain"),
    };
    assert_eq!(composed.determinant_raw(), 0);

    let fixture = input(vec![
        root_with_container(valid_container()),
        layout_node(1, 0, fixed(10), fixed(10), tiny),
        layout_node(2, 1, fixed(10), fixed(10), tiny),
    ]);
    expect_valid(validate(&fixture, VIEWPORT, limits(1, 3, 3)));
}

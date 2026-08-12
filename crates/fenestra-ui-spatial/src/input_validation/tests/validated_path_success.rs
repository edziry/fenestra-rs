use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::validated_path_support::{
    expect_valid, fixture, flattening_poison_limits, limits, line_to, move_to, path, validate,
};
use crate::path::SpatialPathVerbV2;

#[test]
fn empty_paths_retain_zero_validated_facts_and_total() {
    let fixture = fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_validated_paths!(&fixture, VIEWPORT, limits(0, 0)));

    assert!(proof.validated_path_facts().is_empty());
    assert_eq!(proof.subpath_total(), 0);
    assert!(proof.path_range_facts().is_empty());
}

#[test]
fn successful_k1_proofs_and_all_prior_facts_survive_the_stage() {
    let identity = identity();
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, identity),
        free_node(2, 0, 10, 10, identity),
        layout_node(3, 2, fixed(10), fixed(10), identity),
        free_node(4, 3, 10, 10, identity),
        layout_node(5, 4, fixed(10), fixed(10), identity),
        layout_node(6, 3, fixed(10), fixed(10), identity),
    ])
    .with_paths(
        vec![path(0, 0, 3), path(1, 3, 4)],
        vec![
            move_to(0, 0),
            line_to(1, 1),
            SpatialPathVerbV2::Close,
            move_to(2, 2),
            line_to(3, 3),
            move_to(4, 4),
            line_to(5, 5),
        ],
    );
    let proof = expect_valid(prepare_validated_paths!(&fixture, VIEWPORT, limits(4, 3)));

    assert_eq!(proof.validated_path_facts(), vec![(0, 3, 1), (1, 4, 2)]);
    assert_eq!(proof.subpath_total(), 3);
    assert_eq!(
        proof.path_range_facts(),
        vec![(0, 0_u128, 3_u128), (1, 3, 7)]
    );
    assert_eq!(
        proof.prepared_island_facts(),
        vec![(0, vec![2, 3, 6]), (1, vec![4, 5])]
    );
}

#[test]
fn shapes_and_k2_flattening_remain_deferred() {
    let fixture = fixture(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)]);

    expect_valid(validate(&fixture, flattening_poison_limits()));
}

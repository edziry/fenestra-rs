use super::local_transform_support::{
    Placement, VIEWPORT, fixed, free_node, identity, identity_values, input, layout_node, node,
    root, set_field, transform,
};
use super::path_structure_support::{
    closes, expect_valid, fixture, limits, path, permissive_limits, validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::path::SpatialPathVerbV2;
use crate::vocabulary::{SpatialNodeFieldV2, SpatialTransformScalarFieldV2};

#[test]
fn empty_and_zero_length_partitions_are_valid() {
    expect_valid(validate(
        &fixture(Vec::new(), Vec::new()),
        permissive_limits(),
    ));

    let zeros = fixture(vec![path(0, 0, 0), path(1, 0, 0)], Vec::new());
    let proof = expect_valid(prepare_path_structure!(
        &zeros,
        VIEWPORT,
        permissive_limits()
    ));
    assert_eq!(
        proof.path_range_facts(),
        vec![(0, 0_u128, 0_u128), (1, 0, 0)]
    );
}

#[test]
fn exact_ranges_and_prepared_islands_survive_the_structure_stage() {
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
    .with_paths(vec![path(0, 0, 2), path(1, 2, 0), path(2, 2, 1)], closes(3));

    let proof = expect_valid(prepare_path_structure!(
        &fixture,
        VIEWPORT,
        permissive_limits()
    ));
    assert_eq!(
        proof.path_range_facts(),
        vec![(0, 0_u128, 2_u128), (1, 2, 2), (2, 2, 3)]
    );
    assert_eq!(
        proof.prepared_island_facts(),
        vec![(0, vec![2, 3, 6]), (1, vec![4, 5])]
    );
}

#[test]
fn k1_limits_grammar_and_later_content_tables_remain_deferred() {
    let grammar_poison = fixture(vec![path(0, 0, 1)], closes(1));
    expect_valid(validate(&grammar_poison, limits(0, 0)));

    let scalar_poison = fixture(
        vec![path(0, 0, 2)],
        vec![
            SpatialPathVerbV2::MoveTo {
                to: SpatialPointV2::new(
                    SpatialScalarV2::new(SpatialScalarV2::MAX_RAW + 1),
                    SpatialScalarV2::new(0),
                ),
            },
            SpatialPathVerbV2::LineTo {
                to: SpatialPointV2::new(SpatialScalarV2::new(0), SpatialScalarV2::new(0)),
            },
        ],
    );
    expect_valid(validate(&scalar_poison, permissive_limits()));

    let valid_subpath = fixture(
        vec![path(0, 0, 2)],
        vec![
            SpatialPathVerbV2::MoveTo {
                to: SpatialPointV2::new(SpatialScalarV2::new(0), SpatialScalarV2::new(0)),
            },
            SpatialPathVerbV2::LineTo {
                to: SpatialPointV2::new(SpatialScalarV2::new(1), SpatialScalarV2::new(1)),
            },
        ],
    );
    expect_valid(validate(&valid_subpath, limits(2, 0)));
}

#[test]
fn local_transform_failure_precedes_path_keys_and_ranges() {
    let mut values = identity_values();
    set_field(
        &mut values,
        SpatialTransformScalarFieldV2::AffineA,
        SpatialScalarV2::MAX_RAW + 1,
    );
    let fixture = input(vec![
        root(),
        node(Placement::Layout, 1, 0, transform(values)),
    ])
    .with_paths(vec![path(u32::MAX, 1, u32::MAX)], Vec::new());

    super::local_transform_support::expect_transform(
        prepare_path_structure!(&fixture, VIEWPORT, permissive_limits()).map(|_| ()),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineA),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::AffineA,
        },
    );
}

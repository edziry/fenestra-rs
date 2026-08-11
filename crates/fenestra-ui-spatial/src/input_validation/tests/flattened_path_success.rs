use super::flattened_path_support::{
    deferred_limits, expect_valid, fixture, limits, line_to, move_to, path, point,
};
use super::local_transform_support::VIEWPORT;
use super::validated_hit_support::stroke as hit_stroke;
use super::validated_paint_support::{destination, image_paint, source};
use super::validated_semantic_support::{fixture_with_items, semantic};
use super::validated_shape_support::{rect, rect_values};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;

#[test]
fn empty_paths_retain_an_empty_distinct_stage_and_zero_total() {
    let fixture = fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_flattened_paths!(&fixture, VIEWPORT, limits(0, 0)));

    assert!(proof.flattened_path_facts().is_empty());
    assert_eq!(proof.accepted_flattened_segment_total(), 0);
    assert!(proof.validated_semantic_facts().is_empty());
}

#[test]
fn exact_points_subpaths_closure_and_key_order_are_retained() {
    let fixture = fixture(
        vec![path(0, 0, 3), path(1, 3, 2), path(2, 5, 4)],
        vec![
            move_to(0, 0),
            line_to(2, 0),
            SpatialPathVerbV2::Close,
            move_to(0, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, 257),
                to: point(512, 0),
            },
            move_to(5, 5),
            line_to(5, 5),
            move_to(10, 10),
            line_to(12, 10),
        ],
    );
    let proof = expect_valid(prepare_flattened_paths!(&fixture, VIEWPORT, limits(2, 6)));

    assert_eq!(
        proof.flattened_path_facts(),
        vec![
            (0, 2, vec![(0, 0), (2, 0), (0, 0)], vec![(0, 3, true)],),
            (
                1,
                2,
                vec![(0, 0), (128, 129), (512, 0)],
                vec![(0, 3, false)],
            ),
            (
                2,
                2,
                vec![(5, 5), (5, 5), (10, 10), (12, 10)],
                vec![(0, 2, false), (2, 2, false)],
            ),
        ]
    );
    assert_eq!(proof.accepted_flattened_segment_total(), 6);
}

#[test]
fn bounds_strokes_image_finish_and_dependencies_remain_deferred() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let paints = vec![image_paint(
        1,
        0,
        0,
        source(0, 0, 1, 1),
        destination(maximum, maximum, maximum, maximum),
        None,
    )];
    let hits = vec![hit_stroke(
        1,
        0,
        0,
        maximum,
        None,
        SpatialInputPolicyV2::Ignore,
    )];
    let semantics = vec![semantic(1, 0, 0, SpatialFillRuleV2::NonZero, None)];
    let fixture = fixture_with_items(paints, hits, semantics)
        .with_paths(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)])
        .with_shapes(
            vec![
                rect_values(0, 1, maximum, maximum, maximum, maximum),
                rect(1, 2),
                rect(2, 3),
                rect(3, 4),
            ],
            Vec::new(),
        );
    let proof = expect_valid(prepare_flattened_paths!(
        &fixture,
        VIEWPORT,
        deferred_limits(1, 1)
    ));

    assert_eq!(proof.accepted_flattened_segment_total(), 1);
}

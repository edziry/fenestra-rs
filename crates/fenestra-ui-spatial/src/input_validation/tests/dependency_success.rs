use std::ptr;

use super::dependency_support::{
    VIEWPORT, dependency_limits, expect_valid, fixture, free, free_with, identity, layout,
    node_target, offset, root,
};
use super::flattened_path_support::{line_to, move_to, path, point};
use super::local_bounds_support::aabb;
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_clip_support::root_clip;
use super::validated_hit_support::fill as hit_fill;
use super::validated_image_support::blank_image;
use super::validated_paint_support::{destination, fill, image_paint, source};
use super::validated_semantic_support::semantic;
use super::validated_shape_support::{path_shape, polygon, rect_values};
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::model::{SpatialAnchorTargetV2, SpatialScalarV2};
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_tables_retain_a_distinct_empty_dependency_graph_stage() {
    let fixture = fixture(vec![root()]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(0, 0)
    ));

    assert!(proof.dependency_unit_facts().is_empty());
    assert!(proof.dependency_order_facts().is_empty());
    assert_eq!(proof.dependency_edge_count(), 0);
    assert!(proof.shape_local_bounds_facts().is_empty());
}

#[test]
fn exact_graph_local_bounds_content_and_input_facts_survive_the_stage() {
    let paints = vec![
        fill(1, 0, 0, 0, Some(0), SpatialFillRuleV2::EvenOdd),
        image_paint(1, 1, 0, source(0, 0, 1, 1), destination(10, 20, 3, 4), None),
    ];
    let hits = vec![hit_fill(
        2,
        0,
        1,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )];
    let semantics = vec![semantic(3, 0, 2, SpatialFillRuleV2::EvenOdd, None)];
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        layout(2, 1),
        free(3, 2, SpatialAnchorTargetV2::Parent),
    ])
    .with_paths(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(2, 0)])
    .with_shapes(
        vec![
            rect_values(0, 1, 1, 2, 3, 4),
            polygon(1, 2, 0, 3),
            path_shape(2, 3, 0),
        ],
        vec![point(-2, 3), point(4, -1), point(1, 5)],
    )
    .with_brushes(
        vec![solid_color(0, color(10, 20, 30, 255)), gradient(1, 0, 2)],
        valid_stops(),
    )
    .with_images(vec![blank_image(0, 1, 1)])
    .with_clips(vec![root_clip(0, 1, 0)])
    .with_paint_items(paints)
    .with_hit_items(hits)
    .with_semantic_items(semantics);
    let requested_limits = dependency_limits(3, 2);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_semantics = raw_input.items().semantic_items();
    let raw_images = raw_input.resources().images();
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, None, vec![1], vec![]),
            (2, Some((0, 1)), vec![2], vec![1]),
            (3, None, vec![3], vec![2]),
        ]
    );
    assert_eq!(proof.dependency_edge_count(), 2);
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 3]);
    assert_eq!(
        proof.shape_local_bounds_facts(),
        vec![
            (0, aabb(1, 2, 4, 6), aabb(1, 2, 4, 6)),
            (1, aabb(-2, -1, 4, 5), aabb(-2, -1, 4, 5)),
            (2, aabb(0, 0, 2, 0), aabb(0, 0, 2, 0)),
        ]
    );
    assert_eq!(
        proof.paint_local_bounds_facts(),
        vec![(0, aabb(1, 2, 4, 6)), (1, aabb(10, 20, 13, 24))]
    );
    assert_eq!(
        proof.hit_local_bounds_facts(),
        vec![(0, aabb(-2, -1, 4, 5))]
    );
    assert_eq!(
        proof.finalized_image_paint_facts(),
        vec![(
            1,
            source(0, 0, 1, 1),
            destination(10, 20, 3, 4),
            211,
            aabb(10, 20, 13, 24)
        )]
    );
    let bytes = proof
        .finalized_image_paint_bytes(1)
        .expect("the dependency proof retains the final P5 token");
    assert!(ptr::eq(bytes, raw_images[0].bytes()));

    assert_eq!(
        proof.flattened_path_facts(),
        vec![(0, 1, vec![(0, 0), (2, 0)], vec![(0, 2, false)])]
    );
    assert_eq!(proof.accepted_flattened_segment_total(), 1);
    assert_eq!(proof.validated_semantic_facts().len(), 1);
    assert_eq!(proof.validated_hit_facts().len(), 1);
    assert_eq!(proof.validated_paint_facts().len(), 2);
    assert_eq!(
        proof.validated_clip_facts(),
        vec![(0, 1, None, 0, SpatialFillRuleV2::NonZero, 1)]
    );
    assert_eq!(proof.validated_image_facts().len(), 1);
    assert_eq!(proof.accepted_pixel_total(), 1);
    assert_eq!(
        proof.prepared_brush_facts(),
        vec![
            (0, SpatialBrushKindV2::Solid, 0),
            (1, SpatialBrushKindV2::LinearGradient, 2),
        ]
    );
    assert_eq!(proof.gradient_range_facts(), vec![(1, 0, 2)]);
    assert_eq!(
        proof.validated_shape_facts(),
        vec![
            (0, SpatialShapeKindV2::Rect, 0),
            (1, SpatialShapeKindV2::Polygon, 3),
            (2, SpatialShapeKindV2::Path, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(1, 0, 3)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.input().items().semantic_items(), raw_semantics);
    assert_eq!(proof.limits(), requested_limits);
}

#[test]
fn anchor_arithmetic_and_execution_remain_deferred_after_dry_graph_planning() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture(vec![
        root(),
        free_with(1, 0, node_target(2), offset(maximum, maximum), identity()),
        free(2, 0, SpatialAnchorTargetV2::Viewport),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(2, 1)
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![(1, None, vec![1], vec![2]), (2, None, vec![2], vec![]),]
    );
    assert_eq!(proof.dependency_order_facts(), vec![2, 1]);
}

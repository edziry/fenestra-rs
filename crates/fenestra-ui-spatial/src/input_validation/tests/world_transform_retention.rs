use std::ptr;

use super::dependency_support::{free, layout, root};
use super::flattened_path_support::{line_to, move_to, path, point};
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_clip_support::root_clip;
use super::validated_hit_support::fill as hit_fill;
use super::validated_image_support::blank_image;
use super::validated_paint_support::{destination, fill, image_paint, source};
use super::validated_semantic_support::semantic;
use super::validated_shape_support::{path_shape, polygon, rect_values};
use super::world_transform_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, expect_valid, fixture, logical, output, placement, world,
};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialAnchorTargetV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn world_transforms_retain_the_complete_predecessor_and_exact_raw_borrows() {
    let paints = vec![
        fill(1, 0, 0, 0, Some(0), SpatialFillRuleV2::EvenOdd),
        image_paint(1, 1, 0, source(0, 0, 1, 1), destination(10, 20, 3, 4), None),
    ];
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
    .with_hit_items(vec![hit_fill(
        2,
        0,
        1,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )])
    .with_semantic_items(vec![semantic(3, 0, 2, SpatialFillRuleV2::EvenOdd, None)]);
    let requested_limits = SpatialLimitsV2::new([usize::MAX; SpatialLimitKindV2::ALL.len()]);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_images = raw_input.resources().images();
    let raw_semantics = raw_input.items().semantic_items();
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 10, 10), (1, 1, 2, 3, 4)]))]);
    let proof = expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        requested_limits,
        &engine
    ));

    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [SCALE, 0, 0, SCALE, 0, logical(10)]),
            world(2, [SCALE, 0, 0, SCALE, logical(1), logical(12)]),
            world(3, [SCALE, 0, 0, SCALE, logical(1), logical(6)]),
        ]
    );
    assert_eq!(
        proof.placement_facts(),
        vec![
            placement(0, 0, 0, 20, 20, logical(20), logical(20), 0, 0),
            placement(
                1,
                0,
                logical(10),
                10,
                10,
                logical(10),
                logical(20),
                0,
                logical(10)
            ),
            placement(
                2,
                logical(1),
                logical(12),
                3,
                4,
                logical(4),
                logical(16),
                logical(1),
                logical(2)
            ),
            placement(
                3,
                logical(1),
                logical(6),
                10,
                10,
                logical(11),
                logical(16),
                0,
                -logical(6)
            ),
        ]
    );
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 3]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.polygon_range_facts(), vec![(1, 0, 3)]);
    assert_eq!(
        proof.validated_shape_facts()[1].1,
        SpatialShapeKindV2::Polygon
    );
    assert_eq!(proof.gradient_range_facts(), vec![(1, 0, 2)]);
    assert_eq!(proof.prepared_brush_facts().len(), 2);
    assert_eq!(proof.validated_image_facts().len(), 1);
    assert_eq!(proof.validated_clip_facts().len(), 1);
    assert_eq!(proof.validated_paint_facts().len(), 2);
    assert_eq!(proof.validated_hit_facts().len(), 1);
    assert_eq!(proof.validated_semantic_facts().len(), 1);
    assert_eq!(proof.flattened_path_facts().len(), 1);
    assert_eq!(proof.shape_local_bounds_facts().len(), 3);
    assert_eq!(proof.paint_local_bounds_facts().len(), 2);
    assert_eq!(proof.hit_local_bounds_facts().len(), 1);
    let bytes = proof
        .finalized_image_paint_bytes(1)
        .expect("world transforms retain the exact final P5 token");
    assert!(ptr::eq(bytes, raw_images[0].bytes()));
    assert_eq!(proof.input().items().semantic_items(), raw_semantics);
    assert_eq!(proof.limits(), requested_limits);
    assert_eq!(engine.call_count(), 1);
}

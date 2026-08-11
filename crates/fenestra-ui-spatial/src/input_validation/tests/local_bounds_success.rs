use std::ptr;

use super::flattened_path_support::{move_to, path, point};
use super::local_bounds_support::{aabb, deferred_limits, expect_valid, fixture, limits};
use super::local_transform_support::VIEWPORT;
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_clip_support::root_clip;
use super::validated_hit_support::{fill as hit_fill, stroke as hit_stroke};
use super::validated_paint_support::{destination, fill, image_paint, source, stroke};
use super::validated_semantic_support::{fixture_with_items, semantic};
use super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::paint::SpatialPaintKindV2;
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_tables_retain_a_distinct_empty_local_bounds_stage() {
    let fixture = fixture(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let proof = expect_valid(prepare_local_bounds!(&fixture, VIEWPORT, limits()));

    assert!(proof.shape_local_bounds_facts().is_empty());
    assert!(proof.paint_local_bounds_facts().is_empty());
    assert!(proof.hit_local_bounds_facts().is_empty());
    assert!(proof.finalized_image_paint_facts().is_empty());
    assert_eq!(proof.accepted_flattened_segment_total(), 0);
}

#[test]
fn exact_shape_item_image_and_predecessor_facts_survive_local_bounds() {
    let paints = vec![
        fill(1, 0, 0, 1, Some(0), SpatialFillRuleV2::EvenOdd),
        stroke(1, 1, 1, 3, 0, None),
        image_paint(1, 2, 0, source(0, 0, 1, 1), destination(10, 20, 3, 4), None),
        fill(1, 3, 5, 0, None, SpatialFillRuleV2::NonZero),
        fill(1, 4, 3, 0, None, SpatialFillRuleV2::EvenOdd),
    ];
    let hits = vec![
        hit_fill(
            1,
            0,
            2,
            Some(0),
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Accept,
        ),
        hit_fill(
            1,
            1,
            3,
            None,
            SpatialFillRuleV2::EvenOdd,
            SpatialInputPolicyV2::Ignore,
        ),
        hit_stroke(1, 2, 3, 3, None, SpatialInputPolicyV2::Accept),
        hit_fill(
            1,
            3,
            5,
            None,
            SpatialFillRuleV2::EvenOdd,
            SpatialInputPolicyV2::Accept,
        ),
    ];
    let semantics = vec![semantic(1, 0, 4, SpatialFillRuleV2::NonZero, Some(0))];
    let polygon_points = vec![point(-2, 3), point(4, -1), point(1, 5)];
    let fixture = fixture_with_items(paints, hits, semantics)
        .with_paths(
            vec![path(0, 0, 2)],
            vec![
                move_to(0, 0),
                SpatialPathVerbV2::QuadraticTo {
                    control: point(0, 257),
                    to: point(512, 0),
                },
            ],
        )
        .with_shapes(
            vec![
                rect_values(0, 1, 1, 2, 3, 4),
                rect_values(1, 1, 2, 3, 0, 5),
                circle_values(2, 1, 8, 9, 2),
                circle_values(3, 1, 4, -5, 0),
                polygon(4, 1, 0, 3),
                path_shape(5, 1, 0),
            ],
            polygon_points,
        )
        .with_brushes(
            vec![solid_color(0, color(10, 20, 30, 255)), gradient(1, 0, 2)],
            valid_stops(),
        )
        .with_clips(vec![root_clip(0, 1, 0)]);
    let requested_limits = deferred_limits();
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_images = raw_input.resources().images();
    let raw_semantics = raw_input.items().semantic_items();
    let proof = expect_valid(prepare_local_bounds!(&fixture, VIEWPORT, requested_limits));

    assert_eq!(
        proof.shape_local_bounds_facts(),
        vec![
            (0, aabb(1, 2, 4, 6), aabb(1, 2, 4, 6)),
            (1, aabb(2, 3, 2, 8), SpatialAabbV2::empty()),
            (2, aabb(6, 7, 10, 11), aabb(6, 7, 10, 11)),
            (3, aabb(4, -5, 4, -5), SpatialAabbV2::empty()),
            (4, aabb(-2, -1, 4, 5), aabb(-2, -1, 4, 5)),
            (5, aabb(0, 0, 512, 257), aabb(0, 0, 512, 257)),
        ]
    );
    assert_eq!(
        proof.paint_local_bounds_facts(),
        vec![
            (0, aabb(1, 2, 4, 6)),
            (1, aabb(0, 1, 4, 10)),
            (2, aabb(10, 20, 13, 24)),
            (3, aabb(0, 0, 512, 257)),
            (4, SpatialAabbV2::empty()),
        ]
    );
    assert_eq!(
        proof.hit_local_bounds_facts(),
        vec![
            (0, aabb(6, 7, 10, 11)),
            (1, SpatialAabbV2::empty()),
            (2, aabb(2, -7, 6, -3)),
            (3, aabb(0, 0, 512, 257)),
        ]
    );
    assert_eq!(
        proof.finalized_image_paint_facts(),
        vec![(
            2,
            source(0, 0, 1, 1),
            destination(10, 20, 3, 4),
            211,
            aabb(10, 20, 13, 24),
        )]
    );
    let bytes = proof
        .finalized_image_paint_bytes(2)
        .expect("the final P5 token retains the exact raw image bytes");
    assert!(ptr::eq(bytes, raw_images[0].bytes()));

    assert_eq!(
        proof.flattened_path_facts(),
        vec![(
            0,
            2,
            vec![(0, 0), (128, 129), (512, 0)],
            vec![(0, 3, false)],
        )]
    );
    assert_eq!(proof.accepted_flattened_segment_total(), 2);
    assert_eq!(proof.validated_semantic_facts().len(), 1);
    assert_eq!(proof.validated_hit_facts().len(), 4);
    assert_eq!(proof.validated_paint_facts().len(), 5);
    assert_eq!(
        proof.validated_clip_facts(),
        vec![(0, 1, None, 0, SpatialFillRuleV2::NonZero, 1)]
    );
    assert_eq!(proof.validated_image_facts().len(), 2);
    assert_eq!(proof.accepted_pixel_total(), 20);
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
            (1, SpatialShapeKindV2::Rect, 0),
            (2, SpatialShapeKindV2::Circle, 0),
            (3, SpatialShapeKindV2::Circle, 0),
            (4, SpatialShapeKindV2::Polygon, 3),
            (5, SpatialShapeKindV2::Path, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(4, 0, 3)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.input().items().semantic_items(), raw_semantics);
    assert_eq!(proof.limits(), requested_limits);
    assert_eq!(
        proof.validated_paint_facts()[0].3,
        SpatialPaintKindV2::CoveragePaint
    );
}

use std::ptr;

use super::flattened_path_support::{expect_valid, limits, line_to, move_to, path};
use super::local_transform_support::VIEWPORT;
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_hit_support::{fill as hit_fill, stroke as hit_stroke};
use super::validated_paint_support::{destination, image_paint, source};
use super::validated_semantic_support::{fixture_with_items, semantic};
use super::validated_shape_support::{path_shape, point, polygon, rect};
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn flattened_paths_retain_the_exact_input_limits_and_every_predecessor_stage() {
    let paints = vec![
        super::validated_paint_support::fill(1, 0, 0, 1, Some(0), SpatialFillRuleV2::EvenOdd),
        image_paint(
            2,
            0,
            1,
            source(0, 0, 2, 2),
            destination(-5, 6, 7, 8),
            Some(0),
        ),
        super::validated_paint_support::stroke(3, 0, 2, 5, 0, Some(0)),
    ];
    let hits = vec![
        hit_fill(
            1,
            0,
            0,
            Some(0),
            SpatialFillRuleV2::EvenOdd,
            SpatialInputPolicyV2::Accept,
        ),
        hit_stroke(1, 1, 0, 7, Some(0), SpatialInputPolicyV2::Ignore),
        hit_fill(
            3,
            0,
            2,
            Some(0),
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Ignore,
        ),
        hit_stroke(4, 0, 3, 9, Some(3), SpatialInputPolicyV2::Accept),
    ];
    let semantics = vec![
        semantic(1, 0, 0, SpatialFillRuleV2::EvenOdd, Some(0)),
        semantic(1, 1, 0, SpatialFillRuleV2::NonZero, None),
        semantic(3, 0, 2, SpatialFillRuleV2::EvenOdd, Some(0)),
        semantic(4, 0, 3, SpatialFillRuleV2::NonZero, Some(3)),
    ];
    let fixture = fixture_with_items(paints, hits, semantics)
        .with_paths(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)])
        .with_shapes(
            vec![
                polygon(0, 1, 0, 3),
                path_shape(1, 2, 0),
                rect(2, 3),
                rect(3, 4),
            ],
            vec![point(0, 0), point(2, 0), point(0, 2)],
        )
        .with_brushes(
            vec![solid_color(0, color(10, 20, 30, 255)), gradient(1, 0, 2)],
            valid_stops(),
        );
    let requested_limits = limits(1, 1);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_semantics = raw_input.items().semantic_items();
    let raw_images = raw_input.resources().images();
    let proof = expect_valid(prepare_flattened_paths!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.flattened_path_facts(),
        vec![(0, 1, vec![(0, 0), (1, 1)], vec![(0, 2, false)],)]
    );
    assert_eq!(proof.accepted_flattened_segment_total(), 1);
    assert_eq!(proof.validated_semantic_facts().len(), 4);
    assert_eq!(proof.validated_hit_facts().len(), 4);
    assert_eq!(
        proof.validated_fill_hit_facts(),
        vec![
            (0, 0, SpatialFillRuleV2::EvenOdd),
            (2, 2, SpatialFillRuleV2::NonZero),
        ]
    );
    assert_eq!(
        proof.validated_stroke_hit_facts(),
        vec![(1, 0, 7), (3, 3, 9)]
    );
    assert_eq!(
        proof.validated_paint_facts(),
        vec![
            (0, 1, 0, SpatialPaintKindV2::CoveragePaint),
            (1, 2, 0, SpatialPaintKindV2::ImagePaint),
            (2, 3, 0, SpatialPaintKindV2::CoveragePaint),
        ]
    );
    let bytes = proof
        .validated_image_paint_bytes(1)
        .expect("the flattened-path proof retains the exact bound image bytes");
    assert!(ptr::eq(bytes, raw_images[1].bytes()));
    assert_eq!(proof.validated_clip_facts().len(), 4);
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
            (0, SpatialShapeKindV2::Polygon, 3),
            (1, SpatialShapeKindV2::Path, 0),
            (2, SpatialShapeKindV2::Rect, 0),
            (3, SpatialShapeKindV2::Rect, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(0, 0, 3)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.input().items().semantic_items(), raw_semantics);
    assert_eq!(proof.limits(), requested_limits);
    assert_eq!(
        proof.validated_hit_facts()[0].3,
        SpatialCoverageKindV2::Fill
    );
}

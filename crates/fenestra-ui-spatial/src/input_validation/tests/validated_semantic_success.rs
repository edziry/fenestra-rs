use std::ptr;

use super::local_transform_support::VIEWPORT;
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_hit_support::{deferred_limits, fill as hit_fill, stroke as hit_stroke};
use super::validated_paint_support::{destination, image_paint, source};
use super::validated_path_support::{move_to, path};
use super::validated_semantic_support::{expect_valid, fixture_with_items, limits, semantic};
use super::validated_shape_support::{path_shape, point, polygon, rect, rect_values};
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::model::SpatialScalarV2;
use crate::paint::SpatialPaintKindV2;
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_semantic_table_retains_an_empty_distinct_stage_and_the_hit_proof() {
    let hits = vec![hit_fill(
        1,
        0,
        0,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )];
    let fixture = fixture_with_items(Vec::new(), hits, Vec::new());
    let proof = expect_valid(prepare_validated_semantic_items!(
        &fixture,
        VIEWPORT,
        limits()
    ));

    assert!(proof.validated_semantic_facts().is_empty());
    assert_eq!(proof.validated_hit_facts().len(), 1);
}

#[test]
fn mixed_semantics_retain_fill_rules_clips_independent_order_and_every_prior_stage() {
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
        .with_paths(
            vec![path(0, 0, 2)],
            vec![move_to(0, 0), SpatialPathVerbV2::LineTo { to: point(1, 1) }],
        )
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
    let requested_limits = limits();
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_semantics = raw_input.items().semantic_items();
    let raw_images = raw_input.resources().images();
    let proof = expect_valid(prepare_validated_semantic_items!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.validated_semantic_facts(),
        vec![
            (0, 1, 0, 0, SpatialFillRuleV2::EvenOdd, Some(0)),
            (1, 1, 1, 0, SpatialFillRuleV2::NonZero, None),
            (2, 3, 0, 2, SpatialFillRuleV2::EvenOdd, Some(0)),
            (3, 4, 0, 3, SpatialFillRuleV2::NonZero, Some(3)),
        ]
    );
    assert_eq!(
        proof.validated_hit_facts(),
        vec![
            (
                0,
                1,
                0,
                SpatialCoverageKindV2::Fill,
                SpatialInputPolicyV2::Accept,
                Some(0),
            ),
            (
                1,
                1,
                1,
                SpatialCoverageKindV2::RoundStroke,
                SpatialInputPolicyV2::Ignore,
                Some(0),
            ),
            (
                2,
                3,
                0,
                SpatialCoverageKindV2::Fill,
                SpatialInputPolicyV2::Ignore,
                Some(0),
            ),
            (
                3,
                4,
                0,
                SpatialCoverageKindV2::RoundStroke,
                SpatialInputPolicyV2::Accept,
                Some(3),
            ),
        ]
    );
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
        .expect("the semantic proof retains the exact bound image bytes");
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
}

#[test]
fn p5_far_edges_k2_k3_bounds_and_dependencies_remain_deferred() {
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
    let semantics = vec![semantic(1, 0, 0, SpatialFillRuleV2::EvenOdd, None)];
    let fixture = fixture_with_items(paints, hits, semantics)
        .with_paths(
            vec![path(0, 0, 2)],
            vec![
                move_to(0, 0),
                SpatialPathVerbV2::QuadraticTo {
                    control: point(2, 4),
                    to: point(4, 0),
                },
            ],
        )
        .with_shapes(
            vec![
                rect_values(0, 1, maximum, maximum, maximum, maximum),
                rect(1, 2),
                rect(2, 3),
                rect(3, 4),
            ],
            Vec::new(),
        );

    expect_valid(prepare_validated_semantic_items!(
        &fixture,
        VIEWPORT,
        deferred_limits(1)
    ));
}

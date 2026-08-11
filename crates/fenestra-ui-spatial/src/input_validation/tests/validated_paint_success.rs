use std::ptr;

use super::local_transform_support::VIEWPORT;
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_paint_support::{
    deferred_limits, destination, expect_valid, fill, fixture, image_paint, limits,
    second_image_bytes, source, stroke,
};
use super::validated_path_support::{move_to, path};
use super::validated_shape_support::{path_shape, point, polygon, rect, rect_values};
use crate::brush::SpatialBrushKindV2;
use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2, SpatialSemanticGeometryV2};
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::paint::SpatialPaintKindV2;
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_paint_table_retains_an_empty_distinct_stage_and_its_predecessor() {
    let fixture = fixture(Vec::new());
    let proof = expect_valid(prepare_validated_paint_items!(
        &fixture,
        VIEWPORT,
        limits(0)
    ));

    assert!(proof.validated_paint_facts().is_empty());
    assert!(proof.validated_fill_paint_facts().is_empty());
    assert!(proof.validated_stroke_paint_facts().is_empty());
    assert!(proof.validated_image_paint_facts().is_empty());
    assert_eq!(proof.validated_clip_facts().len(), 4);
}

#[test]
fn mixed_paints_retain_exact_variant_facts_preclip_images_and_all_prior_stages() {
    let paints = vec![
        fill(1, 0, 0, 1, Some(0), SpatialFillRuleV2::EvenOdd),
        stroke(2, 0, 1, 7, 0, Some(0)),
        image_paint(
            3,
            0,
            1,
            source(0, 0, 2, 2),
            destination(-5, 6, 7, 8),
            Some(0),
        ),
    ];
    let fixture = fixture(paints)
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
    let requested_limits = limits(1);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_paints = raw_input.items().paint_items();
    let raw_images = raw_input.resources().images();
    let proof = expect_valid(prepare_validated_paint_items!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.validated_paint_facts(),
        vec![
            (0, 1, 0, SpatialPaintKindV2::CoveragePaint),
            (1, 2, 0, SpatialPaintKindV2::CoveragePaint),
            (2, 3, 0, SpatialPaintKindV2::ImagePaint),
        ]
    );
    assert_eq!(
        proof.validated_fill_paint_facts(),
        vec![(0, 0, SpatialFillRuleV2::EvenOdd, 1, 173, Some(0))]
    );
    assert_eq!(
        proof.validated_stroke_paint_facts(),
        vec![(1, 1, 7, 0, 173, Some(0))]
    );
    assert_eq!(
        proof.validated_image_paint_facts(),
        vec![(
            2,
            1,
            source(0, 0, 2, 2),
            destination(-5, 6, 7, 8),
            211,
            Some(0),
        )]
    );
    let bound_bytes = proof
        .validated_image_paint_bytes(2)
        .expect("image paint retains its bound P4 bytes");
    assert!(ptr::eq(bound_bytes, raw_images[1].bytes()));
    assert!(!ptr::eq(bound_bytes, raw_images[0].bytes()));

    assert_eq!(
        proof.validated_clip_facts(),
        vec![
            (0, 1, None, 0, SpatialFillRuleV2::NonZero, 1),
            (1, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd, 2),
            (2, 3, Some(1), 2, SpatialFillRuleV2::NonZero, 3),
            (3, 4, None, 3, SpatialFillRuleV2::NonZero, 1),
        ]
    );
    assert_eq!(
        proof.validated_image_facts(),
        vec![
            (0, 4, 4, 16, vec![0; 64]),
            (1, 2, 2, 8, second_image_bytes()),
        ]
    );
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
    assert_eq!(proof.input().items().paint_items(), raw_paints);
    assert_eq!(proof.limits(), requested_limits);
}

#[test]
fn image_far_edges_k2_k3_hit_semantic_and_dependencies_remain_deferred() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture(vec![
        stroke(1, 0, 0, maximum, 0, None),
        image_paint(
            1,
            1,
            0,
            source(0, 0, 1, 1),
            destination(maximum, maximum, maximum, maximum),
            None,
        ),
    ])
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
    )
    .with_hit_items(vec![SpatialHitV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        SpatialCoverageV2::Fill {
            shape: SpatialShapeKeyV2::new(u32::MAX),
            rule: SpatialFillRuleV2::EvenOdd,
        },
        Some(SpatialClipKeyV2::new(u32::MAX)),
        SpatialInputPolicyV2::Accept,
    )])
    .with_semantic_items(vec![SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        SpatialShapeKeyV2::new(u32::MAX),
        SpatialFillRuleV2::EvenOdd,
        Some(SpatialClipKeyV2::new(u32::MAX)),
    )]);

    expect_valid(prepare_validated_paint_items!(
        &fixture,
        VIEWPORT,
        deferred_limits(2)
    ));
}

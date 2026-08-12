use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_clip_support::{
    clip, deferred_limits, expect_valid, fixture_with, limits, root_clip, standard_fixture,
};
use super::validated_image_support::blank_image;
use super::validated_path_support::{move_to, path};
use super::validated_shape_support::{fixture_with_paths, path_shape, point, polygon, rect_values};
use crate::brush::SpatialBrushKindV2;
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_clips_retain_empty_plan_and_the_predecessor() {
    let fixture = standard_fixture(Vec::new());
    let requested_limits = limits(0);
    let proof = expect_valid(prepare_validated_clips!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert!(proof.validated_clip_facts().is_empty());
    assert_eq!(proof.limits(), requested_limits);
    assert!(proof.validated_image_facts().is_empty());
}

#[test]
fn multiple_roots_branches_future_shapes_and_both_fill_rules_are_retained() {
    let transform = identity();
    let fixture = fixture_with(
        vec![
            root(),
            free_node(1, 0, 10, 10, transform),
            free_node(2, 1, 10, 10, transform),
        ],
        vec![rect_values(0, 1, 0, 0, 1, 1), rect_values(1, 2, 0, 0, 1, 1)],
        vec![
            clip(0, 2, None, 1, SpatialFillRuleV2::EvenOdd),
            root_clip(1, 1, 0),
            clip(2, 2, Some(1), 1, SpatialFillRuleV2::NonZero),
            clip(3, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd),
        ],
    );
    let proof = expect_valid(prepare_validated_clips!(&fixture, VIEWPORT, limits(2)));

    assert_eq!(
        proof.validated_clip_facts(),
        vec![
            (0, 2, None, 1, SpatialFillRuleV2::EvenOdd, 1),
            (1, 1, None, 0, SpatialFillRuleV2::NonZero, 1),
            (2, 2, Some(1), 1, SpatialFillRuleV2::NonZero, 2),
            (3, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd, 2),
        ]
    );
}

#[test]
fn mixed_clip_plan_and_every_prior_stage_fact_survive() {
    let transform = identity();
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        layout_node(2, 1, fixed(10), fixed(10), transform),
    ])
    .with_paths(
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), SpatialPathVerbV2::LineTo { to: point(1, 1) }],
    )
    .with_shapes(
        vec![polygon(0, 1, 0, 3), path_shape(1, 2, 0)],
        vec![point(0, 0), point(2, 0), point(0, 2)],
    )
    .with_brushes(
        vec![solid_color(0, color(9, 8, 7, 255)), gradient(1, 0, 2)],
        valid_stops(),
    )
    .with_images(vec![blank_image(0, 1, 1)])
    .with_clips(vec![
        root_clip(0, 1, 0),
        clip(1, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd),
    ]);
    let requested_limits = limits(2);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_clips = raw_input.geometry().clips();
    let proof = expect_valid(prepare_validated_clips!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.validated_clip_facts(),
        vec![
            (0, 1, None, 0, SpatialFillRuleV2::NonZero, 1),
            (1, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd, 2),
        ]
    );
    assert_eq!(
        proof.validated_image_facts(),
        vec![(0, 1, 1, 4, vec![0; 4])]
    );
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
            (0, SpatialShapeKindV2::Polygon, 3),
            (1, SpatialShapeKindV2::Path, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(0, 0, 3)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.input().geometry().clips(), raw_clips);
    assert_eq!(proof.limits(), requested_limits);
}

#[test]
fn item_clip_relations_flattening_bounds_and_dependencies_remain_deferred() {
    let fixture = deferred_fixture();

    expect_valid(prepare_validated_clips!(&fixture, VIEWPORT, limits(1)));
    expect_valid(prepare_validated_clips!(
        &fixture,
        VIEWPORT,
        deferred_limits(1)
    ));
}

fn deferred_fixture() -> super::fixture::RawInputFixture {
    let maximum = SpatialScalarV2::MAX_RAW;
    fixture_with_paths(
        vec![
            rect_values(0, 1, maximum, 0, maximum, 0),
            rect_values(1, 2, 0, 0, 1, 1),
            path_shape(2, 1, 0),
        ],
        Vec::new(),
        vec![path(0, 0, 2)],
        vec![
            move_to(0, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(2, 4),
                to: point(4, 0),
            },
        ],
    )
    .with_brushes(vec![solid_color(0, color(255, 255, 255, 255))], Vec::new())
    .with_images(vec![blank_image(0, 1, 1)])
    .with_clips(vec![root_clip(0, 1, 0)])
    .with_paint_items(vec![
        SpatialPaintV2::new(
            SpatialNodeKeyV2::new(2),
            0,
            SpatialPaintContentV2::CoveragePaint {
                coverage: SpatialCoverageV2::Fill {
                    shape: SpatialShapeKeyV2::new(1),
                    rule: SpatialFillRuleV2::NonZero,
                },
                brush: SpatialBrushKeyV2::new(0),
                opacity: 255,
                clip: Some(SpatialClipKeyV2::new(0)),
            },
        ),
        SpatialPaintV2::new(
            SpatialNodeKeyV2::new(2),
            1,
            SpatialPaintContentV2::ImagePaint {
                image: SpatialImageKeyV2::new(0),
                source: SpatialImageSourceRectV2::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX),
                destination: SpatialImageDestinationRectV2::new(
                    SpatialScalarV2::new(SpatialScalarV2::MAX_RAW + 1),
                    SpatialScalarV2::new(SpatialScalarV2::MIN_RAW - 1),
                    SpatialScalarV2::new(-1),
                    SpatialScalarV2::new(-1),
                ),
                opacity: 0,
                clip: Some(SpatialClipKeyV2::new(0)),
            },
        ),
    ])
}

use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::prepared_brush_support::{color, gradient, solid_color, valid_stops};
use super::validated_image_support::{
    blank_image, deferred_limits, expect_valid, fixture, image, limits,
};
use super::validated_path_support::{move_to, path};
use super::validated_shape_support::{fixture_with_paths, path_shape, point, rect_values};
use crate::brush::SpatialBrushKindV2;
use crate::content_key::SpatialImageKeyV2;
use crate::geometry_key::SpatialClipKeyV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_images_retain_empty_validated_and_prior_facts() {
    let fixture = fixture(Vec::new());
    let proof = expect_valid(prepare_validated_images!(&fixture, VIEWPORT, limits(0, 0)));

    assert!(proof.validated_image_facts().is_empty());
    assert_eq!(proof.accepted_pixel_total(), 0);
    assert!(proof.prepared_brush_facts().is_empty());
    assert!(proof.validated_shape_facts().is_empty());
}

#[test]
fn validated_images_preserve_stable_metadata_bytes_and_all_prior_facts() {
    let first = vec![0, 1, 2, 2, 3, 3, 3, 3];
    let second = vec![4, 4, 4, 4, 0, 0, 0, 0];
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
        vec![
            super::validated_shape_support::polygon(0, 1, 0, 3),
            path_shape(1, 2, 0),
        ],
        vec![point(0, 0), point(2, 0), point(0, 2)],
    )
    .with_brushes(
        vec![solid_color(0, color(9, 8, 7, 255)), gradient(1, 0, 2)],
        valid_stops(),
    )
    .with_images(vec![
        image(0, 1, 2, 4, first.clone()),
        image(1, 2, 1, 8, second.clone()),
    ]);
    let requested_limits = limits(2, 4);
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_images = raw_input.resources().images();
    let proof = expect_valid(prepare_validated_images!(
        &fixture,
        VIEWPORT,
        requested_limits
    ));

    assert_eq!(
        proof.validated_image_facts(),
        vec![(0, 1, 2, 4, first), (1, 2, 1, 8, second)]
    );
    assert_eq!(proof.accepted_pixel_total(), 4);
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
    assert_eq!(proof.input().resources().images(), raw_images);
    assert_eq!(proof.limits(), requested_limits);
    assert_eq!(
        fixture.input_with_viewport(VIEWPORT).resources().images(),
        raw_images
    );
}

#[test]
fn p5_k2_k3_clips_and_later_item_validation_remain_deferred() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture_with_paths(
        vec![
            rect_values(0, 1, maximum, 0, maximum, 0),
            path_shape(1, 1, 0),
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
    .with_brushes(Vec::new(), Vec::new())
    .with_images(vec![blank_image(0, 1, 1)])
    .with_paint_items(vec![SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        u32::MAX,
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
            clip: Some(SpatialClipKeyV2::new(u32::MAX)),
        },
    )]);

    expect_valid(prepare_validated_images!(
        &fixture,
        VIEWPORT,
        deferred_limits(1, 1)
    ));
}

use super::local_transform_support::{VIEWPORT, free_node, identity, input, root};
use super::prepared_brush_support::{
    color, expect_valid, gradient_values, later_poison_limits, limits, point, solid_color, stop,
    stop_color, validate,
};
use super::validated_path_support::{move_to, path};
use super::validated_shape_support::{fixture_with_paths, path_shape, polygon, rect};
use crate::brush::SpatialBrushKindV2;
use crate::limits::SpatialLimitKindV2;
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_brushes_retain_empty_prepared_and_prior_facts() {
    let fixture = super::prepared_brush_support::fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_prepared_brushes!(&fixture, VIEWPORT, limits(0)));

    assert!(proof.prepared_brush_facts().is_empty());
    assert!(proof.gradient_range_facts().is_empty());
    assert!(proof.validated_shape_facts().is_empty());
    assert_eq!(
        proof
            .limits()
            .limit(SpatialLimitKindV2::GradientStopsPerBrush),
        0
    );
}

#[test]
fn mixed_brushes_normalize_once_retain_values_and_preserve_all_prior_facts() {
    let transform = identity();
    let starts = [0, 0, 32_768, 32_768, u16::MAX, u16::MAX];
    let colors = [
        color(255, 128, 1, 0),
        color(254, 1, 255, 128),
        color(5, 11, 201, 128),
        color(17, 89, 203, 137),
        color(1, 127, 254, 255),
        color(64, 32, 7, 128),
    ];
    let mut stops: Vec<_> = starts
        .into_iter()
        .zip(colors)
        .map(|(offset, color)| stop_color(offset, color))
        .collect();
    stops.extend([
        stop_color(0, color(9, 8, 7, 255)),
        stop_color(u16::MAX, color(6, 5, 4, 255)),
    ]);
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        free_node(2, 0, 10, 10, transform),
    ])
    .with_paths(
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), SpatialPathVerbV2::LineTo { to: point(1, 1) }],
    )
    .with_shapes(
        vec![rect(0, 1), polygon(1, 2, 0, 3), path_shape(2, 1, 0)],
        vec![point(0, 0), point(2, 0), point(0, 2)],
    )
    .with_brushes(
        vec![
            solid_color(0, color(17, 89, 203, 137)),
            gradient_values(1, 0, 6, point(minimum, minimum), point(maximum, maximum)),
            solid_color(2, color(255, 128, 1, 0)),
            gradient_values(3, 6, 2, point(0, 0), point(0, 1)),
        ],
        stops,
    );
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_brushes = raw_input.resources().brushes().to_vec();
    let raw_stops = raw_input.resources().gradient_stops().to_vec();
    let proof = expect_valid(prepare_prepared_brushes!(&fixture, VIEWPORT, limits(8)));

    assert_eq!(
        proof.prepared_brush_facts(),
        vec![
            (0, SpatialBrushKindV2::Solid, 0),
            (1, SpatialBrushKindV2::LinearGradient, 6),
            (2, SpatialBrushKindV2::Solid, 0),
            (3, SpatialBrushKindV2::LinearGradient, 2),
        ]
    );
    assert_eq!(proof.prepared_solid_color(0), color(9, 48, 109, 137));
    assert_eq!(proof.prepared_solid_color(2), color(0, 0, 0, 0));
    let expected = [
        (0, color(0, 0, 0, 0)),
        (0, color(127, 1, 128, 128)),
        (32_768, color(3, 6, 101, 128)),
        (32_768, color(9, 48, 109, 137)),
        (u16::MAX, color(1, 127, 254, 255)),
        (u16::MAX, color(32, 16, 4, 128)),
    ];
    assert_eq!(
        proof.prepared_gradient_facts(1),
        (
            point(minimum, minimum),
            point(maximum, maximum),
            expected.to_vec()
        )
    );
    assert_eq!(
        proof.prepared_gradient_facts(3),
        (
            point(0, 0),
            point(0, 1),
            vec![(0, color(9, 8, 7, 255)), (u16::MAX, color(6, 5, 4, 255))],
        )
    );
    assert_eq!(proof.gradient_range_facts(), vec![(1, 0, 6), (3, 6, 8)]);
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
    assert!(proof.prepared_island_facts().is_empty());
    assert_eq!(proof.input().resources().brushes(), raw_brushes.as_slice());
    assert_eq!(
        proof.input().resources().gradient_stops(),
        raw_stops.as_slice()
    );
    assert_eq!(
        fixture.input_with_viewport(VIEWPORT).resources().brushes(),
        raw_brushes.as_slice()
    );
    assert_eq!(
        fixture
            .input_with_viewport(VIEWPORT)
            .resources()
            .gradient_stops(),
        raw_stops.as_slice()
    );
}

#[test]
fn p3_p4_k2_and_later_global_content_remain_deferred() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let minimum = SpatialScalarV2::MIN_RAW;
    let fixture = fixture_with_paths(
        vec![path_shape(0, 1, 0)],
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
    .with_brushes(
        vec![gradient_values(
            0,
            0,
            5,
            point(minimum, minimum),
            point(maximum, maximum),
        )],
        vec![
            stop(0),
            stop(0),
            stop(32_768),
            stop(u16::MAX),
            stop(u16::MAX),
        ],
    );

    expect_valid(validate(&fixture, later_poison_limits(5)));
}

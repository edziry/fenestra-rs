use super::local_bounds_support::{expect_bounds_error, fixture, map_error, validate};
use super::validated_hit_support::stroke as hit_stroke;
use super::validated_paint_support::{destination, fill, image_paint, source, stroke};
use super::validated_shape_support::{circle_values, rect_values};
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;
use crate::geometry_kernel::{
    GeometryK1StrokeSource, GeometryK3Error, derive_circle_bounds_k3, derive_rect_bounds_k3,
    stroke_bounds_k3, validate_circle_k1, validate_rect_k1, validate_stroke_k1,
};
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::vocabulary::SpatialAxisV2;

const TARGET: u32 = 1;

#[test]
fn mapper_preserves_all_real_shape_paint_and_hit_k3_locations() {
    let maximum = SpatialScalarV2::MAX_RAW;
    for (error, axis, location) in [
        (
            rect_error(maximum, maximum, 1, 1),
            SpatialAxisV2::X,
            shape_location(SpatialShapeFieldV2::RectWidth),
        ),
        (
            rect_error(0, maximum, 0, 1),
            SpatialAxisV2::Y,
            shape_location(SpatialShapeFieldV2::RectHeight),
        ),
        (
            circle_error(maximum, maximum, 1),
            SpatialAxisV2::X,
            shape_location(SpatialShapeFieldV2::CircleRadius),
        ),
        (
            circle_error(0, maximum, 1),
            SpatialAxisV2::Y,
            shape_location(SpatialShapeFieldV2::CircleRadius),
        ),
        (
            stroke_error(
                GeometryK1StrokeSource::Paint { index: TARGET },
                maximum,
                maximum,
            ),
            SpatialAxisV2::X,
            paint_location(SpatialPaintFieldV2::StrokeWidth),
        ),
        (
            stroke_error(GeometryK1StrokeSource::Paint { index: TARGET }, 0, maximum),
            SpatialAxisV2::Y,
            paint_location(SpatialPaintFieldV2::StrokeWidth),
        ),
        (
            stroke_error(
                GeometryK1StrokeSource::Hit { index: TARGET },
                maximum,
                maximum,
            ),
            SpatialAxisV2::X,
            hit_location(SpatialHitFieldV2::StrokeWidth),
        ),
        (
            stroke_error(GeometryK1StrokeSource::Hit { index: TARGET }, 0, maximum),
            SpatialAxisV2::Y,
            hit_location(SpatialHitFieldV2::StrokeWidth),
        ),
    ] {
        expect_bounds_error::<()>(Err(map_error(error)), axis, location);
    }
}

#[test]
fn aggregate_shape_dispatch_maps_both_axes_for_rect_and_circle() {
    let maximum = SpatialScalarV2::MAX_RAW;
    for (shape, axis, field) in [
        (
            rect_values(1, 1, maximum, maximum, 1, 1),
            SpatialAxisV2::X,
            SpatialShapeFieldV2::RectWidth,
        ),
        (
            rect_values(1, 1, 0, maximum, 0, 1),
            SpatialAxisV2::Y,
            SpatialShapeFieldV2::RectHeight,
        ),
        (
            circle_values(1, 1, maximum, maximum, 1),
            SpatialAxisV2::X,
            SpatialShapeFieldV2::CircleRadius,
        ),
        (
            circle_values(1, 1, 0, maximum, 1),
            SpatialAxisV2::Y,
            SpatialShapeFieldV2::CircleRadius,
        ),
    ] {
        let fixture = fixture(
            vec![rect_values(0, 1, 0, 0, 1, 1), shape],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        expect_bounds_error(validate(&fixture), axis, shape_location(field));
    }
}

#[test]
fn aggregate_paint_and_hit_strokes_map_both_axes_at_the_target_record() {
    let maximum = SpatialScalarV2::MAX_RAW;
    for (x, axis) in [(maximum, SpatialAxisV2::X), (0, SpatialAxisV2::Y)] {
        let shapes = vec![
            rect_values(0, 1, 0, 0, 0, 0),
            rect_values(1, 1, x, maximum, 0, 0),
        ];
        let paints = vec![
            fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero),
            stroke(1, 1, 1, 1, 0, None),
        ];
        let paint_fixture = fixture(shapes.clone(), Vec::new(), paints, Vec::new());
        expect_bounds_error(
            validate(&paint_fixture),
            axis,
            paint_location(SpatialPaintFieldV2::StrokeWidth),
        );

        let hits = vec![
            super::validated_hit_support::fill(
                1,
                0,
                0,
                None,
                SpatialFillRuleV2::NonZero,
                crate::content_item::SpatialInputPolicyV2::Accept,
            ),
            hit_stroke(
                1,
                1,
                1,
                1,
                None,
                crate::content_item::SpatialInputPolicyV2::Accept,
            ),
        ];
        let hit_fixture = fixture(shapes, Vec::new(), Vec::new(), hits);
        expect_bounds_error(
            validate(&hit_fixture),
            axis,
            hit_location(SpatialHitFieldV2::StrokeWidth),
        );
    }
}

#[test]
fn image_p5_finish_maps_destination_x_then_y_at_the_target_record() {
    let maximum = SpatialScalarV2::MAX_RAW;
    for (destination, axis, field) in [
        (
            destination(maximum, maximum, 1, 1),
            SpatialAxisV2::X,
            SpatialPaintFieldV2::DestinationWidth,
        ),
        (
            destination(0, maximum, 1, 1),
            SpatialAxisV2::Y,
            SpatialPaintFieldV2::DestinationHeight,
        ),
    ] {
        let shapes = vec![rect_values(0, 1, 0, 0, 1, 1)];
        let paints = vec![
            fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero),
            image_paint(1, 1, 0, source(0, 0, 1, 1), destination, None),
        ];
        let fixture = fixture(shapes, Vec::new(), paints, Vec::new());
        expect_bounds_error(validate(&fixture), axis, paint_location(field));
    }
}

fn rect_error(x: i64, y: i64, width: i64, height: i64) -> GeometryK3Error {
    let rect = validate_rect_k1(TARGET, point(x, y), scalar(width), scalar(height))
        .expect("test rect is K1-valid");
    derive_rect_bounds_k3(TARGET, rect).expect_err("test rect must fail K3")
}

fn circle_error(x: i64, y: i64, radius: i64) -> GeometryK3Error {
    let circle =
        validate_circle_k1(TARGET, point(x, y), scalar(radius)).expect("test circle is K1-valid");
    derive_circle_bounds_k3(TARGET, circle).expect_err("test circle must fail K3")
}

fn stroke_error(source: GeometryK1StrokeSource, x: i64, y: i64) -> GeometryK3Error {
    let rect = validate_rect_k1(7, point(x, y), scalar(0), scalar(0))
        .expect("test stroke base is K1-valid");
    let derived = derive_rect_bounds_k3(7, rect).expect("test stroke base is K3-valid");
    let stroke = validate_stroke_k1(source, scalar(1)).expect("test stroke is K1-valid");
    stroke_bounds_k3(&derived, source, stroke).expect_err("test stroke must fail K3")
}

const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

const fn shape_location(field: SpatialShapeFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Shape {
        index: TARGET,
        field,
    }
}

const fn paint_location(field: SpatialPaintFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Paint {
        index: TARGET,
        field,
    }
}

const fn hit_location(field: SpatialHitFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Hit {
        index: TARGET,
        field,
    }
}

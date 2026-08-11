//! Mapping from Geometry K1 shape failures into aggregate diagnostics.

use super::make_resolve_error;
use crate::content_diagnostic::SpatialShapeErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPolygonPointFieldV2, SpatialShapeFieldV2};
use crate::geometry_kernel::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1ShapeKind,
};
use crate::limits::SpatialLimitKindV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_shape_k1_error(error: GeometryK1Error) -> SpatialResolveErrorV2 {
    match error.kind() {
        GeometryK1ErrorKind::ScalarOutOfDomain => content_error(
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            map_shape_location(error.location()),
        ),
        GeometryK1ErrorKind::InvalidShape(kind) => content_error(
            SpatialContentErrorKindV2::InvalidShape(map_shape_kind(kind)),
            map_shape_location(error.location()),
        ),
        GeometryK1ErrorKind::LimitExceeded(GeometryK1LimitKind::PolygonPointsPerShape) => {
            SpatialResolveErrorV2::limit_exceeded(
                SpatialLimitKindV2::PolygonPointsPerShape,
                map_shape_location(error.location()),
                error
                    .observed()
                    .expect("K1 polygon limit errors carry observed evidence"),
                error
                    .maximum()
                    .expect("K1 polygon limit errors carry maximum evidence"),
            )
        }
        GeometryK1ErrorKind::InvalidPathGrammar(_)
        | GeometryK1ErrorKind::InvalidStroke(_)
        | GeometryK1ErrorKind::LimitExceeded(GeometryK1LimitKind::PathSubpathsTotal) => {
            unreachable!("shape K1 cannot return a path or stroke failure")
        }
    }
}

fn map_shape_location(location: GeometryK1Location) -> SpatialErrorLocationV2 {
    match location {
        GeometryK1Location::Shape { index, field } => SpatialErrorLocationV2::Shape {
            index,
            field: map_shape_field(field),
        },
        GeometryK1Location::PolygonPoint {
            shape,
            point,
            field,
        } => SpatialErrorLocationV2::PolygonPoint {
            shape,
            point,
            field: map_polygon_field(field),
        },
        _ => unreachable!("shape K1 failures use shape or polygon-point locations"),
    }
}

fn map_shape_field(field: GeometryK1Field) -> SpatialShapeFieldV2 {
    match field {
        GeometryK1Field::RectX => SpatialShapeFieldV2::RectX,
        GeometryK1Field::RectY => SpatialShapeFieldV2::RectY,
        GeometryK1Field::RectWidth => SpatialShapeFieldV2::RectWidth,
        GeometryK1Field::RectHeight => SpatialShapeFieldV2::RectHeight,
        GeometryK1Field::CircleCenterX => SpatialShapeFieldV2::CircleCenterX,
        GeometryK1Field::CircleCenterY => SpatialShapeFieldV2::CircleCenterY,
        GeometryK1Field::CircleRadius => SpatialShapeFieldV2::CircleRadius,
        GeometryK1Field::PolygonPointLength => SpatialShapeFieldV2::PolygonPointLength,
        _ => unreachable!("shape K1 shape locations use registered shape fields"),
    }
}

fn map_polygon_field(field: GeometryK1Field) -> SpatialPolygonPointFieldV2 {
    match field {
        GeometryK1Field::X => SpatialPolygonPointFieldV2::X,
        GeometryK1Field::Y => SpatialPolygonPointFieldV2::Y,
        _ => unreachable!("shape K1 polygon locations use coordinate fields"),
    }
}

const fn map_shape_kind(kind: GeometryK1ShapeKind) -> SpatialShapeErrorV2 {
    match kind {
        GeometryK1ShapeKind::NegativeExtent => SpatialShapeErrorV2::NegativeExtent,
        GeometryK1ShapeKind::NegativeRadius => SpatialShapeErrorV2::NegativeRadius,
        GeometryK1ShapeKind::PolygonTooShort => SpatialShapeErrorV2::PolygonTooShort,
        GeometryK1ShapeKind::PolygonRepeatedFirst => SpatialShapeErrorV2::PolygonRepeatedFirst,
        GeometryK1ShapeKind::PolygonAdjacentEqual => SpatialShapeErrorV2::PolygonAdjacentEqual,
    }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

use super::{
    DerivedLocalBoundsK3, FlattenedPathK2, GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field,
    GeometryK1LimitKind, GeometryK1Location, GeometryK1PathGrammarKind, GeometryK1ShapeKind,
    GeometryK1StrokeKind, GeometryK1StrokeSource, GeometryK2Error, GeometryK2ErrorKind,
    GeometryK2LimitKind, GeometryK3Error, GeometryK3ErrorKind, ValidatedCircleK1, ValidatedPathK1,
    ValidatedPolygonK1, ValidatedRectK1, ValidatedStrokeK1, clip_bounds_k3,
    derive_circle_bounds_k3, derive_path_bounds_k3, derive_polygon_bounds_k3,
    derive_rect_bounds_k3, fill_bounds_k3, flatten_path_k2, rect_stroke_bounds_k3,
    stroke_bounds_k3, validate_circle_k1, validate_path_k1, validate_polygon_k1, validate_rect_k1,
    validate_stroke_k1,
};

use crate::aabb::SpatialAabbV2;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::path::SpatialPathVerbV2;
use crate::vocabulary::SpatialAxisV2;

mod bounds;
mod flatten;
mod path;
mod scalar_order;
mod shape;
mod stroke;

const PATH_INDEX: u32 = 7;
const SHAPE_INDEX: u32 = 11;
const PATH_SUBPATH_MAXIMUM: usize =
    REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PathSubpathsTotal);
const POLYGON_POINT_MAXIMUM: usize =
    REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PolygonPointsPerShape);

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn move_to(x: i64, y: i64) -> SpatialPathVerbV2 {
    SpatialPathVerbV2::MoveTo { to: point(x, y) }
}

fn line_to(x: i64, y: i64) -> SpatialPathVerbV2 {
    SpatialPathVerbV2::LineTo { to: point(x, y) }
}

fn expect_valid<T>(result: Result<T, GeometryK1Error>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected K1 validation success, got {error:?}"),
    }
}

fn expect_error<T>(
    result: Result<T, GeometryK1Error>,
    kind: GeometryK1ErrorKind,
    location: GeometryK1Location,
) {
    let error = match result {
        Ok(_) => panic!("expected K1 validation failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn expect_limit<T>(
    result: Result<T, GeometryK1Error>,
    limit: GeometryK1LimitKind,
    location: GeometryK1Location,
    observed: usize,
    maximum: usize,
) {
    let error = match result {
        Ok(_) => panic!("expected K1 limit failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), GeometryK1ErrorKind::LimitExceeded(limit));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
}

fn path_location(verb: u32, field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::PathVerb {
        path: PATH_INDEX,
        verb,
        field,
    }
}

fn shape_location(field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::Shape {
        index: SHAPE_INDEX,
        field,
    }
}

fn polygon_location(point: u32, field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::PolygonPoint {
        shape: SHAPE_INDEX,
        point,
        field,
    }
}

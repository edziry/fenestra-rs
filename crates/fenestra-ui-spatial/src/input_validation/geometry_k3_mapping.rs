//! Geometry K3 failure mapping for aggregate local bounds.

use super::make_resolve_error;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;
use crate::geometry_kernel::{
    GeometryK1Field, GeometryK1Location, GeometryK3Error, GeometryK3ErrorKind,
};
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_geometry_k3_error(error: GeometryK3Error) -> SpatialResolveErrorV2 {
    let GeometryK3ErrorKind::LocalBoundsOutOfDomain(axis) = error.kind();
    make_resolve_error(
        SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::LocalBoundsOutOfDomain(axis)),
        map_location(error.location()),
    )
}

fn map_location(location: GeometryK1Location) -> SpatialErrorLocationV2 {
    match location {
        GeometryK1Location::Shape { index, field } => SpatialErrorLocationV2::Shape {
            index,
            field: match field {
                GeometryK1Field::RectWidth => SpatialShapeFieldV2::RectWidth,
                GeometryK1Field::RectHeight => SpatialShapeFieldV2::RectHeight,
                GeometryK1Field::CircleRadius => SpatialShapeFieldV2::CircleRadius,
                _ => unreachable!("K3 shape failures use derived extent fields"),
            },
        },
        GeometryK1Location::Paint {
            index,
            field: GeometryK1Field::StrokeWidth,
        } => SpatialErrorLocationV2::Paint {
            index,
            field: SpatialPaintFieldV2::StrokeWidth,
        },
        GeometryK1Location::Hit {
            index,
            field: GeometryK1Field::StrokeWidth,
        } => SpatialErrorLocationV2::Hit {
            index,
            field: SpatialHitFieldV2::StrokeWidth,
        },
        _ => unreachable!("K3 failures use shape or stroke-owner locations"),
    }
}

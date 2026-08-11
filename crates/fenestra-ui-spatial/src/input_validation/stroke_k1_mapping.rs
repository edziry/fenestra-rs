//! Mapping from Geometry K1 item-stroke failures into aggregate diagnostics.

use super::make_resolve_error;
use crate::content_diagnostic::SpatialStrokeErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_kernel::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1Location, GeometryK1StrokeKind,
};
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_stroke_k1_error(error: GeometryK1Error) -> SpatialResolveErrorV2 {
    let kind = match error.kind() {
        GeometryK1ErrorKind::ScalarOutOfDomain => SpatialContentErrorKindV2::ScalarOutOfDomain,
        GeometryK1ErrorKind::InvalidStroke(kind) => {
            SpatialContentErrorKindV2::InvalidStroke(map_stroke_kind(kind))
        }
        GeometryK1ErrorKind::InvalidPathGrammar(_)
        | GeometryK1ErrorKind::InvalidShape(_)
        | GeometryK1ErrorKind::LimitExceeded(_) => {
            unreachable!("stroke K1 cannot return path, shape, or limit failures")
        }
    };
    make_resolve_error(
        SpatialResolveErrorKindV2::Content(kind),
        map_stroke_location(error.location()),
    )
}

fn map_stroke_location(location: GeometryK1Location) -> SpatialErrorLocationV2 {
    match location {
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
        _ => unreachable!("stroke K1 uses an item StrokeWidth location"),
    }
}

const fn map_stroke_kind(kind: GeometryK1StrokeKind) -> SpatialStrokeErrorV2 {
    match kind {
        GeometryK1StrokeKind::NegativeWidth => SpatialStrokeErrorV2::NegativeWidth,
        GeometryK1StrokeKind::ZeroWidth => SpatialStrokeErrorV2::ZeroWidth,
    }
}

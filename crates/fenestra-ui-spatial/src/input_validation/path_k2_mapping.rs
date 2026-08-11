//! Geometry K2 failure mapping for aggregate path flattening.

use super::make_resolve_error;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathVerbFieldV2;
use crate::geometry_kernel::{
    GeometryK1Field, GeometryK1Location, GeometryK2Error, GeometryK2ErrorKind, GeometryK2LimitKind,
};
use crate::limits::SpatialLimitKindV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_path_k2_error(error: GeometryK2Error) -> SpatialResolveErrorV2 {
    let location = source_location(error.location());
    match error.kind() {
        GeometryK2ErrorKind::LimitExceeded(kind) => SpatialResolveErrorV2::limit_exceeded(
            limit_kind(kind),
            location,
            error
                .observed()
                .expect("K2 limit failures carry observed evidence"),
            error
                .maximum()
                .expect("K2 limit failures carry maximum evidence"),
        ),
        GeometryK2ErrorKind::NonFlatAtMaximumDepth => make_resolve_error(
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::NonFlatAtMaximumDepth),
            location,
        ),
    }
}

fn source_location(location: GeometryK1Location) -> SpatialErrorLocationV2 {
    let GeometryK1Location::PathVerb {
        path,
        verb,
        field: GeometryK1Field::Kind,
    } = location
    else {
        unreachable!("K2 failures use their authored source-verb location")
    };
    SpatialErrorLocationV2::PathVerb {
        path,
        verb,
        field: SpatialPathVerbFieldV2::Kind,
    }
}

const fn limit_kind(kind: GeometryK2LimitKind) -> SpatialLimitKindV2 {
    match kind {
        GeometryK2LimitKind::FlattenedSegmentsPerPath => {
            SpatialLimitKindV2::FlattenedSegmentsPerPath
        }
        GeometryK2LimitKind::FlattenedSegmentsTotal => SpatialLimitKindV2::FlattenedSegmentsTotal,
    }
}

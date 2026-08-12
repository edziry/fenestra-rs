//! Mapping from Paint P5 failures into aggregate diagnostics.

use super::make_resolve_error;
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::item_field::SpatialPaintFieldV2;
use crate::paint_kernel::{
    PaintP5Error, PaintP5ErrorKind, PaintP5Field, PaintP5ImageKind, PaintP5Location,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_paint_p5_error(error: PaintP5Error) -> SpatialResolveErrorV2 {
    let kind = match error.kind() {
        PaintP5ErrorKind::ScalarOutOfDomain => SpatialContentErrorKindV2::ScalarOutOfDomain,
        PaintP5ErrorKind::InvalidImage(kind) => {
            SpatialContentErrorKindV2::InvalidImage(map_image_kind(kind))
        }
        PaintP5ErrorKind::LocalBoundsOutOfDomain(axis) => {
            SpatialContentErrorKindV2::LocalBoundsOutOfDomain(axis)
        }
    };
    make_resolve_error(
        SpatialResolveErrorKindV2::Content(kind),
        map_paint_location(error.location()),
    )
}

fn map_paint_location(location: PaintP5Location) -> SpatialErrorLocationV2 {
    let PaintP5Location::Paint { index, field } = location;
    SpatialErrorLocationV2::Paint {
        index,
        field: map_paint_field(field),
    }
}

const fn map_paint_field(field: PaintP5Field) -> SpatialPaintFieldV2 {
    match field {
        PaintP5Field::SourceX => SpatialPaintFieldV2::SourceX,
        PaintP5Field::SourceY => SpatialPaintFieldV2::SourceY,
        PaintP5Field::SourceWidth => SpatialPaintFieldV2::SourceWidth,
        PaintP5Field::SourceHeight => SpatialPaintFieldV2::SourceHeight,
        PaintP5Field::DestinationX => SpatialPaintFieldV2::DestinationX,
        PaintP5Field::DestinationY => SpatialPaintFieldV2::DestinationY,
        PaintP5Field::DestinationWidth => SpatialPaintFieldV2::DestinationWidth,
        PaintP5Field::DestinationHeight => SpatialPaintFieldV2::DestinationHeight,
    }
}

const fn map_image_kind(kind: PaintP5ImageKind) -> SpatialImageErrorV2 {
    match kind {
        PaintP5ImageKind::EmptySource => SpatialImageErrorV2::EmptySource,
        PaintP5ImageKind::SourceOutOfBounds => SpatialImageErrorV2::SourceOutOfBounds,
        PaintP5ImageKind::NegativeDestinationExtent(extent) => {
            SpatialImageErrorV2::NegativeDestinationExtent(extent)
        }
        PaintP5ImageKind::EmptyDestination => SpatialImageErrorV2::EmptyDestination,
    }
}

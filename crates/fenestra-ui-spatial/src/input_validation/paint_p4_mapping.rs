//! Mapping from Paint P4 failures into aggregate diagnostics.

use super::make_resolve_error;
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialColorChannelV2, SpatialImageFieldV2};
use crate::limits::SpatialLimitKindV2;
use crate::paint_kernel::{
    PaintP4Channel, PaintP4Error, PaintP4ErrorKind, PaintP4Field, PaintP4ImageKind,
    PaintP4LimitKind, PaintP4Location,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_image_p4_error(error: PaintP4Error) -> SpatialResolveErrorV2 {
    match error.kind() {
        PaintP4ErrorKind::LimitExceeded(kind) => SpatialResolveErrorV2::limit_exceeded(
            map_limit_kind(kind),
            map_location(error.location()),
            error
                .observed()
                .expect("P4 limit failures carry observed evidence"),
            error
                .maximum()
                .expect("P4 limit failures carry maximum evidence"),
        ),
        PaintP4ErrorKind::InvalidImage(kind) => content_error(
            SpatialContentErrorKindV2::InvalidImage(map_image_kind(kind)),
            map_location(error.location()),
        ),
    }
}

const fn map_limit_kind(kind: PaintP4LimitKind) -> SpatialLimitKindV2 {
    match kind {
        PaintP4LimitKind::ImageEdge => SpatialLimitKindV2::ImageEdge,
        PaintP4LimitKind::ImagePixelsTotal => SpatialLimitKindV2::ImagePixelsTotal,
    }
}

fn map_location(location: PaintP4Location) -> SpatialErrorLocationV2 {
    match location {
        PaintP4Location::Image { index, field } => SpatialErrorLocationV2::Image {
            index,
            field: map_image_field(field),
        },
        PaintP4Location::ImagePixel {
            image,
            pixel,
            channel,
        } => SpatialErrorLocationV2::ImagePixel {
            image,
            pixel,
            channel: map_channel(channel),
        },
    }
}

const fn map_image_field(field: PaintP4Field) -> SpatialImageFieldV2 {
    match field {
        PaintP4Field::Width => SpatialImageFieldV2::Width,
        PaintP4Field::Height => SpatialImageFieldV2::Height,
        PaintP4Field::Stride => SpatialImageFieldV2::Stride,
        PaintP4Field::ByteLength => SpatialImageFieldV2::ByteLength,
        PaintP4Field::Pixel => SpatialImageFieldV2::Pixel,
    }
}

const fn map_channel(channel: PaintP4Channel) -> SpatialColorChannelV2 {
    match channel {
        PaintP4Channel::R => SpatialColorChannelV2::R,
        PaintP4Channel::G => SpatialColorChannelV2::G,
        PaintP4Channel::B => SpatialColorChannelV2::B,
    }
}

const fn map_image_kind(kind: PaintP4ImageKind) -> SpatialImageErrorV2 {
    match kind {
        PaintP4ImageKind::ZeroExtent => SpatialImageErrorV2::ZeroExtent,
        PaintP4ImageKind::StrideMismatch => SpatialImageErrorV2::StrideMismatch,
        PaintP4ImageKind::LengthMismatch => SpatialImageErrorV2::LengthMismatch,
        PaintP4ImageKind::InvalidPremultipliedPixel => {
            SpatialImageErrorV2::InvalidPremultipliedPixel
        }
    }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

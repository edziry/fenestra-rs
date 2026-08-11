//! Mapping from Paint P2 failures into aggregate diagnostics.

use super::make_resolve_error;
use crate::content_diagnostic::SpatialGradientErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialBrushFieldV2, SpatialGradientStopFieldV2};
use crate::limits::SpatialLimitKindV2;
use crate::paint_kernel::{
    PaintP2Error, PaintP2ErrorKind, PaintP2Field, PaintP2GradientKind, PaintP2LimitKind,
    PaintP2Location,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn map_paint_p2_error(error: PaintP2Error) -> SpatialResolveErrorV2 {
    match error.kind() {
        PaintP2ErrorKind::LimitExceeded(PaintP2LimitKind::GradientStopsPerBrush) => {
            SpatialResolveErrorV2::limit_exceeded(
                SpatialLimitKindV2::GradientStopsPerBrush,
                map_location(error.location()),
                error
                    .observed()
                    .expect("P2 limit failures carry observed evidence") as u128,
                error
                    .maximum()
                    .expect("P2 limit failures carry maximum evidence") as u128,
            )
        }
        PaintP2ErrorKind::ScalarOutOfDomain => content_error(
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            map_location(error.location()),
        ),
        PaintP2ErrorKind::InvalidGradient(kind) => content_error(
            SpatialContentErrorKindV2::InvalidGradient(map_gradient_kind(kind)),
            map_location(error.location()),
        ),
    }
}

fn map_location(location: PaintP2Location) -> SpatialErrorLocationV2 {
    match location {
        PaintP2Location::Brush { index, field } => SpatialErrorLocationV2::Brush {
            index,
            field: map_brush_field(field),
        },
        PaintP2Location::GradientStop { brush, stop, field } => {
            SpatialErrorLocationV2::GradientStop {
                brush,
                stop,
                field: map_stop_field(field),
            }
        }
    }
}

fn map_brush_field(field: PaintP2Field) -> SpatialBrushFieldV2 {
    match field {
        PaintP2Field::GradientStopLength => SpatialBrushFieldV2::GradientStopLength,
        PaintP2Field::GradientStartX => SpatialBrushFieldV2::GradientStartX,
        PaintP2Field::GradientStartY => SpatialBrushFieldV2::GradientStartY,
        PaintP2Field::GradientEndX => SpatialBrushFieldV2::GradientEndX,
        PaintP2Field::GradientEndY => SpatialBrushFieldV2::GradientEndY,
        PaintP2Field::Offset => unreachable!("P2 brush locations do not use stop fields"),
    }
}

fn map_stop_field(field: PaintP2Field) -> SpatialGradientStopFieldV2 {
    match field {
        PaintP2Field::Offset => SpatialGradientStopFieldV2::Offset,
        _ => unreachable!("P2 gradient-stop locations use the offset field"),
    }
}

const fn map_gradient_kind(kind: PaintP2GradientKind) -> SpatialGradientErrorV2 {
    match kind {
        PaintP2GradientKind::CoincidentEndpoints => SpatialGradientErrorV2::CoincidentEndpoints,
        PaintP2GradientKind::TooFewStops => SpatialGradientErrorV2::TooFewStops,
        PaintP2GradientKind::FirstOffset => SpatialGradientErrorV2::FirstOffset,
        PaintP2GradientKind::LastOffset => SpatialGradientErrorV2::LastOffset,
        PaintP2GradientKind::DecreasingOffset => SpatialGradientErrorV2::DecreasingOffset,
    }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

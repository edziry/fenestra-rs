use super::error::{
    PaintP2Error, PaintP2ErrorKind, PaintP2Field, PaintP2GradientKind, PaintP2Location,
};
use super::model::{PreparedGradientP2, PreparedGradientStopP2};
use super::normalize_straight_p1;

use crate::brush::{SpatialGradientStopV2, SpatialRgba8V2};
use crate::model::{SpatialPointV2, SpatialScalarV2};

pub(crate) fn prepare_solid_p2(color: SpatialRgba8V2) -> SpatialRgba8V2 {
    normalize_straight_p1(color)
}

pub(crate) fn prepare_gradient_p2(
    brush: u32,
    _raw_stop_start: u32,
    raw_stop_length: u32,
    start: SpatialPointV2,
    end: SpatialPointV2,
    stops: &[SpatialGradientStopV2],
    maximum_stops: usize,
) -> Result<PreparedGradientP2, PaintP2Error> {
    let stop_count = raw_stop_length as usize;
    if stop_count > maximum_stops {
        return Err(PaintP2Error::limit(
            brush_location(brush, PaintP2Field::GradientStopLength),
            stop_count,
            maximum_stops,
        ));
    }

    validate_scalar(brush, PaintP2Field::GradientStartX, start.x())?;
    validate_scalar(brush, PaintP2Field::GradientStartY, start.y())?;
    validate_scalar(brush, PaintP2Field::GradientEndX, end.x())?;
    validate_scalar(brush, PaintP2Field::GradientEndY, end.y())?;

    if start == end {
        return Err(gradient_error(
            PaintP2GradientKind::CoincidentEndpoints,
            brush_location(brush, PaintP2Field::GradientEndX),
        ));
    }
    if stop_count < 2 {
        return Err(gradient_error(
            PaintP2GradientKind::TooFewStops,
            brush_location(brush, PaintP2Field::GradientStopLength),
        ));
    }
    if stops[0].offset() != 0 {
        return Err(gradient_error(
            PaintP2GradientKind::FirstOffset,
            stop_location(brush, 0),
        ));
    }

    let last = stop_count - 1;
    if stops[last].offset() != u16::MAX {
        return Err(gradient_error(
            PaintP2GradientKind::LastOffset,
            stop_location(brush, last as u32),
        ));
    }
    for index in 1..stop_count {
        if stops[index].offset() < stops[index - 1].offset() {
            return Err(gradient_error(
                PaintP2GradientKind::DecreasingOffset,
                stop_location(brush, index as u32),
            ));
        }
    }

    let prepared_stops = stops
        .iter()
        .map(|stop| PreparedGradientStopP2::new(stop.offset(), normalize_straight_p1(stop.color())))
        .collect();
    Ok(PreparedGradientP2::new(start, end, prepared_stops))
}

fn validate_scalar(
    brush: u32,
    field: PaintP2Field,
    scalar: SpatialScalarV2,
) -> Result<(), PaintP2Error> {
    if scalar.is_in_domain() {
        Ok(())
    } else {
        Err(PaintP2Error::new(
            PaintP2ErrorKind::ScalarOutOfDomain,
            brush_location(brush, field),
        ))
    }
}

const fn gradient_error(kind: PaintP2GradientKind, location: PaintP2Location) -> PaintP2Error {
    PaintP2Error::new(PaintP2ErrorKind::InvalidGradient(kind), location)
}

const fn brush_location(brush: u32, field: PaintP2Field) -> PaintP2Location {
    PaintP2Location::Brush {
        index: brush,
        field,
    }
}

const fn stop_location(brush: u32, stop: u32) -> PaintP2Location {
    PaintP2Location::GradientStop {
        brush,
        stop,
        field: PaintP2Field::Offset,
    }
}

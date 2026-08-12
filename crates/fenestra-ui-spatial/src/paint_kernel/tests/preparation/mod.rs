use super::*;

mod limits;
mod priority;
mod solid;
mod stops;
mod success;

fn brush_location(field: PaintP2Field) -> PaintP2Location {
    PaintP2Location::Brush {
        index: BRUSH_INDEX,
        field,
    }
}

fn stop_location(stop: u32) -> PaintP2Location {
    PaintP2Location::GradientStop {
        brush: BRUSH_INDEX,
        stop,
        field: PaintP2Field::Offset,
    }
}

fn valid_stops() -> [SpatialGradientStopV2; 2] {
    [
        stop(0, color(255, 0, 0, 255)),
        stop(65_535, color(0, 0, 255, 255)),
    ]
}

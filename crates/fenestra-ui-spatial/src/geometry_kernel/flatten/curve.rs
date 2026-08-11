use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::numeric::{round_ratio_v2, scalar_from_i128};

use super::{SegmentEmitter, error::GeometryK2Error};

const FLATNESS_TOLERANCE: i128 = 256;
const MAXIMUM_DEPTH: u8 = 16;

#[derive(Clone, Copy)]
struct CurveSource {
    path: u32,
    verb: u32,
}

#[derive(Clone, Copy)]
struct Quadratic {
    start: SpatialPointV2,
    control: SpatialPointV2,
    end: SpatialPointV2,
}

#[derive(Clone, Copy)]
struct Cubic {
    start: SpatialPointV2,
    control1: SpatialPointV2,
    control2: SpatialPointV2,
    end: SpatialPointV2,
}

pub(super) fn flatten_quadratic(
    path: u32,
    source_verb: u32,
    start: SpatialPointV2,
    control: SpatialPointV2,
    end: SpatialPointV2,
    emitter: &mut SegmentEmitter,
) -> Result<(), GeometryK2Error> {
    recurse_quadratic(
        CurveSource {
            path,
            verb: source_verb,
        },
        Quadratic {
            start,
            control,
            end,
        },
        0,
        emitter,
    )
}

pub(super) fn flatten_cubic(
    path: u32,
    source_verb: u32,
    start: SpatialPointV2,
    control1: SpatialPointV2,
    control2: SpatialPointV2,
    end: SpatialPointV2,
    emitter: &mut SegmentEmitter,
) -> Result<(), GeometryK2Error> {
    recurse_cubic(
        CurveSource {
            path,
            verb: source_verb,
        },
        Cubic {
            start,
            control1,
            control2,
            end,
        },
        0,
        emitter,
    )
}

fn recurse_quadratic(
    source: CurveSource,
    curve: Quadratic,
    depth: u8,
    emitter: &mut SegmentEmitter,
) -> Result<(), GeometryK2Error> {
    if curve_is_flat(curve.start, curve.end, &[curve.control]) {
        return emitter.emit(curve.end, source.path, source.verb);
    }
    if depth == MAXIMUM_DEPTH {
        return Err(GeometryK2Error::nonflat(source.path, source.verb));
    }

    let start_control = midpoint(curve.start, curve.control);
    let control_end = midpoint(curve.control, curve.end);
    let split = midpoint(start_control, control_end);
    let next_depth = depth + 1;
    recurse_quadratic(
        source,
        Quadratic {
            start: curve.start,
            control: start_control,
            end: split,
        },
        next_depth,
        emitter,
    )?;
    recurse_quadratic(
        source,
        Quadratic {
            start: split,
            control: control_end,
            end: curve.end,
        },
        next_depth,
        emitter,
    )
}

fn recurse_cubic(
    source: CurveSource,
    curve: Cubic,
    depth: u8,
    emitter: &mut SegmentEmitter,
) -> Result<(), GeometryK2Error> {
    if curve_is_flat(curve.start, curve.end, &[curve.control1, curve.control2]) {
        return emitter.emit(curve.end, source.path, source.verb);
    }
    if depth == MAXIMUM_DEPTH {
        return Err(GeometryK2Error::nonflat(source.path, source.verb));
    }

    let start_control1 = midpoint(curve.start, curve.control1);
    let control1_control2 = midpoint(curve.control1, curve.control2);
    let control2_end = midpoint(curve.control2, curve.end);
    let left_control2 = midpoint(start_control1, control1_control2);
    let right_control1 = midpoint(control1_control2, control2_end);
    let split = midpoint(left_control2, right_control1);
    let next_depth = depth + 1;
    recurse_cubic(
        source,
        Cubic {
            start: curve.start,
            control1: start_control1,
            control2: left_control2,
            end: split,
        },
        next_depth,
        emitter,
    )?;
    recurse_cubic(
        source,
        Cubic {
            start: split,
            control1: right_control1,
            control2: control2_end,
            end: curve.end,
        },
        next_depth,
        emitter,
    )
}

fn curve_is_flat(start: SpatialPointV2, end: SpatialPointV2, controls: &[SpatialPointV2]) -> bool {
    controls
        .iter()
        .copied()
        .all(|control| control_is_flat(start, end, control))
}

fn control_is_flat(start: SpatialPointV2, end: SpatialPointV2, control: SpatialPointV2) -> bool {
    let chord_x = i128::from(end.x().raw()) - i128::from(start.x().raw());
    let chord_y = i128::from(end.y().raw()) - i128::from(start.y().raw());
    let control_x = i128::from(control.x().raw()) - i128::from(start.x().raw());
    let control_y = i128::from(control.y().raw()) - i128::from(start.y().raw());
    let cross = (control_x * chord_y - control_y * chord_x).abs();
    let chord_extent = chord_x.abs().max(chord_y.abs());

    cross <= FLATNESS_TOLERANCE * chord_extent
        && coordinate_in_expanded_range(control.x(), start.x(), end.x())
        && coordinate_in_expanded_range(control.y(), start.y(), end.y())
}

fn coordinate_in_expanded_range(
    control: SpatialScalarV2,
    start: SpatialScalarV2,
    end: SpatialScalarV2,
) -> bool {
    let control = i128::from(control.raw());
    let start = i128::from(start.raw());
    let end = i128::from(end.raw());
    let minimum = start.min(end) - FLATNESS_TOLERANCE;
    let maximum = start.max(end) + FLATNESS_TOLERANCE;
    control >= minimum && control <= maximum
}

fn midpoint(left: SpatialPointV2, right: SpatialPointV2) -> SpatialPointV2 {
    SpatialPointV2::new(
        midpoint_scalar(left.x(), right.x()),
        midpoint_scalar(left.y(), right.y()),
    )
}

fn midpoint_scalar(left: SpatialScalarV2, right: SpatialScalarV2) -> SpatialScalarV2 {
    let sum = i128::from(left.raw()) + i128::from(right.raw());
    let rounded = round_ratio_v2(sum, 2);
    match rounded.and_then(scalar_from_i128) {
        Some(midpoint) => midpoint,
        None => unreachable!("canonical scalar midpoint remains canonical"),
    }
}

use super::{
    PaintP2Error, PaintP2ErrorKind, PaintP2Field, PaintP2GradientKind, PaintP2LimitKind,
    PaintP2Location, PaintP4Channel, PaintP4Error, PaintP4ErrorKind, PaintP4Field,
    PaintP4ImageKind, PaintP4LimitKind, PaintP4Location, PaintP5Error, PaintP5ErrorKind,
    PaintP5Field, PaintP5ImageKind, PaintP5Location, apply_opacity_p1,
    finish_image_paint_bounds_after_item_phase_p5, normalize_straight_p1, prepare_gradient_p2,
    prepare_image_p4, prepare_image_paint_p5, prepare_solid_p2, sample_gradient_p3,
    sample_image_p6, source_over_p1, test_p4_pixel_error,
};

use crate::aabb::SpatialAabbV2;
use crate::brush::{SpatialGradientStopV2, SpatialRgba8V2};
use crate::content_key::SpatialImageKeyV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2, SpatialImageV2};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2};

mod image_paint;
mod image_validation;
mod invariants;
mod normalize;
mod opacity;
mod preparation;
mod sampling;
mod source_over;

const BRUSH_INDEX: u32 = 7;
const STOP_START: u32 = 41;

fn color(r: u8, g: u8, b: u8, a: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(r, g, b, a)
}

fn assert_color(actual: SpatialRgba8V2, expected: [u8; 4]) {
    assert_eq!([actual.r(), actual.g(), actual.b(), actual.a()], expected);
}

fn reference_scale(channel: u8, factor: u8) -> u8 {
    ((u16::from(channel) * u16::from(factor) + 127) / 255) as u8
}

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn stop(offset: u16, color: SpatialRgba8V2) -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(offset, color)
}

fn expect_p2_error<T>(
    result: Result<T, PaintP2Error>,
    kind: PaintP2ErrorKind,
    location: PaintP2Location,
) {
    let error = match result {
        Ok(_) => panic!("expected P2 preparation failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn expect_p2_limit<T>(
    result: Result<T, PaintP2Error>,
    location: PaintP2Location,
    observed: usize,
    maximum: usize,
) {
    let error = match result {
        Ok(_) => panic!("expected P2 limit failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        PaintP2ErrorKind::LimitExceeded(PaintP2LimitKind::GradientStopsPerBrush)
    );
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
}

use std::error::Error;

pub(super) use super::brush_structure_support::{
    expect_content, expect_invalid_range, expect_non_dense, fixture, fixture_with_shapes, gradient,
    gradient_values, outside_high, point, rect_values, solid, stop,
};
use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use crate::brush::{SpatialBrushContentV2, SpatialBrushV2, SpatialGradientStopV2, SpatialRgba8V2};
use crate::content_diagnostic::SpatialGradientErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_key::SpatialBrushKeyV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialBrushFieldV2, SpatialGradientStopFieldV2};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) const fn color(r: u8, g: u8, b: u8, a: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(r, g, b, a)
}

pub(super) const fn solid_color(key: u32, color: SpatialRgba8V2) -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(key),
        SpatialBrushContentV2::Solid { color },
    )
}

pub(super) const fn stop_color(offset: u16, color: SpatialRgba8V2) -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(offset, color)
}

pub(super) fn valid_stops() -> Vec<SpatialGradientStopV2> {
    vec![
        stop_color(0, color(255, 0, 0, 255)),
        stop_color(u16::MAX, color(0, 0, 255, 255)),
    ]
}

pub(super) fn ordered_stops(count: usize) -> Vec<SpatialGradientStopV2> {
    assert!(count >= 2);
    (0..count)
        .map(|index| {
            let offset = (index * usize::from(u16::MAX) / (count - 1)) as u16;
            stop_color(offset, color(255, 255, 255, 255))
        })
        .collect()
}

pub(super) fn limits(maximum_stops: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::GradientStopsPerBrush {
            *value = maximum_stops;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) const fn registered_limits() -> SpatialLimitsV2 {
    REGISTERED_SPATIAL_LIMITS_V2
}

pub(super) fn later_poison_limits(maximum_stops: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::GradientStopsPerBrush => *value = maximum_stops,
            SpatialLimitKindV2::ImageEdge
            | SpatialLimitKindV2::ImagePixelsTotal
            | SpatialLimitKindV2::ClipDepth
            | SpatialLimitKindV2::PaintItemsPerNode
            | SpatialLimitKindV2::HitItemsPerNode
            | SpatialLimitKindV2::FlattenedSegmentsPerPath
            | SpatialLimitKindV2::FlattenedSegmentsTotal => *value = 0,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_prepared_brushes!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected prepared-brush success, got {error:?}"),
    }
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    brush: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected prepared-brush limit failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::GradientStopsPerBrush)
    );
    assert_eq!(
        error.location(),
        brush_location(brush, SpatialBrushFieldV2::GradientStopLength)
    );
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_scalar<T>(
    result: Result<T, SpatialResolveErrorV2>,
    brush: u32,
    field: SpatialBrushFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        brush_location(brush, field),
    );
}

pub(super) fn expect_gradient<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialGradientErrorV2,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidGradient(kind),
        location,
    );
}

pub(super) const fn brush_location(
    index: u32,
    field: SpatialBrushFieldV2,
) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Brush { index, field }
}

pub(super) const fn stop_location(brush: u32, stop: u32) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::GradientStop {
        brush,
        stop,
        field: SpatialGradientStopFieldV2::Offset,
    }
}

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
pub(super) use super::validated_shape_support::{expect_content, outside_high, point, rect_values};
use crate::brush::{SpatialBrushContentV2, SpatialBrushV2, SpatialGradientStopV2, SpatialRgba8V2};
use crate::content_diagnostic::{SpatialKeyedContentTableV2, SpatialPayloadTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_key::SpatialBrushKeyV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialPointV2;
use crate::resolve_error::SpatialResolveErrorV2;
use crate::shape::SpatialShapeV2;

pub(super) fn fixture(
    brushes: Vec<SpatialBrushV2>,
    stops: Vec<SpatialGradientStopV2>,
) -> RawInputFixture {
    fixture_with_shapes(Vec::new(), Vec::new(), brushes, stops)
}

pub(super) fn fixture_with_shapes(
    shapes: Vec<SpatialShapeV2>,
    points: Vec<SpatialPointV2>,
    brushes: Vec<SpatialBrushV2>,
    stops: Vec<SpatialGradientStopV2>,
) -> RawInputFixture {
    super::validated_shape_support::fixture(shapes, points).with_brushes(brushes, stops)
}

pub(super) const fn solid(key: u32) -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(key),
        SpatialBrushContentV2::Solid {
            color: color(key as u8),
        },
    )
}

pub(super) const fn gradient(key: u32, start: u32, length: u32) -> SpatialBrushV2 {
    gradient_values(key, start, length, point(0, 0), point(1, 1))
}

pub(super) const fn gradient_values(
    key: u32,
    stop_start: u32,
    stop_length: u32,
    start: SpatialPointV2,
    end: SpatialPointV2,
) -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(key),
        SpatialBrushContentV2::LinearGradient {
            stop_start,
            stop_length,
            start,
            end,
        },
    )
}

pub(super) const fn stop(offset: u16) -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(offset, color(offset as u8))
}

const fn color(value: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(value, value.wrapping_add(1), value.wrapping_add(2), 255)
}

pub(super) fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; SpatialLimitKindV2::ALL.len()])
}

pub(super) fn deferred_p2_limits() -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::GradientStopsPerBrush {
            *value = 0;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_brush_structure!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected brush-structure success, got {error:?}"),
    }
}

pub(super) fn expect_non_dense<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Brush),
        location,
    );
}

pub(super) fn expect_invalid_range<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::GradientStop),
        location,
    );
}

use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use crate::content_diagnostic::{SpatialImageErrorV2, SpatialKeyedContentTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_key::SpatialImageKeyV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::image::SpatialImageV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(images: Vec<SpatialImageV2>) -> RawInputFixture {
    super::prepared_brush_support::fixture(Vec::new(), Vec::new()).with_images(images)
}

pub(super) fn image(
    key: u32,
    width: u32,
    height: u32,
    stride: u32,
    bytes: Vec<u8>,
) -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(key),
        width,
        height,
        stride,
        bytes.into_boxed_slice(),
    )
}

pub(super) fn blank_image(key: u32, width: u32, height: u32) -> SpatialImageV2 {
    let stride = width.checked_mul(4).expect("test image stride fits u32");
    let length =
        usize::try_from(u64::from(stride) * u64::from(height)).expect("test image bytes fit usize");
    image(key, width, height, stride, vec![0; length])
}

pub(super) fn limits(maximum_edge: usize, maximum_pixels: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::ImageEdge => *value = maximum_edge,
            SpatialLimitKindV2::ImagePixelsTotal => *value = maximum_pixels,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_limits(maximum_edge: usize, maximum_pixels: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::ImageEdge => *value = maximum_edge,
            SpatialLimitKindV2::ImagePixelsTotal => *value = maximum_pixels,
            SpatialLimitKindV2::ClipDepth
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
    prepare_validated_images!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-image success, got {error:?}"),
    }
}

pub(super) fn expect_non_dense<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Image),
        location,
    );
}

pub(super) fn expect_image<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialImageErrorV2,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidImage(kind),
        location,
    );
}

pub(super) fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-image content failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Content(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(content)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(content))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    location: SpatialErrorLocationV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-image limit failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::LimitExceeded(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) const fn image_location(
    index: u32,
    field: SpatialImageFieldV2,
) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Image { index, field }
}

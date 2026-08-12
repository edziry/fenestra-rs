use std::error::Error;

use super::raster_support::*;
use super::*;
use crate::error::SpatialErrorLocationV2;

#[test]
fn pixel_limits_are_closed_indexed_and_registered_exactly() {
    for maximum in [0, 1, 17, usize::MAX] {
        let value = limits(maximum);
        assert_eq!(value.limit(ReferenceRasterLimitKindV2::Pixels), maximum);
    }
    assert_eq!(ReferenceRasterLimitKindV2::ALL.len(), 1);
    assert_eq!(
        REGISTERED_REFERENCE_RASTER_LIMITS_V2.limit(ReferenceRasterLimitKindV2::Pixels),
        4_194_304
    );
    assert_eq!(4_194_304_u64 * 4, 16_777_216);
}

#[test]
fn caller_pixel_limit_accepts_equality_and_rejects_one_over_before_painting() {
    let source = empty_owned(viewport(2, 2));
    let raster = snapshot(source.clone())
        .rasterize_reference(limits(4))
        .expect("limit equality passes");
    assert_raster(&raster, 2, 2, &[0; 16]);

    let error = expect_raster_error(snapshot(source).rasterize_reference(limits(3)));
    assert_limit_error(error, 4, 3);
}

#[test]
fn allocation_ceiling_is_folded_into_the_single_pixel_limit() {
    let width = i32::MAX;
    let height = i32::MAX;
    let observed = (width as u128) * (height as u128);
    let effective = (isize::MAX as usize / 4) as u128;
    assert!(observed > effective);

    let error = expect_raster_error(
        snapshot(empty_owned(viewport(width, height))).rasterize_reference(limits(usize::MAX)),
    );
    assert_limit_error(error, observed, effective);
}

#[test]
fn zero_area_viewports_keep_exact_metadata_without_allocating_pixels() {
    let wide = snapshot(empty_owned(viewport(i32::MAX, 0)))
        .rasterize_reference(limits(0))
        .expect("zero-height viewport");
    assert_eq!(wide.width(), i32::MAX as u32);
    assert_eq!(wide.height(), 0);
    assert_eq!(wide.stride(), u64::from(i32::MAX as u32) * 4);
    assert!(wide.bytes().is_empty());

    let tall = snapshot(empty_owned(viewport(0, i32::MAX)))
        .rasterize_reference(limits(0))
        .expect("zero-width viewport");
    assert_eq!(tall.width(), 0);
    assert_eq!(tall.height(), i32::MAX as u32);
    assert_eq!(tall.stride(), 0);
    assert!(tall.bytes().is_empty());
}

fn assert_limit_error(error: ReferenceRasterErrorV2, observed: u128, maximum: u128) {
    assert_eq!(
        error.kind(),
        ReferenceRasterErrorKindV2::LimitExceeded(ReferenceRasterLimitKindV2::Pixels)
    );
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "reference-raster-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "ReferenceRasterErrorV2(reference-raster-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

fn expect_raster_error(
    result: Result<ReferenceRasterV2, ReferenceRasterErrorV2>,
) -> ReferenceRasterErrorV2 {
    match result {
        Ok(_) => panic!("expected reference raster limit error"),
        Err(error) => error,
    }
}

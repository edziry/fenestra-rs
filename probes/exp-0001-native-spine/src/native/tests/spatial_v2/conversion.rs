use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use super::super::super::spatial_v2::{
    SpatialPhysicalExtentV2, SpatialPresentErrorKindV2, SpatialPresentationLimitKindV2,
    SpatialPresentationLimitsV2, SpatialPresentationOutcomeV2, SpatialRasterInputV2,
    present_spatial_offer_v2, stage_reference_pixels_with_reserver_v2,
};
use super::fixture::{
    LOGICAL_PACKED, LOGICAL_VIEWPORT, REFERENCE_RGBA, offer_at, spatial_scheduler,
};
use super::support::{BackendFault, limits, reference_presenter, surface};

#[test]
fn center_nearest_conversion_is_exact_at_one_one_point_two_five_and_two() {
    let cases: &[(u32, u32, &[u32])] = &[
        (4, 1, &LOGICAL_PACKED),
        (
            5,
            1,
            &[
                LOGICAL_PACKED[0],
                LOGICAL_PACKED[1],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[3],
            ],
        ),
        (
            8,
            2,
            &[
                LOGICAL_PACKED[0],
                LOGICAL_PACKED[0],
                LOGICAL_PACKED[1],
                LOGICAL_PACKED[1],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[3],
                LOGICAL_PACKED[3],
                LOGICAL_PACKED[0],
                LOGICAL_PACKED[0],
                LOGICAL_PACKED[1],
                LOGICAL_PACKED[1],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[2],
                LOGICAL_PACKED[3],
                LOGICAL_PACKED[3],
            ],
        ),
    ];

    for &(width, height, expected) in cases {
        let mut scheduler = spatial_scheduler();
        let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
        let (mut presenter, state) = reference_presenter(BackendFault::None, limits());
        let outcome = present_spatial_offer_v2(
            &mut scheduler,
            &work,
            surface(width, height, LOGICAL_VIEWPORT),
            &mut presenter,
            SchedulerTick::new(12),
        )
        .expect("registered conversion should present");
        assert!(matches!(
            outcome,
            SpatialPresentationOutcomeV2::Completed(_)
        ));
        assert_eq!(state.borrow().pixels, expected, "{width}x{height}");
    }
}

#[test]
fn physical_zero_suspends_without_reserving_or_forming_pixels() {
    let reserve_calls = Cell::new(0);
    let staged = stage_reference_pixels_with_reserver_v2(
        LOGICAL_VIEWPORT,
        SpatialPhysicalExtentV2::new(0, 1),
        SpatialRasterInputV2::new(4, 1, 16, &REFERENCE_RGBA),
        limits(),
        |count| {
            reserve_calls.set(reserve_calls.get() + 1);
            Ok(Vec::with_capacity(count))
        },
    )
    .expect("physical zero is suspension, not a staging failure");

    assert!(staged.is_none());
    assert_eq!(reserve_calls.get(), 0);
}

#[test]
fn nonzero_physical_surface_rejects_zero_logical_and_inconsistent_metadata() {
    let physical = SpatialPhysicalExtentV2::new(1, 1);
    let zero = stage_error(
        SpatialViewportV2::new(0, 0),
        physical,
        SpatialRasterInputV2::new(0, 0, 0, &[]),
        limits(),
    );
    assert_eq!(zero, SpatialPresentErrorKindV2::ZeroLogicalRaster);

    for raster in [
        SpatialRasterInputV2::new(3, 1, 12, &REFERENCE_RGBA),
        SpatialRasterInputV2::new(4, 2, 16, &REFERENCE_RGBA),
        SpatialRasterInputV2::new(4, 1, 15, &REFERENCE_RGBA),
        SpatialRasterInputV2::new(4, 1, 16, &REFERENCE_RGBA[..15]),
    ] {
        assert_eq!(
            stage_error(LOGICAL_VIEWPORT, physical, raster, limits()),
            SpatialPresentErrorKindV2::RasterMetadata
        );
    }
}

#[test]
fn staging_limits_and_reservation_fail_before_pixel_materialization() {
    let raster = SpatialRasterInputV2::new(4, 1, 16, &REFERENCE_RGBA);
    let cases = [
        (
            SpatialPresentationLimitsV2::new(3, 8, 2, 16, 64),
            SpatialPhysicalExtentV2::new(4, 1),
            SpatialPresentationLimitKindV2::ReferencePixels,
        ),
        (
            SpatialPresentationLimitsV2::new(4, 3, 2, 16, 64),
            SpatialPhysicalExtentV2::new(4, 1),
            SpatialPresentationLimitKindV2::PhysicalWidth,
        ),
        (
            SpatialPresentationLimitsV2::new(4, 8, 1, 16, 64),
            SpatialPhysicalExtentV2::new(4, 2),
            SpatialPresentationLimitKindV2::PhysicalHeight,
        ),
        (
            SpatialPresentationLimitsV2::new(4, 8, 2, 7, 64),
            SpatialPhysicalExtentV2::new(4, 2),
            SpatialPresentationLimitKindV2::PhysicalPixels,
        ),
        (
            SpatialPresentationLimitsV2::new(4, 8, 2, 16, 31),
            SpatialPhysicalExtentV2::new(4, 2),
            SpatialPresentationLimitKindV2::PhysicalBytes,
        ),
    ];
    for (limits, physical, expected) in cases {
        assert_eq!(
            stage_error(LOGICAL_VIEWPORT, physical, raster, limits),
            SpatialPresentErrorKindV2::LimitExceeded(expected)
        );
    }

    let error = match stage_reference_pixels_with_reserver_v2(
        LOGICAL_VIEWPORT,
        SpatialPhysicalExtentV2::new(4, 1),
        raster,
        limits(),
        |_| Err(()),
    ) {
        Ok(_) => panic!("reservation failure should remain typed"),
        Err(error) => error,
    };
    assert_eq!(error, SpatialPresentErrorKindV2::Allocation);
}

fn stage_error(
    logical: SpatialViewportV2,
    physical: SpatialPhysicalExtentV2,
    raster: SpatialRasterInputV2<'_>,
    limits: SpatialPresentationLimitsV2,
) -> SpatialPresentErrorKindV2 {
    match stage_reference_pixels_with_reserver_v2(logical, physical, raster, limits, |count| {
        Ok(Vec::with_capacity(count))
    }) {
        Ok(_) => panic!("fixture should fail before staging"),
        Err(error) => error,
    }
}

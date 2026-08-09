use fenestra_ui_runtime::prototype::HeadlessRect;

use super::super::{
    NativeContractErrorKindV1, NativeFrameLimitsV1, NativeLimitKindV1, NativePhysicalExtentV1,
    NativeScaleFactorV1, NativeSceneRectangleV1, build_cpu_frame_v1,
    build_cpu_frame_with_reserver_v1,
};
use super::generation_zero;

const LIMITS: NativeFrameLimitsV1 = NativeFrameLimitsV1::new(8, 8, 64, 256);

#[test]
fn raster_clears_clips_and_overwrites_in_authored_order() {
    let scene = [
        rectangle(-1, 0, 3, 2, [1, 2, 3, 255]),
        rectangle(1, 1, 2, 2, [4, 5, 6, 255]),
    ];
    let frame = build_cpu_frame_v1(
        generation_zero(),
        7,
        NativePhysicalExtentV1::new(4, 3),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &scene,
        LIMITS,
    )
    .expect("scene should rasterize")
    .expect("nonzero surface should produce a frame");

    assert_eq!(frame.runtime_generation(), generation_zero());
    assert_eq!(frame.surface_generation(), 7);
    assert_eq!(frame.accounted_bytes(), 48);
    assert_eq!(
        frame.pixels(),
        &[
            0x0001_0203,
            0x0001_0203,
            0,
            0,
            0x0001_0203,
            0x0004_0506,
            0x0004_0506,
            0,
            0,
            0x0004_0506,
            0x0004_0506,
            0,
        ]
    );
    assert_eq!(frame.digest(), 0xe16c_9417_6cb9_7ef9);
}

#[test]
fn raster_scales_edges_instead_of_independent_widths() {
    let scene = [rectangle(1, 1, 2, 1, [9, 8, 7, 255])];
    let frame = build_cpu_frame_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(5, 4),
        NativeScaleFactorV1::try_from_f64(1.5).expect("1.5 scale"),
        &scene,
        LIMITS,
    )
    .expect("scene should rasterize")
    .expect("nonzero surface should produce a frame");

    let colored = frame
        .pixels()
        .iter()
        .enumerate()
        .filter_map(|(index, pixel)| (*pixel != 0).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(colored, vec![6, 7, 8, 9, 11, 12, 13, 14]);
}

#[test]
fn zero_extent_skips_without_allocating_a_frame() {
    let result = build_cpu_frame_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(0, 4),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &[rectangle(0, 0, 1, 1, [1, 1, 1, 255])],
        LIMITS,
    )
    .expect("zero extent is a valid suspension");

    assert!(result.is_none());
}

#[test]
fn raster_rejects_nonopaque_scene_before_returning_a_frame() {
    let error = build_cpu_frame_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(2, 2),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &[rectangle(0, 0, 2, 2, [1, 2, 3, 254])],
        LIMITS,
    )
    .expect_err("nonopaque color must fail");

    assert_eq!(error, NativeContractErrorKindV1::UnsupportedAlpha(0));
}

#[test]
fn scene_validation_precedes_storage_and_reports_the_first_record() {
    let storage_called = std::cell::Cell::new(false);
    let error = build_cpu_frame_with_reserver_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(2, 2),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &[
            rectangle(0, 0, 1, 1, [1, 2, 3, 255]),
            rectangle(0, 0, -1, 1, [4, 5, 6, 255]),
        ],
        LIMITS,
        |_| {
            storage_called.set(true);
            Err(())
        },
    )
    .expect_err("invalid rectangle must fail before storage");
    assert_eq!(error, NativeContractErrorKindV1::InvalidRectangle(1));
    assert!(!storage_called.get());

    let error = build_cpu_frame_with_reserver_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(2, 2),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &[
            rectangle(0, 0, 1, 1, [1, 2, 3, 255]),
            rectangle(0, 0, 1, 1, [4, 5, 6, 254]),
        ],
        LIMITS,
        |_| {
            storage_called.set(true);
            Err(())
        },
    )
    .expect_err("nonopaque record must fail before storage");
    assert_eq!(error, NativeContractErrorKindV1::UnsupportedAlpha(1));
    assert!(!storage_called.get());
}

#[test]
fn storage_failure_is_closed_and_returns_no_partial_frame() {
    let error = build_cpu_frame_with_reserver_v1(
        generation_zero(),
        0,
        NativePhysicalExtentV1::new(2, 2),
        NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
        &[rectangle(0, 0, 2, 2, [1, 2, 3, 255])],
        LIMITS,
        |pixel_count| {
            assert_eq!(pixel_count, 4);
            Err(())
        },
    )
    .expect_err("storage failure must be typed");

    assert_eq!(error, NativeContractErrorKindV1::Allocation);
}

#[test]
fn frame_limits_use_width_height_pixels_then_bytes_priority() {
    let scene = [rectangle(0, 0, 9, 9, [1, 2, 3, 255])];
    let cases = [
        (
            NativeFrameLimitsV1::new(8, 8, 64, 256),
            NativePhysicalExtentV1::new(9, 9),
            NativeLimitKindV1::Width,
        ),
        (
            NativeFrameLimitsV1::new(9, 8, 64, 256),
            NativePhysicalExtentV1::new(9, 9),
            NativeLimitKindV1::Height,
        ),
        (
            NativeFrameLimitsV1::new(9, 9, 80, 400),
            NativePhysicalExtentV1::new(9, 9),
            NativeLimitKindV1::Pixels,
        ),
        (
            NativeFrameLimitsV1::new(9, 9, 81, 323),
            NativePhysicalExtentV1::new(9, 9),
            NativeLimitKindV1::Bytes,
        ),
    ];

    for (limits, extent, expected) in cases {
        assert_eq!(
            build_cpu_frame_v1(
                generation_zero(),
                0,
                extent,
                NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale"),
                &scene,
                limits,
            )
            .expect_err("one-over limit must fail"),
            NativeContractErrorKindV1::LimitExceeded(expected)
        );
    }
}

fn rectangle(x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) -> NativeSceneRectangleV1 {
    NativeSceneRectangleV1::new(HeadlessRect::new(x, y, width, height), color)
}

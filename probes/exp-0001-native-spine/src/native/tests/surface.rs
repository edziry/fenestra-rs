use fenestra_ui_runtime::prototype::{HeadlessPoint, HeadlessSurface};

use super::super::{
    NativeContractErrorKindV1, NativePhysicalExtentV1, NativePhysicalPointV1, NativeScaleFactorV1,
    NativeSurfaceChangeV1, NativeSurfaceStateV1,
};

#[test]
fn scale_normalizes_extents_and_points_with_directed_rounding() {
    let scale = NativeScaleFactorV1::try_from_f64(1.25).expect("1.25 is supported");

    assert_eq!(scale.micros(), 1_250_000);
    assert_eq!(
        scale
            .logical_surface(NativePhysicalExtentV1::new(151, 101))
            .expect("extent should fit"),
        HeadlessSurface::new(121, 81)
    );
    assert_eq!(
        scale
            .logical_point(NativePhysicalPointV1::new(5.0, 6.25))
            .expect("point should fit"),
        HeadlessPoint::new(4, 5)
    );
    assert_eq!(
        scale
            .logical_point(NativePhysicalPointV1::new(-0.1, -1.26))
            .expect("negative point should fit"),
        HeadlessPoint::new(-1, -2)
    );
}

#[test]
fn invalid_scale_and_point_values_fail_closed() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY, 8.001] {
        assert_eq!(
            NativeScaleFactorV1::try_from_f64(value).expect_err("scale must fail"),
            NativeContractErrorKindV1::InvalidScale
        );
    }

    let scale = NativeScaleFactorV1::try_from_f64(1.0).expect("unit scale is supported");
    for point in [
        NativePhysicalPointV1::new(f64::NAN, 0.0),
        NativePhysicalPointV1::new(0.0, f64::INFINITY),
        NativePhysicalPointV1::new(f64::from(i32::MAX) + 1.0, 0.0),
    ] {
        assert_eq!(
            scale.logical_point(point).expect_err("point must fail"),
            NativeContractErrorKindV1::InvalidPoint
        );
    }
}

#[test]
fn surface_generation_changes_only_for_effective_native_state() {
    let mut state = NativeSurfaceStateV1::new();
    let size = NativePhysicalExtentV1::new(200, 100);

    let initialized = state.observe(size, 2.0).expect("first state should apply");
    assert_eq!(initialized, NativeSurfaceChangeV1::Initialized);
    assert_eq!(state.generation().expect("initialized generation").get(), 0);
    assert_eq!(state.logical_surface(), Some(HeadlessSurface::new(100, 50)));

    assert_eq!(
        state.observe(size, 2.0).expect("same state is valid"),
        NativeSurfaceChangeV1::NoChange
    );
    assert_eq!(state.generation().expect("same generation").get(), 0);

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(201, 100), 2.0)
            .expect("resize should apply"),
        NativeSurfaceChangeV1::LogicalResize
    );
    assert_eq!(state.generation().expect("resize generation").get(), 1);
    assert_eq!(state.logical_surface(), Some(HeadlessSurface::new(101, 50)));

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(202, 100), 2.01)
            .expect("native-only change should apply"),
        NativeSurfaceChangeV1::NativeOnly
    );
    assert_eq!(state.generation().expect("native generation").get(), 2);
    assert_eq!(state.logical_surface(), Some(HeadlessSurface::new(101, 50)));
}

#[test]
fn zero_extent_suspends_and_nonzero_extent_restores() {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(80, 60), 1.0)
        .expect("initial state should apply");

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(0, 60), 1.0)
            .expect("zero state should apply"),
        NativeSurfaceChangeV1::Suspended
    );
    assert_eq!(state.logical_surface(), Some(HeadlessSurface::new(0, 0)));
    assert!(state.is_suspended());

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(80, 60), 1.0)
            .expect("restored state should apply"),
        NativeSurfaceChangeV1::Restored
    );
    assert_eq!(state.generation().expect("restored generation").get(), 2);
    assert!(!state.is_suspended());
}

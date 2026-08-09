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

    assert_eq!(
        NativeScaleFactorV1::try_from_f64(8.000_000_4)
            .expect("scale rounds before range validation")
            .micros(),
        8_000_000
    );
    assert_eq!(
        NativeScaleFactorV1::try_from_f64(8.000_000_6).expect_err("rounded scale must fail"),
        NativeContractErrorKindV1::InvalidScale
    );

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
fn surface_limits_and_generation_exhaustion_are_atomic() {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(4_096, 4_096), 1.0)
        .expect("inclusive surface limit should apply");
    promote(&mut state);
    let accepted = state.accepted_tuple();
    let pending = state.pending_tuple();

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(4_097, 4_096), 1.0)
            .expect_err("width one-over must fail"),
        NativeContractErrorKindV1::LimitExceeded(super::super::NativeLimitKindV1::Width)
    );
    assert_eq!(state.accepted_tuple(), accepted);
    assert_eq!(state.pending_tuple(), pending);

    state.force_generation_for_test(u64::MAX);
    let exhausted = state.accepted_tuple();
    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(4_095, 4_096), 1.0)
            .expect_err("generation exhaustion must fail"),
        NativeContractErrorKindV1::ArithmeticExhausted
    );
    assert_eq!(
        state
            .accepted_tuple()
            .expect("forced generation must remain")
            .generation()
            .get(),
        u64::MAX
    );
    assert_eq!(state.accepted_tuple(), exhausted);
    assert_eq!(state.pending_tuple(), pending);

    let mut overflow = NativeSurfaceStateV1::new();
    overflow
        .observe(NativePhysicalExtentV1::new(1, 1), 0.000_001)
        .expect("small extent at minimum scale should fit");
    promote(&mut overflow);
    let accepted = overflow.accepted_tuple();
    assert_eq!(
        overflow
            .observe(NativePhysicalExtentV1::new(4_096, 4_096), 0.000_001)
            .expect_err("logical overflow must fail"),
        NativeContractErrorKindV1::ArithmeticExhausted
    );
    assert_eq!(overflow.accepted_tuple(), accepted);
    assert!(overflow.pending_tuple().is_none());
}

#[test]
fn surface_generation_changes_only_for_effective_native_state() {
    let mut state = NativeSurfaceStateV1::new();
    let size = NativePhysicalExtentV1::new(200, 100);

    let initialized = state.observe(size, 2.0).expect("first state should apply");
    assert_eq!(initialized, NativeSurfaceChangeV1::Initialized);
    assert!(state.accepted_tuple().is_none());
    assert_eq!(
        state
            .pending_tuple()
            .expect("initialized tuple")
            .generation()
            .get(),
        0
    );
    assert_eq!(
        state
            .pending_tuple()
            .expect("initialized tuple")
            .logical_surface(),
        HeadlessSurface::new(100, 50)
    );
    let initial = state.pending_tuple().expect("initialized tuple");
    assert_eq!(initial.physical(), size);
    assert_eq!(initial.scale().micros(), 2_000_000);
    assert_eq!(state.pending_count(), 1);
    assert_eq!(state.input_tuple(), None);
    assert_eq!(promote(&mut state), initial);
    assert_eq!(state.input_tuple(), Some(initial));

    assert_eq!(
        state.observe(size, 2.0).expect("same state is valid"),
        NativeSurfaceChangeV1::NoChange
    );
    assert!(state.pending_tuple().is_none());
    assert_eq!(
        state
            .accepted_tuple()
            .expect("accepted tuple")
            .generation()
            .get(),
        0
    );

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(201, 100), 2.0)
            .expect("resize should apply"),
        NativeSurfaceChangeV1::LogicalResize
    );
    let prior = state
        .accepted_tuple()
        .expect("prior tuple remains accepted");
    assert_eq!(prior.logical_surface(), HeadlessSurface::new(100, 50));
    let resize = state.pending_tuple().expect("resize tuple is pending");
    assert_eq!(resize.generation().get(), 1);
    assert_eq!(resize.logical_surface(), HeadlessSurface::new(101, 50));
    assert_eq!(state.input_tuple(), Some(prior));
    let slots = (state.accepted_tuple(), state.pending_tuple());
    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(201, 100), 2.01)
            .expect_err("scale changes cannot replace a pending resize"),
        NativeContractErrorKindV1::EnvironmentScaleChanged
    );
    assert_eq!((state.accepted_tuple(), state.pending_tuple()), slots);
    assert_eq!(promote(&mut state), resize);

    let accepted = state.accepted_tuple();
    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(201, 100), 2.01)
            .expect_err("post-initial scale changes are outside the fixed run"),
        NativeContractErrorKindV1::EnvironmentScaleChanged
    );
    assert_eq!(state.accepted_tuple(), accepted);
    assert!(state.pending_tuple().is_none());

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(202, 100), 2.0)
            .expect("native-only change should apply"),
        NativeSurfaceChangeV1::NativeOnly
    );
    let native_only = state
        .pending_tuple()
        .expect("native-only tuple is retained");
    assert_eq!(native_only.generation().get(), 2);
    assert_eq!(native_only.logical_surface(), HeadlessSurface::new(101, 50));
    assert_eq!(state.accepted_tuple(), accepted);
}

#[test]
fn zero_extent_suspends_and_nonzero_extent_restores() {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(80, 60), 1.0)
        .expect("initial state should apply");
    promote(&mut state);

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(0, 60), 1.0)
            .expect("zero state should apply"),
        NativeSurfaceChangeV1::Suspended
    );
    assert!(!state.accepted_is_suspended());
    assert!(state.pending_is_suspended());
    promote(&mut state);
    assert!(state.accepted_is_suspended());

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(0, 60), 1.0)
            .expect("duplicate suspension is valid"),
        NativeSurfaceChangeV1::NoChange
    );
    assert!(state.pending_tuple().is_none());

    assert_eq!(
        state
            .observe(NativePhysicalExtentV1::new(80, 60), 1.0)
            .expect("restored state should apply"),
        NativeSurfaceChangeV1::Restored
    );
    assert!(state.accepted_is_suspended());
    assert!(!state.pending_is_suspended());
    let restored = promote(&mut state);
    assert_eq!(restored.generation().get(), 2);
    assert!(!state.accepted_is_suspended());
}

#[test]
fn superseded_resize_keeps_one_generation_and_rejects_stale_promotion() {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(100, 80), 1.0)
        .expect("initial tuple should apply");
    let accepted = promote(&mut state);

    state
        .observe(NativePhysicalExtentV1::new(110, 80), 1.0)
        .expect("first resize should apply");
    let stale = state.pending_tuple().expect("first resize is pending");
    state
        .observe(NativePhysicalExtentV1::new(120, 90), 1.0)
        .expect("newer resize should supersede");
    let latest = state.pending_tuple().expect("latest resize is pending");
    assert_eq!(stale.generation(), latest.generation());
    assert_eq!(latest.generation().get(), 1);
    assert_eq!(state.pending_count(), 1);
    assert_eq!(state.input_tuple(), Some(accepted));

    let slots = (state.accepted_tuple(), state.pending_tuple());
    assert_eq!(
        state
            .promote_pending(stale)
            .expect_err("a superseded tuple cannot promote"),
        NativeContractErrorKindV1::Invariant
    );
    assert_eq!((state.accepted_tuple(), state.pending_tuple()), slots);
    assert_eq!(state.promote_pending(latest), Ok(latest));
    assert_eq!(state.input_tuple(), Some(latest));
    assert_eq!(state.pending_count(), 0);
}

fn promote(state: &mut NativeSurfaceStateV1) -> super::super::NativeSurfaceTupleV1 {
    let expected = state.pending_tuple().expect("a tuple should be pending");
    state
        .promote_pending(expected)
        .expect("the exact pending tuple should promote")
}

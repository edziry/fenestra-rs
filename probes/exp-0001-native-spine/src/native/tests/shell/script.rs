use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::super::artifact::NativeProbeResultV1;
use super::super::super::shell::script::{
    NativeReferenceScriptV1, NativeRunDirectiveV1, NativeRunEvidenceV1,
};
use super::super::super::trace::NativeFailureCauseV1;
use super::super::super::types::NativePhysicalPointV1;

#[test]
fn fixed_script_accepts_only_the_registered_identity_milestones() {
    let mut script = NativeReferenceScriptV1::new();
    assert_eq!(
        script.current(),
        NativeRunDirectiveV1::AwaitInitialPublication
    );

    assert_eq!(
        script.advance(initial()).expect("initial milestone"),
        NativeRunDirectiveV1::AwaitRedraw
    );
    assert_eq!(
        script
            .advance(first_frame())
            .expect("first frame milestone"),
        NativeRunDirectiveV1::ScriptPrimaryPress {
            physical: NativePhysicalPointV1::new(6.25, 6.25),
        }
    );
    assert_eq!(
        script
            .advance(NativeRunEvidenceV1::PointerTarget(
                HeadlessPointerTargetV1::StaticControl,
            ))
            .expect("pointer milestone"),
        NativeRunDirectiveV1::RequestLogicalResize {
            width: 360,
            height: 260,
        }
    );
    assert_eq!(
        script.advance(resize()).expect("resize milestone"),
        NativeRunDirectiveV1::AwaitRedraw
    );
    assert_eq!(
        script
            .advance(second_frame())
            .expect("second frame milestone"),
        NativeRunDirectiveV1::ScriptClose
    );
    assert_eq!(
        script.advance(stopped(2, 2, 1)).expect("stop milestone"),
        NativeRunDirectiveV1::Exit(NativeProbeResultV1::Pass)
    );
}

#[test]
fn every_wrong_initial_and_first_frame_identity_is_typed_and_atomic() {
    for evidence in [
        NativeRunEvidenceV1::InitialPublished {
            runtime_generation: 0,
            surface_generation: 0,
            scale_micros: 1_250_000,
        },
        NativeRunEvidenceV1::InitialPublished {
            runtime_generation: 1,
            surface_generation: 1,
            scale_micros: 1_250_000,
        },
        NativeRunEvidenceV1::InitialPublished {
            runtime_generation: 1,
            surface_generation: 0,
            scale_micros: 0,
        },
        first_frame(),
    ] {
        assert_atomic_failure(&mut NativeReferenceScriptV1::new(), evidence);
    }

    for evidence in [
        presented(2, 0, 0, 0, 0),
        presented(1, 1, 0, 0, 0),
        presented(1, 0, 1, 0, 0),
        presented(1, 0, 0, 1, 0),
        presented(1, 0, 0, 0, 1),
        resize(),
    ] {
        assert_atomic_failure(&mut after_initial(), evidence);
    }
}

#[test]
fn every_wrong_pointer_resize_second_frame_and_stop_identity_is_atomic() {
    for evidence in [
        NativeRunEvidenceV1::PointerTarget(HeadlessPointerTargetV1::None),
        NativeRunEvidenceV1::PointerTarget(HeadlessPointerTargetV1::Key(7)),
        resize(),
    ] {
        assert_atomic_failure(&mut after_first_frame(), evidence);
    }

    for evidence in [
        NativeRunEvidenceV1::ResizePublished {
            runtime_generation: 1,
            surface_generation: 1,
            logical_width: 360,
            logical_height: 260,
        },
        NativeRunEvidenceV1::ResizePublished {
            runtime_generation: 2,
            surface_generation: 0,
            logical_width: 360,
            logical_height: 260,
        },
        resize_with_size(350, 250),
        second_frame(),
    ] {
        assert_atomic_failure(&mut after_pointer(), evidence);
    }

    for evidence in [
        presented(1, 1, 1, 1, 1),
        presented(2, 0, 1, 1, 1),
        presented(2, 1, 0, 1, 1),
        presented(2, 1, 1, 0, 1),
        presented(2, 1, 1, 1, 0),
        stopped(2, 2, 1),
    ] {
        assert_atomic_failure(&mut after_resize(), evidence);
    }

    for control in [0, 1, 3, u64::MAX] {
        assert_atomic_failure(&mut after_second_frame(), stopped(control, 2, 1));
    }
    for evidence in [stopped(2, 1, 1), stopped(2, 2, 0), stopped(2, 3, 1)] {
        assert_atomic_failure(&mut after_second_frame(), evidence);
    }
}

#[test]
fn adapt_and_stop_are_terminal_while_early_pass_is_rejected() {
    for result in [NativeProbeResultV1::Adapt, NativeProbeResultV1::Stop] {
        let mut script = after_first_frame();
        assert_eq!(
            script
                .finish(result)
                .expect("non-pass result should terminate"),
            NativeRunDirectiveV1::Exit(result)
        );
        assert_eq!(script.current(), NativeRunDirectiveV1::Exit(result));
    }

    let mut script = after_first_frame();
    let before = script.current();
    assert_eq!(
        script
            .finish(NativeProbeResultV1::Pass)
            .expect_err("pass requires every fixed milestone"),
        NativeFailureCauseV1::Invariant
    );
    assert_eq!(script.current(), before);
}

const fn initial() -> NativeRunEvidenceV1 {
    NativeRunEvidenceV1::InitialPublished {
        runtime_generation: 1,
        surface_generation: 0,
        scale_micros: 1_250_000,
    }
}

const fn first_frame() -> NativeRunEvidenceV1 {
    presented(1, 0, 0, 0, 0)
}

const fn resize() -> NativeRunEvidenceV1 {
    resize_with_size(360, 260)
}

const fn resize_with_size(logical_width: i32, logical_height: i32) -> NativeRunEvidenceV1 {
    NativeRunEvidenceV1::ResizePublished {
        runtime_generation: 2,
        surface_generation: 1,
        logical_width,
        logical_height,
    }
}

const fn second_frame() -> NativeRunEvidenceV1 {
    presented(2, 1, 1, 1, 1)
}

const fn stopped(
    control: u64,
    runtime_generation: u64,
    surface_generation: u64,
) -> NativeRunEvidenceV1 {
    NativeRunEvidenceV1::Stopped {
        control,
        runtime_generation,
        surface_generation,
    }
}

const fn presented(
    runtime_generation: u64,
    surface_generation: u64,
    frame: u64,
    submission: u64,
    completion_control: u64,
) -> NativeRunEvidenceV1 {
    NativeRunEvidenceV1::Presented {
        runtime_generation,
        surface_generation,
        frame,
        submission,
        completion_control,
    }
}

fn after_initial() -> NativeReferenceScriptV1 {
    advance(NativeReferenceScriptV1::new(), initial())
}

fn after_first_frame() -> NativeReferenceScriptV1 {
    advance(after_initial(), first_frame())
}

fn after_pointer() -> NativeReferenceScriptV1 {
    advance(
        after_first_frame(),
        NativeRunEvidenceV1::PointerTarget(HeadlessPointerTargetV1::StaticControl),
    )
}

fn after_resize() -> NativeReferenceScriptV1 {
    advance(after_pointer(), resize())
}

fn after_second_frame() -> NativeReferenceScriptV1 {
    advance(after_resize(), second_frame())
}

fn advance(
    mut script: NativeReferenceScriptV1,
    evidence: NativeRunEvidenceV1,
) -> NativeReferenceScriptV1 {
    script
        .advance(evidence)
        .expect("registered setup milestone should apply");
    script
}

fn assert_atomic_failure(script: &mut NativeReferenceScriptV1, evidence: NativeRunEvidenceV1) {
    let before = script.current();
    assert_eq!(
        script
            .advance(evidence)
            .expect_err("wrong milestone must fail"),
        NativeFailureCauseV1::Invariant
    );
    assert_eq!(script.current(), before);
}

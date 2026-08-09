#[path = "headless/artifact_verify_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1,
    verify_headless_artifact_v1,
};

#[test]
fn result_generation_and_surface_mismatches_are_unindexed() {
    let canonical = support::canonical_bytes();
    assert_error(&result_mismatch(&canonical), Kind::ResultMismatch, None);
    assert_error(
        &support::shift_all_generations(&canonical),
        Kind::FinalGenerationMismatch,
        None,
    );
    assert_error(
        &support::shift_all_surfaces(&canonical),
        Kind::SurfaceMismatch,
        None,
    );
}

#[test]
fn trace_mismatches_report_the_first_differing_event_index() {
    let canonical = support::canonical_bytes();
    assert_error(
        &headless_mismatch(&canonical),
        Kind::HeadlessTraceMismatch,
        Some(4),
    );
    assert_error(
        &scheduler_mismatch(&canonical),
        Kind::SchedulerTraceMismatch,
        Some(0),
    );
}

#[test]
fn result_generation_surface_and_trace_priority_is_stable() {
    let canonical = support::canonical_bytes();
    let result_and_generation = support::shift_all_generations(&result_mismatch(&canonical));
    assert_error(&result_and_generation, Kind::ResultMismatch, None);

    let generation_and_surface =
        support::shift_all_surfaces(&support::shift_all_generations(&canonical));
    assert_error(&generation_and_surface, Kind::FinalGenerationMismatch, None);

    let surface_and_headless = headless_mismatch(&support::shift_all_surfaces(&canonical));
    assert_error(&surface_and_headless, Kind::SurfaceMismatch, None);

    let both_traces = scheduler_mismatch(&headless_mismatch(&canonical));
    assert_error(&both_traces, Kind::HeadlessTraceMismatch, Some(4));
}

pub fn result_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::replace_once(bytes, "result|pass", "result|adapt")
}

pub fn headless_mismatch(bytes: &[u8]) -> Vec<u8> {
    let changed = support::set_field(bytes, "h-event|1|4|", 6, "resize");
    support::set_field(&changed, "h-event|1|4|", 10, "none")
}

pub fn scheduler_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "s-event|1|0|", 10, "faulted")
}

fn assert_error(bytes: &[u8], kind: Kind, index: Option<usize>) {
    let artifact = support::fixed_point(bytes);
    let error: HeadlessArtifactVerificationErrorV1 =
        verify_headless_artifact_v1(&artifact).expect_err("mismatched artifact should not verify");
    assert_eq!(error.kind(), kind);
    assert_eq!(error.index(), index);
}

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};

use super::{
    assert_trace_mismatch, directed_parts, identity_before_transaction, rejection_like,
    shaped_event, with_comparison,
};
use crate::case::TransactionIdV1;
use crate::failure::ReplayFailureV1;
use crate::fingerprint::{
    FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::trace::{
    LogicalTraceV1, TraceComparisonV1, TraceEventV1, TraceOutcomeV1, TraceTerminationV1,
};

#[test]
fn terminal_prefixes_do_not_require_successful_case_coverage() {
    let (case, replay_config, fault, _identity, events, max_bytes) = directed_parts();
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let identity_before_two = identity_before_transaction(&fixture, &case, TransactionIdV1::new(2));
    let identity_before_one = identity_before_transaction(&fixture, &case, TransactionIdV1::new(1));

    let mut rejected_events = events[..3].to_vec();
    rejected_events[2] = rejection_like(&events[2]);
    let rejected_failure = matching_failure(&rejected_events);
    let rejected = LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity_before_two,
        rejected_events,
        rejected_failure,
        max_bytes,
    )
    .expect("a terminal rejection may end before the case does");
    assert!(matches!(
        rejected.termination(),
        TraceTerminationV1::Rejected(_)
    ));
    assert_eq!(rejected.events().len(), 3);
    assert_eq!(rejected.report().verified_step_count(), 2);
    assert_eq!(rejected.report().publication_count(), 1);
    assert_eq!(rejected.report().noop_count(), 1);
    assert_eq!(rejected.report().final_generation(), 1);
    assert_eq!(rejected.report().identity(), identity_before_two);

    let mut mismatched_events = events[..3].to_vec();
    mismatched_events[2] = with_comparison(&events[2], TraceComparisonV1::Mismatch);
    let mismatched_failure = matching_failure(&mismatched_events);
    let mismatched = LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity_before_two,
        mismatched_events,
        mismatched_failure,
        max_bytes,
    )
    .expect("a terminal commit mismatch may end before the case does");
    assert_eq!(mismatched.termination(), TraceTerminationV1::Mismatch);
    assert_eq!(mismatched.report().verified_step_count(), 2);
    assert_eq!(mismatched.report().publication_count(), 2);
    assert_eq!(mismatched.report().noop_count(), 1);
    assert_eq!(mismatched.report().final_generation(), 2);
    assert_eq!(mismatched.report().identity(), identity_before_two);

    let mut noop_events = events[..2].to_vec();
    noop_events[1] = with_comparison(&events[1], TraceComparisonV1::Mismatch);
    let noop_failure = matching_failure(&noop_events);
    let noop_mismatch = LogicalTraceV1::build(
        case,
        replay_config,
        fault,
        identity_before_one,
        noop_events,
        noop_failure,
        max_bytes,
    )
    .expect("a terminal no-op mismatch is representable");
    assert_eq!(noop_mismatch.termination(), TraceTerminationV1::Mismatch);
    assert_eq!(noop_mismatch.report().verified_step_count(), 1);
    assert_eq!(noop_mismatch.report().publication_count(), 1);
    assert_eq!(noop_mismatch.report().noop_count(), 1);
    assert_eq!(noop_mismatch.report().final_generation(), 1);
    assert_eq!(noop_mismatch.report().identity(), identity_before_one);
}

#[test]
fn each_outcome_shape_is_validated_independently() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();
    let paint = InvalidationSet::from_class(InvalidationClass::Paint);

    let mut empty_invalidation_commit = events.clone();
    empty_invalidation_commit[7] = shaped_event(
        TraceOutcomeV1::Commit,
        7,
        1,
        InvalidationSet::NONE,
        TraceComparisonV1::Mismatch,
    );
    let empty_invalidation_failure = matching_failure(&empty_invalidation_commit);
    LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity,
        empty_invalidation_commit,
        empty_invalidation_failure,
        max_bytes,
    )
    .expect("invalidation is an observation and may be empty for commit");

    let invalid = [
        shaped_event(
            TraceOutcomeV1::Commit,
            7,
            0,
            paint,
            TraceComparisonV1::Match,
        ),
        shaped_event(
            TraceOutcomeV1::Noop,
            7,
            0,
            InvalidationSet::NONE,
            TraceComparisonV1::Match,
        ),
        shaped_event(
            TraceOutcomeV1::Noop,
            6,
            1,
            InvalidationSet::NONE,
            TraceComparisonV1::Match,
        ),
        shaped_event(
            TraceOutcomeV1::Reject(crate::trace::CandidateRejectionV1::CapacityOperations),
            6,
            1,
            InvalidationSet::NONE,
            TraceComparisonV1::Mismatch,
        ),
        shaped_event(
            TraceOutcomeV1::Reject(crate::trace::CandidateRejectionV1::CapacityOperations),
            6,
            0,
            paint,
            TraceComparisonV1::Mismatch,
        ),
    ];
    for terminal in invalid {
        let mut invalid_events = events.clone();
        invalid_events[7] = terminal;
        assert_trace_mismatch(
            &case,
            replay_config,
            fault,
            identity,
            invalid_events,
            max_bytes,
        );
    }
}

#[test]
fn terminal_failure_must_match_the_trace_completion() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();

    let unexpected_failure = state_failure(TransactionIdV1::new(7), None);
    assert_build_fails(
        &case,
        replay_config,
        fault,
        identity,
        events.clone(),
        Some(unexpected_failure),
        max_bytes,
    );

    let mut mismatch = events.clone();
    mismatch[7] = with_comparison(&mismatch[7], TraceComparisonV1::Mismatch);
    for failure in [
        None,
        Some(candidate_failure(
            TransactionIdV1::new(7),
            Some(crate::case::OperationIdV1::new(9)),
            crate::trace::CandidateRejectionV1::CapacityOperations,
        )),
        Some(state_failure(TransactionIdV1::new(6), None)),
        Some(state_failure(
            TransactionIdV1::new(7),
            Some(crate::case::OperationIdV1::new(9)),
        )),
    ] {
        assert_build_fails(
            &case,
            replay_config,
            fault,
            identity,
            mismatch.clone(),
            failure,
            max_bytes,
        );
    }

    let mut rejected = events;
    rejected[7] = rejection_like(&rejected[7]);
    for failure in [
        None,
        Some(state_failure(TransactionIdV1::new(7), None)),
        Some(candidate_failure(
            TransactionIdV1::new(7),
            Some(crate::case::OperationIdV1::new(9)),
            crate::trace::CandidateRejectionV1::MissingNode,
        )),
        Some(candidate_failure(
            TransactionIdV1::new(7),
            Some(crate::case::OperationIdV1::new(99)),
            crate::trace::CandidateRejectionV1::CapacityOperations,
        )),
    ] {
        assert_build_fails(
            &case,
            replay_config,
            fault,
            identity,
            rejected.clone(),
            failure,
            max_bytes,
        );
    }
}

#[test]
fn terminal_trace_accepts_a_structured_identity_mismatch() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();
    let mut mismatched = events;
    mismatched[7] = with_comparison(&mismatched[7], TraceComparisonV1::Mismatch);
    let fingerprint = FailureFingerprintV1::identity_mismatch(
        FingerprintLocationV1::Node(crate::semantic::NodePathV1::root()),
        FingerprintSummaryV1::LifecyclePreserved,
        FingerprintSummaryV1::LifecycleFresh,
    )
    .expect("identity fingerprint should be valid");
    let failure = ReplayFailureV1::new(TransactionIdV1::new(7), None, fingerprint.clone());

    let trace = LogicalTraceV1::build(
        case,
        replay_config,
        fault,
        identity,
        mismatched,
        Some(failure),
        max_bytes,
    )
    .expect("a terminal identity mismatch should build");

    assert_eq!(trace.termination(), TraceTerminationV1::Mismatch);
    assert_eq!(
        trace
            .failure()
            .expect("identity mismatch should be retained")
            .fingerprint(),
        &fingerprint
    );
}

pub(super) fn matching_failure(events: &[TraceEventV1]) -> Option<ReplayFailureV1> {
    let event = events.last()?;
    if event.comparison() == TraceComparisonV1::Match {
        return None;
    }
    let (operation, fingerprint) = match event.outcome() {
        TraceOutcomeV1::Reject(rejection) => {
            return Some(candidate_failure(
                event.transaction(),
                event.operations().first().copied(),
                rejection,
            ));
        }
        TraceOutcomeV1::Commit | TraceOutcomeV1::Noop => (None, state_fingerprint()),
    };
    Some(ReplayFailureV1::new(
        event.transaction(),
        operation,
        fingerprint,
    ))
}

fn candidate_failure(
    transaction: TransactionIdV1,
    operation: Option<crate::case::OperationIdV1>,
    rejection: crate::trace::CandidateRejectionV1,
) -> ReplayFailureV1 {
    ReplayFailureV1::new(
        transaction,
        operation,
        FailureFingerprintV1::candidate_rejected(rejection),
    )
}

fn state_failure(
    transaction: TransactionIdV1,
    operation: Option<crate::case::OperationIdV1>,
) -> ReplayFailureV1 {
    ReplayFailureV1::new(transaction, operation, state_fingerprint())
}

fn state_fingerprint() -> FailureFingerprintV1 {
    FailureFingerprintV1::state_mismatch(
        FingerprintLocationV1::Global,
        FingerprintFieldV1::NodeCount,
        FingerprintSummaryV1::Count(1),
        FingerprintSummaryV1::Count(2),
    )
    .expect("synthetic state fingerprint should be valid")
}

#[allow(clippy::too_many_arguments)]
fn assert_build_fails(
    case: &crate::case::GeneratedCaseV1,
    replay_config: crate::fixture::ReplayConfigV1,
    fault: Option<crate::trace::TraceFaultV1>,
    identity: crate::identity::IdentitySummaryV1,
    events: Vec<TraceEventV1>,
    failure: Option<ReplayFailureV1>,
    max_bytes: usize,
) {
    let error = LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity,
        events,
        failure,
        max_bytes,
    )
    .expect_err("incoherent trace failure should be rejected");
    assert_eq!(error.kind(), crate::error::HarnessErrorKind::TraceMismatch);
}

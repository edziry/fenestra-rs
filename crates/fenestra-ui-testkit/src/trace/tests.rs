use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};

use super::{
    CandidateRejectionV1, LogicalTraceV1, TraceComparisonV1, TraceEventV1, TraceFaultV1,
    TraceOutcomeV1, TraceProvenanceV1, TraceTerminationV1,
};
use crate::case::{GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, SeedV1, TransactionIdV1};
use crate::error::{HarnessErrorKind, HarnessLimitKind};
use crate::fixture::{ReplayConfigV1, RuntimeOracleFixtureV1};
use crate::generate::generate_case_v1;
use crate::identity::IdentitySummaryV1;
use crate::replay::{replay_case_v1, replay_case_with_trace_v1};
use crate::wire::{CaseDecodeContextV1, decode_case_v1, encode_case_v1};

mod terminal;

#[test]
fn successful_trace_retains_provenance_and_exact_case_coverage() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let case = directed_case(&fixture);
    let trace = replay_case_with_trace_v1(&fixture, &case).expect("case should replay");

    assert_eq!(trace.termination(), TraceTerminationV1::Success);
    let provenance: TraceProvenanceV1 = trace.provenance();
    assert_eq!(provenance.fixture_revision(), case.fixture_revision());
    assert_eq!(provenance.replay_config(), fixture.replay_config());
    assert_eq!(provenance.generator_config(), case.config());
    assert_eq!(provenance.seed(), case.seed());
    let fault: Option<TraceFaultV1> = provenance.fault();
    assert_eq!(fault, None);

    assert_eq!(trace.events().len(), case.transactions().len());
    for (sequence, (event, transaction)) in
        trace.events().iter().zip(case.transactions()).enumerate()
    {
        assert_eq!(
            event.sequence(),
            u32::try_from(sequence).expect("sequence fits")
        );
        assert_eq!(event.transaction(), transaction.id());
        assert_eq!(
            event.operations(),
            transaction
                .operations()
                .iter()
                .map(|operation| operation.id())
                .collect::<Vec<_>>()
        );
        assert_eq!(event.comparison(), TraceComparisonV1::Match);
        if let Some(previous) = sequence.checked_sub(1) {
            assert_eq!(
                event.before_generation(),
                trace.events()[previous].after_generation()
            );
        }
    }
}

#[test]
fn terminal_rejection_and_mismatch_derive_coherent_reports() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();
    let mut rejected_events = events.clone();
    rejected_events[7] = rejection_event(TraceComparisonV1::Mismatch, 6, 0, InvalidationSet::NONE);
    let rejected_failure = terminal::matching_failure(&rejected_events);
    let rejected = LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity,
        rejected_events,
        rejected_failure,
        max_bytes,
    )
    .expect("coherent terminal rejection should build");

    assert_eq!(
        rejected.termination(),
        TraceTerminationV1::Rejected(CandidateRejectionV1::CapacityOperations)
    );
    assert_eq!(rejected.report().transaction_count(), 8);
    assert_eq!(rejected.report().operation_count(), 10);
    assert_eq!(rejected.report().verified_step_count(), 7);
    assert_eq!(rejected.report().publication_count(), 6);
    assert_eq!(rejected.report().noop_count(), 1);
    assert_eq!(rejected.report().final_generation(), 6);
    assert_eq!(rejected.report().identity(), identity);
    assert!(
        rejected
            .canonical_bytes()
            .ends_with(b"event|7|7|9|6|6|reject:capacity-operations|0|-|mismatch\n")
    );

    let mut mismatched_events = events;
    mismatched_events[7] = with_comparison(&mismatched_events[7], TraceComparisonV1::Mismatch);
    let mismatched_failure = terminal::matching_failure(&mismatched_events);
    let mismatched = LogicalTraceV1::build(
        case,
        replay_config,
        fault,
        identity,
        mismatched_events,
        mismatched_failure,
        max_bytes,
    )
    .expect("coherent terminal mismatch should build");

    assert_eq!(mismatched.termination(), TraceTerminationV1::Mismatch);
    assert_eq!(mismatched.report().transaction_count(), 8);
    assert_eq!(mismatched.report().operation_count(), 10);
    assert_eq!(mismatched.report().verified_step_count(), 7);
    assert_eq!(mismatched.report().publication_count(), 7);
    assert_eq!(mismatched.report().noop_count(), 1);
    assert_eq!(mismatched.report().final_generation(), 7);
    assert_eq!(mismatched.report().identity(), identity);
    assert!(
        mismatched
            .canonical_bytes()
            .ends_with(b"event|7|7|9|6|7|commit|1|structure,layout,paint|mismatch\n")
    );
}

#[test]
fn logical_trace_rejects_invalid_shapes_and_nonterminal_completion() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();
    let paint = InvalidationSet::from_class(InvalidationClass::Paint);
    let invalid_last_events = [
        rejection_event(TraceComparisonV1::Match, 6, 0, InvalidationSet::NONE),
        rejection_event(TraceComparisonV1::Mismatch, 7, 0, InvalidationSet::NONE),
        rejection_event(TraceComparisonV1::Mismatch, 6, 1, paint),
        shaped_event(
            TraceOutcomeV1::Commit,
            6,
            1,
            paint,
            TraceComparisonV1::Match,
        ),
        shaped_event(TraceOutcomeV1::Noop, 6, 0, paint, TraceComparisonV1::Match),
    ];
    for invalid_last in invalid_last_events {
        let mut invalid = events.clone();
        invalid[7] = invalid_last;
        assert_trace_mismatch(&case, replay_config, fault, identity, invalid, max_bytes);
    }

    let mut nonterminal_rejection = events.clone();
    nonterminal_rejection[6] = rejection_like(&events[6]);
    nonterminal_rejection[7] = with_coordinates(&events[7], 7, 7, vec![9], 5, 6);
    assert_trace_mismatch(
        &case,
        replay_config,
        fault,
        identity,
        nonterminal_rejection,
        max_bytes,
    );

    let mut nonterminal_mismatch = events.clone();
    nonterminal_mismatch[6] = with_comparison(&events[6], TraceComparisonV1::Mismatch);
    assert_trace_mismatch(
        &case,
        replay_config,
        fault,
        identity,
        nonterminal_mismatch,
        max_bytes,
    );

    let too_small = LogicalTraceV1::build(case, replay_config, fault, identity, events, None, 1)
        .expect_err("trace bytes should remain bounded");
    assert_eq!(
        too_small.kind(),
        HarnessErrorKind::LimitExceeded(HarnessLimitKind::TraceBytes)
    );
}

#[test]
fn logical_trace_rejects_incomplete_or_misaligned_events() {
    let (case, replay_config, fault, identity, events, max_bytes) = directed_parts();
    let mut invalid_cases = Vec::new();

    let mut incomplete = events.clone();
    incomplete.pop();
    invalid_cases.push(incomplete);

    let mut wrong_sequence = events.clone();
    wrong_sequence[3] = with_coordinates(&events[3], 4, 3, vec![5], 2, 3);
    invalid_cases.push(wrong_sequence);

    let mut wrong_transaction = events.clone();
    wrong_transaction[3] = with_coordinates(&events[3], 3, 4, vec![5], 2, 3);
    invalid_cases.push(wrong_transaction);

    let mut wrong_operations = events.clone();
    wrong_operations[3] = with_coordinates(&events[3], 3, 3, vec![6], 2, 3);
    invalid_cases.push(wrong_operations);

    let mut discontinuous = events.clone();
    discontinuous[3] = with_coordinates(&events[3], 3, 3, vec![5], 99, 100);
    invalid_cases.push(discontinuous);

    let mut wrong_origin = events.clone();
    wrong_origin[0] = with_coordinates(&events[0], 0, 0, vec![0, 1], 1, 2);
    invalid_cases.push(wrong_origin);

    for invalid in invalid_cases {
        assert_trace_mismatch(&case, replay_config, fault, identity, invalid, max_bytes);
    }
}

fn directed_parts() -> (
    GeneratedCaseV1,
    ReplayConfigV1,
    Option<TraceFaultV1>,
    IdentitySummaryV1,
    Vec<TraceEventV1>,
    usize,
) {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let case = directed_case(&fixture);
    let trace = replay_case_with_trace_v1(&fixture, &case).expect("case should replay");
    let identity = prefix_identity(&fixture, &case);
    let events = trace.events().to_vec();
    (
        case,
        fixture.replay_config(),
        None,
        identity,
        events,
        fixture.harness_limits().trace_bytes(),
    )
}

fn directed_case(fixture: &RuntimeOracleFixtureV1) -> GeneratedCaseV1 {
    generate_case_v1(fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate")
}

fn prefix_identity(fixture: &RuntimeOracleFixtureV1, case: &GeneratedCaseV1) -> IdentitySummaryV1 {
    identity_before_transaction(fixture, case, TransactionIdV1::new(7))
}

fn identity_before_transaction(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
    transaction: TransactionIdV1,
) -> IdentitySummaryV1 {
    let bytes = encode_case_v1(case).expect("case should encode");
    let marker = format!("tx|{}|", transaction.get());
    let last = bytes
        .windows(marker.len())
        .position(|window| window == marker.as_bytes())
        .expect("directed transaction should exist");
    let prefix = decode_case_v1(
        &bytes[..last],
        CaseDecodeContextV1::new(case.fixture_revision(), case.config(), case.seed()),
    )
    .expect("directed prefix should decode");
    replay_case_v1(fixture, &prefix)
        .expect("directed prefix should replay")
        .identity()
}

fn rejection_event(
    comparison: TraceComparisonV1,
    after: u64,
    mutations: usize,
    invalidation: InvalidationSet,
) -> TraceEventV1 {
    shaped_event(
        TraceOutcomeV1::Reject(CandidateRejectionV1::CapacityOperations),
        after,
        mutations,
        invalidation,
        comparison,
    )
}

fn rejection_like(event: &TraceEventV1) -> TraceEventV1 {
    TraceEventV1::new(
        event.sequence(),
        event.transaction(),
        event.operations().to_vec(),
        event.before_generation(),
        event.before_generation(),
        TraceOutcomeV1::Reject(CandidateRejectionV1::CapacityOperations),
        0,
        InvalidationSet::NONE,
        TraceComparisonV1::Mismatch,
    )
    .expect("event scalars should fit")
}

fn shaped_event(
    outcome: TraceOutcomeV1,
    after: u64,
    mutations: usize,
    invalidation: InvalidationSet,
    comparison: TraceComparisonV1,
) -> TraceEventV1 {
    TraceEventV1::new(
        7,
        TransactionIdV1::new(7),
        vec![OperationIdV1::new(9)],
        6,
        after,
        outcome,
        mutations,
        invalidation,
        comparison,
    )
    .expect("event scalars should fit")
}

fn with_comparison(event: &TraceEventV1, comparison: TraceComparisonV1) -> TraceEventV1 {
    with_coordinates(
        event,
        event.sequence(),
        event.transaction().get(),
        event.operations().iter().map(|id| id.get()).collect(),
        event.before_generation(),
        event.after_generation(),
    )
    .with_comparison(comparison)
}

trait EventComparisonTestExt {
    fn with_comparison(self, comparison: TraceComparisonV1) -> Self;
}

impl EventComparisonTestExt for TraceEventV1 {
    fn with_comparison(self, comparison: TraceComparisonV1) -> Self {
        TraceEventV1::new(
            self.sequence(),
            self.transaction(),
            self.operations().to_vec(),
            self.before_generation(),
            self.after_generation(),
            self.outcome(),
            usize::try_from(self.mutation_count()).expect("u32 fits usize"),
            self.invalidation(),
            comparison,
        )
        .expect("event scalars should fit")
    }
}

fn with_coordinates(
    event: &TraceEventV1,
    sequence: u32,
    transaction: u32,
    operations: Vec<u32>,
    before: u64,
    after: u64,
) -> TraceEventV1 {
    TraceEventV1::new(
        sequence,
        TransactionIdV1::new(transaction),
        operations.into_iter().map(OperationIdV1::new).collect(),
        before,
        after,
        event.outcome(),
        usize::try_from(event.mutation_count()).expect("u32 fits usize"),
        event.invalidation(),
        event.comparison(),
    )
    .expect("event scalars should fit")
}

fn assert_trace_mismatch(
    case: &GeneratedCaseV1,
    replay_config: ReplayConfigV1,
    fault: Option<TraceFaultV1>,
    identity: IdentitySummaryV1,
    events: Vec<TraceEventV1>,
    max_bytes: usize,
) {
    let failure = terminal::matching_failure(&events);
    let error = LogicalTraceV1::build(
        case.clone(),
        replay_config,
        fault,
        identity,
        events,
        failure,
        max_bytes,
    )
    .expect_err("invalid trace should fail");
    assert_eq!(error.kind(), HarnessErrorKind::TraceMismatch);
}

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, TemplateNodeId};

use super::super::runner::{ObservationPhaseV1, ReplayRunV1, run_case_with_observer_v1};
use crate::case::{GeneratorConfigV1, SeedV1, TransactionIdV1};
use crate::error::{HarnessError, HarnessErrorKind};
use crate::fingerprint::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::generate::generate_case_v1;
use crate::observe::{ObservationOutcomeV1, observe_snapshot_against_view_v1};
use crate::semantic::NodePathV1;
use crate::trace::{TraceComparisonV1, TraceOutcomeV1};

#[test]
fn initial_observation_mismatch_is_an_unlocated_harness_error() {
    let (result, phases) = run_with_root_template_override(ObservationPhaseV1::Initial);
    let Err(error) = result else {
        panic!("initial observation mismatch should stop replay");
    };

    assert_error(error, None);
    assert_eq!(phases, vec![ObservationPhaseV1::Initial]);
}

#[test]
fn before_observation_mismatch_is_located_at_the_transaction() {
    let transaction = TransactionIdV1::new(0);
    let selected = ObservationPhaseV1::Before(transaction);
    let (result, phases) = run_with_root_template_override(selected);
    let Err(error) = result else {
        panic!("before observation mismatch should stop replay");
    };

    assert_error(error, Some(transaction));
    assert_eq!(phases, vec![ObservationPhaseV1::Initial, selected]);
}

#[test]
fn after_commit_observation_mismatch_retains_the_exact_terminal_run() {
    let transaction = TransactionIdV1::new(0);
    let selected = ObservationPhaseV1::AfterCommit(transaction);
    let (result, phases) = run_with_root_template_override(selected);
    let run = result.expect("post-commit mismatch should produce a terminal run");

    assert_eq!(
        phases,
        vec![
            ObservationPhaseV1::Initial,
            ObservationPhaseV1::Before(transaction),
            selected,
        ]
    );
    assert_terminal_failure(&run, transaction);
    assert_eq!(run.events.len(), 1);
    assert_terminal_event(&run.events[0], transaction);

    let report = run.report;
    assert_eq!(report.transaction_count(), 8);
    assert_eq!(report.operation_count(), 10);
    assert_eq!(report.verified_step_count(), 0);
    assert_eq!(report.publication_count(), 1);
    assert_eq!(report.noop_count(), 0);
    assert_eq!(report.final_generation(), 1);
    assert_eq!(report.identity().preserved(), 0);
    assert_eq!(report.identity().retired(), 0);
    assert_eq!(report.identity().fresh(), 0);
    assert_eq!(report.identity().alias_free_snapshots(), 1);
}

fn assert_terminal_event(event: &crate::trace::TraceEventV1, transaction: TransactionIdV1) {
    assert_eq!(event.sequence(), 0);
    assert_eq!(event.transaction(), transaction);
    assert_eq!(
        event.operations(),
        &[
            crate::case::OperationIdV1::new(0),
            crate::case::OperationIdV1::new(1)
        ]
    );
    assert_eq!(event.before_generation(), 0);
    assert_eq!(event.after_generation(), 1);
    assert_eq!(event.outcome(), TraceOutcomeV1::Commit);
    assert_eq!(event.mutation_count(), 1);
    assert_eq!(
        event.invalidation(),
        InvalidationSet::from_class(InvalidationClass::Layout)
            .union(InvalidationSet::from_class(InvalidationClass::Paint))
    );
    assert_eq!(event.comparison(), TraceComparisonV1::Mismatch);
}

fn run_with_root_template_override(
    selected: ObservationPhaseV1,
) -> (Result<ReplayRunV1, HarnessError>, Vec<ObservationPhaseV1>) {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let case = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");
    let construction = fixture.construction().clone();
    let limits = fixture.harness_limits();
    let mut phases = Vec::new();
    let result = run_case_with_observer_v1(&fixture, &case, None, |phase, expected, snapshot| {
        phases.push(phase);
        if phase == selected {
            return Ok(ObservationOutcomeV1::Mismatch(root_template_override()));
        }
        observe_snapshot_against_view_v1(&construction, expected, snapshot, limits)
    });
    (result, phases)
}

fn root_template_override() -> FailureFingerprintV1 {
    FailureFingerprintV1::state_mismatch(
        FingerprintLocationV1::Node(NodePathV1::root()),
        FingerprintFieldV1::Template,
        FingerprintSummaryV1::Template(TemplateNodeId::new(0)),
        FingerprintSummaryV1::Template(TemplateNodeId::new(1)),
    )
    .expect("root template override should be a valid fingerprint")
}

fn assert_error(error: HarnessError, transaction: Option<TransactionIdV1>) {
    assert_eq!(error.kind(), HarnessErrorKind::StateMismatch);
    assert_eq!(error.transaction(), transaction);
    assert_eq!(error.operation(), None);
}

fn assert_terminal_failure(run: &ReplayRunV1, transaction: TransactionIdV1) {
    let failure = run
        .failure
        .as_ref()
        .expect("terminal run should retain the mismatch");
    assert_eq!(failure.transaction(), transaction);
    assert_eq!(failure.operation(), None);
    let fingerprint = failure.fingerprint();
    assert_eq!(fingerprint.kind(), FailureFingerprintKindV1::StateMismatch);
    assert_eq!(
        fingerprint.location(),
        &FingerprintLocationV1::Node(NodePathV1::root())
    );
    assert_eq!(fingerprint.field(), FingerprintFieldV1::Template);
    assert_eq!(
        fingerprint.expected(),
        &FingerprintSummaryV1::Template(TemplateNodeId::new(0))
    );
    assert_eq!(
        fingerprint.observed(),
        &FingerprintSummaryV1::Template(TemplateNodeId::new(1))
    );
}

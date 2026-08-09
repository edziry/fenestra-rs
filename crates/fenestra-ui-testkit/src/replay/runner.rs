mod observation;

use fenestra_ui_ir::prototype::InvalidationSet;
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, RuntimeInitializationError, UiRuntime,
};

use super::commit::{
    CommitShapeV1, RejectionShapeV1, observe_after_verified_commit_v1, verify_rejection_shape,
};
use super::fault::FaultAdapterV1;
use super::rejection::candidate_rejection_v1;
use super::{ReplayReportV1, validate_case};
use crate::case::{GeneratedCaseV1, TransactionV1};
use crate::desired::DesiredStateV1;
use crate::error::{HarnessError, HarnessErrorKind};
use crate::failure::ReplayFailureV1;
use crate::fixture::RuntimeOracleFixtureV1;
use crate::identity::IdentityLedgerV1;
use crate::model::clean_rebuild_v1;
use crate::observe::{ObservationOutcomeV1, observe_snapshot_against_view_v1};
use crate::resolve::ResolvedBaseV1;
use crate::semantic::NormalizedStateV1;
use crate::trace::{TraceComparisonV1, TraceEventV1, TraceFaultV1, TraceOutcomeV1};
use observation::{observation_error_v1, require_complete_observation_v1};

pub(super) use observation::ObservationPhaseV1;

pub(super) struct ReplayRunV1 {
    pub(super) report: ReplayReportV1,
    pub(super) events: Vec<TraceEventV1>,
    pub(super) failure: Option<ReplayFailureV1>,
}

enum ReplayStepV1 {
    Match {
        desired: DesiredStateV1,
        event: TraceEventV1,
    },
    Failure {
        event: TraceEventV1,
        failure: ReplayFailureV1,
    },
}

pub(super) fn run_case_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
    fault: Option<TraceFaultV1>,
) -> Result<ReplayRunV1, HarnessError> {
    let construction = fixture.construction();
    let limits = fixture.harness_limits();
    run_case_with_observer_v1(fixture, case, fault, |_, expected, snapshot| {
        observe_snapshot_against_view_v1(construction, expected, snapshot, limits)
    })
}

pub(super) fn run_case_with_observer_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
    fault: Option<TraceFaultV1>,
    mut observer: impl FnMut(
        ObservationPhaseV1,
        &NormalizedStateV1,
        &CommittedRuntimeSnapshot,
    ) -> Result<ObservationOutcomeV1, HarnessError>,
) -> Result<ReplayRunV1, HarnessError> {
    let (transaction_count, operation_count) = validate_case(fixture, case)?;
    let fault = FaultAdapterV1::validate(case, fault)?;
    let limits = fixture.harness_limits();
    let mut desired = DesiredStateV1::from_construction(fixture.construction(), limits)?;
    let mut runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture.replay_config().runtime_capacity(),
    )
    .map_err(initialization_error)?;
    let mut ledger = IdentityLedgerV1::new();

    {
        let initial_snapshot = runtime.committed();
        let expected = clean_rebuild_v1(fixture.construction(), &desired, limits)?;
        let observed = require_complete_observation_v1(observer(
            ObservationPhaseV1::Initial,
            &expected,
            &initial_snapshot,
        )?)?;
        if let Some(fingerprint) = ledger.verify_initial_aliases(observed.identities())? {
            return Err(observation_error_v1(&fingerprint));
        }
    }

    let mut verified_step_count = 0_u32;
    let mut publication_count = 0_u32;
    let mut noop_count = 0_u32;
    let mut events = Vec::with_capacity(case.transactions().len());
    for transaction in case.transactions() {
        let sequence = u32::try_from(events.len()).map_err(|_| arithmetic_error())?;
        let step = replay_step_v1(
            fixture,
            &mut runtime,
            &desired,
            &mut ledger,
            transaction,
            sequence,
            &fault,
            &mut observer,
        )?;
        match step {
            ReplayStepV1::Match {
                desired: next,
                event,
            } => {
                count_outcome(event.outcome(), &mut publication_count, &mut noop_count)?;
                events.push(event);
                increment(&mut verified_step_count)?;
                desired = next;
            }
            ReplayStepV1::Failure { event, failure } => {
                count_outcome(event.outcome(), &mut publication_count, &mut noop_count)?;
                let final_generation = event.after_generation();
                events.push(event);
                return Ok(ReplayRunV1 {
                    report: ReplayReportV1::new(
                        transaction_count,
                        operation_count,
                        verified_step_count,
                        publication_count,
                        noop_count,
                        final_generation,
                        ledger.summary(),
                    ),
                    events,
                    failure: Some(failure),
                });
            }
        }
    }

    let final_generation = runtime.committed().generation().get();
    Ok(ReplayRunV1 {
        report: ReplayReportV1::new(
            transaction_count,
            operation_count,
            verified_step_count,
            publication_count,
            noop_count,
            final_generation,
            ledger.summary(),
        ),
        events,
        failure: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn replay_step_v1(
    fixture: &RuntimeOracleFixtureV1,
    runtime: &mut UiRuntime,
    desired: &DesiredStateV1,
    ledger: &mut IdentityLedgerV1,
    transaction: &TransactionV1,
    sequence: u32,
    fault: &FaultAdapterV1,
    observer: &mut impl FnMut(
        ObservationPhaseV1,
        &NormalizedStateV1,
        &CommittedRuntimeSnapshot,
    ) -> Result<ObservationOutcomeV1, HarnessError>,
) -> Result<ReplayStepV1, HarnessError> {
    let limits = fixture.harness_limits();
    let transaction_id = transaction.id();
    let base_snapshot = runtime.committed();
    let expected_before = clean_rebuild_v1(fixture.construction(), desired, limits)
        .map_err(|error| error.at_transaction(transaction_id))?;
    let observed_before = require_complete_observation_v1(
        observer(
            ObservationPhaseV1::Before(transaction_id),
            &expected_before,
            &base_snapshot,
        )
        .map_err(|error| error.at_transaction(transaction_id))?,
    )
    .map_err(|error| error.at_transaction(transaction_id))?;
    if let Some(fingerprint) = IdentityLedgerV1::first_alias(observed_before.identities())
        .map_err(|error| error.at_transaction(transaction_id))?
    {
        return Err(observation_error_v1(&fingerprint).at_transaction(transaction_id));
    }

    let before_generation = base_snapshot.generation().get();
    let mut draft = desired.clone();
    let mut candidate = runtime.begin_transaction();
    let mut candidate_ids = Vec::with_capacity(transaction.operations().len());
    {
        let resolved = ResolvedBaseV1::new(observed_before.identities(), desired);
        for operation in transaction.operations() {
            let resolved_operation = resolved.resolve(transaction_id, operation, &draft)?;
            draft
                .apply_operation(operation.operation(), limits)
                .map_err(|error| error.at_operation(transaction_id, operation.id()))?;
            if fault.omits(operation) {
                continue;
            }
            candidate_ids.push(operation.id());
            if let Err(error) = resolved_operation.stage(&mut candidate) {
                let (rejection, failure) =
                    candidate_rejection_v1(error, transaction_id, &candidate_ids)?;
                verify_rejected_runtime(runtime, &base_snapshot, transaction_id)?;
                return Ok(ReplayStepV1::Failure {
                    event: rejection_event(sequence, transaction, before_generation, rejection)?,
                    failure,
                });
            }
        }
    }

    let receipt = match runtime.commit(candidate) {
        Ok(receipt) => receipt,
        Err(error) => {
            let (rejection, failure) =
                candidate_rejection_v1(error, transaction_id, &candidate_ids)?;
            verify_rejected_runtime(runtime, &base_snapshot, transaction_id)?;
            return Ok(ReplayStepV1::Failure {
                event: rejection_event(sequence, transaction, before_generation, rejection)?,
                failure,
            });
        }
    };
    let after_snapshot = runtime.committed();
    let outcome = if receipt.is_empty() {
        TraceOutcomeV1::Noop
    } else {
        TraceOutcomeV1::Commit
    };
    let commit_shape = CommitShapeV1::observe(&base_snapshot, &after_snapshot, &receipt);
    let observation = observe_after_verified_commit_v1(commit_shape, || {
        let expected = clean_rebuild_v1(fixture.construction(), &draft, limits)?;
        observer(
            ObservationPhaseV1::AfterCommit(transaction_id),
            &expected,
            &after_snapshot,
        )
    })
    .map_err(|error| error.at_transaction(transaction_id))?;
    let observed_after = match observation {
        ObservationOutcomeV1::Complete(observed) => observed,
        ObservationOutcomeV1::Mismatch(fingerprint) => {
            let event = committed_event(
                sequence,
                transaction,
                commit_shape,
                outcome,
                TraceComparisonV1::Mismatch,
            )?;
            return Ok(ReplayStepV1::Failure {
                event,
                failure: ReplayFailureV1::new(transaction_id, None, fingerprint),
            });
        }
    };

    if let Some(fingerprint) = ledger
        .verify_transition(
            observed_before.identities(),
            desired,
            observed_after.identities(),
            &draft,
            &after_snapshot,
        )
        .map_err(|error| error.at_transaction(transaction_id))?
    {
        let event = committed_event(
            sequence,
            transaction,
            commit_shape,
            outcome,
            TraceComparisonV1::Mismatch,
        )?;
        return Ok(ReplayStepV1::Failure {
            event,
            failure: ReplayFailureV1::new(transaction_id, None, fingerprint),
        });
    }
    let event = committed_event(
        sequence,
        transaction,
        commit_shape,
        outcome,
        TraceComparisonV1::Match,
    )?;
    Ok(ReplayStepV1::Match {
        desired: draft,
        event,
    })
}

fn verify_rejected_runtime(
    runtime: &UiRuntime,
    before: &fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot,
    transaction: crate::case::TransactionIdV1,
) -> Result<(), HarnessError> {
    let after = runtime.committed();
    verify_rejection_shape(RejectionShapeV1::observe(before, &after))
        .map_err(|error| error.at_transaction(transaction))
}

fn committed_event(
    sequence: u32,
    transaction: &TransactionV1,
    shape: CommitShapeV1,
    outcome: TraceOutcomeV1,
    comparison: TraceComparisonV1,
) -> Result<TraceEventV1, HarnessError> {
    TraceEventV1::new(
        sequence,
        transaction.id(),
        operation_ids(transaction),
        shape.before_generation(),
        shape.after_generation(),
        outcome,
        shape.mutation_count(),
        shape.invalidation(),
        comparison,
    )
    .map_err(|error| error.at_transaction(transaction.id()))
}

fn rejection_event(
    sequence: u32,
    transaction: &TransactionV1,
    generation: u64,
    rejection: crate::trace::CandidateRejectionV1,
) -> Result<TraceEventV1, HarnessError> {
    TraceEventV1::new(
        sequence,
        transaction.id(),
        operation_ids(transaction),
        generation,
        generation,
        TraceOutcomeV1::Reject(rejection),
        0,
        InvalidationSet::NONE,
        TraceComparisonV1::Mismatch,
    )
    .map_err(|error| error.at_transaction(transaction.id()))
}

fn operation_ids(transaction: &TransactionV1) -> Vec<crate::case::OperationIdV1> {
    transaction
        .operations()
        .iter()
        .map(|operation| operation.id())
        .collect()
}

fn count_outcome(
    outcome: TraceOutcomeV1,
    publications: &mut u32,
    noops: &mut u32,
) -> Result<(), HarnessError> {
    match outcome {
        TraceOutcomeV1::Commit => increment(publications),
        TraceOutcomeV1::Noop => increment(noops),
        TraceOutcomeV1::Reject(_) => Ok(()),
    }
}

fn increment(value: &mut u32) -> Result<(), HarnessError> {
    *value = value.checked_add(1).ok_or_else(arithmetic_error)?;
    Ok(())
}

fn initialization_error(_: RuntimeInitializationError) -> HarnessError {
    HarnessError::new(HarnessErrorKind::RuntimeInitialization)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}

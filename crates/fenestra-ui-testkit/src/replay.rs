use crate::case::{GeneratedCaseV1, REGISTERED_FIXTURE_REVISION_V1};
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::FailureFingerprintKindV1;
use crate::fixture::RuntimeOracleFixtureV1;
use crate::identity::IdentitySummaryV1;
use crate::trace::{LogicalTraceV1, TraceFaultV1};

mod commit;
mod fault;
mod rejection;
mod runner;
#[cfg(test)]
mod tests;

use runner::{ReplayRunV1, run_case_v1};

/// Scalar, physical-identity-free result of one complete V1 replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayReportV1 {
    transaction_count: u32,
    operation_count: usize,
    verified_step_count: u32,
    publication_count: u32,
    noop_count: u32,
    final_generation: u64,
    identity: IdentitySummaryV1,
}

impl ReplayReportV1 {
    pub(crate) const fn new(
        transaction_count: u32,
        operation_count: usize,
        verified_step_count: u32,
        publication_count: u32,
        noop_count: u32,
        final_generation: u64,
        identity: IdentitySummaryV1,
    ) -> Self {
        Self {
            transaction_count,
            operation_count,
            verified_step_count,
            publication_count,
            noop_count,
            final_generation,
            identity,
        }
    }

    /// Returns the number of transactions supplied by the exact case.
    #[must_use]
    pub const fn transaction_count(self) -> u32 {
        self.transaction_count
    }

    /// Returns the number of operations supplied by the exact case.
    #[must_use]
    pub const fn operation_count(self) -> usize {
        self.operation_count
    }

    /// Returns transactions fully checked before and after candidate commit.
    #[must_use]
    pub const fn verified_step_count(self) -> u32 {
        self.verified_step_count
    }

    /// Returns nonempty logical publications observed during replay.
    #[must_use]
    pub const fn publication_count(self) -> u32 {
        self.publication_count
    }

    /// Returns true no-op candidate commits observed during replay.
    #[must_use]
    pub const fn noop_count(self) -> u32 {
        self.noop_count
    }

    /// Returns the final logical runtime generation.
    #[must_use]
    pub const fn final_generation(self) -> u64 {
        self.final_generation
    }

    /// Returns aggregated transient identity checks.
    #[must_use]
    pub const fn identity(self) -> IdentitySummaryV1 {
        self.identity
    }
}

/// Replays one exact semantic case and compares every candidate transaction.
pub fn replay_case_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
) -> Result<ReplayReportV1, HarnessError> {
    let run = run_case_v1(fixture, case, None)?;
    require_success(&run)?;
    Ok(run.report)
}

/// Replays one exact case and retains its bounded canonical logical trace.
pub fn replay_case_with_trace_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
) -> Result<LogicalTraceV1, HarnessError> {
    let run = run_case_v1(fixture, case, None)?;
    require_success(&run)?;
    build_trace(fixture, case, None, run)
}

/// Replays one exact case through a closed test-only candidate fault.
pub fn replay_case_with_fault_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
    fault: TraceFaultV1,
) -> Result<LogicalTraceV1, HarnessError> {
    let run = run_case_v1(fixture, case, Some(fault))?;
    build_trace(fixture, case, Some(fault), run)
}

fn build_trace(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
    fault: Option<TraceFaultV1>,
    run: ReplayRunV1,
) -> Result<LogicalTraceV1, HarnessError> {
    LogicalTraceV1::build(
        case.clone(),
        fixture.replay_config(),
        fault,
        run.report.identity(),
        run.events,
        run.failure,
        fixture.harness_limits().trace_bytes(),
    )
}

fn require_success(run: &ReplayRunV1) -> Result<(), HarnessError> {
    match run.failure.as_ref() {
        None => Ok(()),
        Some(failure) => Err(failure_error(failure)),
    }
}

fn failure_error(failure: &ReplayFailureV1) -> HarnessError {
    let kind = match failure.fingerprint().kind() {
        FailureFingerprintKindV1::CandidateRejected => {
            HarnessErrorKind::UnexpectedCandidateRejection
        }
        FailureFingerprintKindV1::StateMismatch => HarnessErrorKind::StateMismatch,
        FailureFingerprintKindV1::IdentityMismatch => HarnessErrorKind::IdentityMismatch,
    };
    let error = HarnessError::new(kind);
    failure.operation().map_or_else(
        || error.at_transaction(failure.transaction()),
        |operation| error.at_operation(failure.transaction(), operation),
    )
}

fn validate_case(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
) -> Result<(u32, usize), HarnessError> {
    if case.fixture_revision() != REGISTERED_FIXTURE_REVISION_V1 {
        return Err(HarnessError::new(HarnessErrorKind::UnsupportedVersion));
    }
    let limits = fixture.harness_limits();
    if case.transactions().len() > limits.transactions() {
        return Err(HarnessError::limit(HarnessLimitKind::Transactions));
    }
    let mut operation_count = 0_usize;
    for transaction in case.transactions() {
        if transaction.operations().len() > limits.operations_per_transaction() {
            return Err(
                HarnessError::limit(HarnessLimitKind::OperationsPerTransaction)
                    .at_transaction(transaction.id()),
            );
        }
        operation_count = operation_count
            .checked_add(transaction.operations().len())
            .ok_or_else(arithmetic_error)?;
    }
    if operation_count > limits.operations() {
        return Err(HarnessError::limit(HarnessLimitKind::Operations));
    }
    for transaction in case.transactions() {
        if transaction.operations().is_empty() {
            return Err(HarnessError::new(HarnessErrorKind::InvalidOperation)
                .at_transaction(transaction.id()));
        }
    }
    let transaction_count =
        u32::try_from(case.transactions().len()).map_err(|_| arithmetic_error())?;
    Ok((transaction_count, operation_count))
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}

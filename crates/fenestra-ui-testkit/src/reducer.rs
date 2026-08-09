use std::error::Error;
use std::fmt;

use crate::case::GeneratedCaseV1;
use crate::failure::ReplayFailureV1;
use crate::fixture::RuntimeOracleFixtureV1;
use crate::replay::replay_case_with_fault_v1;
use crate::trace::TraceFaultV1;

mod metric;
mod transform;

use metric::{ReductionMetricV1, measure_case_v1};
use transform::{CandidateDispositionV1, SearchOutcomeV1, search_candidates_v1};

const MAX_EVALUATIONS_V1: u32 = 4_096;

/// Bounded configuration for deterministic V1 failure reduction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReducerConfigV1 {
    max_evaluations: u32,
}

impl ReducerConfigV1 {
    /// Creates an unvalidated reduction configuration.
    #[must_use]
    pub const fn new(max_evaluations: u32) -> Self {
        Self { max_evaluations }
    }

    /// Returns the maximum candidate replay evaluations.
    #[must_use]
    pub const fn max_evaluations(self) -> u32 {
        self.max_evaluations
    }
}

/// Completion status produced by the deterministic V1 reducer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReductionCompletionV1 {
    /// No remaining V1 transform preserved the target failure.
    FixedPoint,
    /// The configured evaluation budget was consumed first.
    BudgetExhausted,
}

impl ReductionCompletionV1 {
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::FixedPoint => "fixed-point",
            Self::BudgetExhausted => "budget-exhausted",
        }
    }
}

/// Closed failure classes produced by the deterministic V1 reducer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReducerErrorKind {
    /// The configured evaluation budget was outside `1..=4096`.
    InvalidConfiguration,
    /// The input case did not reproduce the exact supplied first failure.
    InvalidInputFailure,
    /// A case metric could not be encoded within the bounded V1 case format.
    MetricLimitExceeded,
    /// Checked reducer arithmetic could not represent a required value.
    ArithmeticExhausted,
}

/// Privacy-safe failure returned by deterministic V1 reduction.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReducerError {
    kind: ReducerErrorKind,
}

impl ReducerError {
    pub(super) const fn new(kind: ReducerErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the closed reducer failure class.
    #[must_use]
    pub const fn kind(self) -> ReducerErrorKind {
        self.kind
    }
}

impl fmt::Debug for ReducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReducerError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for ReducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime oracle reduction failed: {:?}",
            self.kind
        )
    }
}

impl Error for ReducerError {}

/// Bounded result of one deterministic V1 reduction run.
#[derive(Clone, Eq, PartialEq)]
pub struct ReductionResultV1 {
    minimized_case: GeneratedCaseV1,
    used_evaluations: u32,
    completion: ReductionCompletionV1,
}

impl ReductionResultV1 {
    const fn new(
        minimized_case: GeneratedCaseV1,
        used_evaluations: u32,
        completion: ReductionCompletionV1,
    ) -> Self {
        Self {
            minimized_case,
            used_evaluations,
            completion,
        }
    }

    /// Returns the exact reduced case without renumbering retained IDs.
    #[must_use]
    pub const fn minimized_case(&self) -> &GeneratedCaseV1 {
        &self.minimized_case
    }

    /// Returns the number of candidate replay evaluations consumed.
    #[must_use]
    pub const fn used_evaluations(&self) -> u32 {
        self.used_evaluations
    }

    /// Returns how the bounded reduction stopped.
    #[must_use]
    pub const fn completion(&self) -> ReductionCompletionV1 {
        self.completion
    }
}

impl fmt::Debug for ReductionResultV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReductionResultV1")
            .field(
                "transaction_count",
                &self.minimized_case.transactions().len(),
            )
            .field("operation_count", &self.minimized_case.operation_count())
            .field("used_evaluations", &self.used_evaluations)
            .field("completion", &self.completion)
            .finish()
    }
}

/// Reduces one exact reproducible failure under the ordered V1 transforms.
pub fn reduce_failure_case_v1(
    fixture: &RuntimeOracleFixtureV1,
    original: &GeneratedCaseV1,
    fault: TraceFaultV1,
    target: &ReplayFailureV1,
    config: ReducerConfigV1,
) -> Result<ReductionResultV1, ReducerError> {
    validate_config(config)?;
    validate_input_failure(fixture, original, fault, target)?;

    let mut current = original.clone();
    let mut current_metric = measure_case_v1(&current)?;
    let mut used_evaluations = 0_u32;

    loop {
        let outcome = search_candidates_v1(&current, &mut |candidate| {
            evaluate_candidate_v1(
                fixture,
                candidate,
                fault,
                target,
                config.max_evaluations(),
                &current_metric,
                &mut used_evaluations,
            )
        })?;
        match outcome {
            SearchOutcomeV1::Accepted(candidate) => {
                current_metric = measure_case_v1(&candidate)?;
                current = candidate;
            }
            SearchOutcomeV1::Exhausted(Some(candidate)) => {
                return Ok(ReductionResultV1::new(
                    candidate,
                    used_evaluations,
                    ReductionCompletionV1::BudgetExhausted,
                ));
            }
            SearchOutcomeV1::Exhausted(None) => {
                return Ok(ReductionResultV1::new(
                    current,
                    used_evaluations,
                    ReductionCompletionV1::BudgetExhausted,
                ));
            }
            SearchOutcomeV1::FixedPoint => {
                return Ok(ReductionResultV1::new(
                    current,
                    used_evaluations,
                    ReductionCompletionV1::FixedPoint,
                ));
            }
        }
    }
}

fn validate_config(config: ReducerConfigV1) -> Result<(), ReducerError> {
    if !(1..=MAX_EVALUATIONS_V1).contains(&config.max_evaluations()) {
        return Err(ReducerError::new(ReducerErrorKind::InvalidConfiguration));
    }
    Ok(())
}

fn validate_input_failure(
    fixture: &RuntimeOracleFixtureV1,
    original: &GeneratedCaseV1,
    fault: TraceFaultV1,
    target: &ReplayFailureV1,
) -> Result<(), ReducerError> {
    let trace = replay_case_with_fault_v1(fixture, original, fault)
        .map_err(|_| ReducerError::new(ReducerErrorKind::InvalidInputFailure))?;
    if trace.failure() != Some(target) {
        return Err(ReducerError::new(ReducerErrorKind::InvalidInputFailure));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn evaluate_candidate_v1(
    fixture: &RuntimeOracleFixtureV1,
    candidate: GeneratedCaseV1,
    fault: TraceFaultV1,
    target: &ReplayFailureV1,
    max_evaluations: u32,
    current_metric: &ReductionMetricV1,
    used_evaluations: &mut u32,
) -> Result<CandidateDispositionV1, ReducerError> {
    if measure_case_v1(&candidate)? >= *current_metric {
        return Ok(CandidateDispositionV1::Continue);
    }

    *used_evaluations = used_evaluations
        .checked_add(1)
        .ok_or_else(arithmetic_error)?;
    let accepted = replay_case_with_fault_v1(fixture, &candidate, fault)
        .ok()
        .and_then(|trace| trace.failure().cloned())
        .as_ref()
        == Some(target);
    if *used_evaluations == max_evaluations {
        Ok(CandidateDispositionV1::Exhausted { accepted })
    } else if accepted {
        Ok(CandidateDispositionV1::Accept)
    } else {
        Ok(CandidateDispositionV1::Continue)
    }
}

fn arithmetic_error() -> ReducerError {
    ReducerError::new(ReducerErrorKind::ArithmeticExhausted)
}

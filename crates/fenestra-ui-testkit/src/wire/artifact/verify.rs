mod path;
mod semantic;

#[cfg(test)]
mod tests;

use fenestra_ui_ir::prototype::{SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT};

use self::path::validate_artifact_paths_v1;
use self::semantic::validate_artifact_operations_v1;
use super::FailureArtifactV1;
use crate::case::REGISTERED_FIXTURE_REVISION_V1;
use crate::error::HarnessError;
use crate::failure::ReplayFailureV1;
use crate::fixture::{RuntimeOracleFixtureV1, SCHEMA_NAMESPACE, SCHEMA_REVISION};
use crate::generate::generate_case_v1;
use crate::reducer::{ReducerConfigV1, ReductionCompletionV1, reduce_failure_case_v1};
use crate::replay::{replay_case_v1, replay_case_with_fault_v1};
use crate::trace::{LogicalTraceV1, TraceEventV1};
use crate::wire::case::encode_case_v1;
use crate::wire::error::{ArtifactVerificationError, ArtifactVerificationErrorKind};

/// Verifies decoded artifact provenance, replay, trace, and reduction semantics.
pub fn verify_failure_artifact_v1(
    artifact: &FailureArtifactV1,
) -> Result<(), ArtifactVerificationError> {
    let fixture = RuntimeOracleFixtureV1::build().map_err(|_| fixture_mismatch())?;
    verify_fixture_v1(artifact)?;
    verify_replay_config_v1(&fixture, artifact)?;
    validate_artifact_paths_v1(fixture.construction(), artifact)?;
    validate_artifact_operations_v1(
        &fixture,
        artifact.original_case(),
        artifact.minimized_case(),
    )?;
    verify_seed_v1(&fixture, artifact)?;

    let original_trace =
        replay_case_with_fault_v1(&fixture, artifact.original_case(), artifact.fault()).map_err(
            |error| {
                from_harness(
                    ArtifactVerificationErrorKind::OriginalFailureMismatch,
                    error,
                )
            },
        )?;
    verify_replayed_failure_v1(
        &original_trace,
        artifact.original_failure(),
        ArtifactVerificationErrorKind::OriginalFailureMismatch,
    )?;

    let minimized_trace =
        replay_case_with_fault_v1(&fixture, artifact.minimized_case(), artifact.fault()).map_err(
            |error| {
                from_harness(
                    ArtifactVerificationErrorKind::MinimizedFailureMismatch,
                    error,
                )
            },
        )?;
    verify_replayed_failure_v1(
        &minimized_trace,
        artifact.minimized_failure(),
        ArtifactVerificationErrorKind::MinimizedFailureMismatch,
    )?;

    verify_fingerprint_v1(artifact)?;
    verify_trace_v1(&minimized_trace, artifact.events())?;
    verify_fault_free_replay_v1(&fixture, artifact.minimized_case())?;
    verify_reduction_v1(&fixture, artifact)
}

fn verify_fixture_v1(artifact: &FailureArtifactV1) -> Result<(), ArtifactVerificationError> {
    let metadata = artifact.fixture();
    if metadata.fixture_revision() != REGISTERED_FIXTURE_REVISION_V1
        || metadata.schema_format() != SUPPORTED_SCHEMA_FORMAT.get()
        || metadata.schema_namespace() != SCHEMA_NAMESPACE.get()
        || metadata.schema_revision() != SCHEMA_REVISION.get()
        || metadata.construction_format() != SUPPORTED_CONSTRUCTION_FORMAT.get()
    {
        return Err(fixture_mismatch());
    }
    Ok(())
}

fn verify_replay_config_v1(
    fixture: &RuntimeOracleFixtureV1,
    artifact: &FailureArtifactV1,
) -> Result<(), ArtifactVerificationError> {
    if artifact.replay_config() != fixture.replay_config() {
        return Err(ArtifactVerificationError::new(
            ArtifactVerificationErrorKind::ReplayConfigMismatch,
        ));
    }
    Ok(())
}

fn verify_seed_v1(
    fixture: &RuntimeOracleFixtureV1,
    artifact: &FailureArtifactV1,
) -> Result<(), ArtifactVerificationError> {
    let regenerated = generate_case_v1(fixture, artifact.seed(), artifact.generator_config())
        .map_err(|_| seed_mismatch())?;
    let regenerated = encode_case_v1(&regenerated).map_err(|_| seed_mismatch())?;
    let original = encode_case_v1(artifact.original_case()).map_err(|_| seed_mismatch())?;
    if regenerated != original {
        return Err(seed_mismatch());
    }
    Ok(())
}

fn verify_replayed_failure_v1(
    trace: &LogicalTraceV1,
    stored: &ReplayFailureV1,
    kind: ArtifactVerificationErrorKind,
) -> Result<(), ArtifactVerificationError> {
    if trace.failure() != Some(stored) {
        return Err(at_failure(ArtifactVerificationError::new(kind), stored));
    }
    Ok(())
}

fn verify_fingerprint_v1(artifact: &FailureArtifactV1) -> Result<(), ArtifactVerificationError> {
    if artifact.original_failure().fingerprint() != artifact.minimized_failure().fingerprint() {
        return Err(at_failure(
            ArtifactVerificationError::new(ArtifactVerificationErrorKind::FingerprintMismatch),
            artifact.minimized_failure(),
        ));
    }
    Ok(())
}

fn verify_trace_v1(
    replayed: &LogicalTraceV1,
    stored: &[TraceEventV1],
) -> Result<(), ArtifactVerificationError> {
    if replayed.events() == stored {
        return Ok(());
    }
    let first = stored
        .iter()
        .zip(replayed.events())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| stored.len().min(replayed.events().len()));
    let error = ArtifactVerificationError::new(ArtifactVerificationErrorKind::TraceMismatch);
    Err(stored
        .get(first)
        .or_else(|| replayed.events().get(first))
        .map_or(error, |event| error.at_transaction(event.transaction())))
}

fn verify_fault_free_replay_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &crate::case::GeneratedCaseV1,
) -> Result<(), ArtifactVerificationError> {
    replay_case_v1(fixture, case)
        .map(|_| ())
        .map_err(|error| from_harness(ArtifactVerificationErrorKind::FaultFreeReplayFailed, error))
}

fn verify_reduction_v1(
    fixture: &RuntimeOracleFixtureV1,
    artifact: &FailureArtifactV1,
) -> Result<(), ArtifactVerificationError> {
    let stored = artifact.reduction();
    let config = ReducerConfigV1::new(stored.max_evaluations());
    let result = reduce_failure_case_v1(
        fixture,
        artifact.original_case(),
        artifact.fault(),
        artifact.original_failure(),
        config,
    )
    .map_err(|_| reduction_mismatch())?;
    if result.minimized_case() != artifact.minimized_case()
        || result.used_evaluations() != stored.used_evaluations()
        || result.completion() != stored.completion()
    {
        return Err(reduction_mismatch());
    }
    if stored.completion() == ReductionCompletionV1::FixedPoint {
        let fixed = reduce_failure_case_v1(
            fixture,
            artifact.minimized_case(),
            artifact.fault(),
            artifact.minimized_failure(),
            config,
        )
        .map_err(|_| reduction_mismatch())?;
        if fixed.minimized_case() != artifact.minimized_case()
            || fixed.completion() != ReductionCompletionV1::FixedPoint
        {
            return Err(reduction_mismatch());
        }
    }
    Ok(())
}

fn from_harness(
    kind: ArtifactVerificationErrorKind,
    error: HarnessError,
) -> ArtifactVerificationError {
    let verification = ArtifactVerificationError::new(kind);
    match (error.transaction(), error.operation()) {
        (Some(transaction), Some(operation)) => verification.at_operation(transaction, operation),
        (Some(transaction), None) => verification.at_transaction(transaction),
        _ => verification,
    }
}

fn at_failure(
    error: ArtifactVerificationError,
    failure: &ReplayFailureV1,
) -> ArtifactVerificationError {
    failure.operation().map_or_else(
        || error.at_transaction(failure.transaction()),
        |operation| error.at_operation(failure.transaction(), operation),
    )
}

fn fixture_mismatch() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::FixtureMismatch)
}

fn seed_mismatch() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::SeedMismatch)
}

fn reduction_mismatch() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::ReductionMismatch)
}

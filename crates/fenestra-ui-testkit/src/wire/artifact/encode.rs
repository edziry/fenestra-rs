use crate::trace::TraceFaultV1;

use super::super::case::encode_case_v1;
use super::super::error::{ArtifactEncodeError, ArtifactLimitKind};
use super::super::scan::{ARTIFACT_BYTES_LIMIT, LINE_BYTES_LIMIT, LINES_LIMIT};
use super::fingerprint::{FailureScopeV1, write_failure_v1};
use super::model::FailureArtifactV1;

const HEADER: &str = "fenestra-oracle-failure|1";
const VERSIONS: &str =
    "versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|1|reducer|1";

/// Encodes one typed V1 failure artifact in canonical bounded form without I/O.
pub fn encode_failure_artifact_v1(
    artifact: &FailureArtifactV1,
) -> Result<Vec<u8>, ArtifactEncodeError> {
    let original = encode_case_v1(artifact.original_case())?;
    let minimized = encode_case_v1(artifact.minimized_case())?;
    let trace = super::trace::encode_trace_records_v1(artifact.events())?;
    let mut output = Vec::new();

    push_line(&mut output, HEADER);
    push_line(&mut output, VERSIONS);
    push_line(&mut output, &fixture_line(artifact));
    push_line(&mut output, &replay_line(artifact));
    push_line(&mut output, &generator_line(artifact));
    push_line(&mut output, &format!("seed|{}", artifact.seed().get()));
    push_line(
        &mut output,
        &format!(
            "original-begin|{}|{}|{}",
            artifact.original_case().transactions().len(),
            artifact.original_case().operation_count(),
            original.len()
        ),
    );
    output.extend_from_slice(&original);
    push_line(&mut output, "original-end");
    push_line(&mut output, &fault_line(artifact.fault()));
    push_failure(
        &mut output,
        FailureScopeV1::Original,
        artifact.original_failure(),
    );
    push_line(&mut output, &reduction_line(artifact));
    push_line(
        &mut output,
        &format!(
            "minimized-begin|{}|{}|{}",
            artifact.minimized_case().transactions().len(),
            artifact.minimized_case().operation_count(),
            minimized.len()
        ),
    );
    output.extend_from_slice(&minimized);
    push_line(&mut output, "minimized-end");
    push_failure(
        &mut output,
        FailureScopeV1::Minimized,
        artifact.minimized_failure(),
    );
    push_line(
        &mut output,
        &format!("trace-begin|{}|{}", artifact.events().len(), trace.len()),
    );
    output.extend_from_slice(&trace);
    push_line(&mut output, "trace-end");
    push_line(&mut output, "end");

    validate_envelope_limits(&output)?;
    Ok(output)
}

fn fixture_line(artifact: &FailureArtifactV1) -> String {
    let fixture = artifact.fixture();
    format!(
        "fixture|runtime-oracle|{}|{}|{}|{}|{}",
        fixture.fixture_revision(),
        fixture.schema_format(),
        fixture.schema_namespace(),
        fixture.schema_revision(),
        fixture.construction_format()
    )
}

fn replay_line(artifact: &FailureArtifactV1) -> String {
    let [
        operations,
        structural,
        nodes,
        fragments,
        properties,
        retained,
    ] = artifact.replay_config().fields();
    format!("replay|{operations}|{structural}|{nodes}|{fragments}|{properties}|{retained}")
}

fn generator_line(artifact: &FailureArtifactV1) -> String {
    let config = artifact.generator_config();
    format!(
        "generator|{}|{}|{}",
        config.transaction_count(),
        config.max_operations_per_transaction(),
        config.max_live_memberships()
    )
}

fn fault_line(fault: TraceFaultV1) -> String {
    match fault {
        TraceFaultV1::OmitMove { target } => format!("fault|omit-move|{}", target.get()),
    }
}

fn reduction_line(artifact: &FailureArtifactV1) -> String {
    let reduction = artifact.reduction();
    format!(
        "reducer|{}|{}|{}",
        reduction.max_evaluations(),
        reduction.used_evaluations(),
        reduction.completion().word()
    )
}

fn push_failure(
    output: &mut Vec<u8>,
    scope: FailureScopeV1,
    failure: &crate::failure::ReplayFailureV1,
) {
    let mut line = String::new();
    write_failure_v1(&mut line, scope, failure);
    push_line(output, &line);
}

fn push_line(output: &mut Vec<u8>, line: &str) {
    output.extend_from_slice(line.as_bytes());
    output.push(b'\n');
}

fn validate_envelope_limits(output: &[u8]) -> Result<(), ArtifactEncodeError> {
    if output.len() > ARTIFACT_BYTES_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::ArtifactBytes));
    }
    if output
        .split(|byte| *byte == b'\n')
        .any(|line| line.len() > LINE_BYTES_LIMIT)
    {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::LineBytes));
    }
    if output.iter().filter(|byte| **byte == b'\n').count() > LINES_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::Lines));
    }
    Ok(())
}

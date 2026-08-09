use crate::case::{GeneratorConfigV1, OperationIdV1, SeedV1};
use crate::trace::TraceFaultV1;

use super::super::case::{CaseDecodeContextV1, decode_case_records_v1};
use super::super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, CountKind};
use super::super::primitive::{parse_u32, parse_u64};
use super::super::scan::ScannedLine;
use super::boundary::{EnvelopeBoundariesV1, SectionBoundaryV1};
use super::fingerprint::{FailureScopeV1, finish_failure_v1, parse_failure_parts_v1};
use super::model::{
    ArtifactFixtureMetadataV1, ArtifactReductionV1, ArtifactReplayConfigV1, FailureArtifactV1,
};
use crate::reducer::ReductionCompletionV1;

/// Decodes one complete bounded V1 failure artifact structurally.
///
/// This does not verify fixture provenance, replay behavior, or reduction.
pub fn decode_failure_artifact_v1(bytes: &[u8]) -> Result<FailureArtifactV1, ArtifactDecodeError> {
    let envelope = super::scan_envelope_v1(bytes)?;
    super::preflight::preflight_envelope_v1(&envelope)?;
    decode_envelope_v1(&envelope)
}

fn decode_envelope_v1(
    envelope: &EnvelopeBoundariesV1<'_>,
) -> Result<FailureArtifactV1, ArtifactDecodeError> {
    let fixture = parse_fixture(&envelope.lines[2])?;
    let replay_config = parse_replay_config(&envelope.lines[3])?;
    let generator_config = parse_generator_config(&envelope.lines[4])?;
    let seed = parse_seed(&envelope.lines[5])?;
    let context = CaseDecodeContextV1::new(fixture.fixture_revision(), generator_config, seed);

    let original_boundary = envelope.original();
    let original_case = parse_case(original_boundary, context)?;
    let fault_line = following_line(envelope, original_boundary, 0);
    let original_failure_line = following_line(envelope, original_boundary, 1);
    let reduction_line = following_line(envelope, original_boundary, 2);
    let fault = parse_fault(fault_line)?;
    let original_failure_parts = parse_failure_parts_v1(original_failure_line)?;
    let reduction = parse_reduction(reduction_line)?;

    let minimized_boundary = envelope.minimized();
    let minimized_case = parse_case(minimized_boundary, context)?;
    let minimized_failure_line = following_line(envelope, minimized_boundary, 0);
    let minimized_failure_parts = parse_failure_parts_v1(minimized_failure_line)?;

    let trace_boundary = envelope.trace();
    let trace_declaration = parse_trace_declaration(trace_boundary.begin())?;
    let trace_bytes = records_byte_count(trace_boundary.records())?;
    let events = super::trace::parse_trace_records_v1(trace_boundary.records())?;
    check_count(
        trace_declaration.events,
        events.len(),
        CountKind::TraceEvents,
        trace_declaration.line,
    )?;
    check_count(
        trace_declaration.bytes,
        trace_bytes,
        CountKind::TraceBytes,
        trace_declaration.line,
    )?;

    super::reference::validate_references_v1(super::reference::ReferenceInputV1 {
        original_case: &original_case,
        original_boundary,
        minimized_case: &minimized_case,
        minimized_boundary,
        fault,
        original_failure: &original_failure_parts,
        minimized_failure: &minimized_failure_parts,
        trace_events: &events,
        trace_boundary,
    })?;

    let original_failure = finish_failure_v1(original_failure_parts)?;
    require_scope(
        &original_failure,
        FailureScopeV1::Original,
        original_failure_line,
    )?;
    let minimized_failure = finish_failure_v1(minimized_failure_parts)?;
    require_scope(
        &minimized_failure,
        FailureScopeV1::Minimized,
        minimized_failure_line,
    )?;

    let terminal_index = usize::try_from(trace_boundary.end().number).map_err(|_| {
        ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::NonCanonicalValue,
            trace_boundary.end().number,
        )
    })?;
    if envelope.lines.len() > terminal_index + 1 {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::TrailingData,
            envelope.lines[terminal_index + 1].number,
        ));
    }

    Ok(FailureArtifactV1::new(
        fixture,
        replay_config,
        generator_config,
        seed,
        original_case,
        fault,
        original_failure.failure().clone(),
        reduction,
        minimized_case,
        minimized_failure.failure().clone(),
        events,
    ))
}

pub(super) fn parse_fixture(
    line: &ScannedLine<'_>,
) -> Result<ArtifactFixtureMetadataV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let [
        "fixture",
        "runtime-oracle",
        fixture,
        format,
        namespace,
        revision,
        construction,
    ] = fields.as_slice()
    else {
        return Err(malformed(line.number));
    };
    Ok(ArtifactFixtureMetadataV1::new(
        parse_u32(fixture, line.number)?,
        parse_u32(format, line.number)?,
        parse_u64(namespace, line.number)?,
        parse_u32(revision, line.number)?,
        parse_u32(construction, line.number)?,
    ))
}

pub(super) fn parse_replay_config(
    line: &ScannedLine<'_>,
) -> Result<ArtifactReplayConfigV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let [
        "replay",
        operations,
        structural,
        nodes,
        fragments,
        properties,
        retained,
    ] = fields.as_slice()
    else {
        return Err(malformed(line.number));
    };
    Ok(ArtifactReplayConfigV1::new([
        parse_u32(operations, line.number)?,
        parse_u32(structural, line.number)?,
        parse_u32(nodes, line.number)?,
        parse_u32(fragments, line.number)?,
        parse_u32(properties, line.number)?,
        parse_u32(retained, line.number)?,
    ]))
}

pub(super) fn parse_generator_config(
    line: &ScannedLine<'_>,
) -> Result<GeneratorConfigV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let ["generator", transactions, operations, memberships] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    Ok(GeneratorConfigV1::new(
        parse_u32(transactions, line.number)?,
        parse_u32(operations, line.number)?,
        parse_u32(memberships, line.number)?,
    ))
}

pub(super) fn parse_seed(line: &ScannedLine<'_>) -> Result<SeedV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let ["seed", seed] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    Ok(SeedV1::new(parse_u64(seed, line.number)?))
}

fn parse_case(
    boundary: SectionBoundaryV1<'_, '_>,
    context: CaseDecodeContextV1,
) -> Result<crate::case::GeneratedCaseV1, ArtifactDecodeError> {
    let declaration = parse_case_declaration(boundary.begin())?;
    let byte_count = records_byte_count(boundary.records())?;
    let case = decode_case_records_v1(boundary.records(), byte_count, context)?;
    check_count(
        declaration.transactions,
        case.transactions().len(),
        CountKind::Transactions,
        declaration.line,
    )?;
    check_count(
        declaration.operations,
        case.operation_count(),
        CountKind::Operations,
        declaration.line,
    )?;
    check_count(
        declaration.bytes,
        byte_count,
        CountKind::CaseBytes,
        declaration.line,
    )?;
    Ok(case)
}

pub(super) fn parse_case_declaration(
    line: &ScannedLine<'_>,
) -> Result<CaseDeclarationV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let [_, transactions, operations, bytes] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    Ok(CaseDeclarationV1 {
        transactions: parse_u32(transactions, line.number)?,
        operations: parse_u32(operations, line.number)?,
        bytes: parse_u32(bytes, line.number)?,
        line: line.number,
    })
}

pub(super) fn parse_fault(line: &ScannedLine<'_>) -> Result<TraceFaultV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let ["fault", "omit-move", target] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    Ok(TraceFaultV1::OmitMove {
        target: OperationIdV1::new(parse_u32(target, line.number)?),
    })
}

pub(super) fn parse_reduction(
    line: &ScannedLine<'_>,
) -> Result<ArtifactReductionV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let ["reducer", maximum, used, completion] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    let completion = match *completion {
        "fixed-point" => ReductionCompletionV1::FixedPoint,
        "budget-exhausted" => ReductionCompletionV1::BudgetExhausted,
        _ => return Err(malformed(line.number)),
    };
    Ok(ArtifactReductionV1::new(
        parse_u32(maximum, line.number)?,
        parse_u32(used, line.number)?,
        completion,
    ))
}

pub(super) fn parse_trace_declaration(
    line: &ScannedLine<'_>,
) -> Result<TraceDeclarationV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let ["trace-begin", events, bytes] = fields.as_slice() else {
        return Err(malformed(line.number));
    };
    Ok(TraceDeclarationV1 {
        events: parse_u32(events, line.number)?,
        bytes: parse_u32(bytes, line.number)?,
        line: line.number,
    })
}

fn require_scope(
    failure: &super::fingerprint::ParsedFailureV1,
    expected: FailureScopeV1,
    line: &ScannedLine<'_>,
) -> Result<(), ArtifactDecodeError> {
    if failure.scope() != expected {
        return Err(malformed(line.number));
    }
    Ok(())
}

pub(super) fn following_line<'scan, 'source>(
    envelope: &'scan EnvelopeBoundariesV1<'source>,
    boundary: SectionBoundaryV1<'_, 'source>,
    offset: usize,
) -> &'scan ScannedLine<'source> {
    &envelope.lines[boundary.end().number as usize + offset]
}

pub(super) fn records_byte_count(lines: &[ScannedLine<'_>]) -> Result<usize, ArtifactDecodeError> {
    lines.iter().try_fold(0_usize, |total, line| {
        total
            .checked_add(line.text.len() + 1)
            .ok_or_else(|| noncanonical(line.number))
    })
}

fn check_count(
    declared: u32,
    actual: usize,
    kind: CountKind,
    line: u32,
) -> Result<(), ArtifactDecodeError> {
    if usize::try_from(declared).ok() != Some(actual) {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::CountMismatch(kind),
            line,
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct CaseDeclarationV1 {
    pub(super) transactions: u32,
    pub(super) operations: u32,
    pub(super) bytes: u32,
    pub(super) line: u32,
}

#[derive(Clone, Copy)]
pub(super) struct TraceDeclarationV1 {
    pub(super) events: u32,
    pub(super) bytes: u32,
    pub(super) line: u32,
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}

fn noncanonical(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::NonCanonicalValue, line)
}

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};

use crate::case::{OperationIdV1, TransactionIdV1};
use crate::trace::{CandidateRejectionV1, TraceComparisonV1, TraceEventV1, TraceOutcomeV1};

use super::super::error::{
    ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactEncodeError, ArtifactLimitKind,
};
use super::super::primitive::{parse_u32, parse_u64};
use super::super::scan::ScannedLine;

const TRACE_BYTES_LIMIT: usize = 65_536;
const TRACE_EVENTS_LIMIT: usize = 64;

pub(super) fn parse_trace_records_v1(
    lines: &[ScannedLine<'_>],
) -> Result<Vec<TraceEventV1>, ArtifactDecodeError> {
    lines.iter().map(parse_event).collect()
}

pub(super) fn inspect_trace_records_v1(
    lines: &[ScannedLine<'_>],
) -> Result<(), ArtifactDecodeError> {
    for line in lines {
        let _ = parse_event(line)?;
    }
    Ok(())
}

pub(super) fn encode_trace_records_v1(
    events: &[TraceEventV1],
) -> Result<Vec<u8>, ArtifactEncodeError> {
    let bytes = crate::trace::encode_events_v1(events, TRACE_BYTES_LIMIT)
        .map_err(|_| ArtifactEncodeError::limit(ArtifactLimitKind::TraceBytes))?;
    if events.len() > TRACE_EVENTS_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::TraceEvents));
    }
    Ok(bytes)
}

fn parse_event(line: &ScannedLine<'_>) -> Result<TraceEventV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let [
        "event",
        sequence,
        transaction,
        operations,
        before_generation,
        after_generation,
        outcome,
        mutation_count,
        invalidation,
        comparison,
    ] = fields.as_slice()
    else {
        return Err(malformed(line.number));
    };

    let mutation_count = parse_u32(mutation_count, line.number)?;
    TraceEventV1::new(
        parse_u32(sequence, line.number)?,
        TransactionIdV1::new(parse_u32(transaction, line.number)?),
        parse_operation_ids(operations, line.number)?,
        parse_u64(before_generation, line.number)?,
        parse_u64(after_generation, line.number)?,
        parse_outcome(outcome, line.number)?,
        usize::try_from(mutation_count).map_err(|_| noncanonical(line.number))?,
        parse_invalidation(invalidation, line.number)?,
        parse_comparison(comparison, line.number)?,
    )
    .map_err(|_| noncanonical(line.number))
}

fn parse_operation_ids(value: &str, line: u32) -> Result<Vec<OperationIdV1>, ArtifactDecodeError> {
    if value.is_empty() || value == "-" {
        return Err(noncanonical(line));
    }
    value
        .split(',')
        .map(|value| parse_u32(value, line).map(OperationIdV1::new))
        .collect()
}

fn parse_outcome(value: &str, line: u32) -> Result<TraceOutcomeV1, ArtifactDecodeError> {
    match value {
        "commit" => Ok(TraceOutcomeV1::Commit),
        "noop" => Ok(TraceOutcomeV1::Noop),
        _ => value
            .strip_prefix("reject:")
            .and_then(parse_rejection)
            .map(TraceOutcomeV1::Reject)
            .ok_or_else(|| malformed(line)),
    }
}

fn parse_rejection(value: &str) -> Option<CandidateRejectionV1> {
    Some(match value {
        "capacity-operations" => CandidateRejectionV1::CapacityOperations,
        "capacity-structural" => CandidateRejectionV1::CapacityStructural,
        "capacity-live-nodes" => CandidateRejectionV1::CapacityLiveNodes,
        "capacity-live-fragments" => CandidateRejectionV1::CapacityLiveFragments,
        "capacity-live-properties" => CandidateRejectionV1::CapacityLiveProperties,
        "capacity-retained-generations" => CandidateRejectionV1::CapacityRetainedGenerations,
        "stale-base" => CandidateRejectionV1::StaleBase,
        "missing-node" => CandidateRejectionV1::MissingNode,
        "missing-fragment" => CandidateRejectionV1::MissingFragment,
        "missing-key" => CandidateRejectionV1::MissingKey,
        "duplicate-key" => CandidateRejectionV1::DuplicateKey,
        "unknown-property" => CandidateRejectionV1::UnknownProperty,
        "property-type-mismatch" => CandidateRejectionV1::PropertyTypeMismatch,
        "index-out-of-bounds" => CandidateRejectionV1::IndexOutOfBounds,
        "generation-exhausted" => CandidateRejectionV1::GenerationExhausted,
        "invariant-violation" => CandidateRejectionV1::InvariantViolation,
        _ => return None,
    })
}

fn parse_invalidation(value: &str, line: u32) -> Result<InvalidationSet, ArtifactDecodeError> {
    if value == "-" {
        return Ok(InvalidationSet::NONE);
    }
    if value.is_empty() {
        return Err(noncanonical(line));
    }

    let mut set = InvalidationSet::NONE;
    let mut previous = None;
    for word in value.split(',') {
        let class = parse_invalidation_class(word).ok_or_else(|| malformed(line))?;
        let rank = invalidation_rank(class);
        if previous.is_some_and(|previous| rank <= previous) {
            return Err(noncanonical(line));
        }
        previous = Some(rank);
        set = set.union(InvalidationSet::from_class(class));
    }
    Ok(set)
}

fn parse_invalidation_class(value: &str) -> Option<InvalidationClass> {
    Some(match value {
        "structure" => InvalidationClass::Structure,
        "style-match" => InvalidationClass::StyleMatch,
        "intrinsic" => InvalidationClass::Intrinsic,
        "layout" => InvalidationClass::Layout,
        "semantics" => InvalidationClass::Semantics,
        "hit-test" => InvalidationClass::HitTest,
        "paint" => InvalidationClass::Paint,
        "composition" => InvalidationClass::Composition,
        "surface" => InvalidationClass::Surface,
        _ => return None,
    })
}

const fn invalidation_rank(class: InvalidationClass) -> u8 {
    match class {
        InvalidationClass::Structure => 0,
        InvalidationClass::StyleMatch => 1,
        InvalidationClass::Intrinsic => 2,
        InvalidationClass::Layout => 3,
        InvalidationClass::Semantics => 4,
        InvalidationClass::HitTest => 5,
        InvalidationClass::Paint => 6,
        InvalidationClass::Composition => 7,
        InvalidationClass::Surface => 8,
    }
}

fn parse_comparison(value: &str, line: u32) -> Result<TraceComparisonV1, ArtifactDecodeError> {
    match value {
        "match" => Ok(TraceComparisonV1::Match),
        "mismatch" => Ok(TraceComparisonV1::Mismatch),
        _ => Err(malformed(line)),
    }
}

fn noncanonical(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::NonCanonicalValue, line)
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}

use super::super::case::{
    CASE_BYTES_LIMIT, CaseInspectionV1, OPERATIONS_LIMIT, TRANSACTIONS_LIMIT,
    inspect_case_records_v1,
};
use super::super::error::{
    ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactLimitKind, CountKind, VersionKind,
};
use super::super::primitive::parse_u32;
use super::boundary::{EnvelopeBoundariesV1, SectionBoundaryV1};
use super::decode::{
    CaseDeclarationV1, TraceDeclarationV1, following_line, parse_case_declaration, parse_fault,
    parse_fixture, parse_generator_config, parse_reduction, parse_replay_config, parse_seed,
    parse_trace_declaration, records_byte_count,
};
use super::fingerprint::parse_failure_parts_v1;
use super::grammar::{CaseRecordRoleV1, RecordKindV1, classify_line_v1};
use super::model::ArtifactReductionV1;

const TRACE_BYTES_LIMIT: usize = 65_536;
const TRACE_EVENTS_LIMIT: usize = 64;
const REDUCTION_EVALUATIONS_LIMIT: u32 = 4_096;

pub(super) fn preflight_envelope_v1(
    envelope: &EnvelopeBoundariesV1<'_>,
) -> Result<(), ArtifactDecodeError> {
    validate_nonempty_transactions_v1(envelope.original())?;
    validate_nonempty_transactions_v1(envelope.minimized())?;
    validate_version_canonicality_v1(envelope)?;
    let _ = parse_fixture(&envelope.lines[2])?;
    let _ = parse_replay_config(&envelope.lines[3])?;
    let _ = parse_generator_config(&envelope.lines[4])?;
    let _ = parse_seed(&envelope.lines[5])?;

    let original_boundary = envelope.original();
    let original = inspect_case_v1(original_boundary)?;
    let fault_line = following_line(envelope, original_boundary, 0);
    let original_failure_line = following_line(envelope, original_boundary, 1);
    let reduction_line = following_line(envelope, original_boundary, 2);
    let _ = parse_fault(fault_line)?;
    inspect_failure_v1(original_failure_line)?;
    let reduction = parse_reduction(reduction_line)?;

    let minimized_boundary = envelope.minimized();
    let minimized = inspect_case_v1(minimized_boundary)?;
    let minimized_failure_line = following_line(envelope, minimized_boundary, 0);
    inspect_failure_v1(minimized_failure_line)?;

    let trace_boundary = envelope.trace();
    let trace_declaration = parse_trace_declaration(trace_boundary.begin())?;
    let trace_bytes = records_byte_count(trace_boundary.records())?;
    super::trace::inspect_trace_records_v1(trace_boundary.records())?;
    let facts = EnvelopeLimitFactsV1 {
        original,
        minimized,
        trace_declaration,
        trace_actual_bytes: trace_bytes,
        trace_actual_events: trace_boundary.records().len(),
        reduction,
        reduction_line: reduction_line.number,
    };
    validate_limits_v1(&facts)?;
    validate_counts_v1(&facts)
}

fn validate_nonempty_transactions_v1(
    boundary: SectionBoundaryV1<'_, '_>,
) -> Result<(), ArtifactDecodeError> {
    let mut explicitly_empty = None;
    for line in boundary.records() {
        match classify_line_v1(line)? {
            RecordKindV1::Case(CaseRecordRoleV1::Transaction {
                explicitly_empty: true,
            }) => {
                if let Some(line) = explicitly_empty {
                    return Err(malformed(line));
                }
                explicitly_empty = Some(line.number);
            }
            RecordKindV1::Case(CaseRecordRoleV1::Transaction {
                explicitly_empty: false,
            }) => {
                if let Some(line) = explicitly_empty {
                    return Err(malformed(line));
                }
            }
            RecordKindV1::Case(CaseRecordRoleV1::Operation) => explicitly_empty = None,
            RecordKindV1::Marker { .. } | RecordKindV1::Trace => {
                return Err(malformed(line.number));
            }
        }
    }
    explicitly_empty.map_or(Ok(()), |line| Err(malformed(line)))
}

fn validate_version_canonicality_v1(
    envelope: &EnvelopeBoundariesV1<'_>,
) -> Result<(), ArtifactDecodeError> {
    let header = envelope.lines[0].text.split('|').collect::<Vec<_>>();
    let ["fenestra-oracle-failure", envelope_version] = header.as_slice() else {
        return Err(malformed(envelope.lines[0].number));
    };
    validate_version(
        envelope_version,
        VersionKind::Envelope,
        envelope.lines[0].number,
    )?;

    let versions_line = &envelope.lines[1];
    let versions = versions_line.text.split('|').collect::<Vec<_>>();
    let [
        "versions",
        "fixture",
        fixture,
        "generator",
        generator,
        "case",
        case,
        "state",
        state,
        "trace",
        trace,
        "fingerprint",
        fingerprint,
        "reducer",
        reducer,
    ] = versions.as_slice()
    else {
        return Err(malformed(versions_line.number));
    };
    for (value, kind) in [
        (*fixture, VersionKind::Fixture),
        (*generator, VersionKind::Generator),
        (*case, VersionKind::Case),
        (*state, VersionKind::State),
        (*trace, VersionKind::Trace),
        (*fingerprint, VersionKind::Fingerprint),
        (*reducer, VersionKind::Reducer),
    ] {
        validate_version(value, kind, versions_line.number)?;
    }
    Ok(())
}

fn inspect_case_v1(
    boundary: SectionBoundaryV1<'_, '_>,
) -> Result<CaseLimitFactsV1, ArtifactDecodeError> {
    let declaration = parse_case_declaration(boundary.begin())?;
    let bytes = records_byte_count(boundary.records())?;
    let inspection = inspect_case_records_v1(boundary.records(), bytes)?;
    Ok(CaseLimitFactsV1 {
        declaration,
        inspection,
    })
}

fn inspect_failure_v1(
    line: &super::super::scan::ScannedLine<'_>,
) -> Result<(), ArtifactDecodeError> {
    let _ = parse_failure_parts_v1(line)?;
    Ok(())
}

fn validate_limits_v1(facts: &EnvelopeLimitFactsV1) -> Result<(), ArtifactDecodeError> {
    for kind in ArtifactLimitKind::ALL {
        if let Some(line) = facts.crossing(kind) {
            return Err(ArtifactDecodeError::at(
                ArtifactDecodeErrorKind::LimitExceeded(kind),
                line,
            ));
        }
    }
    Ok(())
}

fn validate_counts_v1(facts: &EnvelopeLimitFactsV1) -> Result<(), ArtifactDecodeError> {
    validate_case_counts_v1(facts.original)?;
    validate_case_counts_v1(facts.minimized)?;
    check_count(
        facts.trace_declaration.events,
        facts.trace_actual_events,
        CountKind::TraceEvents,
        facts.trace_declaration.line,
    )?;
    check_count(
        facts.trace_declaration.bytes,
        facts.trace_actual_bytes,
        CountKind::TraceBytes,
        facts.trace_declaration.line,
    )
}

fn validate_case_counts_v1(case: CaseLimitFactsV1) -> Result<(), ArtifactDecodeError> {
    check_count(
        case.declaration.transactions,
        case.inspection.transaction_count(),
        CountKind::Transactions,
        case.declaration.line,
    )?;
    check_count(
        case.declaration.operations,
        case.inspection.operation_count(),
        CountKind::Operations,
        case.declaration.line,
    )?;
    check_count(
        case.declaration.bytes,
        case.inspection.byte_count(),
        CountKind::CaseBytes,
        case.declaration.line,
    )?;
    if let Some(line) = case.inspection.transaction_count_mismatch_line() {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::CountMismatch(CountKind::OperationsPerTransaction),
            line,
        ));
    }
    Ok(())
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
struct CaseLimitFactsV1 {
    declaration: CaseDeclarationV1,
    inspection: CaseInspectionV1,
}

struct EnvelopeLimitFactsV1 {
    original: CaseLimitFactsV1,
    minimized: CaseLimitFactsV1,
    trace_declaration: TraceDeclarationV1,
    trace_actual_bytes: usize,
    trace_actual_events: usize,
    reduction: ArtifactReductionV1,
    reduction_line: u32,
}

impl EnvelopeLimitFactsV1 {
    fn crossing(&self, kind: ArtifactLimitKind) -> Option<u32> {
        match kind {
            ArtifactLimitKind::ArtifactBytes
            | ArtifactLimitKind::LineBytes
            | ArtifactLimitKind::Lines => None,
            ArtifactLimitKind::CaseBytes => self
                .case_bytes_crossing(self.original)
                .or_else(|| self.case_bytes_crossing(self.minimized)),
            ArtifactLimitKind::TraceBytes => ((self.trace_declaration.bytes as u64
                > TRACE_BYTES_LIMIT as u64)
                || self.trace_actual_bytes > TRACE_BYTES_LIMIT)
                .then_some(self.trace_declaration.line),
            ArtifactLimitKind::Transactions => self
                .transactions_crossing(self.original)
                .or_else(|| self.transactions_crossing(self.minimized)),
            ArtifactLimitKind::OperationsPerTransaction => self
                .original
                .inspection
                .operations_per_transaction_limit_line()
                .or_else(|| {
                    self.minimized
                        .inspection
                        .operations_per_transaction_limit_line()
                }),
            ArtifactLimitKind::Operations => self
                .operations_crossing(self.original)
                .or_else(|| self.operations_crossing(self.minimized)),
            ArtifactLimitKind::PathDepth => self
                .original
                .inspection
                .path_depth_limit_line()
                .or_else(|| self.minimized.inspection.path_depth_limit_line()),
            ArtifactLimitKind::TraceEvents => ((self.trace_declaration.events as u64
                > TRACE_EVENTS_LIMIT as u64)
                || self.trace_actual_events > TRACE_EVENTS_LIMIT)
                .then_some(self.trace_declaration.line),
            ArtifactLimitKind::ReductionEvaluations => ((self.reduction.max_evaluations()
                > REDUCTION_EVALUATIONS_LIMIT)
                || (self.reduction.used_evaluations() > REDUCTION_EVALUATIONS_LIMIT))
                .then_some(self.reduction_line),
        }
    }

    fn case_bytes_crossing(&self, case: CaseLimitFactsV1) -> Option<u32> {
        ((case.declaration.bytes as u64 > CASE_BYTES_LIMIT as u64)
            || case.inspection.byte_count() > CASE_BYTES_LIMIT)
            .then_some(case.declaration.line)
    }

    fn transactions_crossing(&self, case: CaseLimitFactsV1) -> Option<u32> {
        (case.declaration.transactions as u64 > TRANSACTIONS_LIMIT as u64)
            .then_some(case.declaration.line)
            .or_else(|| case.inspection.transactions_limit_line())
    }

    fn operations_crossing(&self, case: CaseLimitFactsV1) -> Option<u32> {
        (case.declaration.operations as u64 > OPERATIONS_LIMIT as u64)
            .then_some(case.declaration.line)
            .or_else(|| case.inspection.operations_limit_line())
    }
}

fn validate_version(value: &str, kind: VersionKind, line: u32) -> Result<(), ArtifactDecodeError> {
    if parse_u32(value, line)? != 1 {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::UnsupportedVersion(kind),
            line,
        ));
    }
    Ok(())
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}

use fenestra_ui_ir::prototype::{ComponentTypeId, PropertyId, TemplateNodeId};

use super::super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind};
use super::super::path::{parse_fragment_path, parse_node_path};
use super::super::primitive::{parse_property_value, parse_u32, parse_u64};
use super::super::scan::ScannedLine;
use crate::case::{OperationIdV1, TransactionIdV1};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1,
};
use crate::semantic::NormalizedChildGroupV1;
use crate::trace::CandidateRejectionV1;

mod encode;

pub(super) use encode::write_failure_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FailureScopeV1 {
    Original,
    Minimized,
}

pub(super) struct ParsedFailureV1 {
    scope: FailureScopeV1,
    failure: ReplayFailureV1,
}

impl ParsedFailureV1 {
    pub(super) const fn scope(&self) -> FailureScopeV1 {
        self.scope
    }

    pub(super) const fn failure(&self) -> &ReplayFailureV1 {
        &self.failure
    }
}

pub(super) struct ParsedFailurePartsV1 {
    scope: FailureScopeV1,
    transaction: TransactionIdV1,
    operation: Option<OperationIdV1>,
    kind: FailureFingerprintKindV1,
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
    line: u32,
}

impl ParsedFailurePartsV1 {
    pub(super) const fn scope(&self) -> FailureScopeV1 {
        self.scope
    }

    pub(super) const fn transaction(&self) -> TransactionIdV1 {
        self.transaction
    }

    pub(super) const fn operation(&self) -> Option<OperationIdV1> {
        self.operation
    }
}

pub(super) fn parse_failure_parts_v1(
    line: &ScannedLine<'_>,
) -> Result<ParsedFailurePartsV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    let [
        "failure",
        scope,
        transaction,
        operation,
        kind,
        location,
        field,
        expected,
        observed,
    ] = fields.as_slice()
    else {
        return Err(malformed(line.number));
    };
    let scope = parse_scope(scope, line.number)?;
    let transaction = TransactionIdV1::new(parse_u32(transaction, line.number)?);
    let operation = parse_optional_operation(operation, line.number)?;
    let kind = parse_kind(kind, line.number)?;
    let location = parse_location(location, line.number)?;
    let field = parse_field(field, line.number)?;
    let expected = parse_summary(expected, line.number)?;
    let observed = parse_summary(observed, line.number)?;
    Ok(ParsedFailurePartsV1 {
        scope,
        transaction,
        operation,
        kind,
        location,
        field,
        expected,
        observed,
        line: line.number,
    })
}

pub(super) fn finish_failure_v1(
    parts: ParsedFailurePartsV1,
) -> Result<ParsedFailureV1, ArtifactDecodeError> {
    let scope = parts.scope();
    let transaction = parts.transaction();
    let operation = parts.operation();
    if operation.is_some() && parts.kind != FailureFingerprintKindV1::CandidateRejected {
        return Err(invalid_fingerprint(parts.line));
    }
    let fingerprint = FailureFingerprintV1::from_parts(
        parts.kind,
        parts.location,
        parts.field,
        parts.expected,
        parts.observed,
    )
    .ok_or_else(|| invalid_fingerprint(parts.line))?;
    Ok(ParsedFailureV1 {
        scope,
        failure: ReplayFailureV1::new(transaction, operation, fingerprint),
    })
}

fn parse_scope(value: &str, line: u32) -> Result<FailureScopeV1, ArtifactDecodeError> {
    match value {
        "original" => Ok(FailureScopeV1::Original),
        "minimized" => Ok(FailureScopeV1::Minimized),
        _ => Err(malformed(line)),
    }
}

fn parse_optional_operation(
    value: &str,
    line: u32,
) -> Result<Option<OperationIdV1>, ArtifactDecodeError> {
    if value == "-" {
        Ok(None)
    } else {
        parse_u32(value, line).map(OperationIdV1::new).map(Some)
    }
}

fn parse_kind(value: &str, line: u32) -> Result<FailureFingerprintKindV1, ArtifactDecodeError> {
    match value {
        "candidate-rejected" => Ok(FailureFingerprintKindV1::CandidateRejected),
        "state-mismatch" => Ok(FailureFingerprintKindV1::StateMismatch),
        "identity-mismatch" => Ok(FailureFingerprintKindV1::IdentityMismatch),
        _ => Err(malformed(line)),
    }
}

fn parse_location(value: &str, line: u32) -> Result<FingerprintLocationV1, ArtifactDecodeError> {
    if value == "global" {
        Ok(FingerprintLocationV1::Global)
    } else if let Some(path) = value.strip_prefix("node:") {
        parse_node_path(path, line).map(FingerprintLocationV1::Node)
    } else if let Some(path) = value.strip_prefix("fragment:") {
        parse_fragment_path(path, line).map(FingerprintLocationV1::Fragment)
    } else {
        Err(malformed(line))
    }
}

fn parse_field(value: &str, line: u32) -> Result<FingerprintFieldV1, ArtifactDecodeError> {
    match value {
        "candidate-outcome" => Ok(FingerprintFieldV1::CandidateOutcome),
        "template" => Ok(FingerprintFieldV1::Template),
        "component" => Ok(FingerprintFieldV1::Component),
        "property" => Ok(FingerprintFieldV1::Property),
        "parent" => Ok(FingerprintFieldV1::Parent),
        "child-order" => Ok(FingerprintFieldV1::ChildOrder),
        "fragment-binding" => Ok(FingerprintFieldV1::FragmentBinding),
        "keyed-order" => Ok(FingerprintFieldV1::KeyedOrder),
        "node-count" => Ok(FingerprintFieldV1::NodeCount),
        "fragment-count" => Ok(FingerprintFieldV1::FragmentCount),
        "property-count" => Ok(FingerprintFieldV1::PropertyCount),
        "identity-lifecycle" => Ok(FingerprintFieldV1::IdentityLifecycle),
        _ => Err(malformed(line)),
    }
}

fn parse_summary(value: &str, line: u32) -> Result<FingerprintSummaryV1, ArtifactDecodeError> {
    match value {
        "none" => Ok(FingerprintSummaryV1::None),
        "binding:present" => Ok(FingerprintSummaryV1::BindingPresent),
        "binding:absent" => Ok(FingerprintSummaryV1::BindingAbsent),
        "kind:accept" => Ok(FingerprintSummaryV1::CandidateAccepted),
        "absent" => Ok(FingerprintSummaryV1::LifecycleAbsent),
        "preserved" => Ok(FingerprintSummaryV1::LifecyclePreserved),
        "fresh" => Ok(FingerprintSummaryV1::LifecycleFresh),
        "retired" => Ok(FingerprintSummaryV1::LifecycleRetired),
        "distinct" => Ok(FingerprintSummaryV1::LifecycleDistinct),
        "aliased" => Ok(FingerprintSummaryV1::LifecycleAliased),
        _ => parse_prefixed_summary(value, line),
    }
}

fn parse_prefixed_summary(
    value: &str,
    line: u32,
) -> Result<FingerprintSummaryV1, ArtifactDecodeError> {
    let (kind, payload) = value.split_once(':').ok_or_else(|| malformed(line))?;
    match kind {
        "count" => parse_u32(payload, line).map(FingerprintSummaryV1::Count),
        "template" => parse_u32(payload, line)
            .map(TemplateNodeId::new)
            .map(FingerprintSummaryV1::Template),
        "component" => parse_u32(payload, line)
            .map(ComponentTypeId::new)
            .map(FingerprintSummaryV1::Component),
        "property" => parse_property_summary(payload, line),
        "node" => parse_node_path(payload, line).map(FingerprintSummaryV1::Node),
        "nodes" => parse_list(payload, line, parse_node_path).map(FingerprintSummaryV1::Nodes),
        "children" => parse_list(payload, line, parse_child).map(FingerprintSummaryV1::Children),
        "keys" => parse_list(payload, line, parse_u64).map(FingerprintSummaryV1::Keys),
        "kind" => parse_rejection(payload, line).map(FingerprintSummaryV1::CandidateRejected),
        "binding" => Err(malformed(line)),
        _ => Err(malformed(line)),
    }
}

fn parse_property_summary(
    value: &str,
    line: u32,
) -> Result<FingerprintSummaryV1, ArtifactDecodeError> {
    let (property, value) = value.split_once(':').ok_or_else(|| malformed(line))?;
    Ok(FingerprintSummaryV1::Property(
        PropertyId::new(parse_u32(property, line)?),
        parse_property_value(value, line)?,
    ))
}

fn parse_child(value: &str, line: u32) -> Result<NormalizedChildGroupV1, ArtifactDecodeError> {
    if let Some(path) = value.strip_prefix("s:") {
        parse_node_path(path, line).map(NormalizedChildGroupV1::Static)
    } else if let Some(path) = value.strip_prefix("r:") {
        parse_fragment_path(path, line).map(NormalizedChildGroupV1::Region)
    } else {
        Err(malformed(line))
    }
}

fn parse_list<T>(
    value: &str,
    line: u32,
    parse: impl Fn(&str, u32) -> Result<T, ArtifactDecodeError>,
) -> Result<Vec<T>, ArtifactDecodeError> {
    if value == "-" {
        return Ok(Vec::new());
    }
    if value.is_empty() {
        return Err(noncanonical(line));
    }
    value
        .split(',')
        .map(|item| {
            if item.is_empty() {
                Err(noncanonical(line))
            } else {
                parse(item, line)
            }
        })
        .collect()
}

fn parse_rejection(value: &str, line: u32) -> Result<CandidateRejectionV1, ArtifactDecodeError> {
    match value {
        "capacity-operations" => Ok(CandidateRejectionV1::CapacityOperations),
        "capacity-structural" => Ok(CandidateRejectionV1::CapacityStructural),
        "capacity-live-nodes" => Ok(CandidateRejectionV1::CapacityLiveNodes),
        "capacity-live-fragments" => Ok(CandidateRejectionV1::CapacityLiveFragments),
        "capacity-live-properties" => Ok(CandidateRejectionV1::CapacityLiveProperties),
        "capacity-retained-generations" => Ok(CandidateRejectionV1::CapacityRetainedGenerations),
        "stale-base" => Ok(CandidateRejectionV1::StaleBase),
        "missing-node" => Ok(CandidateRejectionV1::MissingNode),
        "missing-fragment" => Ok(CandidateRejectionV1::MissingFragment),
        "missing-key" => Ok(CandidateRejectionV1::MissingKey),
        "duplicate-key" => Ok(CandidateRejectionV1::DuplicateKey),
        "unknown-property" => Ok(CandidateRejectionV1::UnknownProperty),
        "property-type-mismatch" => Ok(CandidateRejectionV1::PropertyTypeMismatch),
        "index-out-of-bounds" => Ok(CandidateRejectionV1::IndexOutOfBounds),
        "generation-exhausted" => Ok(CandidateRejectionV1::GenerationExhausted),
        "invariant-violation" => Ok(CandidateRejectionV1::InvariantViolation),
        _ => Err(malformed(line)),
    }
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}

fn noncanonical(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::NonCanonicalValue, line)
}

fn invalid_fingerprint(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::InvalidFingerprint, line)
}

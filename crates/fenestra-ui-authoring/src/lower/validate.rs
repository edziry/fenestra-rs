use fenestra_ui_ir::prototype::{
    ConstructionProgram, IrValidationError, SchemaManifest, SourceId, SourceSpan, StyleProgram,
    StyleValidationLimits, ValidationLimits, validate_construction, validate_schema,
    validate_style,
};

use crate::compiled::{SourceMapEntryV1, SourceMapV1};
use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::parsed::ParsedDocumentV1;
use crate::resolved::logical_span;

use super::failure;

#[cfg(test)]
mod tests;

const REGISTERED_TEMPLATE_DEPTH: usize = 3;
const REGISTERED_INITIAL_INSTANCES: usize = 5;

pub(super) fn validate_programs(
    parsed: &ParsedDocumentV1,
    schema: &SchemaManifest,
    construction: &ConstructionProgram,
    style: &StyleProgram,
    limits: AuthoringLimitsV1,
) -> Result<(), AuthoringDiagnosticV1> {
    let ir_limits = ValidationLimits::new(
        limits.limit(AuthoringLimitKindV1::Components),
        limits.limit(AuthoringLimitKindV1::Properties),
        limits.limit(AuthoringLimitKindV1::Templates),
        limits.limit(AuthoringLimitKindV1::Regions),
        limits.limit(AuthoringLimitKindV1::ChildSlots),
        limits.limit(AuthoringLimitKindV1::InitialProperties),
        limits.limit(AuthoringLimitKindV1::InitialKeys),
        REGISTERED_TEMPLATE_DEPTH,
        REGISTERED_INITIAL_INSTANCES,
    );
    let validated_schema =
        validate_schema(schema.clone(), ir_limits).map_err(|error| ir_failure(parsed, error))?;
    let validated_construction =
        validate_construction(&validated_schema, construction.clone(), ir_limits)
            .map_err(|error| ir_failure(parsed, error))?;
    validate_style(
        &validated_construction,
        style.clone(),
        StyleValidationLimits::new(limits.limit(AuthoringLimitKindV1::StyleAssignments)),
    )
    .map_err(|error| ir_failure(parsed, error))?;
    Ok(())
}

fn ir_failure(parsed: &ParsedDocumentV1, error: IrValidationError) -> AuthoringDiagnosticV1 {
    let ordinal = checked_ir_ordinal(parsed, error.span())
        .expect("lowered IR error span must map to a registered non-document anchor");
    failure(
        parsed,
        ordinal,
        AuthoringDiagnosticKindV1::IrValidation(error.kind()),
    )
}

#[derive(Debug)]
struct IrSpanInvariantV1;

fn checked_ir_ordinal(
    parsed: &ParsedDocumentV1,
    span: SourceSpan,
) -> Result<u32, IrSpanInvariantV1> {
    let SourceSpan::Bytes { source, start, end } = span else {
        return Err(IrSpanInvariantV1);
    };
    let expected_end = start.checked_add(1).ok_or(IrSpanInvariantV1)?;
    let ordinal = usize::try_from(start).map_err(|_| IrSpanInvariantV1)?;
    if source != SourceId::new(0)
        || end != expected_end
        || start == parsed.document_anchor
        || ordinal >= parsed.anchors.len()
    {
        return Err(IrSpanInvariantV1);
    }
    Ok(start)
}

pub(super) fn source_map(parsed: &ParsedDocumentV1) -> SourceMapV1 {
    let entries = parsed
        .anchors
        .iter()
        .enumerate()
        .map(|(ordinal, anchor)| {
            SourceMapEntryV1::new(
                logical_span(ordinal as u32),
                anchor.kind,
                anchor.label.clone(),
                anchor.physical,
            )
        })
        .collect();
    SourceMapV1::new(entries)
}

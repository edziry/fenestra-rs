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
    let ordinal = match error.span() {
        SourceSpan::Bytes { source, start, end }
            if source == SourceId::new(0) && end == start + 1 =>
        {
            usize::try_from(start).ok()
        }
        SourceSpan::Synthetic | SourceSpan::Bytes { .. } => None,
    }
    .filter(|ordinal| *ordinal < parsed.anchors.len())
    .unwrap_or(parsed.document_anchor as usize);
    failure(
        parsed,
        ordinal as u32,
        AuthoringDiagnosticKindV1::IrValidation(error.kind()),
    )
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

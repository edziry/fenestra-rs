use fenestra_ui_ir::prototype::{
    ConstructionProgram, IrValidationError, SchemaManifest, SourceId, SourceSpan, SpatialProgramV2,
    SpatialValidationLimitsV2, StyleProgram, StyleValidationLimits, ValidatedStyleProgram,
    ValidationLimits, validate_construction, validate_schema, validate_spatial, validate_style,
};

use crate::compiled_v2::{SourceMapEntryV2, SourceMapV2};
use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
use crate::parsed_v2::ParsedDocumentV2;
use crate::resolved::logical_span;

use super::failure;

const REGISTERED_TEMPLATE_DEPTH_V2: usize = 4;
const REGISTERED_INITIAL_INSTANCES_V2: usize = 8;

pub(crate) fn validate_logical(
    parsed: &ParsedDocumentV2,
    schema: &SchemaManifest,
    construction: &ConstructionProgram,
    style: &StyleProgram,
    limits: AuthoringLimitsV2,
) -> Result<ValidatedStyleProgram, AuthoringDiagnosticV2> {
    let ir_limits = ValidationLimits::new(
        limits.limit(AuthoringLimitKindV2::Components),
        limits.limit(AuthoringLimitKindV2::Properties),
        limits.limit(AuthoringLimitKindV2::Templates),
        limits.limit(AuthoringLimitKindV2::Regions),
        limits.limit(AuthoringLimitKindV2::ChildSlots),
        limits.limit(AuthoringLimitKindV2::InitialProperties),
        limits.limit(AuthoringLimitKindV2::InitialKeys),
        REGISTERED_TEMPLATE_DEPTH_V2,
        REGISTERED_INITIAL_INSTANCES_V2,
    );
    let schema =
        validate_schema(schema.clone(), ir_limits).map_err(|error| ir_failure(parsed, error))?;
    let construction = validate_construction(&schema, construction.clone(), ir_limits)
        .map_err(|error| ir_failure(parsed, error))?;
    validate_style(
        &construction,
        style.clone(),
        StyleValidationLimits::new(limits.limit(AuthoringLimitKindV2::StyleAssignments)),
    )
    .map_err(|error| ir_failure(parsed, error))
}

pub(crate) fn validate_spatial_program(
    parsed: &ParsedDocumentV2,
    style: &ValidatedStyleProgram,
    spatial: &SpatialProgramV2,
    limits: AuthoringLimitsV2,
) -> Result<(), AuthoringDiagnosticV2> {
    let limits = SpatialValidationLimitsV2::new([
        limits.limit(AuthoringLimitKindV2::SpatialNodes),
        limits.limit(AuthoringLimitKindV2::Shapes),
        limits.limit(AuthoringLimitKindV2::Brushes),
        limits.limit(AuthoringLimitKindV2::Clips),
        limits.limit(AuthoringLimitKindV2::PaintItems),
        limits.limit(AuthoringLimitKindV2::HitItems),
        limits.limit(AuthoringLimitKindV2::SemanticItems),
        limits.limit(AuthoringLimitKindV2::Paths),
        limits.limit(AuthoringLimitKindV2::PathVerbs),
        limits.limit(AuthoringLimitKindV2::PolygonPoints),
        limits.limit(AuthoringLimitKindV2::GradientStops),
        limits.limit(AuthoringLimitKindV2::Images),
        limits.limit(AuthoringLimitKindV2::ImageBytes),
    ]);
    validate_spatial(style, spatial.clone(), limits)
        .map(|_| ())
        .map_err(|error| ir_failure(parsed, error))
}

fn ir_failure(parsed: &ParsedDocumentV2, error: IrValidationError) -> AuthoringDiagnosticV2 {
    let ordinal = checked_ir_ordinal(parsed, error.span())
        .expect("lowered V2 IR error span must map to a registered non-document anchor");
    failure(
        parsed,
        ordinal,
        AuthoringDiagnosticKindV2::IrValidation(error.kind()),
    )
}

#[derive(Debug)]
struct IrSpanInvariantV2;

fn checked_ir_ordinal(
    parsed: &ParsedDocumentV2,
    span: SourceSpan,
) -> Result<u32, IrSpanInvariantV2> {
    let SourceSpan::Bytes { source, start, end } = span else {
        return Err(IrSpanInvariantV2);
    };
    let expected_end = start.checked_add(1).ok_or(IrSpanInvariantV2)?;
    let ordinal = usize::try_from(start).map_err(|_| IrSpanInvariantV2)?;
    if source != SourceId::new(0)
        || end != expected_end
        || start == parsed.document_anchor
        || ordinal >= parsed.anchors.len()
    {
        return Err(IrSpanInvariantV2);
    }
    Ok(start)
}

pub(crate) fn source_map(parsed: &ParsedDocumentV2) -> SourceMapV2 {
    SourceMapV2::new(
        parsed
            .anchors
            .iter()
            .enumerate()
            .map(|(ordinal, anchor)| {
                SourceMapEntryV2::new(
                    logical_span(ordinal as u32),
                    anchor.kind,
                    anchor.label.clone(),
                    anchor.physical,
                )
            })
            .collect(),
    )
}

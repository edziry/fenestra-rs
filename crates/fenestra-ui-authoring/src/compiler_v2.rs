use fenestra_ui_ir::prototype::SourceId;
use proc_macro2::TokenStream;

use crate::compiled_v2::CompiledAuthoringV2;
use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::fen_v2::parse_fen_document_v2;
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
use crate::lower_v2::{source_map, validate_logical, validate_spatial_program};
use crate::parsed_v2::ParsedDocumentV2;
use crate::resolved_v2::ResolvedDocumentV2;
use crate::source_v2::{DiagnosticLocationV2, FenSourceV2, PhysicalOriginV2};
use crate::ui_v2::parse_ui_document_v2;
use crate::vocabulary_v2::AuthoringFrontendV2;

/// Compiles one bounded version-2 `.fen` document into the typed IR quadruple.
///
/// # Errors
///
/// Returns the first bounded lexical, grammar, name-resolution, type, or IR
/// validation diagnostic in deterministic validation order.
pub fn compile_fen_v2(
    source: FenSourceV2<'_>,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> {
    let source_id = source.source();
    let bytes = source.bytes();
    let maximum = limits
        .limit(AuthoringLimitKindV2::FenSourceBytes)
        .min(u32::MAX as usize);
    if bytes.len() > maximum {
        return Err(fen_failure(
            source_id,
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::FenSourceBytes),
            maximum,
            maximum.saturating_add(1),
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|error| {
        let start = error.valid_up_to();
        let width = error.error_len().unwrap_or_else(|| bytes.len() - start);
        fen_failure(
            source_id,
            AuthoringDiagnosticKindV2::InvalidUtf8,
            start,
            start.saturating_add(width),
        )
    })?;
    if text.is_empty() {
        return Err(fen_failure(
            source_id,
            AuthoringDiagnosticKindV2::UnexpectedEof,
            0,
            0,
        ));
    }
    let parsed = parse_fen_document_v2(source_id, text, limits)?;
    lower_document_v2(parsed, limits)
}

/// Compiles one bounded version-2 `ui!` stream into the typed IR quadruple.
///
/// # Errors
///
/// Returns the first bounded lexical, grammar, name-resolution, type, or IR
/// validation diagnostic in deterministic validation order.
pub fn compile_ui_v2(
    tokens: TokenStream,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> {
    let parsed = parse_ui_document_v2(tokens, limits)?;
    lower_document_v2(parsed, limits)
}

fn lower_document_v2(
    parsed: ParsedDocumentV2,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> {
    let core = crate::lower_v2::resolve_logical(&parsed)?;
    let (schema, construction, style) = core.raw_programs();
    let validated_style = validate_logical(&parsed, &schema, &construction, &style, limits)?;
    let spatial = crate::lower_v2::spatial::resolve_spatial(&parsed, &core)?;
    validate_spatial_program(&parsed, &validated_style, &spatial, limits)?;
    assert_eq!(parsed.spatial.field_count, spatial_field_count(&spatial));
    let map = source_map(&parsed);
    let catalog = vec![b'@'; parsed.anchors.len()];
    let document_origin = parsed.anchors[parsed.document_anchor as usize].physical;
    let resolved =
        ResolvedDocumentV2::new(core, parsed.spatial.anchor, parsed.spatial.resources_anchor);
    Ok(CompiledAuthoringV2::new(
        parsed.frontend,
        document_origin,
        schema,
        construction,
        style,
        spatial,
        catalog,
        map,
        resolved,
    ))
}

fn spatial_field_count(program: &fenestra_ui_ir::prototype::SpatialProgramV2) -> usize {
    crate::spatial_field_count_v2::spatial_field_count(program)
}

fn fen_failure(
    source: SourceId,
    kind: AuthoringDiagnosticKindV2,
    start: usize,
    end: usize,
) -> AuthoringDiagnosticV2 {
    AuthoringDiagnosticV2::new(
        AuthoringFrontendV2::Fen,
        kind,
        DiagnosticLocationV2::Physical(PhysicalOriginV2::fen_bytes(
            source,
            bounded_offset(start),
            bounded_offset(end),
        )),
    )
}

fn bounded_offset(offset: usize) -> u32 {
    u32::try_from(offset).unwrap_or(u32::MAX)
}

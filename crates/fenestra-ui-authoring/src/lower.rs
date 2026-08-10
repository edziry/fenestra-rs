use crate::compiled::CompiledAuthoringV1;
use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::AuthoringLimitsV1;
use crate::parsed::ParsedDocumentV1;
use crate::resolved::logical_span;
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};

mod semantics;
mod validate;

use semantics::resolve_semantics;
use validate::{source_map, validate_programs};

pub(crate) fn lower_document_v1(
    parsed: ParsedDocumentV1,
    limits: AuthoringLimitsV1,
) -> Result<CompiledAuthoringV1, AuthoringDiagnosticV1> {
    let resolved = resolve_semantics(&parsed)?;
    let (schema, construction, style) = resolved.raw_programs();
    validate_programs(&parsed, &schema, &construction, &style, limits)?;
    let source_map = source_map(&parsed);
    let catalog = vec![b'@'; parsed.anchors.len()];
    let document_origin = parsed.anchors[parsed.document_anchor as usize].physical;
    Ok(CompiledAuthoringV1::new(
        parsed.frontend,
        document_origin,
        (schema, construction, style),
        catalog,
        source_map,
        resolved,
    ))
}

pub(super) fn failure(
    parsed: &ParsedDocumentV1,
    ordinal: u32,
    kind: AuthoringDiagnosticKindV1,
) -> AuthoringDiagnosticV1 {
    let anchor = &parsed.anchors[ordinal as usize];
    failure_at_origin(parsed, ordinal, kind, anchor.physical)
}

pub(super) fn failure_at_origin(
    parsed: &ParsedDocumentV1,
    ordinal: u32,
    kind: AuthoringDiagnosticKindV1,
    physical: PhysicalOriginV1,
) -> AuthoringDiagnosticV1 {
    let anchor = &parsed.anchors[ordinal as usize];
    AuthoringDiagnosticV1::new(
        parsed.frontend,
        kind,
        DiagnosticLocationV1::Anchored {
            logical: logical_span(ordinal),
            anchor_kind: anchor.kind,
            physical,
        },
    )
}

mod logical;
pub(crate) mod spatial;
mod validate;

pub(crate) use logical::resolve_logical;
pub(crate) use validate::{source_map, validate_logical, validate_spatial_program};

use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::parsed_v2::ParsedDocumentV2;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};

pub(crate) fn failure(
    parsed: &ParsedDocumentV2,
    ordinal: u32,
    kind: AuthoringDiagnosticKindV2,
) -> AuthoringDiagnosticV2 {
    let anchor = &parsed.anchors[ordinal as usize];
    failure_at_origin(parsed, ordinal, kind, anchor.physical)
}

pub(crate) fn failure_at_origin(
    parsed: &ParsedDocumentV2,
    ordinal: u32,
    kind: AuthoringDiagnosticKindV2,
    physical: PhysicalOriginV2,
) -> AuthoringDiagnosticV2 {
    let anchor = &parsed.anchors[ordinal as usize];
    AuthoringDiagnosticV2::new(
        parsed.frontend,
        kind,
        DiagnosticLocationV2::Anchored {
            logical: SourceSpan::bytes(SourceId::new(0), ordinal, ordinal.saturating_add(1)),
            anchor_kind: anchor.kind,
            physical,
        },
    )
}

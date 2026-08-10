use std::fmt;

use fenestra_ui_ir::prototype::{ConstructionProgram, SchemaManifest, SourceSpan, StyleProgram};

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::AuthoringLimitKindV1;
use crate::resolved::ResolvedDocumentV1;
use crate::resolved::logical_span;
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};
use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};

/// One host-only mapping from a logical IR anchor to its authored origin.
pub struct SourceMapEntryV1 {
    logical_span: SourceSpan,
    anchor_kind: AnchorKindV1,
    canonical_label: Box<str>,
    physical_origin: PhysicalOriginV1,
}

impl SourceMapEntryV1 {
    pub(crate) fn new(
        logical_span: SourceSpan,
        anchor_kind: AnchorKindV1,
        canonical_label: Box<str>,
        physical_origin: PhysicalOriginV1,
    ) -> Self {
        Self {
            logical_span,
            anchor_kind,
            canonical_label,
            physical_origin,
        }
    }

    /// Returns the logical byte span retained by the lowered IR record.
    #[must_use]
    pub const fn logical_span(&self) -> SourceSpan {
        self.logical_span
    }

    /// Returns the semantic role of the mapped record.
    #[must_use]
    pub const fn anchor_kind(&self) -> AnchorKindV1 {
        self.anchor_kind
    }

    /// Returns the normalized spelling of the defining authored token.
    #[must_use]
    pub fn canonical_label(&self) -> &str {
        &self.canonical_label
    }

    /// Returns the frontend-specific authored origin.
    #[must_use]
    pub const fn physical_origin(&self) -> &PhysicalOriginV1 {
        &self.physical_origin
    }
}

/// Host-only logical-to-physical source map for one compilation.
pub struct SourceMapV1 {
    entries: Vec<SourceMapEntryV1>,
}

impl SourceMapV1 {
    pub(crate) const fn new(entries: Vec<SourceMapEntryV1>) -> Self {
        Self { entries }
    }

    /// Returns entries in authored semantic-record order.
    #[must_use]
    pub fn entries(&self) -> &[SourceMapEntryV1] {
        &self.entries
    }
}

impl fmt::Debug for SourceMapV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceMapV1")
            .field("entries", &self.entries.len())
            .finish()
    }
}

/// Opaque result of a successful version-1 authoring compilation.
pub struct CompiledAuthoringV1 {
    frontend: AuthoringFrontendV1,
    document_origin: PhysicalOriginV1,
    schema: SchemaManifest,
    construction: ConstructionProgram,
    style: StyleProgram,
    logical_source_catalog: Vec<u8>,
    source_map: SourceMapV1,
    resolved: ResolvedDocumentV1,
}

impl CompiledAuthoringV1 {
    pub(crate) fn new(
        frontend: AuthoringFrontendV1,
        document_origin: PhysicalOriginV1,
        programs: (SchemaManifest, ConstructionProgram, StyleProgram),
        logical_source_catalog: Vec<u8>,
        source_map: SourceMapV1,
        resolved: ResolvedDocumentV1,
    ) -> Self {
        let (schema, construction, style) = programs;
        Self {
            frontend,
            document_origin,
            schema,
            construction,
            style,
            logical_source_catalog,
            source_map,
            resolved,
        }
    }

    /// Returns the lowered raw schema manifest.
    #[must_use]
    pub const fn schema(&self) -> &SchemaManifest {
        &self.schema
    }

    /// Returns the lowered raw construction program.
    #[must_use]
    pub const fn construction(&self) -> &ConstructionProgram {
        &self.construction
    }

    /// Returns the lowered raw style program.
    #[must_use]
    pub const fn style(&self) -> &StyleProgram {
        &self.style
    }

    /// Returns the virtual bytes addressed by every logical source span.
    #[must_use]
    pub fn logical_source_catalog(&self) -> &[u8] {
        &self.logical_source_catalog
    }

    /// Returns the host-only source map for this compilation.
    #[must_use]
    pub const fn source_map(&self) -> &SourceMapV1 {
        &self.source_map
    }

    pub(crate) const fn resolved(&self) -> &ResolvedDocumentV1 {
        &self.resolved
    }

    pub(crate) fn generated_rust_limit_failure(&self) -> AuthoringDiagnosticV1 {
        let document_anchor = self.resolved.document_anchor();
        AuthoringDiagnosticV1::new(
            self.frontend,
            AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::GeneratedRustBytes),
            DiagnosticLocationV1::Anchored {
                logical: logical_span(document_anchor),
                anchor_kind: AnchorKindV1::Document,
                physical: self.document_origin,
            },
        )
    }
}

impl fmt::Debug for CompiledAuthoringV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resolved = self.resolved();
        formatter
            .debug_struct("CompiledAuthoringV1")
            .field("authoring_format", &resolved.authoring_format())
            .field("document_anchor", &resolved.document_anchor())
            .field("semantic_records", &resolved.semantic_records())
            .field("authored_name_bytes", &resolved.authored_name_bytes())
            .field("logical_source_bytes", &self.logical_source_catalog.len())
            .field("source_map_entries", &self.source_map.entries.len())
            .finish_non_exhaustive()
    }
}

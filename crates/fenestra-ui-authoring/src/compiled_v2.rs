use std::fmt;

use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, SourceSpan, SpatialProgramV2, StyleProgram,
};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::resolved::logical_span;
use crate::resolved_v2::ResolvedDocumentV2;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};
use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};

/// Opaque canonical Rust source generated from a version-2 document.
pub struct GeneratedRustV2 {
    source: Box<str>,
}

impl GeneratedRustV2 {
    pub(crate) fn new(source: Box<str>) -> Self {
        Self { source }
    }

    /// Returns the canonical Rust source.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }
}

impl fmt::Debug for GeneratedRustV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeneratedRustV2")
            .field("bytes", &self.source.len())
            .finish()
    }
}

/// One host-only mapping from a logical IR anchor to its authored origin.
pub struct SourceMapEntryV2 {
    logical_span: SourceSpan,
    anchor_kind: AnchorKindV2,
    canonical_label: Box<str>,
    physical_origin: PhysicalOriginV2,
}

impl SourceMapEntryV2 {
    pub(crate) fn new(
        logical_span: SourceSpan,
        anchor_kind: AnchorKindV2,
        canonical_label: Box<str>,
        physical_origin: PhysicalOriginV2,
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
    pub const fn anchor_kind(&self) -> AnchorKindV2 {
        self.anchor_kind
    }

    /// Returns the normalized spelling of the defining authored token.
    #[must_use]
    pub fn canonical_label(&self) -> &str {
        &self.canonical_label
    }

    /// Returns the frontend-specific authored origin.
    #[must_use]
    pub const fn physical_origin(&self) -> &PhysicalOriginV2 {
        &self.physical_origin
    }
}

/// Host-only logical-to-physical source map for one version-2 compilation.
pub struct SourceMapV2 {
    entries: Vec<SourceMapEntryV2>,
}

impl SourceMapV2 {
    pub(crate) const fn new(entries: Vec<SourceMapEntryV2>) -> Self {
        Self { entries }
    }

    /// Returns entries in authored semantic-record order.
    #[must_use]
    pub fn entries(&self) -> &[SourceMapEntryV2] {
        &self.entries
    }
}

impl fmt::Debug for SourceMapV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceMapV2")
            .field("entries", &self.entries.len())
            .finish()
    }
}

/// Opaque result of a successful version-2 authoring compilation.
pub struct CompiledAuthoringV2 {
    frontend: AuthoringFrontendV2,
    document_origin: PhysicalOriginV2,
    schema: SchemaManifest,
    construction: ConstructionProgram,
    style: StyleProgram,
    spatial: SpatialProgramV2,
    logical_source_catalog: Vec<u8>,
    source_map: SourceMapV2,
    resolved: ResolvedDocumentV2,
}

impl CompiledAuthoringV2 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        frontend: AuthoringFrontendV2,
        document_origin: PhysicalOriginV2,
        schema: SchemaManifest,
        construction: ConstructionProgram,
        style: StyleProgram,
        spatial: SpatialProgramV2,
        logical_source_catalog: Vec<u8>,
        source_map: SourceMapV2,
        resolved: ResolvedDocumentV2,
    ) -> Self {
        Self {
            frontend,
            document_origin,
            schema,
            construction,
            style,
            spatial,
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

    /// Returns the lowered raw symbolic spatial program.
    #[must_use]
    pub const fn spatial(&self) -> &SpatialProgramV2 {
        &self.spatial
    }

    /// Returns the virtual bytes addressed by every logical source span.
    #[must_use]
    pub fn logical_source_catalog(&self) -> &[u8] {
        &self.logical_source_catalog
    }

    /// Returns the host-only source map for this compilation.
    #[must_use]
    pub const fn source_map(&self) -> &SourceMapV2 {
        &self.source_map
    }

    pub(crate) const fn resolved(&self) -> &ResolvedDocumentV2 {
        &self.resolved
    }

    pub(crate) fn generated_rust_limit_failure(&self) -> AuthoringDiagnosticV2 {
        let document_anchor = self.resolved.document_anchor();
        AuthoringDiagnosticV2::new(
            self.frontend,
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::GeneratedRustBytes),
            DiagnosticLocationV2::Anchored {
                logical: logical_span(document_anchor),
                anchor_kind: AnchorKindV2::Document,
                physical: self.document_origin,
            },
        )
    }
}

impl fmt::Debug for CompiledAuthoringV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledAuthoringV2")
            .field("authoring_format", &self.resolved.authoring_format())
            .field("document_anchor", &self.resolved.document_anchor())
            .field("logical_source_bytes", &self.logical_source_catalog.len())
            .field("source_map_entries", &self.source_map.entries.len())
            .finish_non_exhaustive()
    }
}

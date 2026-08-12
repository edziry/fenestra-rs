use crate::ids::{SchemaNamespace, SchemaRevision, SpatialFormatVersion};
use crate::source::SourceSpan;

use super::{SpatialImageDeclarationV2, SpatialNodeDeclarationV2, SpatialViewportContainerV2};

/// Unvalidated symbolic spatial program used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialProgramV2 {
    format: SpatialFormatVersion,
    schema_namespace: SchemaNamespace,
    schema_revision: SchemaRevision,
    viewport_container: SpatialViewportContainerV2,
    nodes: Vec<SpatialNodeDeclarationV2>,
    images: Vec<SpatialImageDeclarationV2>,
    span: SourceSpan,
}

impl SpatialProgramV2 {
    /// Creates an unvalidated symbolic spatial program.
    #[must_use]
    pub fn new(
        format: SpatialFormatVersion,
        schema_namespace: SchemaNamespace,
        schema_revision: SchemaRevision,
        viewport_container: SpatialViewportContainerV2,
        nodes: Vec<SpatialNodeDeclarationV2>,
        images: Vec<SpatialImageDeclarationV2>,
        span: SourceSpan,
    ) -> Self {
        Self {
            format,
            schema_namespace,
            schema_revision,
            viewport_container,
            nodes,
            images,
            span,
        }
    }

    /// Returns the symbolic spatial format version.
    #[must_use]
    pub const fn format(&self) -> SpatialFormatVersion {
        self.format
    }

    /// Returns the schema namespace referenced by this program.
    #[must_use]
    pub const fn schema_namespace(&self) -> SchemaNamespace {
        self.schema_namespace
    }

    /// Returns the schema revision referenced by this program.
    #[must_use]
    pub const fn schema_revision(&self) -> SchemaRevision {
        self.schema_revision
    }

    /// Returns the literal viewport container settings.
    #[must_use]
    pub const fn viewport_container(&self) -> SpatialViewportContainerV2 {
        self.viewport_container
    }

    /// Returns symbolic nodes in authored preorder.
    #[must_use]
    pub fn nodes(&self) -> &[SpatialNodeDeclarationV2] {
        &self.nodes
    }

    /// Returns program-local image declarations.
    #[must_use]
    pub fn images(&self) -> &[SpatialImageDeclarationV2] {
        &self.images
    }

    /// Returns the program record span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

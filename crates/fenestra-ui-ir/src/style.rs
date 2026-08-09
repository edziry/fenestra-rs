use crate::ids::{PropertyId, SchemaNamespace, SchemaRevision, StyleFormatVersion, TemplateNodeId};
use crate::source::SourceSpan;
use crate::value::PropertyValue;

/// One unvalidated exact-template style assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleAssignment {
    pub(crate) target: TemplateNodeId,
    pub(crate) property: PropertyId,
    pub(crate) value: PropertyValue,
    pub(crate) span: SourceSpan,
}

impl StyleAssignment {
    /// Creates an unvalidated exact-template style assignment.
    #[must_use]
    pub const fn new(
        target: TemplateNodeId,
        property: PropertyId,
        value: PropertyValue,
        span: SourceSpan,
    ) -> Self {
        Self {
            target,
            property,
            value,
            span,
        }
    }
}

/// Unvalidated exact-target style program used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleProgram {
    pub(crate) format: StyleFormatVersion,
    pub(crate) schema_namespace: SchemaNamespace,
    pub(crate) schema_revision: SchemaRevision,
    pub(crate) assignments: Vec<StyleAssignment>,
    pub(crate) span: SourceSpan,
}

impl StyleProgram {
    /// Creates an unvalidated exact-target style program.
    #[must_use]
    pub const fn new(
        format: StyleFormatVersion,
        schema_namespace: SchemaNamespace,
        schema_revision: SchemaRevision,
        assignments: Vec<StyleAssignment>,
        span: SourceSpan,
    ) -> Self {
        Self {
            format,
            schema_namespace,
            schema_revision,
            assignments,
            span,
        }
    }
}

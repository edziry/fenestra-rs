use crate::ids::{
    ComponentTypeId, PropertyId, SchemaFormatVersion, SchemaNamespace, SchemaRevision,
};
use crate::invalidation::InvalidationSet;
use crate::source::SourceSpan;
use crate::value::{PropertyValue, ValueType};

/// Unvalidated property declaration used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertySchema {
    pub(crate) id: PropertyId,
    pub(crate) value_type: ValueType,
    pub(crate) default: PropertyValue,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) span: SourceSpan,
}

impl PropertySchema {
    /// Creates an unvalidated property declaration.
    #[must_use]
    pub const fn new(
        id: PropertyId,
        value_type: ValueType,
        default: PropertyValue,
        invalidation: InvalidationSet,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            value_type,
            default,
            invalidation,
            span,
        }
    }
}

/// Unvalidated component declaration used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSchema {
    pub(crate) id: ComponentTypeId,
    pub(crate) properties: Vec<PropertySchema>,
    pub(crate) span: SourceSpan,
}

impl ComponentSchema {
    /// Creates an unvalidated component declaration.
    #[must_use]
    pub const fn new(
        id: ComponentTypeId,
        properties: Vec<PropertySchema>,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            properties,
            span,
        }
    }
}

/// Unvalidated schema manifest used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaManifest {
    pub(crate) format: SchemaFormatVersion,
    pub(crate) namespace: SchemaNamespace,
    pub(crate) revision: SchemaRevision,
    pub(crate) components: Vec<ComponentSchema>,
    pub(crate) span: SourceSpan,
}

impl SchemaManifest {
    /// Creates an unvalidated schema manifest.
    #[must_use]
    pub const fn new(
        format: SchemaFormatVersion,
        namespace: SchemaNamespace,
        revision: SchemaRevision,
        components: Vec<ComponentSchema>,
        span: SourceSpan,
    ) -> Self {
        Self {
            format,
            namespace,
            revision,
            components,
            span,
        }
    }
}

use crate::ids::{
    ComponentTypeId, ConstructionFormatVersion, PropertyId, SchemaNamespace, SchemaRevision,
    StructuralRegionId, TemplateNodeId,
};
use crate::invalidation::InvalidationSet;
use crate::source::SourceSpan;
use crate::value::PropertyValue;

/// Unvalidated initial property assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialProperty {
    pub(crate) property: PropertyId,
    pub(crate) value: PropertyValue,
    pub(crate) span: SourceSpan,
}

impl InitialProperty {
    /// Creates an unvalidated initial property assignment.
    #[must_use]
    pub const fn new(property: PropertyId, value: PropertyValue, span: SourceSpan) -> Self {
        Self {
            property,
            value,
            span,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChildSlotKind {
    Static(TemplateNodeId),
    Region(StructuralRegionId),
}

/// One ordered child position in a template.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildSlot {
    pub(crate) kind: ChildSlotKind,
    pub(crate) span: SourceSpan,
}

impl ChildSlot {
    /// Creates a static child slot.
    #[must_use]
    pub const fn static_node(child: TemplateNodeId, span: SourceSpan) -> Self {
        Self {
            kind: ChildSlotKind::Static(child),
            span,
        }
    }

    /// Creates a structural region slot.
    #[must_use]
    pub const fn region(region: StructuralRegionId, span: SourceSpan) -> Self {
        Self {
            kind: ChildSlotKind::Region(region),
            span,
        }
    }

    pub(crate) const fn span(self) -> SourceSpan {
        self.span
    }
}

/// Unvalidated template node declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateNode {
    pub(crate) id: TemplateNodeId,
    pub(crate) component: ComponentTypeId,
    pub(crate) initial_properties: Vec<InitialProperty>,
    pub(crate) children: Vec<ChildSlot>,
    pub(crate) span: SourceSpan,
}

impl TemplateNode {
    /// Creates an unvalidated template node declaration.
    #[must_use]
    pub const fn new(
        id: TemplateNodeId,
        component: ComponentTypeId,
        initial_properties: Vec<InitialProperty>,
        children: Vec<ChildSlot>,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            component,
            initial_properties,
            children,
            span,
        }
    }
}

/// One initial keyed member in a structural region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InitialKey {
    pub(crate) value: u64,
    pub(crate) span: SourceSpan,
}

impl InitialKey {
    /// Creates an unvalidated initial key.
    #[must_use]
    pub const fn new(value: u64, span: SourceSpan) -> Self {
        Self { value, span }
    }
}

/// Unvalidated keyed structural region declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralRegion {
    pub(crate) id: StructuralRegionId,
    pub(crate) owner: TemplateNodeId,
    pub(crate) repeat_body: TemplateNodeId,
    pub(crate) initial_keys: Vec<InitialKey>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) span: SourceSpan,
}

impl StructuralRegion {
    /// Creates an unvalidated structural region declaration.
    #[must_use]
    pub const fn new(
        id: StructuralRegionId,
        owner: TemplateNodeId,
        repeat_body: TemplateNodeId,
        initial_keys: Vec<InitialKey>,
        invalidation: InvalidationSet,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            owner,
            repeat_body,
            initial_keys,
            invalidation,
            span,
        }
    }
}

/// Unvalidated construction program used as validator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstructionProgram {
    pub(crate) format: ConstructionFormatVersion,
    pub(crate) schema_namespace: SchemaNamespace,
    pub(crate) schema_revision: SchemaRevision,
    pub(crate) nodes: Vec<TemplateNode>,
    pub(crate) regions: Vec<StructuralRegion>,
    pub(crate) span: SourceSpan,
}

impl ConstructionProgram {
    /// Creates an unvalidated construction program.
    #[must_use]
    pub const fn new(
        format: ConstructionFormatVersion,
        schema_namespace: SchemaNamespace,
        schema_revision: SchemaRevision,
        nodes: Vec<TemplateNode>,
        regions: Vec<StructuralRegion>,
        span: SourceSpan,
    ) -> Self {
        Self {
            format,
            schema_namespace,
            schema_revision,
            nodes,
            regions,
            span,
        }
    }
}

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::construction::{
    ChildSlot, ChildSlotKind, ConstructionProgram, InitialKey, InitialProperty, StructuralRegion,
    TemplateNode,
};
use crate::ids::{PropertyId, StructuralRegionId, TemplateNodeId};
use crate::invalidation::InvalidationSet;
use crate::source::SourceSpan;
use crate::value::PropertyValue;

use super::{ComponentSchemaView, PropertySchemaView, ValidatedSchema};

struct ConstructionData {
    program: ConstructionProgram,
    nodes: HashMap<TemplateNodeId, usize>,
    regions: HashMap<StructuralRegionId, usize>,
    root: usize,
}

/// Immutable construction program linked to an exact validated schema.
#[derive(Clone)]
pub struct ValidatedConstruction {
    schema: ValidatedSchema,
    data: Arc<ConstructionData>,
}

impl ValidatedConstruction {
    pub(crate) fn new(
        schema: ValidatedSchema,
        program: ConstructionProgram,
        nodes: HashMap<TemplateNodeId, usize>,
        regions: HashMap<StructuralRegionId, usize>,
        root: usize,
    ) -> Self {
        Self {
            schema,
            data: Arc::new(ConstructionData {
                program,
                nodes,
                regions,
                root,
            }),
        }
    }

    /// Returns the exact validated schema retained by this construction.
    #[must_use]
    pub const fn schema(&self) -> &ValidatedSchema {
        &self.schema
    }

    /// Returns the only root factory in this provisional program.
    #[must_use]
    pub fn root_factory(&self) -> TemplateFactory<'_> {
        TemplateFactory {
            construction: self,
            index: self.data.root,
        }
    }

    /// Resolves a template symbol through this construction domain.
    #[must_use]
    pub fn template(&self, id: TemplateNodeId) -> Option<TemplateFactory<'_>> {
        let index = *self.data.nodes.get(&id)?;
        Some(TemplateFactory {
            construction: self,
            index,
        })
    }

    /// Resolves a region symbol through this construction domain.
    #[must_use]
    pub fn region(&self, id: StructuralRegionId) -> Option<RegionFactory<'_>> {
        let index = *self.data.regions.get(&id)?;
        Some(RegionFactory {
            construction: self,
            index,
        })
    }

    /// Returns whether another clone shares this exact construction domain.
    #[must_use]
    pub fn shares_domain_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl fmt::Debug for ValidatedConstruction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValidatedConstruction(..)")
    }
}

/// Receiver-scoped view of one validated template factory.
#[derive(Clone, Copy)]
pub struct TemplateFactory<'a> {
    construction: &'a ValidatedConstruction,
    index: usize,
}

impl<'a> TemplateFactory<'a> {
    fn raw(self) -> &'a TemplateNode {
        &self.construction.data.program.nodes[self.index]
    }

    /// Returns the template symbol local to its construction program.
    #[must_use]
    pub fn id(self) -> TemplateNodeId {
        self.raw().id
    }

    /// Returns the resolved component declaration for this template.
    #[must_use]
    pub fn component(self) -> ComponentSchemaView<'a> {
        self.construction
            .schema
            .component(self.raw().component)
            .expect("validated template component must resolve")
    }

    /// Iterates authored initial property overrides in declaration order.
    pub fn initial_properties(self) -> InitialPropertyIter<'a> {
        InitialPropertyIter {
            template: self,
            properties: self.raw().initial_properties.iter(),
        }
    }

    /// Returns an authored override or the schema default for a property.
    #[must_use]
    pub fn effective_value(self, property: PropertyId) -> Option<&'a PropertyValue> {
        let declared = self.component().property(property)?;
        self.raw()
            .initial_properties
            .iter()
            .find(|candidate| candidate.property == property)
            .map_or_else(
                || Some(declared.default()),
                |candidate| Some(&candidate.value),
            )
    }

    /// Iterates ordered static and region child factories.
    pub fn children(self) -> ChildFactoryIter<'a> {
        ChildFactoryIter {
            construction: self.construction,
            slots: self.raw().children.iter(),
        }
    }
}

/// One resolved authored initial property assignment.
#[derive(Clone, Copy)]
pub struct InitialPropertyView<'a> {
    property: PropertySchemaView<'a>,
    value: &'a PropertyValue,
}

impl<'a> InitialPropertyView<'a> {
    /// Returns the resolved property declaration.
    #[must_use]
    pub const fn property(self) -> PropertySchemaView<'a> {
        self.property
    }

    /// Returns the immutable authored override value.
    #[must_use]
    pub const fn value(self) -> &'a PropertyValue {
        self.value
    }
}

/// Iterator over resolved initial property assignments.
pub struct InitialPropertyIter<'a> {
    template: TemplateFactory<'a>,
    properties: std::slice::Iter<'a, InitialProperty>,
}

impl<'a> Iterator for InitialPropertyIter<'a> {
    type Item = InitialPropertyView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let assignment = self.properties.next()?;
        Some(InitialPropertyView {
            property: self
                .template
                .component()
                .property(assignment.property)
                .expect("validated initial property must resolve"),
            value: &assignment.value,
        })
    }
}

/// Receiver-scoped view of one validated structural region.
#[derive(Clone, Copy)]
pub struct RegionFactory<'a> {
    construction: &'a ValidatedConstruction,
    index: usize,
}

impl<'a> RegionFactory<'a> {
    fn raw(self) -> &'a StructuralRegion {
        &self.construction.data.program.regions[self.index]
    }

    /// Returns the region symbol local to its construction program.
    #[must_use]
    pub fn id(self) -> StructuralRegionId {
        self.raw().id
    }

    /// Returns the template factory that owns this region slot.
    #[must_use]
    pub fn owner(self) -> TemplateFactory<'a> {
        self.template(self.raw().owner)
    }

    /// Returns the repeat-body template factory.
    #[must_use]
    pub fn repeat_body(self) -> TemplateFactory<'a> {
        self.template(self.raw().repeat_body)
    }

    /// Returns the validated structural invalidation declaration.
    #[must_use]
    pub fn invalidation(self) -> InvalidationSet {
        self.raw().invalidation
    }

    /// Iterates initial keys in authored order.
    pub fn initial_keys(self) -> InitialKeyIter<'a> {
        InitialKeyIter {
            keys: self.raw().initial_keys.iter(),
        }
    }

    fn template(self, id: TemplateNodeId) -> TemplateFactory<'a> {
        TemplateFactory {
            construction: self.construction,
            index: self.construction.data.nodes[&id],
        }
    }
}

/// One resolved child position in a template factory.
#[derive(Clone, Copy)]
pub enum ChildFactory<'a> {
    /// Statically present template factory.
    Static {
        /// Resolved child template.
        template: TemplateFactory<'a>,
        /// Authored child-slot anchor.
        span: SourceSpan,
    },
    /// Keyed structural region factory.
    Region {
        /// Resolved structural region.
        region: RegionFactory<'a>,
        /// Authored child-slot anchor.
        span: SourceSpan,
    },
}

/// Iterator over resolved child factories.
pub struct ChildFactoryIter<'a> {
    construction: &'a ValidatedConstruction,
    slots: std::slice::Iter<'a, ChildSlot>,
}

impl<'a> Iterator for ChildFactoryIter<'a> {
    type Item = ChildFactory<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slot = *self.slots.next()?;
        Some(match slot.kind {
            ChildSlotKind::Static(child) => ChildFactory::Static {
                template: TemplateFactory {
                    construction: self.construction,
                    index: self.construction.data.nodes[&child],
                },
                span: slot.span,
            },
            ChildSlotKind::Region(region) => ChildFactory::Region {
                region: RegionFactory {
                    construction: self.construction,
                    index: self.construction.data.regions[&region],
                },
                span: slot.span,
            },
        })
    }
}

/// Receiver-scoped view of one initial keyed member.
#[derive(Clone, Copy)]
pub struct InitialKeyView<'a> {
    key: &'a InitialKey,
}

impl InitialKeyView<'_> {
    /// Returns the authored key value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.key.value
    }
}

/// Iterator over initial keyed members in authored order.
pub struct InitialKeyIter<'a> {
    keys: std::slice::Iter<'a, InitialKey>,
}

impl<'a> Iterator for InitialKeyIter<'a> {
    type Item = InitialKeyView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|key| InitialKeyView { key })
    }
}

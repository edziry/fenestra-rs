use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::ids::{ComponentTypeId, PropertyId};
use crate::invalidation::InvalidationSet;
use crate::schema::{ComponentSchema, PropertySchema, SchemaManifest};
use crate::value::{PropertyValue, ValueType};

pub(crate) struct SchemaData {
    pub(crate) manifest: SchemaManifest,
    pub(crate) components: HashMap<ComponentTypeId, usize>,
    pub(crate) properties: Vec<HashMap<PropertyId, usize>>,
}

/// Immutable schema produced by bounded validation.
#[derive(Clone)]
pub struct ValidatedSchema {
    pub(crate) data: Arc<SchemaData>,
}

impl ValidatedSchema {
    pub(crate) fn new(
        manifest: SchemaManifest,
        components: HashMap<ComponentTypeId, usize>,
        properties: Vec<HashMap<PropertyId, usize>>,
    ) -> Self {
        Self {
            data: Arc::new(SchemaData {
                manifest,
                components,
                properties,
            }),
        }
    }

    /// Resolves a component symbol through this schema domain.
    #[must_use]
    pub fn component(&self, id: ComponentTypeId) -> Option<ComponentSchemaView<'_>> {
        let index = *self.data.components.get(&id)?;
        Some(ComponentSchemaView {
            schema: self,
            index,
        })
    }

    /// Returns whether another clone shares this exact validation domain.
    #[must_use]
    pub fn shares_domain_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl fmt::Debug for ValidatedSchema {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValidatedSchema(..)")
    }
}

/// Receiver-scoped view of one validated component.
#[derive(Clone, Copy)]
pub struct ComponentSchemaView<'a> {
    schema: &'a ValidatedSchema,
    index: usize,
}

impl<'a> ComponentSchemaView<'a> {
    fn raw(self) -> &'a ComponentSchema {
        &self.schema.data.manifest.components[self.index]
    }

    /// Returns the local component symbol.
    #[must_use]
    pub fn id(self) -> ComponentTypeId {
        self.raw().id
    }

    /// Iterates properties in authored declaration order.
    pub fn properties(self) -> ComponentPropertiesIter<'a> {
        ComponentPropertiesIter {
            component: self,
            next: 0,
        }
    }

    /// Resolves a property symbol within this component.
    #[must_use]
    pub fn property(self, id: PropertyId) -> Option<PropertySchemaView<'a>> {
        let index = *self.schema.data.properties[self.index].get(&id)?;
        Some(PropertySchemaView {
            component: self,
            index,
        })
    }
}

/// Iterator over validated properties in authored declaration order.
pub struct ComponentPropertiesIter<'a> {
    component: ComponentSchemaView<'a>,
    next: usize,
}

impl<'a> Iterator for ComponentPropertiesIter<'a> {
    type Item = PropertySchemaView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.next < self.component.raw().properties.len()).then(|| {
            let property = PropertySchemaView {
                component: self.component,
                index: self.next,
            };
            self.next += 1;
            property
        })
    }
}

/// Receiver-scoped view of one validated property.
#[derive(Clone, Copy)]
pub struct PropertySchemaView<'a> {
    component: ComponentSchemaView<'a>,
    index: usize,
}

impl<'a> PropertySchemaView<'a> {
    fn raw(self) -> &'a PropertySchema {
        &self.component.raw().properties[self.index]
    }

    /// Returns the property symbol local to its component.
    #[must_use]
    pub fn id(self) -> PropertyId {
        self.raw().id
    }

    /// Returns the declared closed value type.
    #[must_use]
    pub fn value_type(self) -> ValueType {
        self.raw().value_type
    }

    /// Returns the immutable default value.
    #[must_use]
    pub fn default(self) -> &'a PropertyValue {
        &self.raw().default
    }

    /// Returns the validated downstream invalidation declaration.
    #[must_use]
    pub fn invalidation(self) -> InvalidationSet {
        self.raw().invalidation
    }
}

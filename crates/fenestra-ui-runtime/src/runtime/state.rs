use std::collections::HashSet;
use std::sync::Arc;

use fenestra_ui_ir::prototype::{
    ChildFactory, ComponentTypeId, InvalidationSet, PropertyId, PropertyValue, StructuralRegionId,
    TemplateNodeId, ValidatedConstruction,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::logical_tree::{LogicalTree, NodeId};

use super::capacity::RuntimeCapacity;
use super::fragment::{FragmentId, FragmentStore};
use super::headless::HeadlessProjectionState;
use super::spatial::SpatialPublication;

/// Monotonic identity of one committed logical runtime state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeGeneration(u64);

impl RuntimeGeneration {
    pub(crate) const INITIAL: Self = Self(0);

    pub(crate) fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }

    /// Returns the numeric generation used by experiment artifacts.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropertySlot {
    pub(crate) id: PropertyId,
    pub(crate) value: PropertyValue,
    pub(crate) invalidation: InvalidationSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChildGroup {
    Static(NodeId),
    Region(FragmentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeNode {
    pub(crate) template: TemplateNodeId,
    pub(crate) component: ComponentTypeId,
    pub(crate) properties: Vec<PropertySlot>,
    pub(crate) child_groups: Vec<ChildGroup>,
}

pub(crate) struct RuntimeState {
    pub(crate) generation: RuntimeGeneration,
    pub(crate) tree: LogicalTree<RuntimeNode>,
    pub(crate) fragments: FragmentStore,
    pub(crate) property_slot_count: usize,
    pub(crate) headless: Option<HeadlessProjectionState>,
    pub(crate) spatial: Option<Arc<SpatialPublication>>,
}

impl RuntimeState {
    pub(crate) fn spatial_viewport(&self) -> Option<SpatialViewportV2> {
        self.spatial.as_deref().map(SpatialPublication::viewport)
    }

    #[cfg(test)]
    pub(crate) fn set_generation_for_test(&mut self, value: u64) {
        self.generation = RuntimeGeneration(value);
    }

    #[cfg(test)]
    pub(crate) fn corrupt_properties_for_test(&mut self) {
        let root = self.tree.root().expect("test corruption requires a root");
        self.tree
            .value_mut_for_transaction(root)
            .expect("test corruption requires a live root")
            .properties
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_tree_for_test(&mut self) {
        let root = self.tree.root().expect("test corruption requires a root");
        self.tree.clear_children_for_test(root);
    }

    #[cfg(test)]
    pub(crate) fn corrupt_fragment_for_test(&mut self) {
        let root = self.tree.root().expect("test corruption requires a root");
        let fragment = self
            .tree
            .value(root)
            .expect("test corruption requires a live root")
            .child_groups
            .iter()
            .find_map(|group| match group {
                ChildGroup::Region(fragment) => Some(*fragment),
                ChildGroup::Static(_) => None,
            })
            .expect("test corruption requires a fragment");
        self.fragments
            .remove(fragment)
            .expect("test corruption requires a live fragment");
    }

    pub(crate) fn fork_for_transaction(&self) -> Self {
        Self {
            generation: self.generation,
            tree: self.tree.fork_for_transaction(),
            fragments: self.fragments.fork_for_transaction(),
            property_slot_count: self.property_slot_count,
            headless: self.headless.clone(),
            spatial: self.spatial.clone(),
        }
    }

    pub(crate) fn validate(
        &self,
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
    ) -> Result<(), ()> {
        self.tree.validate().map_err(|_| ())?;
        if self.tree.len() > capacity.live_nodes()
            || self.fragments.len() > capacity.live_fragments()
            || self.property_slot_count > capacity.live_property_slots()
        {
            return Err(());
        }

        let mut property_slots = 0usize;
        let mut referenced_fragments = HashSet::new();
        for (node_id, node) in self.tree.live_nodes() {
            let template = construction.template(node.template).ok_or(())?;
            if template.component().id() != node.component {
                return Err(());
            }
            let declared = template.component().properties().collect::<Vec<_>>();
            if declared.len() != node.properties.len() {
                return Err(());
            }
            for (slot, property) in node.properties.iter().zip(declared) {
                if slot.id != property.id()
                    || slot.value.value_type() != property.value_type()
                    || slot.invalidation != property.invalidation()
                {
                    return Err(());
                }
            }
            property_slots = property_slots
                .checked_add(node.properties.len())
                .ok_or(())?;

            let authored = template.children().collect::<Vec<_>>();
            if authored.len() != node.child_groups.len() {
                return Err(());
            }
            let mut flattened = Vec::new();
            for (group, expected) in node.child_groups.iter().zip(authored) {
                match (*group, expected) {
                    (ChildGroup::Static(child), ChildFactory::Static { template, .. }) => {
                        if self.tree.parent(child) != Some(node_id)
                            || self.tree.value(child).map(|value| value.template)
                                != Some(template.id())
                        {
                            return Err(());
                        }
                        flattened.push(child);
                    }
                    (ChildGroup::Region(fragment), ChildFactory::Region { region, .. }) => {
                        let stored = self.fragments.get(fragment).ok_or(())?;
                        if stored.owner != node_id || stored.descriptor != region.id() {
                            return Err(());
                        }
                        if !referenced_fragments.insert(fragment) {
                            return Err(());
                        }
                        let mut keys = HashSet::new();
                        for member in &stored.members {
                            if !keys.insert(member.key)
                                || self.tree.parent(member.root) != Some(node_id)
                                || self.tree.value(member.root).map(|value| value.template)
                                    != Some(region.repeat_body().id())
                            {
                                return Err(());
                            }
                            flattened.push(member.root);
                        }
                    }
                    _ => return Err(()),
                }
            }
            if self.tree.children(node_id) != Some(flattened.as_slice()) {
                return Err(());
            }
        }
        if property_slots != self.property_slot_count
            || referenced_fragments.len() != self.fragments.len()
        {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn fragment_for(
        &self,
        owner: NodeId,
        descriptor: StructuralRegionId,
    ) -> Option<FragmentId> {
        self.tree.value(owner)?;
        self.fragments.find(owner, descriptor)
    }
}

use std::fmt;
use std::slice;
use std::sync::Arc;

use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};

use crate::logical_tree::NodeId;

use super::fragment::{FragmentId, KeyedMember};
use super::state::{RuntimeGeneration, RuntimeState};

/// Immutable handle to one exact committed logical runtime generation.
#[derive(Clone)]
pub struct CommittedRuntimeSnapshot {
    pub(crate) state: Arc<RuntimeState>,
}

impl CommittedRuntimeSnapshot {
    /// Returns the committed runtime generation.
    #[must_use]
    pub fn generation(&self) -> RuntimeGeneration {
        self.state.generation
    }

    /// Returns the always-present logical root.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.state
            .tree
            .root()
            .expect("validated runtime state must retain a root")
    }

    /// Returns the number of live logical nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.state.tree.len()
    }

    /// Returns the number of live fragment instances.
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.state.fragments.len()
    }

    /// Returns the number of live typed property slots.
    #[must_use]
    pub fn property_slot_count(&self) -> usize {
        self.state.property_slot_count
    }

    /// Returns a live node's construction template symbol.
    #[must_use]
    pub fn template(&self, node: NodeId) -> Option<TemplateNodeId> {
        self.state.tree.value(node).map(|stored| stored.template)
    }

    /// Returns a live node's component symbol.
    #[must_use]
    pub fn component(&self, node: NodeId) -> Option<ComponentTypeId> {
        self.state.tree.value(node).map(|stored| stored.component)
    }

    /// Returns a live node's typed property value.
    #[must_use]
    pub fn property(&self, node: NodeId, property: PropertyId) -> Option<&PropertyValue> {
        self.state
            .tree
            .value(node)?
            .properties
            .iter()
            .find(|slot| slot.id == property)
            .map(|slot| &slot.value)
    }

    /// Returns a live non-root node's parent.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.state.tree.parent(node)
    }

    /// Returns the ordered direct children of a live node.
    #[must_use]
    pub fn children(&self, node: NodeId) -> Option<&[NodeId]> {
        self.state.tree.children(node)
    }

    /// Resolves a region instance owned by a live node.
    #[must_use]
    pub fn fragment(&self, owner: NodeId, descriptor: StructuralRegionId) -> Option<FragmentId> {
        self.state.fragment_for(owner, descriptor)
    }

    /// Iterates one fragment's keyed members in committed order.
    #[must_use]
    pub fn keyed_members(&self, fragment: FragmentId) -> Option<KeyedMemberIter<'_>> {
        Some(KeyedMemberIter {
            members: self.state.fragments.get(fragment)?.members.iter(),
        })
    }

    /// Resolves one key to its committed member root.
    #[must_use]
    pub fn keyed_member(&self, fragment: FragmentId, key: u64) -> Option<NodeId> {
        self.state
            .fragments
            .get(fragment)?
            .members
            .iter()
            .find_map(|member| (member.key == key).then_some(member.root))
    }

    /// Returns whether two handles retain the exact same state allocation.
    #[must_use]
    pub fn shares_state_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

impl fmt::Debug for CommittedRuntimeSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CommittedRuntimeSnapshot")
            .field("generation", &self.state.generation)
            .finish_non_exhaustive()
    }
}

/// Iterator over committed `(key, member root)` pairs.
pub struct KeyedMemberIter<'a> {
    members: slice::Iter<'a, KeyedMember>,
}

impl Iterator for KeyedMemberIter<'_> {
    type Item = (u64, NodeId);

    fn next(&mut self) -> Option<Self::Item> {
        self.members.next().map(|member| (member.key, member.root))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members.size_hint()
    }
}

impl ExactSizeIterator for KeyedMemberIter<'_> {}

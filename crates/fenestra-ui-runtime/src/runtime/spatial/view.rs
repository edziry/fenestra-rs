use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialResolvedSnapshotV2};

use crate::logical_tree::NodeId;

use super::types::SpatialPublication;
use crate::runtime::fragment::FragmentId;
use crate::runtime::state::RuntimeState;
use crate::runtime::view::KeyedMemberIter;

/// Borrowed immutable logical state supplied during a spatial build callback.
#[derive(Clone, Copy)]
pub struct RuntimeSpatialBuildViewV2<'a> {
    state: &'a RuntimeState,
}

impl<'a> RuntimeSpatialBuildViewV2<'a> {
    pub(crate) const fn new(state: &'a RuntimeState) -> Self {
        Self { state }
    }

    /// Returns the always-present logical root.
    #[must_use]
    pub fn root(self) -> NodeId {
        self.state
            .tree
            .root()
            .expect("validated runtime state must retain a root")
    }

    /// Returns the number of live logical nodes.
    #[must_use]
    pub fn node_count(self) -> usize {
        self.state.tree.len()
    }

    /// Returns a live node's construction template symbol.
    #[must_use]
    pub fn template(self, node: NodeId) -> Option<TemplateNodeId> {
        self.state.tree.value(node).map(|stored| stored.template)
    }

    /// Returns a live node's component symbol.
    #[must_use]
    pub fn component(self, node: NodeId) -> Option<ComponentTypeId> {
        self.state.tree.value(node).map(|stored| stored.component)
    }

    /// Returns a live node's typed property value.
    #[must_use]
    pub fn property(self, node: NodeId, property: PropertyId) -> Option<&'a PropertyValue> {
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
    pub fn parent(self, node: NodeId) -> Option<NodeId> {
        self.state.tree.parent(node)
    }

    /// Returns the ordered direct children of a live node.
    #[must_use]
    pub fn children(self, node: NodeId) -> Option<&'a [NodeId]> {
        self.state.tree.children(node)
    }

    /// Resolves a region instance owned by a live node.
    #[must_use]
    pub fn fragment(self, owner: NodeId, descriptor: StructuralRegionId) -> Option<FragmentId> {
        self.state.fragment_for(owner, descriptor)
    }

    /// Iterates one fragment's keyed members in committed order.
    #[must_use]
    pub fn keyed_members(self, fragment: FragmentId) -> Option<KeyedMemberIter<'a>> {
        Some(KeyedMemberIter::new(
            &self.state.fragments.get(fragment)?.members,
        ))
    }

    /// Resolves one key to its member root.
    #[must_use]
    pub fn keyed_member(self, fragment: FragmentId, key: u64) -> Option<NodeId> {
        self.state
            .fragments
            .get(fragment)?
            .members
            .iter()
            .find_map(|member| (member.key == key).then_some(member.root))
    }
}

/// Borrowed immutable access to one exact runtime spatial publication.
#[derive(Clone, Copy)]
pub struct RuntimeSpatialViewV2<'a> {
    publication: &'a SpatialPublication,
}

impl<'a> RuntimeSpatialViewV2<'a> {
    pub(crate) const fn new(publication: &'a SpatialPublication) -> Self {
        Self { publication }
    }

    /// Returns the completed reference spatial snapshot.
    #[must_use]
    pub fn snapshot(self) -> &'a SpatialResolvedSnapshotV2 {
        self.publication.snapshot.as_ref()
    }

    /// Resolves a non-sentinel spatial key to its accepted logical node.
    #[must_use]
    pub fn logical_node(self, key: SpatialNodeKeyV2) -> Option<NodeId> {
        let index = key.get().checked_sub(1)?;
        self.publication
            .logical_nodes
            .get(usize::try_from(index).ok()?)
            .copied()
    }

    /// Resolves an accepted logical node to its unique spatial key.
    #[must_use]
    pub fn spatial_key(self, node: NodeId) -> Option<SpatialNodeKeyV2> {
        let index = self
            .publication
            .logical_nodes
            .iter()
            .position(|mapped| *mapped == node)?;
        let value = u32::try_from(index.checked_add(1)?).ok()?;
        Some(SpatialNodeKeyV2::new(value))
    }
}

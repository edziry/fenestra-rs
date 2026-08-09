use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId};

pub(crate) trait SnapshotViewV1 {
    fn root(&self) -> NodeId;

    fn node_count(&self) -> usize;

    fn fragment_count(&self) -> usize;

    fn property_slot_count(&self) -> usize;

    fn template(&self, node: NodeId) -> Option<TemplateNodeId>;

    fn component(&self, node: NodeId) -> Option<ComponentTypeId>;

    fn property(&self, node: NodeId, property: PropertyId) -> Option<PropertyValue>;

    fn parent(&self, node: NodeId) -> Option<NodeId>;

    fn child_count(&self, node: NodeId) -> Option<usize>;

    fn child_at(&self, node: NodeId, index: usize) -> Option<NodeId>;

    fn fragment(&self, owner: NodeId, descriptor: StructuralRegionId) -> Option<FragmentId>;

    fn keyed_count(&self, fragment: FragmentId) -> Option<usize>;

    fn keyed_at(&self, fragment: FragmentId, index: usize) -> Option<(u64, NodeId)>;
}

impl SnapshotViewV1 for CommittedRuntimeSnapshot {
    fn root(&self) -> NodeId {
        CommittedRuntimeSnapshot::root(self)
    }

    fn node_count(&self) -> usize {
        CommittedRuntimeSnapshot::node_count(self)
    }

    fn fragment_count(&self) -> usize {
        CommittedRuntimeSnapshot::fragment_count(self)
    }

    fn property_slot_count(&self) -> usize {
        CommittedRuntimeSnapshot::property_slot_count(self)
    }

    fn template(&self, node: NodeId) -> Option<TemplateNodeId> {
        CommittedRuntimeSnapshot::template(self, node)
    }

    fn component(&self, node: NodeId) -> Option<ComponentTypeId> {
        CommittedRuntimeSnapshot::component(self, node)
    }

    fn property(&self, node: NodeId, property: PropertyId) -> Option<PropertyValue> {
        CommittedRuntimeSnapshot::property(self, node, property).cloned()
    }

    fn parent(&self, node: NodeId) -> Option<NodeId> {
        CommittedRuntimeSnapshot::parent(self, node)
    }

    fn child_count(&self, node: NodeId) -> Option<usize> {
        CommittedRuntimeSnapshot::children(self, node).map(<[NodeId]>::len)
    }

    fn child_at(&self, node: NodeId, index: usize) -> Option<NodeId> {
        CommittedRuntimeSnapshot::children(self, node)?
            .get(index)
            .copied()
    }

    fn fragment(&self, owner: NodeId, descriptor: StructuralRegionId) -> Option<FragmentId> {
        CommittedRuntimeSnapshot::fragment(self, owner, descriptor)
    }

    fn keyed_count(&self, fragment: FragmentId) -> Option<usize> {
        CommittedRuntimeSnapshot::keyed_members(self, fragment).map(|members| members.len())
    }

    fn keyed_at(&self, fragment: FragmentId, index: usize) -> Option<(u64, NodeId)> {
        CommittedRuntimeSnapshot::keyed_members(self, fragment)?.nth(index)
    }
}

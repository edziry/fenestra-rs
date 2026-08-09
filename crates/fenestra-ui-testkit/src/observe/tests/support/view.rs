use fenestra_ui_ir::prototype::{ComponentTypeId, PropertyId, PropertyValue, TemplateNodeId};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId};

mod defect;

pub(in crate::observe::tests) use defect::{NodeViewDefectV1, ViewDefectV1};

use super::super::super::view::SnapshotViewV1;
use crate::identity::IdentityIndexV1;
use crate::semantic::NormalizedStateV1;
use defect::{ResolvedNodeViewDefectV1, ResolvedViewDefectV1, resolve_defect, swapped_index};

pub(super) struct DefectViewV1<'a> {
    snapshot: &'a CommittedRuntimeSnapshot,
    defects: Vec<ResolvedViewDefectV1>,
}

impl<'a> DefectViewV1<'a> {
    pub(super) fn new(
        snapshot: &'a CommittedRuntimeSnapshot,
        identities: &IdentityIndexV1,
        expected: &NormalizedStateV1,
        defects: Vec<ViewDefectV1>,
    ) -> Self {
        let defects = defects
            .into_iter()
            .map(|defect| resolve_defect(defect, identities, expected))
            .collect();
        Self { snapshot, defects }
    }
}

impl SnapshotViewV1 for DefectViewV1<'_> {
    fn root(&self) -> NodeId {
        self.snapshot.root()
    }

    fn node_count(&self) -> usize {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::ReportedCounts { nodes, .. } = defect {
                return *nodes;
            }
        }
        self.snapshot.node_count()
    }

    fn fragment_count(&self) -> usize {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::ReportedCounts { fragments, .. } = defect {
                return *fragments;
            }
        }
        self.snapshot.fragment_count()
    }

    fn property_slot_count(&self) -> usize {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::ReportedCounts { properties, .. } = defect {
                return *properties;
            }
        }
        self.snapshot.property_slot_count()
    }

    fn template(&self, node: NodeId) -> Option<TemplateNodeId> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::Node(ResolvedNodeViewDefectV1::Template {
                node: target,
                observed,
            }) = defect
                && node == *target
            {
                return *observed;
            }
        }
        self.snapshot.template(node)
    }

    fn component(&self, node: NodeId) -> Option<ComponentTypeId> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::Node(ResolvedNodeViewDefectV1::Component {
                node: target,
                observed,
            }) = defect
                && node == *target
            {
                return *observed;
            }
        }
        self.snapshot.component(node)
    }

    fn property(&self, node: NodeId, property: PropertyId) -> Option<PropertyValue> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::Node(ResolvedNodeViewDefectV1::Property {
                node: target,
                property: target_property,
                observed,
            }) = defect
                && node == *target
                && property == *target_property
            {
                return observed.clone();
            }
        }
        self.snapshot.property(node, property).cloned()
    }

    fn parent(&self, node: NodeId) -> Option<NodeId> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::Node(ResolvedNodeViewDefectV1::Parent {
                node: target,
                observed,
            }) = defect
                && node == *target
            {
                return *observed;
            }
        }
        self.snapshot
            .parent(node)
            .map(|parent| self.aliased_node(parent))
    }

    fn child_count(&self, node: NodeId) -> Option<usize> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::ReportedChildCount {
                node: target,
                count,
            } = defect
                && node == *target
            {
                return Some(*count);
            }
        }
        self.snapshot.children(node).map(<[NodeId]>::len)
    }

    fn child_at(&self, node: NodeId, index: usize) -> Option<NodeId> {
        let index = self
            .defects
            .iter()
            .fold(index, |index, defect| match defect {
                ResolvedViewDefectV1::SwappedChildren {
                    node: target,
                    left,
                    right,
                } if node == *target => swapped_index(index, *left, *right),
                _ => index,
            });
        self.snapshot
            .children(node)?
            .get(index)
            .copied()
            .map(|child| self.aliased_node(child))
    }

    fn fragment(
        &self,
        owner: NodeId,
        descriptor: fenestra_ui_ir::prototype::StructuralRegionId,
    ) -> Option<FragmentId> {
        if self.defects.iter().any(|defect| {
            matches!(
                defect,
                ResolvedViewDefectV1::HiddenFragmentBinding {
                    owner: target_owner,
                    descriptor: target_descriptor,
                } if owner == *target_owner && descriptor == *target_descriptor
            )
        }) {
            return None;
        }
        for defect in &self.defects {
            if let ResolvedViewDefectV1::FragmentAlias {
                source,
                target_owner,
                target_descriptor,
            } = defect
                && owner == *target_owner
                && descriptor == *target_descriptor
            {
                return Some(*source);
            }
        }
        self.snapshot.fragment(owner, descriptor)
    }

    fn keyed_count(&self, fragment: FragmentId) -> Option<usize> {
        for defect in &self.defects {
            if let ResolvedViewDefectV1::ReportedKeyedCount {
                fragment: target,
                count,
            } = defect
                && fragment == *target
            {
                return Some(*count);
            }
        }
        self.snapshot
            .keyed_members(fragment)
            .map(|members| members.len())
    }

    fn keyed_at(&self, fragment: FragmentId, index: usize) -> Option<(u64, NodeId)> {
        let index = self
            .defects
            .iter()
            .fold(index, |index, defect| match defect {
                ResolvedViewDefectV1::SwappedKeyedMembers {
                    fragment: target,
                    left,
                    right,
                } if fragment == *target => swapped_index(index, *left, *right),
                _ => index,
            });
        self.snapshot
            .keyed_members(fragment)?
            .nth(index)
            .map(|(key, node)| (key, self.aliased_node(node)))
    }
}

impl DefectViewV1<'_> {
    fn aliased_node(&self, node: NodeId) -> NodeId {
        self.defects.iter().fold(node, |node, defect| match defect {
            ResolvedViewDefectV1::NodeAlias { source, target } if node == *target => *source,
            _ => node,
        })
    }
}

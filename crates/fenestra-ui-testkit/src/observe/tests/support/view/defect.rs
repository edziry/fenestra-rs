use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};
use fenestra_ui_runtime::prototype::{FragmentId, NodeId};

use crate::identity::IdentityIndexV1;
use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedStateV1};

pub(in crate::observe::tests) enum NodeViewDefectV1 {
    Template {
        node: NodePathV1,
        observed: Option<TemplateNodeId>,
    },
    Component {
        node: NodePathV1,
        observed: Option<ComponentTypeId>,
    },
    Property {
        node: NodePathV1,
        property: PropertyId,
        observed: Option<PropertyValue>,
    },
    Parent {
        node: NodePathV1,
        observed: Option<NodePathV1>,
    },
}

pub(in crate::observe::tests) enum ViewDefectV1 {
    Node(NodeViewDefectV1),
    HiddenFragmentBinding {
        fragment: FragmentPathV1,
    },
    SwappedKeyedMembers {
        fragment: FragmentPathV1,
        left: usize,
        right: usize,
    },
    SwappedChildren {
        node: NodePathV1,
        left: usize,
        right: usize,
    },
    ReportedCounts {
        nodes: usize,
        fragments: usize,
        properties: usize,
    },
    ReportedChildCount {
        node: NodePathV1,
        count: usize,
    },
    ReportedKeyedCount {
        fragment: FragmentPathV1,
        count: usize,
    },
    NodeAlias {
        source: NodePathV1,
        target: NodePathV1,
    },
    FragmentAlias {
        source: FragmentPathV1,
        target: FragmentPathV1,
    },
}

impl From<NodeViewDefectV1> for ViewDefectV1 {
    fn from(defect: NodeViewDefectV1) -> Self {
        Self::Node(defect)
    }
}

pub(super) enum ResolvedNodeViewDefectV1 {
    Template {
        node: NodeId,
        observed: Option<TemplateNodeId>,
    },
    Component {
        node: NodeId,
        observed: Option<ComponentTypeId>,
    },
    Property {
        node: NodeId,
        property: PropertyId,
        observed: Option<PropertyValue>,
    },
    Parent {
        node: NodeId,
        observed: Option<NodeId>,
    },
}

pub(super) enum ResolvedViewDefectV1 {
    Node(ResolvedNodeViewDefectV1),
    HiddenFragmentBinding {
        owner: NodeId,
        descriptor: StructuralRegionId,
    },
    SwappedKeyedMembers {
        fragment: FragmentId,
        left: usize,
        right: usize,
    },
    SwappedChildren {
        node: NodeId,
        left: usize,
        right: usize,
    },
    ReportedCounts {
        nodes: usize,
        fragments: usize,
        properties: usize,
    },
    ReportedChildCount {
        node: NodeId,
        count: usize,
    },
    ReportedKeyedCount {
        fragment: FragmentId,
        count: usize,
    },
    NodeAlias {
        source: NodeId,
        target: NodeId,
    },
    FragmentAlias {
        source: FragmentId,
        target_owner: NodeId,
        target_descriptor: StructuralRegionId,
    },
}

pub(super) fn resolve_defect(
    defect: ViewDefectV1,
    identities: &IdentityIndexV1,
    expected: &NormalizedStateV1,
) -> ResolvedViewDefectV1 {
    match defect {
        ViewDefectV1::Node(defect) => {
            ResolvedViewDefectV1::Node(resolve_node_defect(defect, identities))
        }
        ViewDefectV1::HiddenFragmentBinding { fragment } => {
            let descriptor = expected
                .fragment(&fragment)
                .expect("semantic fragment should exist")
                .descriptor();
            ResolvedViewDefectV1::HiddenFragmentBinding {
                owner: resolve_node(identities, fragment.owner()),
                descriptor,
            }
        }
        ViewDefectV1::SwappedKeyedMembers {
            fragment,
            left,
            right,
        } => ResolvedViewDefectV1::SwappedKeyedMembers {
            fragment: resolve_fragment(identities, &fragment),
            left,
            right,
        },
        ViewDefectV1::SwappedChildren { node, left, right } => {
            ResolvedViewDefectV1::SwappedChildren {
                node: resolve_node(identities, &node),
                left,
                right,
            }
        }
        ViewDefectV1::ReportedCounts {
            nodes,
            fragments,
            properties,
        } => ResolvedViewDefectV1::ReportedCounts {
            nodes,
            fragments,
            properties,
        },
        ViewDefectV1::ReportedChildCount { node, count } => {
            ResolvedViewDefectV1::ReportedChildCount {
                node: resolve_node(identities, &node),
                count,
            }
        }
        ViewDefectV1::ReportedKeyedCount { fragment, count } => {
            ResolvedViewDefectV1::ReportedKeyedCount {
                fragment: resolve_fragment(identities, &fragment),
                count,
            }
        }
        ViewDefectV1::NodeAlias { source, target } => ResolvedViewDefectV1::NodeAlias {
            source: resolve_node(identities, &source),
            target: resolve_node(identities, &target),
        },
        ViewDefectV1::FragmentAlias { source, target } => {
            let target_descriptor = expected
                .fragment(&target)
                .expect("semantic target fragment should exist")
                .descriptor();
            ResolvedViewDefectV1::FragmentAlias {
                source: resolve_fragment(identities, &source),
                target_owner: resolve_node(identities, target.owner()),
                target_descriptor,
            }
        }
    }
}

fn resolve_node_defect(
    defect: NodeViewDefectV1,
    identities: &IdentityIndexV1,
) -> ResolvedNodeViewDefectV1 {
    match defect {
        NodeViewDefectV1::Template { node, observed } => ResolvedNodeViewDefectV1::Template {
            node: resolve_node(identities, &node),
            observed,
        },
        NodeViewDefectV1::Component { node, observed } => ResolvedNodeViewDefectV1::Component {
            node: resolve_node(identities, &node),
            observed,
        },
        NodeViewDefectV1::Property {
            node,
            property,
            observed,
        } => ResolvedNodeViewDefectV1::Property {
            node: resolve_node(identities, &node),
            property,
            observed,
        },
        NodeViewDefectV1::Parent { node, observed } => ResolvedNodeViewDefectV1::Parent {
            node: resolve_node(identities, &node),
            observed: observed.map(|path| resolve_node(identities, &path)),
        },
    }
}

pub(super) fn swapped_index(index: usize, left: usize, right: usize) -> usize {
    if index == left {
        right
    } else if index == right {
        left
    } else {
        index
    }
}

fn resolve_node(identities: &IdentityIndexV1, path: &NodePathV1) -> NodeId {
    identities.node(path).expect("semantic node should resolve")
}

fn resolve_fragment(identities: &IdentityIndexV1, path: &FragmentPathV1) -> FragmentId {
    identities
        .fragment(path)
        .expect("semantic fragment should resolve")
}

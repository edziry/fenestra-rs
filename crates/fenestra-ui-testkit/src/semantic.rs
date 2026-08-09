use std::cmp::Ordering;

use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};

/// One authored step from a semantic node to a direct child.
///
/// Paths contain authored slots and keys only. They never retain runtime node
/// or fragment handles.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PathSegmentV1 {
    /// Selects the static child declared at an authored child slot.
    Static {
        /// Ordinal in the owning template's authored child list.
        authored_slot: u16,
    },
    /// Selects one keyed member from the region at an authored child slot.
    Member {
        /// Ordinal in the owning template's authored child list.
        region_slot: u16,
        /// Key local to the selected fragment.
        key: u64,
    },
}

impl PathSegmentV1 {
    /// Returns the authored child-slot ordinal selected by this segment.
    #[must_use]
    pub const fn authored_slot(&self) -> u16 {
        match self {
            Self::Static { authored_slot } => *authored_slot,
            Self::Member { region_slot, .. } => *region_slot,
        }
    }
}

impl Ord for PathSegmentV1 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.authored_slot()
            .cmp(&other.authored_slot())
            .then_with(|| match (self, other) {
                (Self::Static { .. }, Self::Static { .. }) => Ordering::Equal,
                (Self::Static { .. }, Self::Member { .. }) => Ordering::Less,
                (Self::Member { .. }, Self::Static { .. }) => Ordering::Greater,
                (Self::Member { key: left, .. }, Self::Member { key: right, .. }) => {
                    left.cmp(right)
                }
            })
    }
}

impl PartialOrd for PathSegmentV1 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Stable semantic address of one node in the registered fixture.
///
/// Equality, ordering, hashing, and debug output depend only on authored slots
/// and keyed-member values. Runtime identities cannot enter this type.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodePathV1 {
    segments: Vec<PathSegmentV1>,
}

impl NodePathV1 {
    /// Returns the semantic root path.
    #[must_use]
    pub const fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Extends this path with the static child at `authored_slot`.
    #[must_use]
    pub fn static_child(mut self, authored_slot: u16) -> Self {
        self.segments.push(PathSegmentV1::Static { authored_slot });
        self
    }

    /// Extends this path with `key` in the region at `region_slot`.
    #[must_use]
    pub fn member(mut self, region_slot: u16, key: u64) -> Self {
        self.segments
            .push(PathSegmentV1::Member { region_slot, key });
        self
    }

    /// Returns the ordered authored path segments after the root.
    #[must_use]
    pub fn segments(&self) -> &[PathSegmentV1] {
        &self.segments
    }

    /// Returns the number of segments after the root.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.segments.len()
    }
}

/// Stable semantic address of one structural fragment.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FragmentPathV1 {
    owner: NodePathV1,
    region_slot: u16,
}

impl FragmentPathV1 {
    /// Creates the fragment address for an owner's authored region slot.
    #[must_use]
    pub const fn new(owner: NodePathV1, region_slot: u16) -> Self {
        Self { owner, region_slot }
    }

    /// Returns the semantic path of the fragment owner.
    #[must_use]
    pub const fn owner(&self) -> &NodePathV1 {
        &self.owner
    }

    /// Returns the ordinal in the owner's authored child list.
    #[must_use]
    pub const fn region_slot(&self) -> u16 {
        self.region_slot
    }
}

/// One normalized effective property value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedPropertyV1 {
    property: PropertyId,
    value: PropertyValue,
}

impl NormalizedPropertyV1 {
    pub(crate) const fn new(property: PropertyId, value: PropertyValue) -> Self {
        Self { property, value }
    }

    /// Returns the property symbol local to the node's component.
    #[must_use]
    pub const fn property(&self) -> PropertyId {
        self.property
    }

    /// Returns the normalized effective value.
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }
}

/// One normalized authored child group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedChildGroupV1 {
    /// A static child at one authored slot.
    Static(NodePathV1),
    /// A structural fragment at one authored slot.
    Region(FragmentPathV1),
}

/// One node in an authored-preorder normalized state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedNodeV1 {
    path: NodePathV1,
    parent: Option<NodePathV1>,
    template: TemplateNodeId,
    component: ComponentTypeId,
    properties: Vec<NormalizedPropertyV1>,
    child_groups: Vec<NormalizedChildGroupV1>,
}

impl NormalizedNodeV1 {
    pub(crate) fn new(
        path: NodePathV1,
        parent: Option<NodePathV1>,
        template: TemplateNodeId,
        component: ComponentTypeId,
        properties: Vec<NormalizedPropertyV1>,
        child_groups: Vec<NormalizedChildGroupV1>,
    ) -> Self {
        Self {
            path,
            parent,
            template,
            component,
            properties,
            child_groups,
        }
    }

    /// Returns this node's semantic path.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the semantic parent, or `None` for the root.
    #[must_use]
    pub const fn parent(&self) -> Option<&NodePathV1> {
        self.parent.as_ref()
    }

    /// Returns the authored template symbol.
    #[must_use]
    pub const fn template(&self) -> TemplateNodeId {
        self.template
    }

    /// Returns the authored component symbol.
    #[must_use]
    pub const fn component(&self) -> ComponentTypeId {
        self.component
    }

    /// Returns every effective property in schema order.
    #[must_use]
    pub fn properties(&self) -> &[NormalizedPropertyV1] {
        &self.properties
    }

    /// Returns every child group in authored slot order.
    #[must_use]
    pub fn child_groups(&self) -> &[NormalizedChildGroupV1] {
        &self.child_groups
    }
}

/// One keyed member in committed fragment order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedMemberV1 {
    key: u64,
    node: NodePathV1,
}

impl NormalizedMemberV1 {
    pub(crate) const fn new(key: u64, node: NodePathV1) -> Self {
        Self { key, node }
    }

    /// Returns the key local to the fragment.
    #[must_use]
    pub const fn key(&self) -> u64 {
        self.key
    }

    /// Returns the semantic path of the member root.
    #[must_use]
    pub const fn node(&self) -> &NodePathV1 {
        &self.node
    }
}

/// One fragment in an authored-preorder normalized state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedFragmentV1 {
    path: FragmentPathV1,
    descriptor: StructuralRegionId,
    members: Vec<NormalizedMemberV1>,
}

impl NormalizedFragmentV1 {
    pub(crate) const fn new(
        path: FragmentPathV1,
        descriptor: StructuralRegionId,
        members: Vec<NormalizedMemberV1>,
    ) -> Self {
        Self {
            path,
            descriptor,
            members,
        }
    }

    /// Returns this fragment's semantic path.
    #[must_use]
    pub const fn path(&self) -> &FragmentPathV1 {
        &self.path
    }

    /// Returns the authored region descriptor symbol.
    #[must_use]
    pub const fn descriptor(&self) -> StructuralRegionId {
        self.descriptor
    }

    /// Returns keyed members in committed order.
    #[must_use]
    pub fn members(&self) -> &[NormalizedMemberV1] {
        &self.members
    }
}

/// Complete physical-identity-free logical state for oracle comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedStateV1 {
    nodes: Vec<NormalizedNodeV1>,
    fragments: Vec<NormalizedFragmentV1>,
}

impl NormalizedStateV1 {
    pub(crate) const fn new(
        nodes: Vec<NormalizedNodeV1>,
        fragments: Vec<NormalizedFragmentV1>,
    ) -> Self {
        Self { nodes, fragments }
    }

    /// Returns normalized nodes in authored preorder.
    #[must_use]
    pub fn nodes(&self) -> &[NormalizedNodeV1] {
        &self.nodes
    }

    /// Returns normalized fragments in authored preorder.
    #[must_use]
    pub fn fragments(&self) -> &[NormalizedFragmentV1] {
        &self.fragments
    }

    /// Returns the normalized record for `path`, if present.
    #[must_use]
    pub fn node(&self, path: &NodePathV1) -> Option<&NormalizedNodeV1> {
        self.nodes.iter().find(|node| node.path() == path)
    }

    /// Returns the normalized record for `path`, if present.
    #[must_use]
    pub fn fragment(&self, path: &FragmentPathV1) -> Option<&NormalizedFragmentV1> {
        self.fragments
            .iter()
            .find(|fragment| fragment.path() == path)
    }

    /// Returns whether the normalized state contains `path`.
    #[must_use]
    pub fn contains_node(&self, path: &NodePathV1) -> bool {
        self.node(path).is_some()
    }

    /// Returns whether the normalized state contains `path`.
    #[must_use]
    pub fn contains_fragment(&self, path: &FragmentPathV1) -> bool {
        self.fragment(path).is_some()
    }

    /// Returns the number of normalized nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of normalized fragments.
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    /// Returns the total number of normalized property slots.
    #[must_use]
    pub fn property_slot_count(&self) -> usize {
        self.nodes.iter().map(|node| node.properties.len()).sum()
    }
}

/// Inclusive resource categories for one layout computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutLimitKindV1 {
    /// Input nodes and corresponding output records.
    Nodes,
    /// Root-inclusive tree depth.
    Depth,
    /// Direct children of one node.
    ChildrenPerNode,
}

impl LayoutLimitKindV1 {
    /// Every resource category in deterministic validation order.
    pub const ALL: [Self; 3] = [Self::Nodes, Self::Depth, Self::ChildrenPerNode];
}

/// Complete inclusive tree limits for one layout computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutLimitsV1 {
    values: [usize; 3],
}

impl LayoutLimitsV1 {
    /// Creates one explicit complete limit set.
    #[must_use]
    pub const fn new(nodes: usize, depth: usize, children_per_node: usize) -> Self {
        Self {
            values: [nodes, depth, children_per_node],
        }
    }

    /// Returns the inclusive bound for one resource category.
    #[must_use]
    pub const fn limit(self, kind: LayoutLimitKindV1) -> usize {
        self.values[kind as usize]
    }
}

/// Registered WU-0011 direct-corpus profile.
///
/// This is an experiment profile rather than a product capacity.
pub const REGISTERED_LAYOUT_LIMITS_V1: LayoutLimitsV1 = LayoutLimitsV1::new(32, 8, 16);

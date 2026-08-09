/// Explicit inclusive bounds for the runtime transaction prototype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapacity {
    operations: usize,
    structural_changes: usize,
    live_nodes: usize,
    live_fragments: usize,
    live_property_slots: usize,
    retained_generations: usize,
}

impl RuntimeCapacity {
    /// Creates a complete set of runtime bounds.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        operations: usize,
        structural_changes: usize,
        live_nodes: usize,
        live_fragments: usize,
        live_property_slots: usize,
        retained_generations: usize,
    ) -> Self {
        Self {
            operations,
            structural_changes,
            live_nodes,
            live_fragments,
            live_property_slots,
            retained_generations,
        }
    }

    pub(crate) const fn operations(self) -> usize {
        self.operations
    }

    pub(crate) const fn structural_changes(self) -> usize {
        self.structural_changes
    }

    pub(crate) const fn live_nodes(self) -> usize {
        self.live_nodes
    }

    pub(crate) const fn live_fragments(self) -> usize {
        self.live_fragments
    }

    pub(crate) const fn live_property_slots(self) -> usize {
        self.live_property_slots
    }

    pub(crate) const fn retained_generations(self) -> usize {
        self.retained_generations
    }

    /// Returns bounds with a new staged-operation limit.
    #[must_use]
    pub const fn with_operations(mut self, value: usize) -> Self {
        self.operations = value;
        self
    }

    /// Returns bounds with a new structural-change limit.
    #[must_use]
    pub const fn with_structural_changes(mut self, value: usize) -> Self {
        self.structural_changes = value;
        self
    }

    /// Returns bounds with a new live-node limit.
    #[must_use]
    pub const fn with_live_nodes(mut self, value: usize) -> Self {
        self.live_nodes = value;
        self
    }

    /// Returns bounds with a new live-fragment limit.
    #[must_use]
    pub const fn with_live_fragments(mut self, value: usize) -> Self {
        self.live_fragments = value;
        self
    }

    /// Returns bounds with a new live-property-slot limit.
    #[must_use]
    pub const fn with_live_property_slots(mut self, value: usize) -> Self {
        self.live_property_slots = value;
        self
    }

    /// Returns bounds with a new retained-generation limit.
    #[must_use]
    pub const fn with_retained_generations(mut self, value: usize) -> Self {
        self.retained_generations = value;
        self
    }
}

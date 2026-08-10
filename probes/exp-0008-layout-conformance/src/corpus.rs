mod boundaries;
mod geometry;
mod limits;
mod support;

use fenestra_ui_layout::prototype::{LayoutNodeV1, LayoutRecordV1, LayoutViewportV1};

/// One owned direct conformance case with an independently authored oracle.
pub struct RegisteredLayoutCaseV1 {
    name: &'static str,
    viewport: LayoutViewportV1,
    nodes: Vec<LayoutNodeV1>,
    expected_records: Vec<LayoutRecordV1>,
}

impl RegisteredLayoutCaseV1 {
    pub(super) fn new(
        name: &'static str,
        viewport: LayoutViewportV1,
        nodes: Vec<LayoutNodeV1>,
        expected_records: Vec<LayoutRecordV1>,
    ) -> Self {
        Self {
            name,
            viewport,
            nodes,
            expected_records,
        }
    }

    /// Returns the stable case name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the case's logical viewport.
    #[must_use]
    pub const fn viewport(&self) -> LayoutViewportV1 {
        self.viewport
    }

    /// Returns authored nodes in dense preorder.
    #[must_use]
    pub fn nodes(&self) -> &[LayoutNodeV1] {
        &self.nodes
    }

    /// Returns independently authored expected records in key order.
    #[must_use]
    pub fn expected_records(&self) -> &[LayoutRecordV1] {
        &self.expected_records
    }
}

/// Builds the 23 registered direct layout cases in stable evidence order.
#[must_use]
pub fn registered_layout_corpus_v1() -> Vec<RegisteredLayoutCaseV1> {
    let mut cases = Vec::with_capacity(23);
    cases.extend(geometry::cases());
    cases.extend(boundaries::cases());
    cases.extend(limits::cases());
    cases
}

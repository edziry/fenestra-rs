use std::fmt;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

use crate::logical_tree::NodeId;

use super::types::{
    ComputedStyleRecord, GeometryRecord, HeadlessProjectionState, HitRegionRecord,
    SceneRectangleRecord, SemanticRecord,
};
use super::{
    HeadlessPoint, HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};
use crate::runtime::state::RuntimeGeneration;

/// Immutable view of one node's materialized property values.
#[derive(Clone, Copy)]
pub struct ComputedStyleView<'a> {
    record: &'a ComputedStyleRecord,
}

impl<'a> ComputedStyleView<'a> {
    /// Returns the logical node described by this record.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.record.node
    }

    /// Returns one materialized property value.
    #[must_use]
    pub fn property(self, property: PropertyId) -> Option<&'a PropertyValue> {
        self.record
            .properties
            .iter()
            .find(|candidate| candidate.id == property)
            .map(|candidate| &candidate.value)
    }
}

/// Immutable view of one node's absolute geometry and effective clip.
#[derive(Clone, Copy)]
pub struct HeadlessGeometryView<'a> {
    record: &'a GeometryRecord,
}

impl HeadlessGeometryView<'_> {
    /// Returns the logical node described by this record.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.record.node
    }

    /// Returns the unclipped absolute bounds.
    #[must_use]
    pub const fn bounds(self) -> HeadlessRect {
        self.record.bounds
    }

    /// Returns the effective ancestor and surface clip.
    #[must_use]
    pub const fn clip(self) -> HeadlessRect {
        self.record.clip
    }
}

/// Immutable view of one closed semantic record.
#[derive(Clone, Copy)]
pub struct HeadlessSemanticView<'a> {
    record: &'a SemanticRecord,
}

impl HeadlessSemanticView<'_> {
    /// Returns the logical node described by this record.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.record.node
    }

    /// Returns the closed semantic role.
    #[must_use]
    pub const fn role(self) -> HeadlessSemanticRole {
        self.record.role
    }

    /// Returns the closed fixture label symbol.
    #[must_use]
    pub const fn label(self) -> u32 {
        self.record.label
    }

    /// Returns the closed semantic action.
    #[must_use]
    pub const fn action(self) -> HeadlessSemanticAction {
        self.record.action
    }
}

/// Immutable view of one ordered input region.
#[derive(Clone, Copy)]
pub struct HeadlessHitRegionView<'a> {
    record: &'a HitRegionRecord,
}

impl HeadlessHitRegionView<'_> {
    /// Returns the logical input target.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.record.node
    }

    /// Returns the effective half-open hit rectangle.
    #[must_use]
    pub const fn clip(self) -> HeadlessRect {
        self.record.clip
    }
}

/// Immutable view of one authored-order scene rectangle.
#[derive(Clone, Copy)]
pub struct HeadlessSceneRectangleView<'a> {
    record: &'a SceneRectangleRecord,
}

impl HeadlessSceneRectangleView<'_> {
    /// Returns the logical node that produced this rectangle.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.record.node
    }

    /// Returns the clipped scene rectangle.
    #[must_use]
    pub const fn rectangle(self) -> HeadlessRect {
        self.record.rectangle
    }

    /// Returns the materialized fixture color.
    #[must_use]
    pub const fn color(self) -> [u8; 4] {
        self.record.color
    }
}

impl fmt::Debug for ComputedStyleView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComputedStyleView")
            .field("property_count", &self.record.properties.len())
            .finish()
    }
}

/// Immutable view of the headless projection in one committed generation.
#[derive(Clone, Copy)]
pub struct HeadlessProjectionView<'a> {
    state: &'a HeadlessProjectionState,
    generation: RuntimeGeneration,
}

impl<'a> HeadlessProjectionView<'a> {
    pub(crate) const fn new(
        state: &'a HeadlessProjectionState,
        generation: RuntimeGeneration,
    ) -> Self {
        Self { state, generation }
    }

    /// Returns the committed generation shared with the logical snapshot.
    #[must_use]
    pub const fn generation(self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the logical surface retained by this generation.
    #[must_use]
    pub const fn surface(self) -> HeadlessSurface {
        self.state.surface
    }

    /// Resolves one live node's computed-style record.
    #[must_use]
    pub fn computed_style(self, node: NodeId) -> Option<ComputedStyleView<'a>> {
        self.state
            .computed_styles
            .iter()
            .find(|candidate| candidate.node == node)
            .map(|record| ComputedStyleView { record })
    }

    /// Iterates computed-style records in authored tree order.
    pub fn computed_styles(self) -> impl ExactSizeIterator<Item = ComputedStyleView<'a>> + 'a {
        self.state
            .computed_styles
            .iter()
            .map(|record| ComputedStyleView { record })
    }

    /// Resolves one live node's geometry record.
    #[must_use]
    pub fn geometry(self, node: NodeId) -> Option<HeadlessGeometryView<'a>> {
        self.state
            .geometry
            .iter()
            .find(|candidate| candidate.node == node)
            .map(|record| HeadlessGeometryView { record })
    }

    /// Iterates geometry records in authored tree order.
    pub fn geometries(self) -> impl ExactSizeIterator<Item = HeadlessGeometryView<'a>> + 'a {
        self.state
            .geometry
            .iter()
            .map(|record| HeadlessGeometryView { record })
    }

    /// Iterates visible semantic records in authored tree order.
    pub fn semantics(self) -> impl ExactSizeIterator<Item = HeadlessSemanticView<'a>> + 'a {
        self.state
            .semantics
            .iter()
            .map(|record| HeadlessSemanticView { record })
    }

    /// Iterates accepting hit regions in authored scene order.
    pub fn hit_regions(self) -> impl ExactSizeIterator<Item = HeadlessHitRegionView<'a>> + 'a {
        self.state
            .hit_regions
            .iter()
            .map(|record| HeadlessHitRegionView { record })
    }

    /// Iterates visible scene rectangles in authored order.
    pub fn scene_rectangles(
        self,
    ) -> impl ExactSizeIterator<Item = HeadlessSceneRectangleView<'a>> + 'a {
        self.state
            .scene_rectangles
            .iter()
            .map(|record| HeadlessSceneRectangleView { record })
    }

    /// Returns the topmost accepting node at a logical point.
    #[must_use]
    pub fn hit_test(self, point: HeadlessPoint) -> Option<NodeId> {
        self.state
            .hit_regions
            .iter()
            .rev()
            .find_map(|record| record.clip.contains(point).then_some(record.node))
    }

    /// Returns the number of computed-style records.
    #[must_use]
    pub fn computed_style_count(self) -> usize {
        self.state.computed_styles.len()
    }

    /// Returns the number of geometry records.
    #[must_use]
    pub fn geometry_count(self) -> usize {
        self.state.geometry.len()
    }

    /// Returns the number of visible semantic records.
    #[must_use]
    pub fn semantic_count(self) -> usize {
        self.state.semantics.len()
    }

    /// Returns the number of accepting hit regions.
    #[must_use]
    pub fn hit_region_count(self) -> usize {
        self.state.hit_regions.len()
    }

    /// Returns the number of visible scene rectangles.
    #[must_use]
    pub fn scene_rectangle_count(self) -> usize {
        self.state.scene_rectangles.len()
    }
}

impl fmt::Debug for HeadlessProjectionView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessProjectionView")
            .field("generation", &self.generation)
            .field("computed_style_count", &self.state.computed_styles.len())
            .field("geometry_count", &self.state.geometry.len())
            .field("semantic_count", &self.state.semantics.len())
            .field("hit_region_count", &self.state.hit_regions.len())
            .field("scene_rectangle_count", &self.state.scene_rectangles.len())
            .finish_non_exhaustive()
    }
}

//! Aggregate borrowed spatial input.

use crate::content_input::{SpatialItemInputV2, SpatialResourceInputV2};
use crate::geometry_input::SpatialGeometryInputV2;
use crate::topology::SpatialTopologyInputV2;

/// Borrowed aggregate view of every raw spatial input table.
#[derive(Clone, Copy)]
pub struct SpatialInputV2<'a> {
    topology: SpatialTopologyInputV2<'a>,
    geometry: SpatialGeometryInputV2<'a>,
    resources: SpatialResourceInputV2<'a>,
    items: SpatialItemInputV2<'a>,
}

impl<'a> SpatialInputV2<'a> {
    /// Creates one aggregate borrowed input without validating it.
    #[must_use]
    pub const fn new(
        topology: SpatialTopologyInputV2<'a>,
        geometry: SpatialGeometryInputV2<'a>,
        resources: SpatialResourceInputV2<'a>,
        items: SpatialItemInputV2<'a>,
    ) -> Self {
        Self {
            topology,
            geometry,
            resources,
            items,
        }
    }

    /// Returns the borrowed topology view.
    #[must_use]
    pub const fn topology(self) -> SpatialTopologyInputV2<'a> {
        self.topology
    }

    /// Returns the borrowed geometry view.
    #[must_use]
    pub const fn geometry(self) -> SpatialGeometryInputV2<'a> {
        self.geometry
    }

    /// Returns the borrowed resource view.
    #[must_use]
    pub const fn resources(self) -> SpatialResourceInputV2<'a> {
        self.resources
    }

    /// Returns the borrowed ordered-item view.
    #[must_use]
    pub const fn items(self) -> SpatialItemInputV2<'a> {
        self.items
    }
}

//! Borrowed view of raw local geometry tables.

use crate::coverage::SpatialClipV2;
use crate::model::SpatialPointV2;
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::shape::SpatialShapeV2;

/// Borrowed raw geometry input in registered table order.
#[derive(Clone, Copy)]
pub struct SpatialGeometryInputV2<'a> {
    polygon_points: &'a [SpatialPointV2],
    path_verbs: &'a [SpatialPathVerbV2],
    paths: &'a [SpatialPathV2],
    shapes: &'a [SpatialShapeV2],
    clips: &'a [SpatialClipV2],
}

impl<'a> SpatialGeometryInputV2<'a> {
    /// Creates one borrowed raw geometry view.
    #[must_use]
    pub const fn new(
        polygon_points: &'a [SpatialPointV2],
        path_verbs: &'a [SpatialPathVerbV2],
        paths: &'a [SpatialPathV2],
        shapes: &'a [SpatialShapeV2],
        clips: &'a [SpatialClipV2],
    ) -> Self {
        Self {
            polygon_points,
            path_verbs,
            paths,
            shapes,
            clips,
        }
    }

    /// Returns the supplied polygon-point table.
    #[must_use]
    pub const fn polygon_points(self) -> &'a [SpatialPointV2] {
        self.polygon_points
    }

    /// Returns the supplied path-verb table.
    #[must_use]
    pub const fn path_verbs(self) -> &'a [SpatialPathVerbV2] {
        self.path_verbs
    }

    /// Returns the supplied path table.
    #[must_use]
    pub const fn paths(self) -> &'a [SpatialPathV2] {
        self.paths
    }

    /// Returns the supplied shape table.
    #[must_use]
    pub const fn shapes(self) -> &'a [SpatialShapeV2] {
        self.shapes
    }

    /// Returns the supplied clip table.
    #[must_use]
    pub const fn clips(self) -> &'a [SpatialClipV2] {
        self.clips
    }
}

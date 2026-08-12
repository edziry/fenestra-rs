//! Owned immutable spatial input storage.

use crate::aggregate_input::SpatialInputV2;
use crate::brush::{SpatialBrushV2, SpatialGradientStopV2};
use crate::content_input::{SpatialItemInputV2, SpatialResourceInputV2};
use crate::content_item::{SpatialHitV2, SpatialSemanticGeometryV2};
use crate::coverage::SpatialClipV2;
use crate::geometry_input::SpatialGeometryInputV2;
use crate::image::SpatialImageV2;
use crate::model::{SpatialPointV2, SpatialViewportV2};
use crate::paint::SpatialPaintV2;
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::shape::SpatialShapeV2;
use crate::topology::{SpatialNodeV2, SpatialTopologyInputV2};

/// Immutable owner for every raw spatial input table.
pub struct SpatialOwnedInputV2 {
    viewport: SpatialViewportV2,
    nodes: Box<[SpatialNodeV2]>,
    polygon_points: Box<[SpatialPointV2]>,
    path_verbs: Box<[SpatialPathVerbV2]>,
    paths: Box<[SpatialPathV2]>,
    shapes: Box<[SpatialShapeV2]>,
    clips: Box<[SpatialClipV2]>,
    gradient_stops: Box<[SpatialGradientStopV2]>,
    brushes: Box<[SpatialBrushV2]>,
    images: Box<[SpatialImageV2]>,
    paint_items: Box<[SpatialPaintV2]>,
    hit_items: Box<[SpatialHitV2]>,
    semantic_items: Box<[SpatialSemanticGeometryV2]>,
}

impl SpatialOwnedInputV2 {
    /// Takes ownership of every raw spatial input table without validating it.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        viewport: SpatialViewportV2,
        nodes: Box<[SpatialNodeV2]>,
        polygon_points: Box<[SpatialPointV2]>,
        path_verbs: Box<[SpatialPathVerbV2]>,
        paths: Box<[SpatialPathV2]>,
        shapes: Box<[SpatialShapeV2]>,
        clips: Box<[SpatialClipV2]>,
        gradient_stops: Box<[SpatialGradientStopV2]>,
        brushes: Box<[SpatialBrushV2]>,
        images: Box<[SpatialImageV2]>,
        paint_items: Box<[SpatialPaintV2]>,
        hit_items: Box<[SpatialHitV2]>,
        semantic_items: Box<[SpatialSemanticGeometryV2]>,
    ) -> Self {
        Self {
            viewport,
            nodes,
            polygon_points,
            path_verbs,
            paths,
            shapes,
            clips,
            gradient_stops,
            brushes,
            images,
            paint_items,
            hit_items,
            semantic_items,
        }
    }

    /// Borrows every owned table as one aggregate validation input.
    #[must_use]
    pub fn as_input(&self) -> SpatialInputV2<'_> {
        SpatialInputV2::new(
            SpatialTopologyInputV2::new(self.viewport, self.nodes.as_ref()),
            SpatialGeometryInputV2::new(
                self.polygon_points.as_ref(),
                self.path_verbs.as_ref(),
                self.paths.as_ref(),
                self.shapes.as_ref(),
                self.clips.as_ref(),
            ),
            SpatialResourceInputV2::new(
                self.gradient_stops.as_ref(),
                self.brushes.as_ref(),
                self.images.as_ref(),
            ),
            SpatialItemInputV2::new(
                self.paint_items.as_ref(),
                self.hit_items.as_ref(),
                self.semantic_items.as_ref(),
            ),
        )
    }
}

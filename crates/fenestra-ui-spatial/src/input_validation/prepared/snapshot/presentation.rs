//! Borrowed immutable paint projection over one accepted spatial snapshot.

use super::SpatialResolvedSnapshotV2;
use crate::aabb::SpatialAabbV2;
use crate::brush::{SpatialBrushV2, SpatialGradientStopV2};
use crate::coverage::SpatialClipV2;
use crate::image::SpatialImageV2;
use crate::model::{SpatialPointV2, SpatialViewportV2};
use crate::output_geometry::SpatialClipOutputRecordV2;
use crate::output_item::SpatialPaintOutputRecordV2;
use crate::paint::SpatialPaintV2;
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::reference_raster::{ReferenceRasterErrorV2, ReferenceRasterLimitsV2, ReferenceRasterV2};
use crate::shape::SpatialShapeV2;

/// Borrowed paint-only view of one exact accepted spatial snapshot.
#[derive(Clone, Copy)]
pub struct SpatialPaintFrameV2<'a> {
    snapshot: &'a SpatialResolvedSnapshotV2,
}

impl SpatialResolvedSnapshotV2 {
    /// Borrows the coherent paint projection retained by this snapshot.
    #[must_use]
    pub fn paint_frame(&self) -> SpatialPaintFrameV2<'_> {
        SpatialPaintFrameV2 { snapshot: self }
    }
}

impl<'a> SpatialPaintFrameV2<'a> {
    /// Returns the logical viewport used to resolve this paint frame.
    #[must_use]
    pub fn viewport(self) -> SpatialViewportV2 {
        self.snapshot.viewport()
    }

    /// Returns the accepted polygon-point table.
    #[must_use]
    pub fn polygon_points(self) -> &'a [SpatialPointV2] {
        self.input().geometry().polygon_points()
    }

    /// Returns the accepted path-verb table.
    #[must_use]
    pub fn path_verbs(self) -> &'a [SpatialPathVerbV2] {
        self.input().geometry().path_verbs()
    }

    /// Returns the accepted path table.
    #[must_use]
    pub fn paths(self) -> &'a [SpatialPathV2] {
        self.input().geometry().paths()
    }

    /// Returns the accepted shape table.
    #[must_use]
    pub fn shapes(self) -> &'a [SpatialShapeV2] {
        self.input().geometry().shapes()
    }

    /// Returns the accepted clip-primitive table.
    #[must_use]
    pub fn clip_primitives(self) -> &'a [SpatialClipV2] {
        self.input().geometry().clips()
    }

    /// Returns the accepted gradient-stop table.
    #[must_use]
    pub fn gradient_stops(self) -> &'a [SpatialGradientStopV2] {
        self.input().resources().gradient_stops()
    }

    /// Returns the accepted brush table.
    #[must_use]
    pub fn brushes(self) -> &'a [SpatialBrushV2] {
        self.input().resources().brushes()
    }

    /// Returns the accepted normalized-image table.
    #[must_use]
    pub fn images(self) -> &'a [SpatialImageV2] {
        self.input().resources().images()
    }

    /// Returns the accepted ordered paint-item table.
    #[must_use]
    pub fn paint_items(self) -> &'a [SpatialPaintV2] {
        self.input().items().paint_items()
    }

    /// Returns the resolved clip rows paired with clip primitives by ordinal.
    #[must_use]
    pub fn resolved_clips(self) -> &'a [SpatialClipOutputRecordV2] {
        &self.snapshot.clips
    }

    /// Returns the effective clip bounds paired with clip primitives by ordinal.
    #[must_use]
    pub fn effective_clip_aabbs(self) -> &'a [SpatialAabbV2] {
        self.snapshot.effective_clip_aabbs()
    }

    /// Returns resolved paint rows in accepted painter order.
    #[must_use]
    pub fn resolved_paints(self) -> &'a [SpatialPaintOutputRecordV2] {
        &self.snapshot.paints
    }

    /// Renders this exact retained paint projection with the reference rasterizer.
    #[must_use = "reference raster errors must be handled"]
    pub fn rasterize_reference(
        self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2> {
        self.snapshot.rasterize_reference(limits)
    }

    fn input(self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.snapshot.prepared.source.as_input()
    }
}

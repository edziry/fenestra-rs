//! Borrowed views of raw content tables.

use crate::brush::{SpatialBrushV2, SpatialGradientStopV2};
use crate::content_item::{SpatialHitV2, SpatialSemanticGeometryV2};
use crate::image::SpatialImageV2;
use crate::paint::SpatialPaintV2;

/// Borrowed raw resource input in registered table order.
#[derive(Clone, Copy)]
pub struct SpatialResourceInputV2<'a> {
    gradient_stops: &'a [SpatialGradientStopV2],
    brushes: &'a [SpatialBrushV2],
    images: &'a [SpatialImageV2],
}

impl<'a> SpatialResourceInputV2<'a> {
    /// Creates one borrowed raw resource view.
    #[must_use]
    pub const fn new(
        gradient_stops: &'a [SpatialGradientStopV2],
        brushes: &'a [SpatialBrushV2],
        images: &'a [SpatialImageV2],
    ) -> Self {
        Self {
            gradient_stops,
            brushes,
            images,
        }
    }

    /// Returns the supplied gradient-stop table.
    #[must_use]
    pub const fn gradient_stops(self) -> &'a [SpatialGradientStopV2] {
        self.gradient_stops
    }

    /// Returns the supplied brush table.
    #[must_use]
    pub const fn brushes(self) -> &'a [SpatialBrushV2] {
        self.brushes
    }

    /// Returns the supplied owned-image table.
    #[must_use]
    pub const fn images(self) -> &'a [SpatialImageV2] {
        self.images
    }
}

/// Borrowed raw paint, hit, and semantic item input.
#[derive(Clone, Copy)]
pub struct SpatialItemInputV2<'a> {
    paint_items: &'a [SpatialPaintV2],
    hit_items: &'a [SpatialHitV2],
    semantic_items: &'a [SpatialSemanticGeometryV2],
}

impl<'a> SpatialItemInputV2<'a> {
    /// Creates one borrowed raw item view.
    #[must_use]
    pub const fn new(
        paint_items: &'a [SpatialPaintV2],
        hit_items: &'a [SpatialHitV2],
        semantic_items: &'a [SpatialSemanticGeometryV2],
    ) -> Self {
        Self {
            paint_items,
            hit_items,
            semantic_items,
        }
    }

    /// Returns the supplied paint-item table.
    #[must_use]
    pub const fn paint_items(self) -> &'a [SpatialPaintV2] {
        self.paint_items
    }

    /// Returns the supplied hit-item table.
    #[must_use]
    pub const fn hit_items(self) -> &'a [SpatialHitV2] {
        self.hit_items
    }

    /// Returns the supplied semantic-geometry table.
    #[must_use]
    pub const fn semantic_items(self) -> &'a [SpatialSemanticGeometryV2] {
        self.semantic_items
    }
}

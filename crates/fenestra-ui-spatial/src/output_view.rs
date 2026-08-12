//! Borrowed candidate output table view.

use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
use crate::output_item::{
    SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialSemanticOutputRecordV2,
};

/// Borrowed view of every raw candidate output table.
#[derive(Clone, Copy)]
pub struct SpatialOutputV2<'a> {
    geometry: &'a [SpatialGeometryOutputRecordV2],
    clips: &'a [SpatialClipOutputRecordV2],
    paints: &'a [SpatialPaintOutputRecordV2],
    hits: &'a [SpatialHitOutputRecordV2],
    semantics: &'a [SpatialSemanticOutputRecordV2],
}

impl<'a> SpatialOutputV2<'a> {
    /// Creates one borrowed output view without validating its records.
    #[must_use]
    pub const fn new(
        geometry: &'a [SpatialGeometryOutputRecordV2],
        clips: &'a [SpatialClipOutputRecordV2],
        paints: &'a [SpatialPaintOutputRecordV2],
        hits: &'a [SpatialHitOutputRecordV2],
        semantics: &'a [SpatialSemanticOutputRecordV2],
    ) -> Self {
        Self {
            geometry,
            clips,
            paints,
            hits,
            semantics,
        }
    }

    /// Returns the exact supplied geometry-record slice.
    #[must_use]
    pub const fn geometry(self) -> &'a [SpatialGeometryOutputRecordV2] {
        self.geometry
    }

    /// Returns the exact supplied clip-record slice.
    #[must_use]
    pub const fn clips(self) -> &'a [SpatialClipOutputRecordV2] {
        self.clips
    }

    /// Returns the exact supplied paint-record slice.
    #[must_use]
    pub const fn paints(self) -> &'a [SpatialPaintOutputRecordV2] {
        self.paints
    }

    /// Returns the exact supplied hit-record slice.
    #[must_use]
    pub const fn hits(self) -> &'a [SpatialHitOutputRecordV2] {
        self.hits
    }

    /// Returns the exact supplied semantic-record slice.
    #[must_use]
    pub const fn semantics(self) -> &'a [SpatialSemanticOutputRecordV2] {
        self.semantics
    }
}

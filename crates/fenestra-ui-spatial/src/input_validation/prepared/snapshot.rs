//! Immutable resolved snapshot materialized from complete phase-10 state.

use super::PreparedSpatialV2;
use crate::aabb::SpatialAabbV2;
use crate::model::SpatialViewportV2;
use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
use crate::output_item::{
    SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialSemanticOutputRecordV2,
};
use crate::output_view::SpatialOutputV2;

#[cfg(test)]
mod facts;
mod materialize;
mod validate;

/// Immutable lifetime-free spatial state ready for downstream consumers.
pub struct SpatialResolvedSnapshotV2 {
    prepared: PreparedSpatialV2,
    geometry: Box<[SpatialGeometryOutputRecordV2]>,
    clips: Box<[SpatialClipOutputRecordV2]>,
    paints: Box<[SpatialPaintOutputRecordV2]>,
    hits: Box<[SpatialHitOutputRecordV2]>,
    semantics: Box<[SpatialSemanticOutputRecordV2]>,
}

impl SpatialResolvedSnapshotV2 {
    /// Returns the validated viewport used to resolve this snapshot.
    #[must_use]
    pub const fn viewport(&self) -> SpatialViewportV2 {
        self.prepared.state.viewport
    }

    /// Borrows all resolved output tables without copying their records.
    #[must_use]
    pub fn output(&self) -> SpatialOutputV2<'_> {
        SpatialOutputV2::new(
            &self.geometry,
            &self.clips,
            &self.paints,
            &self.hits,
            &self.semantics,
        )
    }

    /// Returns effective clip bounds after parent-chain intersection.
    #[must_use]
    pub fn effective_clip_aabbs(&self) -> &[SpatialAabbV2] {
        &self.prepared.state.effective_clip_aabbs
    }
}

/// Materializes the deterministic reference output from fully prepared state.
#[must_use]
pub fn materialize_reference_spatial_v2(prepared: PreparedSpatialV2) -> SpatialResolvedSnapshotV2 {
    let tables = materialize::materialize_tables(&prepared.state);
    SpatialResolvedSnapshotV2 {
        prepared,
        geometry: tables.geometry,
        clips: tables.clips,
        paints: tables.paints,
        hits: tables.hits,
        semantics: tables.semantics,
    }
}

/// Validates supplied candidate tables against fully prepared spatial state.
#[must_use = "candidate output validation errors must be handled before publication"]
pub fn validate_spatial_output_v2(
    prepared: PreparedSpatialV2,
    supplied: SpatialOutputV2<'_>,
) -> Result<SpatialResolvedSnapshotV2, crate::resolve_error::SpatialResolveErrorV2> {
    validate::validate(prepared, supplied)
}

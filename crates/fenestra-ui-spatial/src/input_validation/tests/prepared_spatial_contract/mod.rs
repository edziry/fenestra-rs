use crate::prototype::validate_spatial_output_v2;
use crate::prototype::{
    PreparedSpatialV2, REGISTERED_REFERENCE_RASTER_LIMITS_V2, ReferenceRasterErrorKindV2,
    ReferenceRasterErrorV2, ReferenceRasterLimitKindV2, ReferenceRasterLimitsV2, ReferenceRasterV2,
    SpatialHitResultV2, SpatialResolvedSnapshotV2, materialize_reference_spatial_v2,
    prepare_spatial_v2, resolve_spatial_v2,
};

mod errors;
mod hit_authority_miss;
mod hit_clips;
mod hit_coverage_fill;
mod hit_coverage_stroke;
mod hit_ownership;
mod hit_selection;
mod hit_support;
mod hit_surface;
mod one_shot_errors;
mod one_shot_success;
mod ownership;
mod raster_authority_clips;
mod raster_geometry;
mod raster_limits;
mod raster_ownership;
mod raster_resources;
mod raster_sampling;
mod raster_support;
mod raster_surface;
mod retention_derived;
mod retention_resources;
mod root_success;
mod signatures;
mod snapshot_empty;
mod snapshot_output;
mod snapshot_ownership;
mod snapshot_retention;
mod support;
mod surface;
mod traits;
mod validator_aabbs;
mod validator_clips;
mod validator_counts_keys;
mod validator_determinants;
mod validator_ownership;
mod validator_priority;
mod validator_projection;
mod validator_reference_priority;
mod validator_references;
mod validator_rounding;
mod validator_scalars_extents;
mod validator_success;
mod validator_support;

#[allow(dead_code)]
trait ReferenceRasterRedBridge {
    fn rasterize_reference(
        &self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2>;
}

impl ReferenceRasterRedBridge for SpatialResolvedSnapshotV2 {
    fn rasterize_reference(
        &self,
        _limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2> {
        panic!("SpatialResolvedSnapshotV2::rasterize_reference is not implemented")
    }
}

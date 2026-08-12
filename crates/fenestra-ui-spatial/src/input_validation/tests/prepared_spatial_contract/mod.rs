use crate::prototype::validate_spatial_output_v2;
use crate::prototype::{
    PreparedSpatialV2, SpatialHitResultV2, SpatialResolvedSnapshotV2,
    materialize_reference_spatial_v2, prepare_spatial_v2, resolve_spatial_v2,
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

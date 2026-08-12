use crate::prototype::{
    PreparedSpatialV2, SpatialResolvedSnapshotV2, materialize_reference_spatial_v2,
    prepare_spatial_v2,
};

mod errors;
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

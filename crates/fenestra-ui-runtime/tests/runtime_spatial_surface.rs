use fenestra_ui_runtime::prototype::{
    RuntimeSpatialBuildViewV2, RuntimeSpatialErrorV2, RuntimeSpatialInputV2,
    RuntimeSpatialProgramV2, RuntimeSpatialViewV2, SpatialViewportChangeViewV2,
};

#[path = "support/spatial/mod.rs"]
mod spatial_support;
#[path = "support/mod.rs"]
mod support;

#[path = "runtime_spatial_surface/contract.rs"]
mod contract;
#[path = "runtime_spatial_surface/dependency.rs"]
mod dependency;
#[path = "runtime_spatial_surface/errors.rs"]
mod errors;
#[path = "runtime_spatial_surface/initialization.rs"]
mod initialization;
#[path = "runtime_spatial_surface/initialization_cleanup.rs"]
mod initialization_cleanup;
#[path = "runtime_spatial_surface/initialization_errors.rs"]
mod initialization_errors;
#[path = "runtime_spatial_surface/registry.rs"]
mod registry;
#[path = "runtime_spatial_surface/signatures.rs"]
mod signatures;
#[path = "runtime_spatial_surface/source.rs"]
mod source;
#[path = "runtime_spatial_surface/traits.rs"]
mod traits;

use fenestra_ui_runtime::prototype::{
    RuntimePaintFrameV2, RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2,
};

#[allow(dead_code)]
#[path = "support/spatial/mod.rs"]
mod spatial_support;
#[path = "support/mod.rs"]
mod support;

#[path = "runtime_paint_frame_contract/behavior.rs"]
mod behavior;
#[path = "runtime_paint_frame_contract/fixture.rs"]
mod fixture;
#[path = "runtime_paint_frame_contract/signatures.rs"]
mod signatures;
#[path = "runtime_spatial_surface/source.rs"]
mod source;
#[path = "runtime_paint_frame_contract/surface.rs"]
mod surface;
#[path = "runtime_paint_frame_contract/traits.rs"]
mod traits;

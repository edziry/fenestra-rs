use fenestra_ui_ir::prototype::ValidatedSpatialProgramV2;
use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_runtime::prototype::{
    RuntimeInitializationError, RuntimeSpatialBuildViewV2, RuntimeSpatialErrorV2,
    RuntimeSpatialInputV2, RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2,
    RuntimeSpatialProgramV2, UiRuntime,
};
use fenestra_ui_spatial::prototype::{SpatialLimitsV2, SpatialViewportV2};

#[path = "support/spatial/mod.rs"]
mod spatial_support;
#[path = "support/mod.rs"]
mod support;

fn new_ir(
    program: ValidatedSpatialProgramV2,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
    capacity: fenestra_ui_runtime::prototype::RuntimeCapacity,
) -> Result<UiRuntime, RuntimeInitializationError> {
    UiRuntime::new_spatial_ir(program, viewport, limits, capacity)
}

fn new_ir_with_engine(
    program: ValidatedSpatialProgramV2,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
    capacity: fenestra_ui_runtime::prototype::RuntimeCapacity,
    layout_engine: Box<dyn LayoutEngineV1>,
) -> Result<UiRuntime, RuntimeInitializationError> {
    UiRuntime::new_spatial_ir_with_layout_engine(program, viewport, limits, capacity, layout_engine)
}

#[path = "runtime_spatial_ir/generation.rs"]
mod generation;
#[path = "runtime_spatial_ir/limits_and_failures.rs"]
mod limits_and_failures;
#[path = "runtime_spatial_ir/manual_compatibility.rs"]
mod manual_compatibility;
#[path = "runtime_spatial_ir/mapper_branches.rs"]
mod mapper_branches;
#[path = "runtime_spatial_ir/transactions.rs"]
mod transactions;

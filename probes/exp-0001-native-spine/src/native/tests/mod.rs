mod driver;
mod raster;
mod shell;
mod surface;
mod trace;
mod trace_applicability;
mod trace_capacity;
mod trace_matrix;

use fenestra_ui_runtime::prototype::{
    HeadlessSurface, RuntimeGeneration, SchedulerState, UiRuntime,
};
use fenestra_ui_testkit::prototype::HeadlessFixtureV1;

use super::trace::{NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1, NativeTraceStepV1};

pub(super) fn generation_zero() -> RuntimeGeneration {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should remain valid");
    UiRuntime::new_headless(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("registered runtime should initialize")
    .committed()
    .generation()
}

pub(super) fn generation_one() -> RuntimeGeneration {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should remain valid");
    let mut runtime = UiRuntime::new_headless(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("registered runtime should initialize");
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(HeadlessSurface::new(121, 90))
        .expect("different surface should stage");
    runtime
        .commit(transaction)
        .expect("different surface should publish");
    runtime.committed().generation()
}

pub(super) fn trace_step(
    stage: NativeTraceStageV1,
    observation: NativeObservationV1,
    outcome: NativeOutcomeV1,
) -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(stage, observation, outcome);
    step.scheduler_state = Some(SchedulerState::Running);
    step.current_generation = Some(generation_zero());
    step
}

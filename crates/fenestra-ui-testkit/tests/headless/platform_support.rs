#![allow(dead_code)]

use fenestra_ui_runtime::prototype::{
    QueueCapacity, SchedulerCapacity, SchedulerInput, UiRuntime, UiScheduler,
};
use fenestra_ui_testkit::prototype::HeadlessFixtureV1;

use crate::fixture_support;

pub fn scheduler(fixture: &HeadlessFixtureV1) -> UiScheduler {
    UiScheduler::new(fixture_support::runtime(fixture), scheduler_capacity())
        .expect("registered headless scheduler should initialize")
}

pub fn ordinary_scheduler(fixture: &HeadlessFixtureV1) -> UiScheduler {
    let runtime = UiRuntime::new(
        fixture.style().construction().clone(),
        fixture.runtime_capacity(),
    )
    .expect("ordinary runtime should initialize from the fixture construction");
    UiScheduler::new(runtime, scheduler_capacity())
        .expect("ordinary scheduler should accept the registered bounds")
}

pub const fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

pub const fn scheduler_input_discriminant(input: SchedulerInput) -> u8 {
    match input {
        SchedulerInput::FrameReady => 0,
        SchedulerInput::AcceptFrame(_) => 1,
        SchedulerInput::RejectFrame(_) => 2,
        SchedulerInput::Complete(_) => 3,
        SchedulerInput::RendererLost(_) => 4,
        SchedulerInput::RequestShutdown => 5,
    }
}

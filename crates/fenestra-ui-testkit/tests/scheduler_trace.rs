#[path = "scheduler_trace/capacity.rs"]
mod capacity;
#[path = "scheduler_trace/projection.rs"]
mod projection;
#[path = "scheduler_renderer/support.rs"]
mod scheduler_support;
#[path = "scheduler_trace/stress.rs"]
mod stress;
#[path = "scheduler_trace/stress_support.rs"]
mod stress_support;

use fenestra_ui_runtime::prototype::{RendererEpoch, SchedulerTick, UiScheduler};
use fenestra_ui_testkit::prototype::{
    FakeClockDomainV1, FakeClockV1, FakeRendererCapacityV1, FakeRendererV1,
    SchedulerTraceCapacityV1, SchedulerTraceStepV1, SchedulerTraceV1,
};

fn scheduler() -> UiScheduler {
    scheduler_support::scheduler()
}

fn renderer() -> FakeRendererV1 {
    FakeRendererV1::new(
        RendererEpoch::new(0),
        FakeRendererCapacityV1::new(2, 192, 100),
    )
}

fn clock(domain: u32, tick: u64) -> FakeClockV1 {
    FakeClockV1::new(FakeClockDomainV1::new(domain), SchedulerTick::new(tick))
}

fn trace(domain: u32, max_events: usize, max_bytes: usize) -> SchedulerTraceV1 {
    SchedulerTraceV1::new(
        FakeClockDomainV1::new(domain),
        SchedulerTraceCapacityV1::new(max_events, max_bytes),
    )
}

fn record(
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    step: SchedulerTraceStepV1,
    scheduler: &UiScheduler,
    renderer: &FakeRendererV1,
) {
    trace
        .record(clock, step, scheduler, renderer)
        .expect("bounded scheduler event should record");
}

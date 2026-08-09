#[path = "headless/trace_expected.rs"]
mod expected;

use fenestra_ui_runtime::prototype::{HeadlessSurface, SchedulerTick};
use fenestra_ui_testkit::prototype::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessPointerTargetV1,
    HeadlessResultV1, HeadlessRunErrorV1, HeadlessRunV1, HeadlessTraceEventV1,
    HeadlessTraceProjectionCountsV1, HeadlessTraceQueueStatsV1, HeadlessTraceRendererStatsV1,
    HeadlessTraceStageV1, HeadlessTraceV1, NormalizedHeadlessProjectionV1, SchedulerTraceV1,
    run_headless_spine_v1,
};

#[test]
fn fixed_runner_returns_both_complete_correlated_traces_and_final_projection() {
    let runner: fn() -> Result<HeadlessRunV1, HeadlessRunErrorV1> = run_headless_spine_v1;
    let run = runner().expect("the registered headless script should pass");
    let _: &NormalizedHeadlessProjectionV1 = run.final_projection();
    let _: &HeadlessTraceV1 = run.headless_trace();
    let _: &SchedulerTraceV1 = run.scheduler_trace();

    assert_eq!(run.result(), HeadlessResultV1::Pass);
    assert_eq!(run.final_generation().get(), 9);
    assert_eq!(
        run.final_projection().surface(),
        HeadlessSurface::new(90, 70)
    );
    assert_eq!(run.final_projection().computed_styles().len(), 5);
    assert_eq!(run.final_projection().geometries().len(), 5);
    assert_eq!(run.final_projection().semantics().len(), 0);
    assert_eq!(run.final_projection().hit_regions().len(), 2);
    assert_eq!(run.final_projection().scene_rectangles().len(), 4);

    let headless = run.headless_trace();
    assert_eq!(headless.capacity().max_events(), 128);
    assert_eq!(headless.capacity().max_bytes(), 20_480);
    assert_eq!(HeadlessTraceEventV1::ACCOUNTED_BYTES, 160);
    assert_eq!(headless.len(), 55);
    assert_eq!(headless.accounted_bytes(), 8_800);
    assert_eq!(headless.events().len(), headless.len());

    let scheduler = run.scheduler_trace();
    assert_eq!(scheduler.capacity().max_events(), 256);
    assert_eq!(scheduler.capacity().max_bytes(), 24_576);
    assert_eq!(scheduler.len(), 41);
    assert_eq!(scheduler.accounted_bytes(), 3_936);
    assert_eq!(headless.domain(), scheduler.domain());

    expected::assert_headless_events(headless.events(), headless.domain());
    expected::assert_scheduler_correlation(headless.events(), scheduler.events());
}

#[test]
fn two_fresh_runs_are_identical() {
    let first = run_headless_spine_v1().expect("the first registered run should pass");
    let second = run_headless_spine_v1().expect("the second registered run should pass");

    assert_eq!(first.result(), second.result());
    assert_eq!(first.final_generation(), second.final_generation());
    assert_eq!(first.final_projection(), second.final_projection());
    assert_eq!(
        first.headless_trace().domain(),
        second.headless_trace().domain()
    );
    assert_eq!(
        first.headless_trace().capacity(),
        second.headless_trace().capacity()
    );
    assert_eq!(
        first.headless_trace().accounted_bytes(),
        second.headless_trace().accounted_bytes()
    );
    assert_eq!(
        first.headless_trace().events(),
        second.headless_trace().events()
    );
    assert_eq!(
        first.scheduler_trace().domain(),
        second.scheduler_trace().domain()
    );
    assert_eq!(
        first.scheduler_trace().capacity(),
        second.scheduler_trace().capacity()
    );
    assert_eq!(
        first.scheduler_trace().accounted_bytes(),
        second.scheduler_trace().accounted_bytes()
    );
    assert_eq!(
        first.scheduler_trace().events(),
        second.scheduler_trace().events()
    );
}

#[test]
fn trace_vocabularies_are_closed_and_events_are_copyable() {
    fn assert_copy<T: Copy>() {}
    fn stage(value: HeadlessTraceStageV1) -> u8 {
        match value {
            HeadlessTraceStageV1::Build => 0,
            HeadlessTraceStageV1::Input => 1,
            HeadlessTraceStageV1::Callback => 2,
            HeadlessTraceStageV1::Transaction => 3,
            HeadlessTraceStageV1::Projection => 4,
            HeadlessTraceStageV1::Scheduler => 5,
            HeadlessTraceStageV1::Renderer => 6,
        }
    }
    fn input(value: HeadlessInputKindV1) -> u8 {
        match value {
            HeadlessInputKindV1::None => 0,
            HeadlessInputKindV1::Pointer => 1,
            HeadlessInputKindV1::Direct => 2,
            HeadlessInputKindV1::Insert => 3,
            HeadlessInputKindV1::Move => 4,
            HeadlessInputKindV1::Update => 5,
            HeadlessInputKindV1::Remove => 6,
            HeadlessInputKindV1::Resize => 7,
            HeadlessInputKindV1::FrameReady => 8,
            HeadlessInputKindV1::Completion => 9,
            HeadlessInputKindV1::Loss => 10,
            HeadlessInputKindV1::Shutdown => 11,
        }
    }
    fn outcome(value: HeadlessOutcomeV1) -> u8 {
        match value {
            HeadlessOutcomeV1::Observed => 0,
            HeadlessOutcomeV1::Deferred => 1,
            HeadlessOutcomeV1::Published => 2,
            HeadlessOutcomeV1::NoChange => 3,
            HeadlessOutcomeV1::Matched => 4,
            HeadlessOutcomeV1::Action => 5,
            HeadlessOutcomeV1::Accepted => 6,
            HeadlessOutcomeV1::Rejected => 7,
            HeadlessOutcomeV1::Completed => 8,
            HeadlessOutcomeV1::Lost => 9,
            HeadlessOutcomeV1::Stopped => 10,
            HeadlessOutcomeV1::Failed(_) => 11,
        }
    }
    fn cause(value: HeadlessFailureCauseV1) -> u8 {
        match value {
            HeadlessFailureCauseV1::Runtime => 0,
            HeadlessFailureCauseV1::Projection => 1,
            HeadlessFailureCauseV1::Oracle => 2,
            HeadlessFailureCauseV1::Scheduler => 3,
            HeadlessFailureCauseV1::Renderer => 4,
            HeadlessFailureCauseV1::Trace => 5,
        }
    }
    fn result(value: HeadlessResultV1) -> u8 {
        match value {
            HeadlessResultV1::Pass => 0,
            HeadlessResultV1::Adapt => 1,
            HeadlessResultV1::Stop => 2,
        }
    }

    fn error_kind(error: &HeadlessRunErrorV1) -> HeadlessFailureCauseV1 {
        error.kind()
    }

    let error_api: fn(&HeadlessRunErrorV1) -> HeadlessFailureCauseV1 = error_kind;
    let _ = error_api;
    assert_copy::<HeadlessTraceEventV1>();
    assert_copy::<HeadlessTraceProjectionCountsV1>();
    assert_copy::<HeadlessTraceQueueStatsV1>();
    assert_copy::<HeadlessTraceRendererStatsV1>();
    assert_eq!(stage(HeadlessTraceStageV1::Renderer), 6);
    assert_eq!(input(HeadlessInputKindV1::Shutdown), 11);
    assert_eq!(outcome(HeadlessOutcomeV1::Stopped), 10);
    assert_eq!(
        outcome(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Trace)),
        11
    );
    assert_eq!(cause(HeadlessFailureCauseV1::Trace), 5);
    assert_eq!(result(HeadlessResultV1::Stop), 2);
    assert_eq!(SchedulerTick::new(19).get(), 19);
    assert_eq!(HeadlessPointerTargetV1::None, HeadlessPointerTargetV1::None);
}

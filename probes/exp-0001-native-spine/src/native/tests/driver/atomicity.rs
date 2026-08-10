use super::super::super::driver::NativeDriverActionV1;
use super::super::super::driver::NativeDriverV1;
use super::super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceCapacityV1,
    NativeTraceEventV1, NativeTraceStageV1,
};
use super::support::{PresenterMode, TestPresenter, physical, tick};

#[test]
fn trace_batch_failure_precedes_surface_publication() {
    let capacity = NativeTraceCapacityV1::new(2, NativeTraceEventV1::ACCOUNTED_BYTES * 2);
    let mut driver = NativeDriverV1::with_trace_capacity_for_test(
        TestPresenter::new(PresenterMode::Success),
        capacity,
    )
    .expect("bounded driver should build");
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface observation should consume the second event");
    let generation = driver.runtime_generation();
    let stats = driver.scheduler_stats();
    let accepted = driver.accepted_surface();
    let pending = driver.pending_surface();
    let trace_len = driver.trace().len();

    assert_eq!(
        driver
            .drain_scheduler(tick(1))
            .expect_err("the publication batch cannot fit"),
        NativeFailureCauseV1::Trace
    );
    assert_eq!(driver.runtime_generation(), generation);
    assert_eq!(driver.scheduler_stats(), stats);
    assert_eq!(driver.accepted_surface(), accepted);
    assert_eq!(driver.pending_surface(), pending);
    assert_eq!(driver.trace().len(), trace_len);
}

#[test]
fn idle_drain_records_one_safe_scheduler_turn() {
    let mut driver = NativeDriverV1::new(TestPresenter::new(PresenterMode::Success))
        .expect("registered driver should build");
    assert_eq!(
        driver
            .drain_scheduler(tick(0))
            .expect("an idle scheduler has no work"),
        NativeDriverActionV1::Idle
    );
    let event = driver
        .trace()
        .events()
        .last()
        .expect("idle turn should record");
    assert_eq!(event.stage(), NativeTraceStageV1::Scheduler);
    assert_eq!(event.observation(), NativeObservationV1::Surface);
    assert_eq!(event.outcome(), NativeOutcomeV1::Ignored);
    assert_eq!(event.scheduler_turn(), Some(0));
    assert_eq!(event.current_generation().get(), 0);
}

#[test]
fn superseded_surface_is_the_only_deferred_publication() {
    let mut driver = NativeDriverV1::new(TestPresenter::new(PresenterMode::Success))
        .expect("registered driver should build");
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("first observation should stage");
    driver
        .observe_surface(physical(720, 520), 2.0, tick(0))
        .expect("newer observation should replace the pending tuple");
    assert_eq!(driver.scheduler_stats().deferred().items(), 0);
    assert_eq!(
        driver
            .pending_surface()
            .expect("latest tuple")
            .generation()
            .get(),
        0
    );

    let action = driver
        .drain_scheduler(tick(1))
        .expect("only the latest tuple should publish");
    assert!(matches!(
        action,
        super::super::super::driver::NativeDriverActionV1::RequestFrame { .. }
    ));
    assert_eq!(driver.runtime_generation().get(), 1);
    assert_eq!(
        driver
            .accepted_surface()
            .expect("latest tuple should publish")
            .physical(),
        physical(720, 520)
    );
    assert_eq!(driver.scheduler_stats().deferred().items(), 0);
}

#[test]
fn publication_that_meets_prior_frame_work_rejects_the_offer_before_returning() {
    let mut driver = NativeDriverV1::new(TestPresenter::new(PresenterMode::FailPreflight))
        .expect("registered driver should build");
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver.drain_scheduler(tick(1)).expect("initial publish");
    assert_eq!(
        driver
            .redraw_requested(tick(2))
            .expect_err("preaccept failure should reject frame zero"),
        NativeFailureCauseV1::Presenter
    );
    assert_eq!(driver.scheduler_stats().visual().items(), 1);

    driver
        .observe_surface(physical(720, 520), 2.0, tick(3))
        .expect("resize should stage against the retained visual request");
    assert_eq!(
        driver
            .drain_scheduler(tick(4))
            .expect_err("combined publication and offer is terminal but recoverable"),
        NativeFailureCauseV1::Invariant
    );
    assert_eq!(driver.runtime_generation().get(), 2);
    assert_eq!(driver.pending_surface(), None);
    assert_eq!(driver.scheduler_stats().in_flight().items(), 0);
    assert_eq!(driver.scheduler_stats().controls().items(), 0);
    assert_eq!(driver.scheduler_stats().visual().items(), 1);
    assert!(!driver.redraw_armed());

    let tail = &driver.trace().events()[driver.trace().len() - 3..];
    assert_eq!(tail[0].stage(), NativeTraceStageV1::Scheduler);
    assert_eq!(tail[0].observation(), NativeObservationV1::Frame);
    assert_eq!(tail[0].outcome(), NativeOutcomeV1::Rejected);
    assert_eq!(tail[0].frame(), Some(1));
    assert_eq!(tail[1].stage(), NativeTraceStageV1::Renderer);
    assert_eq!(tail[1].outcome(), NativeOutcomeV1::Rejected);
    assert_eq!(tail[1].frame(), Some(1));
    assert_eq!(tail[2].stage(), NativeTraceStageV1::Oracle);
    assert_eq!(tail[2].outcome(), NativeOutcomeV1::Matched);
}

use super::super::super::driver::NativeDriverV1;
use super::super::super::trace::{NativeFailureCauseV1, NativeTraceCapacityV1, NativeTraceEventV1};
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

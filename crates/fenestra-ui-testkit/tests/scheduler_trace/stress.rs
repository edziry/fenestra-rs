use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    ControlAdmission, RendererEpoch, RuntimeCapacity, SchedulerAction, SchedulerCapacity,
    SchedulerErrorKind, SchedulerInput, SchedulerInputResult, SchedulerState, SchedulerTick,
    UiRuntime, UiScheduler,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, FakeCallbackMutationV1, FakeCallbackScriptV1, FakeClockV1,
    FakeControlDeliveryV1, FakeFrameReadyDeliveryV1, FakePlatformV1, FakeRendererErrorKindV1,
    FakeRendererModeV1, FakeRendererOfferOutcomeV1, FakeRendererV1, RuntimeOracleFixtureV1,
    SchedulerTraceCallbackOutcomeV1, SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1,
    SchedulerTraceStepV1, SchedulerTraceV1,
};

use super::stress_support::*;

#[test]
fn integrated_scheduler_script_is_bounded_and_deterministic() {
    let first = run_script();
    let second = run_script();

    assert_eq!(first, second);
    assert_events_are_bounded(&first);
    assert_script_coverage(&first);
}

fn run_script() -> Vec<SchedulerTraceEventV1> {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let runtime = UiRuntime::new(
        fixture.construction().clone(),
        RuntimeCapacity::new(4, 64, 256, 128, 1_024, 3),
    )
    .expect("runtime should initialize");
    let mut scheduler = UiScheduler::new(
        runtime,
        SchedulerCapacity::new(DEFERRED, CONTROLS, VISUAL, IN_FLIGHT),
    )
    .expect("registered scheduler capacity should validate");
    let mut platform = FakePlatformV1::new();
    let mut renderer = FakeRendererV1::new(RendererEpoch::new(0), RENDERER);
    let mut clock = FakeClockV1::new(DOMAIN, SchedulerTick::new(0));
    let mut trace = SchedulerTraceV1::new(DOMAIN, TRACE);
    let root = scheduler.committed().root();

    let callback = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(
                FakeCallbackDepthV1::Grandchild,
                Some(FakeCallbackMutationV1::new(
                    root,
                    WIDTH,
                    PropertyValue::ScalarI32(130),
                )),
                false,
            ),
            clock.now(),
        )
        .expect("nested callback should defer one mutation");
    assert_eq!(callback.deepest_depth(), 3);
    assert!(callback.shares_entry_snapshot());
    record(
        &mut trace,
        &clock,
        SchedulerTraceStepV1::Callback {
            depth: FakeCallbackDepthV1::Grandchild,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(callback.finish()),
        },
        &scheduler,
        &renderer,
    );

    advance(&mut clock);
    let first = requested_offer(&mut scheduler, &mut platform, &mut trace, &clock, &renderer);
    let first = accept_late(&mut scheduler, &mut renderer, &mut trace, &clock, first, 1);

    advance(&mut clock);
    commit_width(&mut scheduler, &mut trace, &clock, &renderer, 140);
    let second = requested_offer(&mut scheduler, &mut platform, &mut trace, &clock, &renderer);
    let second = accept_late(&mut scheduler, &mut renderer, &mut trace, &clock, second, 1);

    advance(&mut clock);
    commit_width(&mut scheduler, &mut trace, &clock, &renderer, 150);
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        frame_ready(
            &mut platform,
            &mut scheduler,
            &mut trace,
            &clock,
            &renderer,
            false,
        ),
        FakeFrameReadyDeliveryV1::Accepted
    );
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );

    advance_by(&mut clock, 6);
    for width in 160..=190 {
        commit_width(&mut scheduler, &mut trace, &clock, &renderer, width);
        assert_eq!(
            next_action(&mut scheduler, &mut trace, &clock, &renderer),
            None
        );
    }

    complete(&mut scheduler, &mut renderer, &mut trace, &clock, first);
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );
    let latest = take_offer(next_action(&mut scheduler, &mut trace, &clock, &renderer));
    assert_eq!(latest.generation().get(), 34);
    assert_eq!(latest.earliest_tick(), SchedulerTick::new(3));
    assert_eq!(latest.latest_tick(), SchedulerTick::new(9));
    assert_eq!(
        latest.snapshot().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(190))
    );

    let rejected_frame = latest.id();
    let scheduler_before_rejection = scheduler.stats();
    let renderer_before_rejection = renderer.stats();
    let error = renderer
        .offer(
            &mut scheduler,
            latest,
            &[resource(2), resource(3)],
            FakeRendererModeV1::Late,
            clock.now(),
        )
        .expect_err("three distinct resources must exceed the registered renderer bound");
    assert_eq!(error.kind(), FakeRendererErrorKindV1::CapacityExceeded);
    assert_eq!(renderer.stats(), renderer_before_rejection);
    assert_eq!(scheduler.stats(), scheduler_before_rejection);
    record_input(
        &mut trace,
        &clock,
        SchedulerInput::RejectFrame(rejected_frame),
        SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameRejected(rejected_frame)),
        &scheduler,
        &renderer,
    );

    let retry = take_offer(next_action(&mut scheduler, &mut trace, &clock, &renderer));
    assert_ne!(retry.id(), rejected_frame);
    assert_eq!(retry.generation().get(), 34);
    assert_eq!(retry.earliest_tick(), SchedulerTick::new(3));
    assert_eq!(retry.latest_tick(), SchedulerTick::new(9));
    assert_eq!(
        retry.snapshot().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(190))
    );
    let third = accept_late(&mut scheduler, &mut renderer, &mut trace, &clock, retry, 2);

    commit_width(&mut scheduler, &mut trace, &clock, &renderer, 200);
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        Some(SchedulerAction::RequestFrame)
    );
    complete(&mut scheduler, &mut renderer, &mut trace, &clock, second);
    assert_eq!(
        frame_ready(
            &mut platform,
            &mut scheduler,
            &mut trace,
            &clock,
            &renderer,
            false,
        ),
        FakeFrameReadyDeliveryV1::Retained(SchedulerErrorKind::ControlPending)
    );
    assert!(platform.has_pending_frame_ready());
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );
    assert_eq!(
        frame_ready(
            &mut platform,
            &mut scheduler,
            &mut trace,
            &clock,
            &renderer,
            true,
        ),
        FakeFrameReadyDeliveryV1::Accepted
    );
    assert!(!platform.has_pending_frame_ready());
    let lost_offer = take_offer(next_action(&mut scheduler, &mut trace, &clock, &renderer));
    let FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Accepted(loss)) = renderer
        .offer(
            &mut scheduler,
            lost_offer,
            &[],
            FakeRendererModeV1::Loss,
            clock.now(),
        )
        .expect("loss should enter the ordered control lane")
    else {
        panic!("loss should be accepted");
    };
    record_input(
        &mut trace,
        &clock,
        SchedulerInput::RendererLost(RendererEpoch::new(0)),
        SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(loss)),
        &scheduler,
        &renderer,
    );

    let shutdown = scheduler
        .process_input(SchedulerInput::RequestShutdown, clock.now())
        .expect("shutdown reserve should remain available");
    let SchedulerInputResult::Control(ControlAdmission::Accepted(shutdown_sequence)) = shutdown
    else {
        panic!("first shutdown should receive one control sequence");
    };
    record_input(
        &mut trace,
        &clock,
        SchedulerInput::RequestShutdown,
        SchedulerTraceInputOutcomeV1::Accepted(shutdown),
        &scheduler,
        &renderer,
    );

    let duplicate = scheduler
        .process_input(SchedulerInput::RequestShutdown, clock.now())
        .expect("shutdown should remain idempotent");
    let SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(duplicate_sequence)) =
        duplicate
    else {
        panic!("duplicate shutdown should reuse its control sequence");
    };
    assert_eq!(duplicate_sequence, shutdown_sequence);
    record_input(
        &mut trace,
        &clock,
        SchedulerInput::RequestShutdown,
        SchedulerTraceInputOutcomeV1::Accepted(duplicate),
        &scheduler,
        &renderer,
    );

    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(renderer.stats().items(), 1);
    assert!(matches!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        Some(SchedulerAction::StopRenderer(_))
    ));
    assert_eq!(scheduler.state(), SchedulerState::Draining);

    advance_by(&mut clock, 8);
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );
    let inclusive = trace
        .events()
        .last()
        .copied()
        .expect("age event should exist");
    assert_eq!(inclusive.in_flight().oldest_residence_ticks(), Some(8));
    assert_eq!(inclusive.renderer().oldest_residence_ticks(), Some(8));

    complete(&mut scheduler, &mut renderer, &mut trace, &clock, third);
    assert_eq!(
        next_action(&mut scheduler, &mut trace, &clock, &renderer),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    assert_eq!(renderer.stats().items(), 0);

    assert_eq!(trace.len(), EXPECTED_EVENTS);
    assert_eq!(trace.accounted_bytes(), EXPECTED_TRACE_BYTES);
    assert!(EXPECTED_EVENTS <= TRACE.max_events());
    assert!(EXPECTED_TRACE_BYTES <= TRACE.max_bytes());
    trace.events().to_vec()
}

#[path = "scheduler_renderer/support.rs"]
mod support;

use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, QueueCapacity, RendererEpoch, SchedulerAction,
    SchedulerErrorKind, SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerState,
    SchedulerTick,
};
use fenestra_ui_testkit::prototype::{
    FakeControlDeliveryV1, FakeRendererCapacityV1, FakeRendererErrorKindV1, FakeRendererModeV1,
    FakeRendererOfferOutcomeV1, FakeRendererV1, SyntheticResourceIdV1, SyntheticResourceUseV1,
};

use support::{offer_width, process_control, scheduler_with_controls};

fn resource(id: u64) -> SyntheticResourceUseV1 {
    SyntheticResourceUseV1::new(SyntheticResourceIdV1::new(id), 64)
}

fn renderer() -> FakeRendererV1 {
    FakeRendererV1::new(
        RendererEpoch::new(0),
        FakeRendererCapacityV1::new(2, 192, 100),
    )
}

#[test]
fn immediate_completion_retains_one_control_until_shutdown_opens_its_reserved_slot() {
    let mut scheduler = scheduler_with_controls(QueueCapacity::new(1, 32, 100));
    let mut renderer = renderer();
    let work = offer_width(&mut scheduler, 130, 1);

    let FakeRendererOfferOutcomeV1::Immediate {
        submission,
        completion:
            FakeControlDeliveryV1::Retained(SchedulerErrorKind::CapacityExceeded(
                SchedulerLane::Controls,
            )),
    } = renderer
        .offer(
            &mut scheduler,
            work,
            &[resource(1)],
            FakeRendererModeV1::Immediate,
            SchedulerTick::new(1),
        )
        .expect("accepted immediate work should retain rejected completion")
    else {
        panic!("immediate mode should expose its retained completion");
    };
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(renderer.stats().items(), 1);
    assert!(renderer.stats().has_pending_control());
    assert_eq!(renderer.stats().completed(), None);

    assert_eq!(
        renderer
            .retry_control(&mut scheduler, SchedulerTick::new(1))
            .expect("full reserve should retain the same completion"),
        FakeControlDeliveryV1::Retained(SchedulerErrorKind::CapacityExceeded(
            SchedulerLane::Controls,
        ))
    );
    assert_eq!(renderer.stats().items(), 1);

    let SchedulerInputResult::Control(ControlAdmission::Accepted(shutdown)) = scheduler
        .process_input(SchedulerInput::RequestShutdown, SchedulerTick::new(2))
        .expect("reserved shutdown should be admitted")
    else {
        panic!("shutdown should receive one control identity");
    };
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("shutdown should be delivered"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert_eq!(scheduler.state(), SchedulerState::Draining);

    assert!(matches!(
        renderer
            .retry_control(&mut scheduler, SchedulerTick::new(2))
            .expect("completion should use the post-shutdown control slot"),
        FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
    ));
    assert_eq!(renderer.stats().items(), 0);
    assert!(!renderer.stats().has_pending_control());
    assert_eq!(
        renderer.stats().completed(),
        Some(CompletionWatermark::from_submission(submission))
    );
    process_control(&mut scheduler, 2);
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
}

#[test]
fn loss_backpressure_retains_one_control_and_retry_cancels_the_original_offer() {
    let mut scheduler = scheduler_with_controls(QueueCapacity::new(2, 64, 100));
    let mut renderer = renderer();
    let first = offer_width(&mut scheduler, 130, 1);
    let FakeRendererOfferOutcomeV1::Accepted(first_submission) = renderer
        .offer(
            &mut scheduler,
            first,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(1),
        )
        .expect("first offer should be accepted")
    else {
        panic!("first offer should produce a submission");
    };
    let outstanding = offer_width(&mut scheduler, 140, 2);
    assert!(matches!(
        renderer
            .complete(
                &mut scheduler,
                CompletionWatermark::from_submission(first_submission),
                SchedulerTick::new(2),
            )
            .expect("first completion should fill ordinary control allowance"),
        FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
    ));

    assert_eq!(
        renderer
            .offer(
                &mut scheduler,
                outstanding,
                &[resource(2)],
                FakeRendererModeV1::Loss,
                SchedulerTick::new(2),
            )
            .expect("loss should be retained under typed control pressure"),
        FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Retained(
            SchedulerErrorKind::CapacityExceeded(SchedulerLane::Controls),
        ))
    );
    assert!(renderer.stats().has_pending_control());
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 1);

    assert_eq!(
        renderer
            .retry_control(&mut scheduler, SchedulerTick::new(2))
            .expect("retry while full should retain the same loss"),
        FakeControlDeliveryV1::Retained(SchedulerErrorKind::CapacityExceeded(
            SchedulerLane::Controls,
        ))
    );
    process_control(&mut scheduler, 2);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 1);

    assert!(matches!(
        renderer
            .retry_control(&mut scheduler, SchedulerTick::new(3))
            .expect("loss retry should enter the opened ordinary slot"),
        FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
    ));
    assert!(!renderer.stats().has_pending_control());
    process_control(&mut scheduler, 3);
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    assert!(
        scheduler
            .next_action(SchedulerTick::new(3))
            .expect("loss should leave no phantom retry offer")
            .is_none()
    );
}

#[test]
fn one_late_completion_is_retained_without_overwriting_it_with_a_later_control() {
    let mut scheduler = scheduler_with_controls(QueueCapacity::new(1, 32, 100));
    let mut renderer = renderer();
    let first = offer_width(&mut scheduler, 130, 1);
    let FakeRendererOfferOutcomeV1::Accepted(first_submission) = renderer
        .offer(
            &mut scheduler,
            first,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(1),
        )
        .expect("first offer should be accepted")
    else {
        panic!("first offer should produce a submission");
    };
    let second = offer_width(&mut scheduler, 140, 2);
    let FakeRendererOfferOutcomeV1::Accepted(second_submission) = renderer
        .offer(
            &mut scheduler,
            second,
            &[resource(2)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(2),
        )
        .expect("second offer should be accepted")
    else {
        panic!("second offer should produce a submission");
    };
    let first_watermark = CompletionWatermark::from_submission(first_submission);
    let second_watermark = CompletionWatermark::from_submission(second_submission);

    assert_eq!(
        renderer
            .complete(&mut scheduler, first_watermark, SchedulerTick::new(2))
            .expect("completion should be retained behind the shutdown reserve"),
        FakeControlDeliveryV1::Retained(SchedulerErrorKind::CapacityExceeded(
            SchedulerLane::Controls,
        ))
    );
    assert_eq!(renderer.stats().items(), 2);
    assert_eq!(renderer.stats().completed(), None);
    assert!(renderer.stats().has_pending_control());

    let error = renderer
        .complete(&mut scheduler, second_watermark, SchedulerTick::new(2))
        .expect_err("a second control must not overwrite the retained completion");
    assert_eq!(
        error.kind(),
        FakeRendererErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
    assert_eq!(renderer.stats().items(), 2);
    assert_eq!(renderer.stats().completed(), None);

    let SchedulerInputResult::Control(ControlAdmission::Accepted(shutdown)) = scheduler
        .process_input(SchedulerInput::RequestShutdown, SchedulerTick::new(3))
        .expect("reserved shutdown should be admitted")
    else {
        panic!("shutdown should receive one control identity");
    };
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(3))
            .expect("shutdown should be delivered"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert!(matches!(
        renderer
            .retry_control(&mut scheduler, SchedulerTick::new(3))
            .expect("the first retained completion should be retried"),
        FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
    ));
    assert_eq!(renderer.stats().items(), 1);
    assert_eq!(renderer.stats().completed(), Some(first_watermark));
    assert!(!renderer.stats().has_pending_control());
}

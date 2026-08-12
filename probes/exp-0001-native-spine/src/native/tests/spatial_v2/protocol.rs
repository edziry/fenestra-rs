use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, SchedulerAction, SchedulerInput, SchedulerInputResult,
    SchedulerState, SchedulerTick,
};

use super::super::super::spatial_v2::{
    SpatialPresentErrorKindV2, SpatialPresentationOutcomeV2, present_spatial_offer_v2,
};
use super::fixture::{LOGICAL_VIEWPORT, offer_at, spatial_scheduler, take_offer};
use super::support::{PortPlan, ProtocolPhase, RecordingPort, surface};

#[test]
fn successful_protocol_orders_all_phases_before_same_turn_completion() {
    let mut scheduler = spatial_scheduler();
    let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let mut presenter = RecordingPort::new(PortPlan::Success);

    let outcome = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(4, 1, LOGICAL_VIEWPORT),
        &mut presenter,
        SchedulerTick::new(12),
    )
    .expect("successful port should present and complete");
    let SpatialPresentationOutcomeV2::Completed(receipt) = outcome else {
        panic!("nonzero surface should not suspend");
    };

    assert_eq!(
        presenter.phases(),
        [
            ProtocolPhase::Raster,
            ProtocolPhase::Stage,
            ProtocolPhase::Resize,
            ProtocolPhase::Acquire,
            ProtocolPhase::Copy,
            ProtocolPhase::Notify,
            ProtocolPhase::Accept,
            ProtocolPhase::Present,
        ]
    );
    assert_eq!(presenter.calls(), 1);
    assert_eq!(presenter.accept_calls(), 1);
    assert_eq!(receipt.generation(), work.generation());
    assert_eq!(
        receipt.digest(),
        presenter.last_successful_digest().unwrap()
    );
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    assert!(
        scheduler
            .next_action(SchedulerTick::new(12))
            .expect("completion action should be readable")
            .is_none()
    );
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Running);
}

#[test]
fn physical_zero_invokes_no_presenter_and_rejects_the_unaccepted_offer() {
    let mut scheduler = spatial_scheduler();
    let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let mut presenter = RecordingPort::new(PortPlan::Success);

    let outcome = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(0, 1, LOGICAL_VIEWPORT),
        &mut presenter,
        SchedulerTick::new(12),
    )
    .expect("physical zero should suspend cleanly");
    assert_eq!(outcome, SpatialPresentationOutcomeV2::Suspended);
    assert_eq!(presenter.calls(), 0);
    assert_eq!(presenter.accept_calls(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 1);
    let retry = take_offer(&mut scheduler, 12);
    assert_ne!(retry.id(), work.id());
    assert_eq!(retry.generation(), work.generation());
}

#[test]
fn postaccept_present_failure_reports_renderer_loss_without_rejection() {
    let mut scheduler = spatial_scheduler();
    let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let mut presenter = RecordingPort::new(PortPlan::FailPresent);

    let error = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(4, 1, LOGICAL_VIEWPORT),
        &mut presenter,
        SchedulerTick::new(12),
    )
    .expect_err("present failure should be postaccept");
    assert_eq!(error.kind(), SpatialPresentErrorKindV2::Presenter);
    assert_eq!(presenter.accept_calls(), 1);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 1);

    assert!(
        scheduler
            .next_action(SchedulerTick::new(12))
            .expect("loss control should process")
            .is_none()
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    let submission = error
        .accepted_submission()
        .expect("accepted failure should retain a submission identity");
    let result = scheduler
        .process_input(
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            SchedulerTick::new(13),
        )
        .expect("ordered retirement should remain accepted");
    assert!(matches!(
        result,
        SchedulerInputResult::Control(ControlAdmission::Accepted(_))
    ));
    assert!(
        scheduler
            .next_action(SchedulerTick::new(13))
            .expect("completion should retire")
            .is_none()
    );
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert!(
        !matches!(
            scheduler
                .next_action(SchedulerTick::new(13))
                .expect("postaccept failure should not retry"),
            Some(SchedulerAction::OfferFrame(_))
        ),
        "postaccept failure must not perform RejectFrame"
    );
}

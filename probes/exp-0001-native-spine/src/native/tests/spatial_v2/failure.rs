use fenestra_ui_runtime::prototype::{SchedulerTick, UiScheduler};

use super::super::super::spatial_v2::{
    SpatialPresentErrorKindV2, SpatialPresentationLimitKindV2, present_spatial_offer_v2,
};
use super::fixture::{LOGICAL_VIEWPORT, offer_at, spatial_scheduler, take_offer};
use super::support::{PortPlan, ProtocolPhase, RecordingPort, surface};

#[test]
fn every_preaccept_fault_rejects_once_and_never_accepts() {
    let cases = [
        (
            ProtocolPhase::Raster,
            SpatialPresentErrorKindV2::ReferenceRaster,
        ),
        (
            ProtocolPhase::Stage,
            SpatialPresentErrorKindV2::LimitExceeded(
                SpatialPresentationLimitKindV2::PhysicalPixels,
            ),
        ),
        (ProtocolPhase::Resize, SpatialPresentErrorKindV2::Presenter),
        (ProtocolPhase::Acquire, SpatialPresentErrorKindV2::Presenter),
        (ProtocolPhase::Copy, SpatialPresentErrorKindV2::Presenter),
        (ProtocolPhase::Notify, SpatialPresentErrorKindV2::PrePresent),
    ];

    for (phase, expected) in cases {
        let mut scheduler = spatial_scheduler();
        let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
        let mut presenter = RecordingPort::new(PortPlan::FailBefore(phase, expected));
        let error = present_spatial_offer_v2(
            &mut scheduler,
            &work,
            surface(4, 1, LOGICAL_VIEWPORT),
            &mut presenter,
            SchedulerTick::new(12),
        )
        .expect_err("preaccept fault should reject the outstanding offer");

        assert_eq!(error.kind(), expected);
        assert!(error.accepted_submission().is_none());
        assert_eq!(presenter.accept_calls(), 0);
        assert_eq!(scheduler.stats().in_flight().items(), 0);
        assert_eq!(scheduler.stats().visual().items(), 1);
        let retry = take_offer(&mut scheduler, 12);
        assert_ne!(retry.id(), work.id());
        assert_eq!(retry.generation(), work.generation());
    }
}

#[test]
fn preaccept_failure_preserves_the_last_successful_digest() {
    let (mut scheduler, mut presenter, digest) = successful_baseline();
    let next_viewport = fenestra_ui_spatial::prototype::SpatialViewportV2::new(5, 1);
    let work = offer_at(&mut scheduler, next_viewport, 20);
    presenter.set_plan_for_test(PortPlan::FailBefore(
        ProtocolPhase::Copy,
        SpatialPresentErrorKindV2::Presenter,
    ));

    let error = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(5, 1, next_viewport),
        &mut presenter,
        SchedulerTick::new(22),
    )
    .expect_err("copy fault should reject before acceptance");

    assert_eq!(error.kind(), SpatialPresentErrorKindV2::Presenter);
    assert_eq!(presenter.last_successful_digest(), Some(digest));
    assert_eq!(presenter.accept_calls(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 1);
}

#[test]
fn postaccept_failure_preserves_prior_digest_and_retains_accepted_work() {
    let (mut scheduler, mut presenter, digest) = successful_baseline();
    let next_viewport = fenestra_ui_spatial::prototype::SpatialViewportV2::new(5, 1);
    let work = offer_at(&mut scheduler, next_viewport, 20);
    presenter.set_plan_for_test(PortPlan::FailPresent);

    let error = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(5, 1, next_viewport),
        &mut presenter,
        SchedulerTick::new(22),
    )
    .expect_err("present fault should report renderer loss");

    assert_eq!(error.kind(), SpatialPresentErrorKindV2::Presenter);
    assert!(error.accepted_submission().is_some());
    assert_eq!(presenter.last_successful_digest(), Some(digest));
    assert_eq!(presenter.accept_calls(), 2);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 1);
}

fn successful_baseline() -> (UiScheduler, RecordingPort, u64) {
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
    .expect("baseline should present");
    let digest = outcome
        .completed()
        .expect("baseline should complete")
        .digest();
    assert!(
        scheduler
            .next_action(SchedulerTick::new(12))
            .expect("completion should retire")
            .is_none()
    );
    (scheduler, presenter, digest)
}

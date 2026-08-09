#[path = "scheduler_renderer/support.rs"]
mod support;

use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, RendererEpoch, SchedulerErrorKind, SchedulerState,
    SchedulerTick,
};
use fenestra_ui_testkit::prototype::{
    FakeControlDeliveryV1, FakeRendererCapacityV1, FakeRendererErrorKindV1, FakeRendererModeV1,
    FakeRendererOfferOutcomeV1, FakeRendererV1, SyntheticResourceIdV1, SyntheticResourceUseV1,
};

use support::{next_offer, offer_width, process_control, scheduler};

const RESOURCE_BYTES: usize = 64;

fn resource(id: u64) -> SyntheticResourceUseV1 {
    SyntheticResourceUseV1::new(SyntheticResourceIdV1::new(id), RESOURCE_BYTES)
}

fn renderer(capacity: FakeRendererCapacityV1) -> FakeRendererV1 {
    FakeRendererV1::new(RendererEpoch::new(0), capacity)
}

fn standard_renderer() -> FakeRendererV1 {
    renderer(FakeRendererCapacityV1::new(2, 192, 8))
}

#[test]
fn scripted_failure_restores_the_offer_and_retry_uses_a_fresh_identity() {
    let mut scheduler = scheduler();
    let mut renderer = standard_renderer();
    let first = offer_width(&mut scheduler, 130, 1);

    assert_eq!(
        renderer
            .offer(
                &mut scheduler,
                first.clone(),
                &[resource(1)],
                FakeRendererModeV1::Fail,
                SchedulerTick::new(1),
            )
            .expect("scripted failure should reject the offer"),
        FakeRendererOfferOutcomeV1::Rejected(first.id())
    );
    assert_eq!(renderer.stats().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    let retry = next_offer(&mut scheduler, 1);
    assert_ne!(retry.id(), first.id());
    assert_eq!(retry.generation(), first.generation());
    assert!(retry.snapshot().shares_state_with(first.snapshot()));
    assert_eq!(retry.invalidation(), first.invalidation());
    assert_eq!(retry.earliest_tick(), first.earliest_tick());
    assert_eq!(retry.latest_tick(), first.latest_tick());

    let FakeRendererOfferOutcomeV1::Accepted(_) = renderer
        .offer(
            &mut scheduler,
            retry,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(1),
        )
        .expect("retry should be accepted")
    else {
        panic!("late retry should produce a submission");
    };
    assert_eq!(renderer.stats().items(), 1);
    assert_eq!(renderer.stats().accounted_bytes(), 96);
}

#[test]
fn retirement_item_and_byte_preflight_is_atomic_with_an_existing_record() {
    for capacity in [
        FakeRendererCapacityV1::new(1, 192, 8),
        FakeRendererCapacityV1::new(2, 191, 8),
    ] {
        let mut scheduler = scheduler();
        let mut renderer = renderer(capacity);
        let first = offer_width(&mut scheduler, 130, 1);
        let FakeRendererOfferOutcomeV1::Accepted(first_submission) = renderer
            .offer(
                &mut scheduler,
                first,
                &[resource(1)],
                FakeRendererModeV1::Late,
                SchedulerTick::new(1),
            )
            .expect("first resource should fit")
        else {
            panic!("first offer should be accepted");
        };
        let before = renderer.stats();

        let second = offer_width(&mut scheduler, 140, 2);
        let error = renderer
            .offer(
                &mut scheduler,
                second,
                &[resource(1), resource(2)],
                FakeRendererModeV1::Late,
                SchedulerTick::new(2),
            )
            .expect_err("complete retirement projection must fit before acceptance");
        assert_eq!(error.kind(), FakeRendererErrorKindV1::CapacityExceeded);
        assert_eq!(renderer.stats(), before);
        assert_eq!(scheduler.stats().in_flight().items(), 1);
        assert_eq!(scheduler.stats().visual().items(), 1);

        assert!(matches!(next_offer(&mut scheduler, 2).id().get(), 2..));
        let admission = renderer
            .complete(
                &mut scheduler,
                CompletionWatermark::from_submission(first_submission),
                SchedulerTick::new(2),
            )
            .expect("completion should remain admissible");
        assert!(matches!(
            admission,
            FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
        ));
        assert_eq!(renderer.stats().items(), 0);
        process_control(&mut scheduler, 2);
        assert_eq!(scheduler.stats().in_flight().items(), 0);
    }
}

#[test]
fn ordered_completion_releases_only_prefixes_and_invalid_inputs_are_atomic() {
    let mut scheduler = scheduler();
    let mut renderer = standard_renderer();
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
    let first_admission = renderer
        .complete(&mut scheduler, first_watermark, SchedulerTick::new(3))
        .expect("first prefix should complete");
    assert!(matches!(
        first_admission,
        FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_))
    ));
    assert_eq!(renderer.stats().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(scheduler.stats().controls().items(), 1);
    let before_invalid = renderer.stats();

    assert!(matches!(
        renderer
            .complete(&mut scheduler, first_watermark, SchedulerTick::new(3))
            .expect("equal completion should be idempotent"),
        FakeControlDeliveryV1::Accepted(ControlAdmission::AlreadyAccepted(_))
    ));
    for (watermark, expected) in [
        (
            CompletionWatermark::new(RendererEpoch::new(1), 0),
            SchedulerErrorKind::ForeignRendererEpoch,
        ),
        (
            CompletionWatermark::new(RendererEpoch::new(0), 2),
            SchedulerErrorKind::CompletionBeyondAccepted,
        ),
    ] {
        let error = renderer
            .complete(&mut scheduler, watermark, SchedulerTick::new(3))
            .expect_err("invalid completion must preserve both ledgers");
        assert_eq!(error.kind(), FakeRendererErrorKindV1::Scheduler(expected));
        assert_eq!(renderer.stats(), before_invalid);
        assert_eq!(scheduler.stats().in_flight().items(), 2);
        assert_eq!(scheduler.stats().controls().items(), 1);
    }

    process_control(&mut scheduler, 3);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    let second_watermark = CompletionWatermark::from_submission(second_submission);
    renderer
        .complete(&mut scheduler, second_watermark, SchedulerTick::new(4))
        .expect("second prefix should complete");
    assert_eq!(renderer.stats().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    process_control(&mut scheduler, 4);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    let error = renderer
        .complete(&mut scheduler, first_watermark, SchedulerTick::new(4))
        .expect_err("completion must not regress");
    assert_eq!(
        error.kind(),
        FakeRendererErrorKindV1::Scheduler(SchedulerErrorKind::CompletionRegression)
    );
    assert_eq!(renderer.stats().items(), 0);
}

#[test]
fn immediate_completion_is_control_owned_until_the_next_scheduler_turn() {
    let mut scheduler = scheduler();
    let mut renderer = standard_renderer();
    let work = offer_width(&mut scheduler, 130, 1);

    let FakeRendererOfferOutcomeV1::Immediate {
        submission: _submission,
        completion: FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(_)),
    } = renderer
        .offer(
            &mut scheduler,
            work,
            &[resource(1)],
            FakeRendererModeV1::Immediate,
            SchedulerTick::new(1),
        )
        .expect("immediate mode should accept and observe completion")
    else {
        panic!("immediate mode should report its completed submission");
    };
    assert_eq!(renderer.stats().items(), 0);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    process_control(&mut scheduler, 1);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
}

#[test]
fn loss_cancels_only_unaccepted_work_and_never_retires_resources() {
    let mut scheduler = scheduler();
    let mut renderer = standard_renderer();
    let accepted = offer_width(&mut scheduler, 130, 1);
    let FakeRendererOfferOutcomeV1::Accepted(submission) = renderer
        .offer(
            &mut scheduler,
            accepted,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(1),
        )
        .expect("first offer should be accepted")
    else {
        panic!("first offer should produce a submission");
    };
    let pending = offer_width(&mut scheduler, 140, 2);

    let FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Accepted(
        ControlAdmission::Accepted(_),
    )) = renderer
        .offer(
            &mut scheduler,
            pending,
            &[resource(2)],
            FakeRendererModeV1::Loss,
            SchedulerTick::new(2),
        )
        .expect("loss control should be admitted")
    else {
        panic!("loss mode should return its control admission");
    };
    assert_eq!(renderer.stats().items(), 1);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    process_control(&mut scheduler, 2);
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(renderer.stats().items(), 1);

    renderer
        .complete(
            &mut scheduler,
            CompletionWatermark::from_submission(submission),
            SchedulerTick::new(3),
        )
        .expect("late completion should remain admissible");
    assert_eq!(renderer.stats().items(), 0);
    process_control(&mut scheduler, 3);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
}

#[test]
fn retirement_residence_is_inclusive_terminal_and_completion_stays_admissible() {
    let mut scheduler = scheduler();
    let mut renderer = standard_renderer();
    let first = offer_width(&mut scheduler, 130, 1);
    let FakeRendererOfferOutcomeV1::Accepted(first_submission) = renderer
        .offer(
            &mut scheduler,
            first,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(2),
        )
        .expect("first offer should be accepted")
    else {
        panic!("first offer should produce a submission");
    };
    let second = offer_width(&mut scheduler, 140, 9);
    let FakeRendererOfferOutcomeV1::Accepted(second_submission) = renderer
        .offer(
            &mut scheduler,
            second,
            &[resource(1)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(10),
        )
        .expect("the exact residence deadline should remain inclusive")
    else {
        panic!("second offer should produce a submission");
    };
    assert_eq!(renderer.stats().items(), 1);
    assert_eq!(
        renderer.stats().earliest_tick(),
        Some(SchedulerTick::new(2))
    );
    assert_eq!(renderer.stats().latest_tick(), Some(SchedulerTick::new(10)));

    renderer
        .complete(
            &mut scheduler,
            CompletionWatermark::from_submission(first_submission),
            SchedulerTick::new(10),
        )
        .expect("first runtime submission should complete");
    assert_eq!(renderer.stats().items(), 1);
    process_control(&mut scheduler, 10);

    let third = offer_width(&mut scheduler, 150, 11);
    let before = renderer.stats();
    let error = renderer
        .offer(
            &mut scheduler,
            third,
            &[resource(2)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(11),
        )
        .expect_err("the first tick beyond the deadline should be terminal");
    assert_eq!(error.kind(), FakeRendererErrorKindV1::ResidenceExceeded);
    assert_eq!(renderer.stats(), before);
    assert_eq!(scheduler.stats().visual().items(), 1);

    renderer
        .complete(
            &mut scheduler,
            CompletionWatermark::from_submission(second_submission),
            SchedulerTick::new(11),
        )
        .expect("completion must remain admissible after fake pressure");
    assert_eq!(renderer.stats().items(), 0);
    process_control(&mut scheduler, 11);

    let retry = next_offer(&mut scheduler, 11);
    let error = renderer
        .offer(
            &mut scheduler,
            retry,
            &[resource(2)],
            FakeRendererModeV1::Late,
            SchedulerTick::new(11),
        )
        .expect_err("retirement pressure must remain terminal after cleanup");
    assert_eq!(error.kind(), FakeRendererErrorKindV1::ResidenceExceeded);
}

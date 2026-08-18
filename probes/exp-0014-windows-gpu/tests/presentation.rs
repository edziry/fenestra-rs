use fenestra_ui_exp_0014_windows_gpu::{
    GpuPortReceiptV1, GpuPresentErrorKindV1, GpuPresentPortV1, GpuPresentationOutcomeV1,
    GpuSurfaceExtentV1, build_registered_runtime_v1, present_gpu_offer_v1,
};
use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    FrameWork, QueueCapacity, RuntimePaintFrameV2, SchedulerAction, SchedulerCapacity,
    SchedulerInput, SchedulerInputResult, SchedulerTick, SubmissionId, UiScheduler,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

#[derive(Clone, Copy)]
enum Plan {
    Success,
    FailBeforeAccept,
    FailAfterAccept,
}

struct FakeGpuPort {
    plan: Plan,
    accept_calls: usize,
}

impl FakeGpuPort {
    const fn new(plan: Plan) -> Self {
        Self {
            plan,
            accept_calls: 0,
        }
    }
}

impl GpuPresentPortV1 for FakeGpuPort {
    fn present<A>(
        &mut self,
        _frame: RuntimePaintFrameV2<'_>,
        _surface: GpuSurfaceExtentV1,
        accept_once: A,
    ) -> Result<GpuPortReceiptV1, GpuPresentErrorKindV1>
    where
        A: FnOnce() -> Result<SubmissionId, GpuPresentErrorKindV1>,
    {
        if matches!(self.plan, Plan::FailBeforeAccept) {
            return Err(GpuPresentErrorKindV1::Acquire);
        }
        self.accept_calls += 1;
        let _submission = accept_once()?;
        if matches!(self.plan, Plan::FailAfterAccept) {
            return Err(GpuPresentErrorKindV1::Present);
        }
        Ok(GpuPortReceiptV1::new(0x0123_4567_89ab_cdef))
    }
}

#[test]
fn completed_gpu_port_correlates_offer_submission_and_completion() {
    let (mut scheduler, work) = offered_scheduler();
    let frame = work.id().get();
    let generation = work.generation().get();
    let mut port = FakeGpuPort::new(Plan::Success);

    let outcome = present_gpu_offer_v1(
        &mut scheduler,
        &work,
        GpuSurfaceExtentV1::new(192, 128),
        &mut port,
        SchedulerTick::new(12),
    )
    .expect("completed GPU port");
    let GpuPresentationOutcomeV1::Completed(receipt) = outcome else {
        panic!("nonzero surface should complete");
    };

    assert_eq!(port.accept_calls, 1);
    assert_eq!(receipt.generation(), generation);
    assert_eq!(receipt.frame(), frame);
    assert_eq!(receipt.submission(), 0);
    assert_eq!(receipt.raster_digest(), 0x0123_4567_89ab_cdef);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 1);
}

#[test]
fn zero_or_preaccept_failure_never_accepts_the_offer() {
    let (mut scheduler, work) = offered_scheduler();
    let mut port = FakeGpuPort::new(Plan::Success);
    let outcome = present_gpu_offer_v1(
        &mut scheduler,
        &work,
        GpuSurfaceExtentV1::new(0, 128),
        &mut port,
        SchedulerTick::new(12),
    )
    .expect("zero surface suspends");
    assert_eq!(outcome, GpuPresentationOutcomeV1::Suspended);
    assert_eq!(port.accept_calls, 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    let (mut scheduler, work) = offered_scheduler();
    let mut port = FakeGpuPort::new(Plan::FailBeforeAccept);
    let error = present_gpu_offer_v1(
        &mut scheduler,
        &work,
        GpuSurfaceExtentV1::new(192, 128),
        &mut port,
        SchedulerTick::new(12),
    )
    .expect_err("preaccept failure");
    assert_eq!(error.kind(), GpuPresentErrorKindV1::Acquire);
    assert_eq!(error.accepted_submission(), None);
    assert_eq!(port.accept_calls, 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
}

#[test]
fn postaccept_failure_reports_renderer_loss_and_keeps_submission_identity() {
    let (mut scheduler, work) = offered_scheduler();
    let mut port = FakeGpuPort::new(Plan::FailAfterAccept);

    let error = present_gpu_offer_v1(
        &mut scheduler,
        &work,
        GpuSurfaceExtentV1::new(192, 128),
        &mut port,
        SchedulerTick::new(12),
    )
    .expect_err("postaccept failure");

    assert_eq!(error.kind(), GpuPresentErrorKindV1::Present);
    assert_eq!(
        error.accepted_submission().map(SubmissionId::token),
        Some(0)
    );
    assert_eq!(port.accept_calls, 1);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().controls().items(), 1);
}

fn offered_scheduler() -> (UiScheduler, FrameWork) {
    let runtime =
        build_registered_runtime_v1(SpatialViewportV2::new(192, 128)).expect("registered runtime");
    let mut scheduler = UiScheduler::new(runtime, scheduler_capacity()).expect("scheduler");
    let snapshot = scheduler.committed();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(
            snapshot.root(),
            PropertyId::new(4),
            PropertyValue::Rgba8([80, 40, 24, 255]),
        )
        .expect("tone mutation");
    scheduler
        .commit(transaction, SchedulerTick::new(10))
        .expect("visual commit");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(10))
            .expect("request"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(11))
            .expect("frame ready"),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(11))
        .expect("offer")
    else {
        panic!("registered frame should be offered");
    };
    (scheduler, work)
}

fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 32),
        QueueCapacity::new(4, 128, 32),
        QueueCapacity::new(1, 40, 32),
        QueueCapacity::new(2, 80, 32),
    )
}

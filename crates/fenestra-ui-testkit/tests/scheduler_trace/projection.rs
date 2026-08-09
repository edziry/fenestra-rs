use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CallbackFinish, CompletionWatermark, ControlAdmission, SchedulerAction, SchedulerInput,
    SchedulerInputResult, SchedulerState, SchedulerTick,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, FakeControlDeliveryV1, FakeRendererModeV1, FakeRendererOfferOutcomeV1,
    SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1, SchedulerTraceCommitOutcomeV1,
    SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1, SchedulerTraceStageV1,
    SchedulerTraceStepV1, SyntheticResourceIdV1, SyntheticResourceUseV1,
};

use super::{clock, record, renderer, scheduler, trace};

const WIDTH: PropertyId = PropertyId::new(0);

#[test]
fn events_project_typed_state_without_retaining_runtime_work() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<SchedulerTraceEventV1>();

    let mut scheduler = scheduler();
    let renderer = renderer();
    let mut trace = trace(7, 8, 8 * SchedulerTraceEventV1::ACCOUNTED_BYTES);
    let entry = scheduler.committed();
    let root = entry.root();

    record(
        &mut trace,
        &clock(7, 0),
        SchedulerTraceStepV1::Callback {
            depth: FakeCallbackDepthV1::Outer,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(
                fenestra_ui_runtime::prototype::CallbackFinish::NoChanges,
            ),
        },
        &scheduler,
        &renderer,
    );
    let initial = trace.events()[0];
    assert_eq!(initial.schema_revision(), 1);
    assert_eq!(initial.sequence(), 0);
    assert_eq!(initial.clock_domain().get(), 7);
    assert_eq!(initial.tick(), SchedulerTick::new(0));
    assert_eq!(initial.stage(), SchedulerTraceStageV1::Callback);
    assert_eq!(
        initial.step(),
        SchedulerTraceStepV1::Callback {
            depth: FakeCallbackDepthV1::Outer,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(
                fenestra_ui_runtime::prototype::CallbackFinish::NoChanges,
            ),
        }
    );
    assert_eq!(initial.callback_depth(), Some(1));
    assert_eq!(initial.lifecycle(), SchedulerState::Running);
    assert_eq!(initial.generation(), entry.generation());
    assert_eq!(initial.frame(), None);
    assert_eq!(initial.control(), None);
    for lane in [
        initial.deferred(),
        initial.controls(),
        initial.visual(),
        initial.in_flight(),
    ] {
        assert_eq!(lane.items(), 0);
        assert_eq!(lane.accounted_bytes(), 0);
        assert_eq!(lane.oldest_residence_ticks(), None);
    }
    assert_eq!(initial.renderer().items(), 0);
    assert_eq!(initial.renderer().accounted_bytes(), 0);

    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .expect("property write should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(1))
        .expect("property write should commit");
    record(
        &mut trace,
        &clock(7, 1),
        SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published),
        &scheduler,
        &renderer,
    );
    let committed = trace.events()[1];
    assert_eq!(committed.sequence(), 1);
    assert_eq!(committed.tick(), SchedulerTick::new(1));
    assert_eq!(
        committed.step(),
        SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published)
    );
    assert_eq!(committed.generation().get(), 1);
    assert_eq!(committed.visual().items(), 1);
    assert_eq!(committed.visual().accounted_bytes(), 40);
    assert_eq!(committed.visual().oldest_residence_ticks(), Some(0));

    let action = scheduler
        .next_action(SchedulerTick::new(1))
        .expect("request turn should advance");
    let projected = SchedulerTraceActionV1::from_action(action.as_ref());
    record(
        &mut trace,
        &clock(7, 1),
        SchedulerTraceStepV1::Action(projected),
        &scheduler,
        &renderer,
    );
    assert_eq!(projected, SchedulerTraceActionV1::RequestFrame);
    assert_eq!(trace.events()[2].stage(), SchedulerTraceStageV1::Action);
    assert_eq!(trace.events()[2].sequence(), 2);
    assert_eq!(
        trace.events()[2].step(),
        SchedulerTraceStepV1::Action(SchedulerTraceActionV1::RequestFrame)
    );
    drop(action);

    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(2))
            .expect("frame-ready should be accepted"),
        SchedulerInputResult::FrameReady
    );
    record(
        &mut trace,
        &clock(7, 2),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::FrameReady,
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameReady),
        },
        &scheduler,
        &renderer,
    );
    assert_eq!(trace.events()[3].sequence(), 3);
    assert_eq!(
        trace.events()[3].step(),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::FrameReady,
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameReady),
        }
    );
    let offer = scheduler
        .next_action(SchedulerTick::new(2))
        .expect("offer turn should advance");
    let projected_offer = SchedulerTraceActionV1::from_action(offer.as_ref());
    record(
        &mut trace,
        &clock(7, 2),
        SchedulerTraceStepV1::Action(projected_offer),
        &scheduler,
        &renderer,
    );
    let Some(SchedulerAction::OfferFrame(work)) = offer else {
        panic!("frame-ready should produce one offer");
    };
    assert_eq!(
        projected_offer,
        SchedulerTraceActionV1::OfferFrame(work.id())
    );
    assert_eq!(trace.events()[4].sequence(), 4);
    assert_eq!(
        trace.events()[4].step(),
        SchedulerTraceStepV1::Action(SchedulerTraceActionV1::OfferFrame(work.id()))
    );
    assert_eq!(trace.events()[4].frame(), Some(work.id()));
    drop(work);
    drop(entry);
}

#[test]
fn events_capture_nonempty_deferred_control_submission_and_retirement_lanes() {
    let mut scheduler = scheduler();
    let mut renderer = renderer();
    let mut trace = trace(11, 4, 4 * SchedulerTraceEventV1::ACCOUNTED_BYTES);
    let root = scheduler.committed().root();
    let mut scope = scheduler
        .begin_callback(SchedulerTick::new(10))
        .expect("callback should begin");
    scope
        .transaction()
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .expect("callback property write should stage");
    let finish = scope.finish().expect("callback should enter deferred lane");
    record(
        &mut trace,
        &clock(11, 12),
        SchedulerTraceStepV1::Callback {
            depth: FakeCallbackDepthV1::Outer,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(finish),
        },
        &scheduler,
        &renderer,
    );
    let deferred = trace.events()[0];
    assert_eq!(
        deferred.step(),
        SchedulerTraceStepV1::Callback {
            depth: FakeCallbackDepthV1::Outer,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(CallbackFinish::Deferred {
                operation_count: 1,
                accounted_bytes: 80,
            }),
        }
    );
    assert_eq!(deferred.deferred().items(), 1);
    assert_eq!(deferred.deferred().accounted_bytes(), 80);
    assert_eq!(deferred.deferred().oldest_residence_ticks(), Some(2));

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(12))
            .expect("deferred turn should publish"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(12))
            .expect("frame-ready should be accepted"),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(12))
        .expect("offer turn should advance")
    else {
        panic!("published callback should produce one offer");
    };
    let frame = work.id();
    let resource = SyntheticResourceUseV1::new(SyntheticResourceIdV1::new(1), 64);
    let FakeRendererOfferOutcomeV1::Accepted(submission) = renderer
        .offer(
            &mut scheduler,
            work,
            &[resource],
            FakeRendererModeV1::Late,
            SchedulerTick::new(12),
        )
        .expect("late renderer should accept the offer")
    else {
        panic!("late renderer should return one submission");
    };
    record(
        &mut trace,
        &clock(11, 13),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::AcceptFrame(frame),
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameAccepted(
                submission,
            )),
        },
        &scheduler,
        &renderer,
    );
    let submitted = trace.events()[1];
    assert_eq!(submitted.frame(), Some(frame));
    assert_eq!(submitted.in_flight().items(), 1);
    assert_eq!(submitted.in_flight().accounted_bytes(), 40);
    assert_eq!(submitted.in_flight().oldest_residence_ticks(), Some(1));
    assert_eq!(submitted.renderer().items(), 1);
    assert_eq!(submitted.renderer().accounted_bytes(), 96);
    assert_eq!(submitted.renderer().oldest_residence_ticks(), Some(1));

    let watermark = CompletionWatermark::from_submission(submission);
    let FakeControlDeliveryV1::Accepted(admission) = renderer
        .complete(&mut scheduler, watermark, SchedulerTick::new(13))
        .expect("completion should enter the control lane")
    else {
        panic!("completion should not be retained by the standard fixture");
    };
    record(
        &mut trace,
        &clock(11, 13),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::Complete(watermark),
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(
                admission,
            )),
        },
        &scheduler,
        &renderer,
    );
    let controlled = trace.events()[2];
    let sequence = match admission {
        ControlAdmission::Accepted(sequence) | ControlAdmission::AlreadyAccepted(sequence) => {
            sequence
        }
    };
    assert_eq!(controlled.control(), Some(sequence));
    assert_eq!(controlled.controls().items(), 1);
    assert_eq!(controlled.controls().accounted_bytes(), 32);
    assert_eq!(controlled.controls().oldest_residence_ticks(), Some(0));
    assert_eq!(controlled.in_flight().items(), 1);
    assert_eq!(controlled.renderer().items(), 0);
    assert_eq!(controlled.renderer().accounted_bytes(), 0);
}

#[test]
fn equal_scripts_on_distinct_runtimes_produce_equal_event_vectors() {
    fn run() -> Vec<SchedulerTraceEventV1> {
        let mut scheduler = scheduler();
        let renderer = renderer();
        let mut trace = trace(9, 4, 4 * SchedulerTraceEventV1::ACCOUNTED_BYTES);
        let root = scheduler.committed().root();
        let mut transaction = scheduler.begin_transaction();
        transaction
            .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
            .expect("property write should stage");
        scheduler
            .commit(transaction, SchedulerTick::new(1))
            .expect("property write should commit");
        record(
            &mut trace,
            &clock(9, 1),
            SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published),
            &scheduler,
            &renderer,
        );
        let action = scheduler
            .next_action(SchedulerTick::new(1))
            .expect("request turn should advance");
        record(
            &mut trace,
            &clock(9, 1),
            SchedulerTraceStepV1::Action(SchedulerTraceActionV1::from_action(action.as_ref())),
            &scheduler,
            &renderer,
        );
        trace.events().to_vec()
    }

    assert_eq!(run(), run());
}

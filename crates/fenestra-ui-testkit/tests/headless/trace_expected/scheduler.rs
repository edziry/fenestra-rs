use fenestra_ui_runtime::prototype::{
    CallbackFinish, ControlAdmission, SchedulerErrorKind, SchedulerInput, SchedulerInputResult,
    SchedulerState, TransactionErrorKind,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1,
    SchedulerTraceStepV1,
};

#[derive(Clone, Copy)]
enum ExpectedAction {
    Idle,
    RequestFrame,
    Offer(u64),
    MissingNode,
    Stop(u64),
}

#[derive(Clone, Copy)]
enum ExpectedInput {
    FrameReady,
    Accept { frame: u64, token: u64 },
    Reject(u64),
    Complete { token: u64, control: u64 },
    Loss(u64),
    Shutdown { control: u64, duplicate: bool },
}

pub(super) fn assert_scheduler_steps(events: &[SchedulerTraceEventV1]) {
    for index in [2, 4, 6, 8, 19, 21, 30] {
        assert_eq!(
            events[index].step(),
            SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published)
        );
    }
    for (index, depth) in [
        (0, FakeCallbackDepthV1::Nested),
        (9, FakeCallbackDepthV1::Nested),
        (12, FakeCallbackDepthV1::Outer),
        (14, FakeCallbackDepthV1::Outer),
    ] {
        assert_eq!(
            events[index].step(),
            SchedulerTraceStepV1::Callback {
                depth,
                outcome: SchedulerTraceCallbackOutcomeV1::Finished(CallbackFinish::Deferred {
                    operation_count: 1,
                    accounted_bytes: 80,
                }),
            }
        );
    }
    let actions = [
        (1, ExpectedAction::RequestFrame),
        (3, ExpectedAction::Idle),
        (5, ExpectedAction::Idle),
        (7, ExpectedAction::Idle),
        (10, ExpectedAction::MissingNode),
        (11, ExpectedAction::Idle),
        (13, ExpectedAction::Idle),
        (15, ExpectedAction::Idle),
        (17, ExpectedAction::Offer(0)),
        (20, ExpectedAction::RequestFrame),
        (22, ExpectedAction::Idle),
        (24, ExpectedAction::Offer(1)),
        (26, ExpectedAction::Offer(2)),
        (29, ExpectedAction::Idle),
        (31, ExpectedAction::RequestFrame),
        (33, ExpectedAction::Offer(3)),
        (37, ExpectedAction::Idle),
        (38, ExpectedAction::Stop(2)),
        (40, ExpectedAction::Idle),
    ];
    for (index, action) in actions {
        assert_action(events[index], action);
    }
    let inputs = [
        (16, ExpectedInput::FrameReady),
        (18, ExpectedInput::Accept { frame: 0, token: 0 }),
        (23, ExpectedInput::FrameReady),
        (25, ExpectedInput::Reject(1)),
        (27, ExpectedInput::Accept { frame: 2, token: 1 }),
        (
            28,
            ExpectedInput::Complete {
                token: 0,
                control: 0,
            },
        ),
        (32, ExpectedInput::FrameReady),
        (34, ExpectedInput::Loss(1)),
        (
            35,
            ExpectedInput::Shutdown {
                control: 2,
                duplicate: false,
            },
        ),
        (
            36,
            ExpectedInput::Shutdown {
                control: 2,
                duplicate: true,
            },
        ),
        (
            39,
            ExpectedInput::Complete {
                token: 1,
                control: 3,
            },
        ),
    ];
    for (index, input) in inputs {
        assert_input(events[index], input);
    }
    for (index, event) in events.iter().copied().enumerate() {
        let expected = match index {
            0..=34 => SchedulerState::Running,
            35..=36 => SchedulerState::ShutdownQueued,
            37 => SchedulerState::Faulted,
            38..=39 => SchedulerState::Draining,
            40 => SchedulerState::Stopped,
            _ => unreachable!("scheduler trace has exactly 41 events"),
        };
        assert_eq!(event.lifecycle(), expected);
    }
}

fn assert_action(event: SchedulerTraceEventV1, expected: ExpectedAction) {
    match (event.step(), expected) {
        (SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Idle), ExpectedAction::Idle)
        | (
            SchedulerTraceStepV1::Action(SchedulerTraceActionV1::RequestFrame),
            ExpectedAction::RequestFrame,
        ) => {}
        (
            SchedulerTraceStepV1::Action(SchedulerTraceActionV1::OfferFrame(frame)),
            ExpectedAction::Offer(expected),
        ) => assert_eq!(frame.get(), expected),
        (
            SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Rejected(
                SchedulerErrorKind::Transaction(TransactionErrorKind::MissingNode),
            )),
            ExpectedAction::MissingNode,
        ) => {}
        (
            SchedulerTraceStepV1::Action(SchedulerTraceActionV1::StopRenderer(control)),
            ExpectedAction::Stop(expected),
        ) => assert_eq!(control.get(), expected),
        (actual, _) => panic!("unexpected scheduler action: {actual:?}"),
    }
}

fn assert_input(event: SchedulerTraceEventV1, expected: ExpectedInput) {
    let SchedulerTraceStepV1::Input { input, outcome } = event.step() else {
        panic!("expected scheduler input event")
    };
    match (input, outcome, expected) {
        (
            SchedulerInput::FrameReady,
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameReady),
            ExpectedInput::FrameReady,
        ) => {}
        (
            SchedulerInput::AcceptFrame(frame),
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameAccepted(submission)),
            ExpectedInput::Accept {
                frame: expected_frame,
                token,
            },
        ) => {
            assert_eq!(frame.get(), expected_frame);
            assert_eq!(submission.epoch().get(), 0);
            assert_eq!(submission.token(), token);
        }
        (
            SchedulerInput::RejectFrame(frame),
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameRejected(rejected)),
            ExpectedInput::Reject(expected),
        ) => {
            assert_eq!(frame.get(), expected);
            assert_eq!(rejected, frame);
        }
        (
            SchedulerInput::Complete(watermark),
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(
                ControlAdmission::Accepted(sequence),
            )),
            ExpectedInput::Complete { token, control },
        ) => {
            assert_eq!(watermark.epoch().get(), 0);
            assert_eq!(watermark.token(), token);
            assert_eq!(sequence.get(), control);
        }
        (
            SchedulerInput::RendererLost(epoch),
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(
                ControlAdmission::Accepted(sequence),
            )),
            ExpectedInput::Loss(control),
        ) => {
            assert_eq!(epoch.get(), 0);
            assert_eq!(sequence.get(), control);
        }
        (
            SchedulerInput::RequestShutdown,
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(admission)),
            ExpectedInput::Shutdown { control, duplicate },
        ) => {
            let (sequence, actual_duplicate) = match admission {
                ControlAdmission::Accepted(sequence) => (sequence, false),
                ControlAdmission::AlreadyAccepted(sequence) => (sequence, true),
            };
            assert_eq!(sequence.get(), control);
            assert_eq!(actual_duplicate, duplicate);
        }
        (actual, outcome, _) => panic!("unexpected scheduler input: {actual:?} {outcome:?}"),
    }
}

use fenestra_ui_runtime::prototype::{
    CallbackFinish, ControlAdmission, SchedulerErrorKind, SchedulerInput, SchedulerInputResult,
    SchedulerState,
};

use crate::scheduler::{
    FakeCallbackDepthV1, SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1,
    SchedulerTraceStepV1,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StepRecordV1 {
    Callback {
        depth: FakeCallbackDepthV1,
        outcome: CallbackRecordV1,
    },
    Commit(CommitRecordV1),
    Input {
        input: InputRecordV1,
        outcome: InputOutcomeRecordV1,
    },
    Action(ActionRecordV1),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CallbackRecordV1 {
    NoChanges,
    Deferred { operations: usize, bytes: usize },
    ShutdownRequested,
    Rejected(SchedulerErrorKind),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CommitRecordV1 {
    Published,
    Noop,
    Rejected(SchedulerErrorKind),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum InputRecordV1 {
    FrameReady,
    AcceptFrame(u64),
    RejectFrame(u64),
    Complete { epoch: u64, token: u64 },
    RendererLost(u64),
    RequestShutdown,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum AdmissionRecordV1 {
    Accepted(u64),
    AlreadyAccepted(u64),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum InputResultRecordV1 {
    FrameReady,
    FrameAccepted { epoch: u64, token: u64 },
    FrameRejected(u64),
    Control(AdmissionRecordV1),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum InputOutcomeRecordV1 {
    Accepted(InputResultRecordV1),
    Retained(SchedulerErrorKind),
    Canceled,
    Rejected(SchedulerErrorKind),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ActionRecordV1 {
    Idle,
    RequestFrame,
    OfferFrame(u64),
    StopRenderer(u64),
    Rejected(SchedulerErrorKind),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct LaneRecordV1 {
    pub(crate) items: usize,
    pub(crate) bytes: usize,
    pub(crate) residence: Option<u64>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct SubmissionRecordV1 {
    pub(crate) epoch: u64,
    pub(crate) token: u64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct RendererRecordV1 {
    pub(crate) items: usize,
    pub(crate) bytes: usize,
    pub(crate) residence: Option<u64>,
    pub(crate) last: Option<SubmissionRecordV1>,
    pub(crate) completed: Option<SubmissionRecordV1>,
    pub(crate) pending: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct SchedulerEventRecordV1 {
    pub(crate) schema: u32,
    pub(crate) sequence: u64,
    pub(crate) domain: u32,
    pub(crate) tick: u64,
    pub(crate) step: StepRecordV1,
    pub(crate) lifecycle: SchedulerState,
    pub(crate) generation: u64,
    pub(crate) frame: Option<u64>,
    pub(crate) control: Option<u64>,
    pub(crate) deferred: LaneRecordV1,
    pub(crate) controls: LaneRecordV1,
    pub(crate) visual: LaneRecordV1,
    pub(crate) in_flight: LaneRecordV1,
    pub(crate) renderer: RendererRecordV1,
}

impl SchedulerEventRecordV1 {
    pub(crate) fn from_event(event: SchedulerTraceEventV1) -> Self {
        let renderer = event.renderer();
        Self {
            schema: event.schema_revision(),
            sequence: event.sequence(),
            domain: event.clock_domain().get(),
            tick: event.tick().get(),
            step: step(event.step()),
            lifecycle: event.lifecycle(),
            generation: event.generation().get(),
            frame: event.frame().map(|value| value.get()),
            control: event.control().map(|value| value.get()),
            deferred: lane(event.deferred()),
            controls: lane(event.controls()),
            visual: lane(event.visual()),
            in_flight: lane(event.in_flight()),
            renderer: RendererRecordV1 {
                items: renderer.items(),
                bytes: renderer.accounted_bytes(),
                residence: renderer.oldest_residence_ticks(),
                last: renderer.last_accepted().map(|value| SubmissionRecordV1 {
                    epoch: value.epoch().get(),
                    token: value.token(),
                }),
                completed: renderer.completed().map(|value| SubmissionRecordV1 {
                    epoch: value.epoch().get(),
                    token: value.token(),
                }),
                pending: renderer.has_pending_control(),
            },
        }
    }
}

fn lane(value: crate::scheduler::SchedulerTraceLaneStatsV1) -> LaneRecordV1 {
    LaneRecordV1 {
        items: value.items(),
        bytes: value.accounted_bytes(),
        residence: value.oldest_residence_ticks(),
    }
}

fn step(value: SchedulerTraceStepV1) -> StepRecordV1 {
    match value {
        SchedulerTraceStepV1::Callback { depth, outcome } => StepRecordV1::Callback {
            depth,
            outcome: match outcome {
                SchedulerTraceCallbackOutcomeV1::Finished(CallbackFinish::NoChanges) => {
                    CallbackRecordV1::NoChanges
                }
                SchedulerTraceCallbackOutcomeV1::Finished(CallbackFinish::Deferred {
                    operation_count,
                    accounted_bytes,
                }) => CallbackRecordV1::Deferred {
                    operations: operation_count,
                    bytes: accounted_bytes,
                },
                SchedulerTraceCallbackOutcomeV1::Finished(CallbackFinish::ShutdownRequested) => {
                    CallbackRecordV1::ShutdownRequested
                }
                SchedulerTraceCallbackOutcomeV1::Rejected(error) => {
                    CallbackRecordV1::Rejected(error)
                }
            },
        },
        SchedulerTraceStepV1::Commit(outcome) => StepRecordV1::Commit(match outcome {
            SchedulerTraceCommitOutcomeV1::Published => CommitRecordV1::Published,
            SchedulerTraceCommitOutcomeV1::Noop => CommitRecordV1::Noop,
            SchedulerTraceCommitOutcomeV1::Rejected(error) => CommitRecordV1::Rejected(error),
        }),
        SchedulerTraceStepV1::Input { input, outcome } => StepRecordV1::Input {
            input: input_record(input),
            outcome: input_outcome(outcome),
        },
        SchedulerTraceStepV1::Action(action) => StepRecordV1::Action(match action {
            SchedulerTraceActionV1::Idle => ActionRecordV1::Idle,
            SchedulerTraceActionV1::RequestFrame => ActionRecordV1::RequestFrame,
            SchedulerTraceActionV1::OfferFrame(frame) => ActionRecordV1::OfferFrame(frame.get()),
            SchedulerTraceActionV1::StopRenderer(control) => {
                ActionRecordV1::StopRenderer(control.get())
            }
            SchedulerTraceActionV1::Rejected(error) => ActionRecordV1::Rejected(error),
        }),
    }
}

fn input_record(value: SchedulerInput) -> InputRecordV1 {
    match value {
        SchedulerInput::FrameReady => InputRecordV1::FrameReady,
        SchedulerInput::AcceptFrame(frame) => InputRecordV1::AcceptFrame(frame.get()),
        SchedulerInput::RejectFrame(frame) => InputRecordV1::RejectFrame(frame.get()),
        SchedulerInput::Complete(value) => InputRecordV1::Complete {
            epoch: value.epoch().get(),
            token: value.token(),
        },
        SchedulerInput::RendererLost(epoch) => InputRecordV1::RendererLost(epoch.get()),
        SchedulerInput::RequestShutdown => InputRecordV1::RequestShutdown,
    }
}

fn input_outcome(value: SchedulerTraceInputOutcomeV1) -> InputOutcomeRecordV1 {
    match value {
        SchedulerTraceInputOutcomeV1::Accepted(result) => {
            InputOutcomeRecordV1::Accepted(match result {
                SchedulerInputResult::FrameReady => InputResultRecordV1::FrameReady,
                SchedulerInputResult::FrameAccepted(value) => InputResultRecordV1::FrameAccepted {
                    epoch: value.epoch().get(),
                    token: value.token(),
                },
                SchedulerInputResult::FrameRejected(frame) => {
                    InputResultRecordV1::FrameRejected(frame.get())
                }
                SchedulerInputResult::Control(value) => InputResultRecordV1::Control(match value {
                    ControlAdmission::Accepted(control) => {
                        AdmissionRecordV1::Accepted(control.get())
                    }
                    ControlAdmission::AlreadyAccepted(control) => {
                        AdmissionRecordV1::AlreadyAccepted(control.get())
                    }
                }),
            })
        }
        SchedulerTraceInputOutcomeV1::Retained(error) => InputOutcomeRecordV1::Retained(error),
        SchedulerTraceInputOutcomeV1::Canceled => InputOutcomeRecordV1::Canceled,
        SchedulerTraceInputOutcomeV1::Rejected(error) => InputOutcomeRecordV1::Rejected(error),
    }
}

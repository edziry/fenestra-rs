mod support;

use fenestra_ui_runtime::prototype::{
    ControlAdmission, QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerTick, UiRuntime, UiScheduler,
};

use support::{capacity, construction};

fn scheduler() -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    UiScheduler::new(
        runtime,
        SchedulerCapacity::new(
            QueueCapacity::new(1, 80, 8),
            QueueCapacity::new(1, 32, 8),
            QueueCapacity::new(1, 40, 8),
            QueueCapacity::new(2, 80, 8),
        ),
    )
    .expect("scheduler capacity should be valid")
}

#[test]
fn protocol_enums_are_closed_over_the_v1_control_surface() {
    fn input_tag(input: SchedulerInput) -> u8 {
        match input {
            SchedulerInput::FrameReady => 0,
            SchedulerInput::AcceptFrame(_) => 1,
            SchedulerInput::RejectFrame(_) => 2,
            SchedulerInput::Complete(_) => 3,
            SchedulerInput::RendererLost(_) => 4,
            SchedulerInput::RequestShutdown => 5,
        }
    }

    fn action_tag(action: &SchedulerAction) -> u8 {
        match action {
            SchedulerAction::RequestFrame => 0,
            SchedulerAction::OfferFrame(_) => 1,
            SchedulerAction::StopRenderer(_) => 2,
        }
    }

    fn result_tag(result: SchedulerInputResult) -> u8 {
        match result {
            SchedulerInputResult::FrameReady => 0,
            SchedulerInputResult::FrameAccepted(_) => 1,
            SchedulerInputResult::FrameRejected(_) => 2,
            SchedulerInputResult::Control(ControlAdmission::Accepted(_)) => 3,
            SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(_)) => 4,
        }
    }

    fn error_tag(error: SchedulerErrorKind) -> u8 {
        match error {
            SchedulerErrorKind::CapacityTooSmall(_) => 0,
            SchedulerErrorKind::RetainedGenerationCapacity => 1,
            SchedulerErrorKind::TickRegression => 2,
            SchedulerErrorKind::InputOutOfOrder => 3,
            SchedulerErrorKind::FrameIdMismatch => 4,
            SchedulerErrorKind::ControlPending => 5,
            SchedulerErrorKind::CapacityExceeded(_) => 6,
            SchedulerErrorKind::ArithmeticExhausted => 7,
            SchedulerErrorKind::ResidenceExceeded(_) => 8,
            SchedulerErrorKind::ForeignRendererEpoch => 9,
            SchedulerErrorKind::CompletionRegression => 10,
            SchedulerErrorKind::CompletionBeyondAccepted => 11,
            SchedulerErrorKind::Transaction(_) => 12,
        }
    }

    let mut scheduler = scheduler();
    let result = scheduler
        .process_input(SchedulerInput::RequestShutdown, SchedulerTick::new(1))
        .expect("shutdown should be accepted");
    let action = scheduler
        .next_action(SchedulerTick::new(1))
        .expect("shutdown action turn should advance")
        .expect("shutdown should emit one stop action");

    assert_eq!(input_tag(SchedulerInput::RequestShutdown), 5);
    assert_eq!(action_tag(&action), 2);
    assert_eq!(result_tag(result), 3);
    assert_eq!(error_tag(SchedulerErrorKind::ControlPending), 5);
}

use std::fmt::Write;

use fenestra_ui_runtime::prototype::{
    CapacityKind, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind, SchedulerErrorKind,
    SchedulerLane, TransactionErrorKind,
};

use super::super::super::record::scheduler::{
    ActionRecordV1, AdmissionRecordV1, CallbackRecordV1, CommitRecordV1, InputOutcomeRecordV1,
    InputRecordV1, InputResultRecordV1, StepRecordV1,
};
use crate::scheduler::FakeCallbackDepthV1;

pub(super) fn write_step(line: &mut String, step: StepRecordV1) {
    match step {
        StepRecordV1::Callback { depth, outcome } => callback(line, depth, outcome),
        StepRecordV1::Commit(outcome) => commit(line, outcome),
        StepRecordV1::Input { input, outcome } => input_step(line, input, outcome),
        StepRecordV1::Action(action) => write_action(line, action),
    }
}

fn callback(line: &mut String, depth: FakeCallbackDepthV1, outcome: CallbackRecordV1) {
    let depth = depth_number(depth);
    match outcome {
        CallbackRecordV1::NoChanges => fields(line, "callback-no-changes", &[number(depth)]),
        CallbackRecordV1::Deferred { operations, bytes } => fields(
            line,
            "callback-deferred",
            &[number(depth), operations.to_string(), bytes.to_string()],
        ),
        CallbackRecordV1::ShutdownRequested => {
            fields(line, "callback-shutdown-requested", &[number(depth)]);
        }
        CallbackRecordV1::Rejected(error) => fields(
            line,
            "callback-rejected",
            &[number(depth), scheduler_error(error)],
        ),
    }
}

fn commit(line: &mut String, outcome: CommitRecordV1) {
    match outcome {
        CommitRecordV1::Published => fields(line, "commit-published", &[]),
        CommitRecordV1::Noop => fields(line, "commit-no-change", &[]),
        CommitRecordV1::Rejected(error) => {
            fields(line, "commit-rejected", &[scheduler_error(error)]);
        }
    }
}

fn input_step(line: &mut String, input: InputRecordV1, outcome: InputOutcomeRecordV1) {
    match (input, outcome) {
        (
            InputRecordV1::FrameReady,
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::FrameReady),
        ) => {
            fields(line, "input-frame-ready", &["accepted".to_owned()]);
        }
        (
            InputRecordV1::AcceptFrame(_),
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::FrameAccepted { epoch, token }),
        ) => fields(
            line,
            "input-accept-frame",
            &[epoch.to_string(), token.to_string(), "accepted".to_owned()],
        ),
        (
            InputRecordV1::RejectFrame(_),
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::FrameRejected(_)),
        ) => fields(line, "input-reject-frame", &["accepted".to_owned()]),
        (
            InputRecordV1::Complete { epoch, token },
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
        ) => fields(
            line,
            "input-complete",
            &[
                epoch.to_string(),
                token.to_string(),
                admission_word(admission).to_owned(),
            ],
        ),
        (
            InputRecordV1::RendererLost(epoch),
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
        ) => fields(
            line,
            "input-renderer-lost",
            &[epoch.to_string(), admission_word(admission).to_owned()],
        ),
        (
            InputRecordV1::RequestShutdown,
            InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
        ) => fields(
            line,
            "input-shutdown",
            &[admission_word(admission).to_owned()],
        ),
        (input, outcome) => write_other_input(line, input, outcome),
    }
}

fn write_other_input(line: &mut String, input: InputRecordV1, outcome: InputOutcomeRecordV1) {
    let mut token = format!("input-{}", input_word(input));
    let mut values = input_payload(input);
    match outcome {
        InputOutcomeRecordV1::Accepted(result) => {
            let _ = write!(token, "-accepted-{}", result_word(result));
            values.extend(result_payload(result));
        }
        InputOutcomeRecordV1::Retained(error) => {
            token.push_str("-retained");
            values.push(scheduler_error(error));
        }
        InputOutcomeRecordV1::Canceled => token.push_str("-canceled"),
        InputOutcomeRecordV1::Rejected(error) => {
            token.push_str("-rejected");
            values.push(scheduler_error(error));
        }
    }
    fields(line, &token, &values);
}

fn write_action(line: &mut String, action: ActionRecordV1) {
    match action {
        ActionRecordV1::Idle => fields(line, "action-idle", &[]),
        ActionRecordV1::RequestFrame => fields(line, "action-request-frame", &[]),
        ActionRecordV1::OfferFrame(_) => fields(line, "action-offer-frame", &[]),
        ActionRecordV1::StopRenderer(_) => fields(line, "action-stop-renderer", &[]),
        ActionRecordV1::Rejected(SchedulerErrorKind::Transaction(
            TransactionErrorKind::MissingNode,
        )) => fields(line, "action-transaction-missing-node", &[]),
        ActionRecordV1::Rejected(error) => {
            fields(line, "action-rejected", &[scheduler_error(error)]);
        }
    }
}

fn fields(line: &mut String, token: &str, values: &[String]) {
    line.push('|');
    line.push_str(token);
    for index in 0..4 {
        line.push('|');
        line.push_str(values.get(index).map_or("-", String::as_str));
    }
}

fn input_word(input: InputRecordV1) -> &'static str {
    match input {
        InputRecordV1::FrameReady => "frame-ready",
        InputRecordV1::AcceptFrame(_) => "accept-frame",
        InputRecordV1::RejectFrame(_) => "reject-frame",
        InputRecordV1::Complete { .. } => "complete",
        InputRecordV1::RendererLost(_) => "renderer-lost",
        InputRecordV1::RequestShutdown => "shutdown",
    }
}

fn input_payload(input: InputRecordV1) -> Vec<String> {
    match input {
        InputRecordV1::FrameReady
        | InputRecordV1::AcceptFrame(_)
        | InputRecordV1::RejectFrame(_)
        | InputRecordV1::RequestShutdown => Vec::new(),
        InputRecordV1::Complete { epoch, token } => {
            vec![epoch.to_string(), token.to_string()]
        }
        InputRecordV1::RendererLost(epoch) => vec![epoch.to_string()],
    }
}

fn result_word(result: InputResultRecordV1) -> &'static str {
    match result {
        InputResultRecordV1::FrameReady => "frame-ready",
        InputResultRecordV1::FrameAccepted { .. } => "frame-accepted",
        InputResultRecordV1::FrameRejected(_) => "frame-rejected",
        InputResultRecordV1::Control(_) => "control",
    }
}

fn result_payload(result: InputResultRecordV1) -> Vec<String> {
    match result {
        InputResultRecordV1::FrameReady => Vec::new(),
        InputResultRecordV1::FrameAccepted { epoch, token } => {
            vec![epoch.to_string(), token.to_string()]
        }
        InputResultRecordV1::FrameRejected(frame) => vec![frame.to_string()],
        InputResultRecordV1::Control(admission) => {
            vec![
                admission_word(admission).to_owned(),
                admission_sequence(admission).to_string(),
            ]
        }
    }
}

const fn admission_word(admission: AdmissionRecordV1) -> &'static str {
    match admission {
        AdmissionRecordV1::Accepted(_) => "accepted",
        AdmissionRecordV1::AlreadyAccepted(_) => "already-accepted",
    }
}

const fn admission_sequence(admission: AdmissionRecordV1) -> u64 {
    match admission {
        AdmissionRecordV1::Accepted(value) | AdmissionRecordV1::AlreadyAccepted(value) => value,
    }
}

const fn depth_number(depth: FakeCallbackDepthV1) -> u64 {
    match depth {
        FakeCallbackDepthV1::Outer => 1,
        FakeCallbackDepthV1::Nested => 2,
        FakeCallbackDepthV1::Grandchild => 3,
    }
}

fn scheduler_error(error: SchedulerErrorKind) -> String {
    match error {
        SchedulerErrorKind::CapacityTooSmall(lane) => {
            format!("capacity-too-small-{}", lane_word(lane))
        }
        SchedulerErrorKind::RetainedGenerationCapacity => "retained-generation-capacity".into(),
        SchedulerErrorKind::TickRegression => "tick-regression".into(),
        SchedulerErrorKind::InputOutOfOrder => "input-out-of-order".into(),
        SchedulerErrorKind::FrameIdMismatch => "frame-id-mismatch".into(),
        SchedulerErrorKind::ControlPending => "control-pending".into(),
        SchedulerErrorKind::CapacityExceeded(lane) => {
            format!("capacity-exceeded-{}", lane_word(lane))
        }
        SchedulerErrorKind::ArithmeticExhausted => "arithmetic-exhausted".into(),
        SchedulerErrorKind::ResidenceExceeded(lane) => {
            format!("residence-exceeded-{}", lane_word(lane))
        }
        SchedulerErrorKind::ForeignRendererEpoch => "foreign-renderer-epoch".into(),
        SchedulerErrorKind::CompletionRegression => "completion-regression".into(),
        SchedulerErrorKind::CompletionBeyondAccepted => "completion-beyond-accepted".into(),
        SchedulerErrorKind::Transaction(error) => format!("transaction-{}", transaction(error)),
    }
}

fn transaction(error: TransactionErrorKind) -> String {
    match error {
        TransactionErrorKind::CapacityExceeded(kind) => {
            format!("capacity-exceeded-{}", capacity(kind))
        }
        TransactionErrorKind::Headless(kind) => format!("headless-{}", headless(kind)),
        TransactionErrorKind::Spatial(_) => "invariant-violation".into(),
        TransactionErrorKind::HeadlessUnavailable => "headless-unavailable".into(),
        TransactionErrorKind::SpatialUnavailable => "invariant-violation".into(),
        TransactionErrorKind::StaleBase => "stale-base".into(),
        TransactionErrorKind::MissingNode => "missing-node".into(),
        TransactionErrorKind::MissingFragment => "missing-fragment".into(),
        TransactionErrorKind::MissingKey => "missing-key".into(),
        TransactionErrorKind::DuplicateKey => "duplicate-key".into(),
        TransactionErrorKind::UnknownProperty => "unknown-property".into(),
        TransactionErrorKind::PropertyTypeMismatch => "property-type-mismatch".into(),
        TransactionErrorKind::IndexOutOfBounds => "index-out-of-bounds".into(),
        TransactionErrorKind::GenerationExhausted => "generation-exhausted".into(),
        TransactionErrorKind::InvariantViolation => "invariant-violation".into(),
    }
}

const fn lane_word(lane: SchedulerLane) -> &'static str {
    match lane {
        SchedulerLane::Deferred => "deferred",
        SchedulerLane::Controls => "controls",
        SchedulerLane::Visual => "visual",
        SchedulerLane::InFlight => "in-flight",
    }
}

const fn capacity(kind: CapacityKind) -> &'static str {
    match kind {
        CapacityKind::Operations => "operations",
        CapacityKind::StructuralChanges => "structural",
        CapacityKind::LiveNodes => "live-nodes",
        CapacityKind::LiveFragments => "live-fragments",
        CapacityKind::LivePropertySlots => "live-properties",
        CapacityKind::RetainedGenerations => "retained-generations",
    }
}

fn headless(kind: HeadlessProjectionErrorKind) -> String {
    match kind {
        HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty => {
            "missing-specification-target-or-property".into()
        }
        HeadlessProjectionErrorKind::PropertyTypeMismatch => "property-type-mismatch".into(),
        HeadlessProjectionErrorKind::InvalidSemanticTemplate => "invalid-semantic-template".into(),
        HeadlessProjectionErrorKind::InvalidSurface => "invalid-surface".into(),
        HeadlessProjectionErrorKind::CapacityExceeded(limit) => {
            format!("capacity-exceeded-{}", projection_limit(limit))
        }
        HeadlessProjectionErrorKind::NegativeGeometry => "negative-geometry".into(),
        HeadlessProjectionErrorKind::ArithmeticExhausted => "arithmetic-exhausted".into(),
        HeadlessProjectionErrorKind::InvariantViolation => "invariant-violation".into(),
    }
}

const fn projection_limit(limit: HeadlessProjectionLimitKind) -> &'static str {
    match limit {
        HeadlessProjectionLimitKind::ComputedStyles => "computed-styles",
        HeadlessProjectionLimitKind::Geometry => "geometry",
        HeadlessProjectionLimitKind::Semantics => "semantics",
        HeadlessProjectionLimitKind::HitRegions => "hit-regions",
        HeadlessProjectionLimitKind::SceneRectangles => "scene-rectangles",
    }
}

fn number(value: u64) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use fenestra_ui_runtime::prototype::{RuntimeSpatialErrorV2, TransactionErrorKind};

    use super::transaction;

    #[test]
    fn headless_artifact_v1_folds_unrepresented_spatial_failures_closed() {
        for error in [
            TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::ViewportMismatch),
            TransactionErrorKind::SpatialUnavailable,
        ] {
            assert_eq!(transaction(error), "invariant-violation");
        }
    }
}

use fenestra_ui_runtime::prototype::SchedulerErrorKind;

use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
};
use super::super::value::{parse_u64, parse_usize};
use crate::headless::artifact::record::scheduler::{
    ActionRecordV1, AdmissionRecordV1, CallbackRecordV1, CommitRecordV1, InputOutcomeRecordV1,
    InputRecordV1, InputResultRecordV1, StepRecordV1,
};
use crate::scheduler::FakeCallbackDepthV1;

pub(super) fn step_shape_v1(fields: &[&str]) -> bool {
    let [token, first, second, third, fourth] = fields else {
        return false;
    };
    match *token {
        "callback-no-changes" | "callback-shutdown-requested" => {
            present(first) && absent(second) && absent(third) && absent(fourth)
        }
        "callback-deferred" => {
            present(first) && present(second) && present(third) && absent(fourth)
        }
        "callback-rejected" => {
            present(first) && error_word(second) && absent(third) && absent(fourth)
        }
        "commit-published"
        | "commit-no-change"
        | "action-idle"
        | "action-request-frame"
        | "action-offer-frame"
        | "action-stop-renderer"
        | "action-transaction-missing-node" => all_absent(first, second, third, fourth),
        "commit-rejected" | "action-rejected" => {
            error_word(first) && absent(second) && absent(third) && absent(fourth)
        }
        "input-frame-ready" | "input-reject-frame" => {
            *first == "accepted" && absent(second) && absent(third) && absent(fourth)
        }
        "input-accept-frame" => {
            present(first) && present(second) && *third == "accepted" && absent(fourth)
        }
        "input-complete" => present(first) && present(second) && admission(third) && absent(fourth),
        "input-renderer-lost" => {
            present(first) && admission(second) && absent(third) && absent(fourth)
        }
        "input-shutdown" => admission(first) && absent(second) && absent(third) && absent(fourth),
        _ => other_input_shape(token, [first, second, third, fourth]),
    }
}

pub(super) fn parse_step_v1(
    fields: &[&str],
    frame: Option<u64>,
    control: Option<u64>,
    line: u32,
) -> Result<StepRecordV1, HeadlessArtifactDecodeErrorV1> {
    if !step_shape_v1(fields) {
        return Err(malformed(line));
    }
    let [token, first, second, third, _] = fields else {
        return Err(malformed(line));
    };
    let step = match *token {
        "callback-no-changes" => Ok(callback(first, CallbackRecordV1::NoChanges, line)?),
        "callback-deferred" => Ok(callback(
            first,
            CallbackRecordV1::Deferred {
                operations: parse_usize(second, line)?,
                bytes: parse_usize(third, line)?,
            },
            line,
        )?),
        "callback-shutdown-requested" => {
            Ok(callback(first, CallbackRecordV1::ShutdownRequested, line)?)
        }
        "callback-rejected" => Ok(callback(
            first,
            CallbackRecordV1::Rejected(parse_error(second, line)?),
            line,
        )?),
        "commit-published" => Ok(StepRecordV1::Commit(CommitRecordV1::Published)),
        "commit-no-change" => Ok(StepRecordV1::Commit(CommitRecordV1::Noop)),
        "commit-rejected" => Ok(StepRecordV1::Commit(CommitRecordV1::Rejected(parse_error(
            first, line,
        )?))),
        "input-frame-ready" => Ok(input(
            InputRecordV1::FrameReady,
            InputResultRecordV1::FrameReady,
        )),
        "input-accept-frame" => Ok(input(
            InputRecordV1::AcceptFrame(required(frame, line)?),
            InputResultRecordV1::FrameAccepted {
                epoch: parse_u64(first, line)?,
                token: parse_u64(second, line)?,
            },
        )),
        "input-reject-frame" => Ok(input(
            InputRecordV1::RejectFrame(required(frame, line)?),
            InputResultRecordV1::FrameRejected(required(frame, line)?),
        )),
        "input-complete" => Ok(input(
            InputRecordV1::Complete {
                epoch: parse_u64(first, line)?,
                token: parse_u64(second, line)?,
            },
            InputResultRecordV1::Control(parse_admission(third, control, line)?),
        )),
        "input-renderer-lost" => Ok(input(
            InputRecordV1::RendererLost(parse_u64(first, line)?),
            InputResultRecordV1::Control(parse_admission(second, control, line)?),
        )),
        "input-shutdown" => Ok(input(
            InputRecordV1::RequestShutdown,
            InputResultRecordV1::Control(parse_admission(first, control, line)?),
        )),
        "action-idle" => Ok(StepRecordV1::Action(ActionRecordV1::Idle)),
        "action-request-frame" => Ok(StepRecordV1::Action(ActionRecordV1::RequestFrame)),
        "action-offer-frame" => Ok(StepRecordV1::Action(ActionRecordV1::OfferFrame(required(
            frame, line,
        )?))),
        "action-stop-renderer" => Ok(StepRecordV1::Action(ActionRecordV1::StopRenderer(
            required(control, line)?,
        ))),
        "action-transaction-missing-node" => Ok(StepRecordV1::Action(ActionRecordV1::Rejected(
            SchedulerErrorKind::Transaction(
                fenestra_ui_runtime::prototype::TransactionErrorKind::MissingNode,
            ),
        ))),
        "action-rejected" => Ok(StepRecordV1::Action(ActionRecordV1::Rejected(parse_error(
            first, line,
        )?))),
        _ => parse_other_input(token, [first, second, third, fields[4]], frame, line),
    }?;
    Ok(step)
}

fn callback(
    depth: &str,
    outcome: CallbackRecordV1,
    line: u32,
) -> Result<StepRecordV1, HeadlessArtifactDecodeErrorV1> {
    let depth = match parse_u64(depth, line)? {
        1 => FakeCallbackDepthV1::Outer,
        2 => FakeCallbackDepthV1::Nested,
        3 => FakeCallbackDepthV1::Grandchild,
        _ => return Err(malformed(line)),
    };
    Ok(StepRecordV1::Callback { depth, outcome })
}

fn input(input: InputRecordV1, result: InputResultRecordV1) -> StepRecordV1 {
    StepRecordV1::Input {
        input,
        outcome: InputOutcomeRecordV1::Accepted(result),
    }
}

fn parse_admission(
    value: &str,
    control: Option<u64>,
    line: u32,
) -> Result<AdmissionRecordV1, HeadlessArtifactDecodeErrorV1> {
    let control = required(control, line)?;
    match value {
        "accepted" => Ok(AdmissionRecordV1::Accepted(control)),
        "already-accepted" => Ok(AdmissionRecordV1::AlreadyAccepted(control)),
        _ => Err(malformed(line)),
    }
}

fn parse_error(
    value: &str,
    line: u32,
) -> Result<SchedulerErrorKind, HeadlessArtifactDecodeErrorV1> {
    super::error::parse_scheduler_error_v1(value).ok_or_else(|| malformed(line))
}

fn other_input_shape(token: &str, fields: [&str; 4]) -> bool {
    let Some((input, outcome)) = split_other_input(token) else {
        return false;
    };
    let payload = input_payload_count(input);
    match outcome {
        OtherOutcomeV1::Canceled => used_then_absent(fields, payload),
        OtherOutcomeV1::Retained | OtherOutcomeV1::Rejected => {
            payload < 4 && used_then_absent(fields, payload + 1) && error_word(fields[payload])
        }
        OtherOutcomeV1::Accepted(result) => {
            let result_fields = result_payload_count(result);
            let shape =
                payload + result_fields <= 4 && used_then_absent(fields, payload + result_fields);
            shape && (!matches!(result, ResultWordV1::Control) || admission(fields[payload]))
        }
    }
}

fn parse_other_input(
    token: &str,
    fields: [&str; 4],
    frame: Option<u64>,
    line: u32,
) -> Result<StepRecordV1, HeadlessArtifactDecodeErrorV1> {
    let (input_word, outcome) = split_other_input(token).ok_or_else(|| malformed(line))?;
    let payload = input_payload_count(input_word);
    let input = parse_input(input_word, fields, frame, line)?;
    let outcome = match outcome {
        OtherOutcomeV1::Canceled => InputOutcomeRecordV1::Canceled,
        OtherOutcomeV1::Retained => {
            InputOutcomeRecordV1::Retained(parse_error(fields[payload], line)?)
        }
        OtherOutcomeV1::Rejected => {
            InputOutcomeRecordV1::Rejected(parse_error(fields[payload], line)?)
        }
        OtherOutcomeV1::Accepted(result) => {
            InputOutcomeRecordV1::Accepted(parse_result(result, &fields[payload..], line)?)
        }
    };
    Ok(StepRecordV1::Input { input, outcome })
}

fn parse_input(
    value: InputWordV1,
    fields: [&str; 4],
    frame: Option<u64>,
    line: u32,
) -> Result<InputRecordV1, HeadlessArtifactDecodeErrorV1> {
    Ok(match value {
        InputWordV1::FrameReady => InputRecordV1::FrameReady,
        InputWordV1::AcceptFrame => InputRecordV1::AcceptFrame(required(frame, line)?),
        InputWordV1::RejectFrame => InputRecordV1::RejectFrame(required(frame, line)?),
        InputWordV1::Complete => InputRecordV1::Complete {
            epoch: parse_u64(fields[0], line)?,
            token: parse_u64(fields[1], line)?,
        },
        InputWordV1::RendererLost => InputRecordV1::RendererLost(parse_u64(fields[0], line)?),
        InputWordV1::Shutdown => InputRecordV1::RequestShutdown,
    })
}

fn required(value: Option<u64>, line: u32) -> Result<u64, HeadlessArtifactDecodeErrorV1> {
    value.ok_or_else(|| malformed(line))
}

fn parse_result(
    result: ResultWordV1,
    fields: &[&str],
    line: u32,
) -> Result<InputResultRecordV1, HeadlessArtifactDecodeErrorV1> {
    Ok(match result {
        ResultWordV1::FrameReady => InputResultRecordV1::FrameReady,
        ResultWordV1::FrameAccepted => InputResultRecordV1::FrameAccepted {
            epoch: parse_u64(fields[0], line)?,
            token: parse_u64(fields[1], line)?,
        },
        ResultWordV1::FrameRejected => {
            InputResultRecordV1::FrameRejected(parse_u64(fields[0], line)?)
        }
        ResultWordV1::Control => {
            let sequence = parse_u64(fields[1], line)?;
            let admission = match fields[0] {
                "accepted" => AdmissionRecordV1::Accepted(sequence),
                "already-accepted" => AdmissionRecordV1::AlreadyAccepted(sequence),
                _ => return Err(malformed(line)),
            };
            InputResultRecordV1::Control(admission)
        }
    })
}

#[derive(Clone, Copy)]
enum InputWordV1 {
    FrameReady,
    AcceptFrame,
    RejectFrame,
    Complete,
    RendererLost,
    Shutdown,
}

#[derive(Clone, Copy)]
enum ResultWordV1 {
    FrameReady,
    FrameAccepted,
    FrameRejected,
    Control,
}

#[derive(Clone, Copy)]
enum OtherOutcomeV1 {
    Accepted(ResultWordV1),
    Retained,
    Canceled,
    Rejected,
}

fn split_other_input(token: &str) -> Option<(InputWordV1, OtherOutcomeV1)> {
    let suffixes = [
        (
            "-accepted-frame-ready",
            OtherOutcomeV1::Accepted(ResultWordV1::FrameReady),
        ),
        (
            "-accepted-frame-accepted",
            OtherOutcomeV1::Accepted(ResultWordV1::FrameAccepted),
        ),
        (
            "-accepted-frame-rejected",
            OtherOutcomeV1::Accepted(ResultWordV1::FrameRejected),
        ),
        (
            "-accepted-control",
            OtherOutcomeV1::Accepted(ResultWordV1::Control),
        ),
        ("-retained", OtherOutcomeV1::Retained),
        ("-canceled", OtherOutcomeV1::Canceled),
        ("-rejected", OtherOutcomeV1::Rejected),
    ];
    for (suffix, outcome) in suffixes {
        if let Some(prefix) = token.strip_suffix(suffix) {
            return parse_input_word(prefix).map(|input| (input, outcome));
        }
    }
    None
}

fn parse_input_word(value: &str) -> Option<InputWordV1> {
    match value {
        "input-frame-ready" => Some(InputWordV1::FrameReady),
        "input-accept-frame" => Some(InputWordV1::AcceptFrame),
        "input-reject-frame" => Some(InputWordV1::RejectFrame),
        "input-complete" => Some(InputWordV1::Complete),
        "input-renderer-lost" => Some(InputWordV1::RendererLost),
        "input-shutdown" => Some(InputWordV1::Shutdown),
        _ => None,
    }
}

const fn input_payload_count(value: InputWordV1) -> usize {
    match value {
        InputWordV1::Complete => 2,
        InputWordV1::RendererLost => 1,
        InputWordV1::FrameReady
        | InputWordV1::AcceptFrame
        | InputWordV1::RejectFrame
        | InputWordV1::Shutdown => 0,
    }
}

const fn result_payload_count(value: ResultWordV1) -> usize {
    match value {
        ResultWordV1::FrameReady => 0,
        ResultWordV1::FrameAccepted | ResultWordV1::Control => 2,
        ResultWordV1::FrameRejected => 1,
    }
}

fn used_then_absent(fields: [&str; 4], used: usize) -> bool {
    fields
        .iter()
        .enumerate()
        .all(|(index, value)| (index < used) == present(value))
}

fn error_word(value: &str) -> bool {
    super::error::scheduler_error_word_v1(value)
}

fn admission(value: &str) -> bool {
    matches!(value, "accepted" | "already-accepted")
}

fn present(value: &str) -> bool {
    value != "-"
}

fn absent(value: &str) -> bool {
    value == "-"
}

fn all_absent(first: &str, second: &str, third: &str, fourth: &str) -> bool {
    absent(first) && absent(second) && absent(third) && absent(fourth)
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::MalformedRecord, line)
}

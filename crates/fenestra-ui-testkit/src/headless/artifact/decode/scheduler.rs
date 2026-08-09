mod common;
mod error;
mod step;

use fenestra_ui_runtime::prototype::SchedulerState;

use super::super::error::{HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1};
use super::scan::ScannedLineV1;
use super::value::{
    parse_bool, parse_optional_u64, parse_submission, parse_u32, parse_u64, parse_usize,
};
use crate::headless::artifact::record::scheduler::{
    LaneRecordV1, RendererRecordV1, SchedulerEventRecordV1, SubmissionRecordV1,
};

pub(super) fn step_shape_v1(fields: &[&str]) -> bool {
    step::step_shape_v1(fields)
}

pub(super) fn parse_scheduler_event_v1(
    line: &ScannedLineV1<'_>,
) -> Result<SchedulerEventRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = split_fields::<32>(line)?;
    let frame = parse_optional_u64(fields[12], line.number)?;
    let control = parse_optional_u64(fields[13], line.number)?;
    let step = step::parse_step_v1(&fields[5..10], frame, control, line.number)?;
    common::validate_common_fields_v1(step, frame, control, line.number)?;
    Ok(SchedulerEventRecordV1 {
        schema: parse_u32(fields[1], line.number)?,
        sequence: parse_u64(fields[2], line.number)?,
        domain: parse_u32(fields[3], line.number)?,
        tick: parse_u64(fields[4], line.number)?,
        step,
        lifecycle: parse_lifecycle(fields[10], line.number)?,
        generation: parse_u64(fields[11], line.number)?,
        frame,
        control,
        deferred: lane(&fields[14..17], line.number)?,
        controls: lane(&fields[17..20], line.number)?,
        visual: lane(&fields[20..23], line.number)?,
        in_flight: lane(&fields[23..26], line.number)?,
        renderer: renderer(&fields[26..32], line.number)?,
    })
}

fn lane(fields: &[&str], line: u32) -> Result<LaneRecordV1, HeadlessArtifactDecodeErrorV1> {
    Ok(LaneRecordV1 {
        items: parse_usize(fields[0], line)?,
        bytes: parse_usize(fields[1], line)?,
        residence: parse_optional_u64(fields[2], line)?,
    })
}

fn renderer(fields: &[&str], line: u32) -> Result<RendererRecordV1, HeadlessArtifactDecodeErrorV1> {
    Ok(RendererRecordV1 {
        items: parse_usize(fields[0], line)?,
        bytes: parse_usize(fields[1], line)?,
        residence: parse_optional_u64(fields[2], line)?,
        last: submission(fields[3], line)?,
        completed: submission(fields[4], line)?,
        pending: parse_bool(fields[5], line)?,
    })
}

fn submission(
    value: &str,
    line: u32,
) -> Result<Option<SubmissionRecordV1>, HeadlessArtifactDecodeErrorV1> {
    Ok(parse_submission(value, line)?.map(|(epoch, token)| SubmissionRecordV1 { epoch, token }))
}

fn parse_lifecycle(
    value: &str,
    line: u32,
) -> Result<SchedulerState, HeadlessArtifactDecodeErrorV1> {
    match value {
        "running" => Ok(SchedulerState::Running),
        "shutdown-queued" => Ok(SchedulerState::ShutdownQueued),
        "draining" => Ok(SchedulerState::Draining),
        "stopped" => Ok(SchedulerState::Stopped),
        "faulted" => Ok(SchedulerState::Faulted),
        _ => Err(malformed(line)),
    }
}

fn split_fields<'source, const N: usize>(
    line: &ScannedLineV1<'source>,
) -> Result<[&'source str; N], HeadlessArtifactDecodeErrorV1> {
    let mut fields = [""; N];
    let mut count = 0_usize;
    for field in line.text.split('|') {
        let Some(slot) = fields.get_mut(count) else {
            return Err(malformed(line.number));
        };
        *slot = field;
        count += 1;
    }
    if count == N {
        Ok(fields)
    } else {
        Err(malformed(line.number))
    }
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::MalformedRecord, line)
}

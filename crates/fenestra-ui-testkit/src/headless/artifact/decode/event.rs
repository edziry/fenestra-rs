use fenestra_ui_runtime::prototype::HeadlessSurface;

use super::super::error::{HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1};
use super::scan::ScannedLineV1;
use super::value::{parse_optional_u64, parse_u32, parse_u64, parse_usize};
use crate::headless::artifact::record::{QueueRecordV1, TraceEventRecordV1};
use crate::headless::platform::HeadlessPointerTargetV1;
use crate::headless::trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1,
    HeadlessTraceProjectionCountsV1, HeadlessTraceStageV1,
};

pub(super) fn parse_headless_event_v1(
    line: &ScannedLineV1<'_>,
) -> Result<TraceEventRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = split_fields::<30>(line)?;
    let [
        "h-event",
        schema,
        sequence,
        domain,
        tick,
        stage,
        input,
        outcome,
        captured,
        published,
        target,
        frame,
        control,
        surface_width,
        surface_height,
        computed,
        geometry,
        semantics,
        hits,
        scene,
        deferred_items,
        deferred_bytes,
        control_items,
        control_bytes,
        visual_items,
        visual_bytes,
        in_flight_items,
        in_flight_bytes,
        renderer_items,
        renderer_bytes,
    ] = &fields
    else {
        return Err(malformed(line.number));
    };
    Ok(TraceEventRecordV1 {
        schema: parse_u32(schema, line.number)?,
        sequence: parse_u64(sequence, line.number)?,
        domain: parse_u32(domain, line.number)?,
        tick: parse_u64(tick, line.number)?,
        stage: parse_stage(stage, line.number)?,
        input: parse_input(input, line.number)?,
        outcome: parse_outcome(outcome, line.number)?,
        captured: parse_optional_u64(captured, line.number)?,
        published: parse_optional_u64(published, line.number)?,
        target: parse_target(target, line.number)?,
        frame: parse_optional_u64(frame, line.number)?,
        control: parse_optional_u64(control, line.number)?,
        surface: HeadlessSurface::new(
            super::value::parse_i32(surface_width, line.number)?,
            super::value::parse_i32(surface_height, line.number)?,
        ),
        counts: HeadlessTraceProjectionCountsV1::new(
            parse_usize(computed, line.number)?,
            parse_usize(geometry, line.number)?,
            parse_usize(semantics, line.number)?,
            parse_usize(hits, line.number)?,
            parse_usize(scene, line.number)?,
        ),
        deferred: queue(deferred_items, deferred_bytes, line.number)?,
        controls: queue(control_items, control_bytes, line.number)?,
        visual: queue(visual_items, visual_bytes, line.number)?,
        in_flight: queue(in_flight_items, in_flight_bytes, line.number)?,
        renderer: queue(renderer_items, renderer_bytes, line.number)?,
    })
}

fn queue(
    items: &str,
    bytes: &str,
    line: u32,
) -> Result<QueueRecordV1, HeadlessArtifactDecodeErrorV1> {
    Ok(QueueRecordV1 {
        items: parse_usize(items, line)?,
        bytes: parse_usize(bytes, line)?,
    })
}

fn parse_stage(
    value: &str,
    line: u32,
) -> Result<HeadlessTraceStageV1, HeadlessArtifactDecodeErrorV1> {
    match value {
        "build" => Ok(HeadlessTraceStageV1::Build),
        "input" => Ok(HeadlessTraceStageV1::Input),
        "callback" => Ok(HeadlessTraceStageV1::Callback),
        "transaction" => Ok(HeadlessTraceStageV1::Transaction),
        "projection" => Ok(HeadlessTraceStageV1::Projection),
        "scheduler" => Ok(HeadlessTraceStageV1::Scheduler),
        "renderer" => Ok(HeadlessTraceStageV1::Renderer),
        _ => Err(malformed(line)),
    }
}

fn parse_input(
    value: &str,
    line: u32,
) -> Result<HeadlessInputKindV1, HeadlessArtifactDecodeErrorV1> {
    match value {
        "none" => Ok(HeadlessInputKindV1::None),
        "pointer" => Ok(HeadlessInputKindV1::Pointer),
        "direct" => Ok(HeadlessInputKindV1::Direct),
        "insert" => Ok(HeadlessInputKindV1::Insert),
        "move" => Ok(HeadlessInputKindV1::Move),
        "update" => Ok(HeadlessInputKindV1::Update),
        "remove" => Ok(HeadlessInputKindV1::Remove),
        "resize" => Ok(HeadlessInputKindV1::Resize),
        "frame-ready" => Ok(HeadlessInputKindV1::FrameReady),
        "completion" => Ok(HeadlessInputKindV1::Completion),
        "loss" => Ok(HeadlessInputKindV1::Loss),
        "shutdown" => Ok(HeadlessInputKindV1::Shutdown),
        _ => Err(malformed(line)),
    }
}

fn parse_outcome(
    value: &str,
    line: u32,
) -> Result<HeadlessOutcomeV1, HeadlessArtifactDecodeErrorV1> {
    match value {
        "observed" => Ok(HeadlessOutcomeV1::Observed),
        "deferred" => Ok(HeadlessOutcomeV1::Deferred),
        "published" => Ok(HeadlessOutcomeV1::Published),
        "no-change" => Ok(HeadlessOutcomeV1::NoChange),
        "matched" => Ok(HeadlessOutcomeV1::Matched),
        "action" => Ok(HeadlessOutcomeV1::Action),
        "accepted" => Ok(HeadlessOutcomeV1::Accepted),
        "rejected" => Ok(HeadlessOutcomeV1::Rejected),
        "completed" => Ok(HeadlessOutcomeV1::Completed),
        "lost" => Ok(HeadlessOutcomeV1::Lost),
        "stopped" => Ok(HeadlessOutcomeV1::Stopped),
        "failed:runtime" => Ok(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Runtime)),
        "failed:projection" => Ok(HeadlessOutcomeV1::Failed(
            HeadlessFailureCauseV1::Projection,
        )),
        "failed:oracle" => Ok(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Oracle)),
        "failed:scheduler" => Ok(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Scheduler)),
        "failed:renderer" => Ok(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Renderer)),
        "failed:trace" => Ok(HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Trace)),
        _ => Err(malformed(line)),
    }
}

fn parse_target(
    value: &str,
    line: u32,
) -> Result<HeadlessPointerTargetV1, HeadlessArtifactDecodeErrorV1> {
    match value {
        "none" => Ok(HeadlessPointerTargetV1::None),
        "static-control" => Ok(HeadlessPointerTargetV1::StaticControl),
        _ => value.strip_prefix("key:").map_or_else(
            || Err(malformed(line)),
            |key| Ok(HeadlessPointerTargetV1::Key(parse_u64(key, line)?)),
        ),
    }
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::MalformedRecord, line)
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

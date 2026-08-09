mod event;
mod grammar;
mod preflight;
mod projection;
mod reference;
pub(super) mod scan;
mod scheduler;
mod state;
mod value;

use super::error::{HeadlessArtifactDecodeErrorKindV1 as ErrorKind, HeadlessArtifactDecodeErrorV1};

use self::event::parse_headless_event_v1;
use self::grammar::validate_closed_grammar_v1;
use self::preflight::preflight_v1;
use self::projection::parse_projection_v1;
use self::scan::scan_artifact_v1;
use self::scheduler::parse_scheduler_event_v1;
use self::state::{LayoutV1, SectionRangeV1, scan_layout_v1};
use super::model::HeadlessArtifactV1;
use super::record::{SchedulerEventRecordV1, TraceEventRecordV1};
use crate::headless::runner::HeadlessResultV1;

/// Decodes one complete bounded headless V1 artifact structurally.
///
/// This validates canonical storage, grammar, counts, and artifact-local
/// references. Semantic fixture provenance and replay remain verifier work.
pub fn decode_headless_artifact_v1(
    bytes: &[u8],
) -> Result<HeadlessArtifactV1, HeadlessArtifactDecodeErrorV1> {
    let scanned = scan_artifact_v1(bytes)?;
    validate_closed_grammar_v1(scanned.lines())?;
    let layout = scan_layout_v1(scanned.lines())?;
    let preflight = preflight_v1(&scanned, layout)?;
    let headless_events = parse_headless_events(&scanned, layout.headless)?;
    let scheduler_events = parse_scheduler_events(&scanned, layout.scheduler)?;
    let (final_generation, projection) = parse_projection_v1(&scanned, layout)?;
    let artifact = HeadlessArtifactV1 {
        metadata: preflight.metadata,
        capacities: preflight.capacities,
        headless_events,
        scheduler_events,
        final_generation,
        projection,
        result: parse_result(&scanned, layout)?,
    };
    reference::validate_references_v1(&artifact, &scanned, layout)?;
    if let Some(line) = scanned.lines().get(layout.trailing_start) {
        return Err(HeadlessArtifactDecodeErrorV1::at(
            ErrorKind::TrailingData,
            line.number,
        ));
    }
    Ok(artifact)
}

fn parse_headless_events(
    scanned: &scan::ScannedArtifactV1<'_>,
    range: SectionRangeV1,
) -> Result<Vec<TraceEventRecordV1>, HeadlessArtifactDecodeErrorV1> {
    let mut events = Vec::with_capacity(range.records_end - range.records_start);
    for line in &scanned.lines()[range.records_start..range.records_end] {
        events.push(parse_headless_event_v1(line)?);
    }
    Ok(events)
}

fn parse_scheduler_events(
    scanned: &scan::ScannedArtifactV1<'_>,
    range: SectionRangeV1,
) -> Result<Vec<SchedulerEventRecordV1>, HeadlessArtifactDecodeErrorV1> {
    let mut events = Vec::with_capacity(range.records_end - range.records_start);
    for line in &scanned.lines()[range.records_start..range.records_end] {
        events.push(parse_scheduler_event_v1(line)?);
    }
    Ok(events)
}

fn parse_result(
    scanned: &scan::ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<HeadlessResultV1, HeadlessArtifactDecodeErrorV1> {
    match scanned.lines()[layout.result].text {
        "result|pass" => Ok(HeadlessResultV1::Pass),
        "result|adapt" => Ok(HeadlessResultV1::Adapt),
        "result|stop" => Ok(HeadlessResultV1::Stop),
        _ => Err(HeadlessArtifactDecodeErrorV1::at(
            ErrorKind::MalformedRecord,
            scanned.lines()[layout.result].number,
        )),
    }
}

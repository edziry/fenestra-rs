mod event;
mod projection;

use super::error::{HeadlessArtifactEncodeErrorKindV1, HeadlessArtifactEncodeErrorV1};
use super::model::HeadlessArtifactV1;
use crate::headless::runner::HeadlessResultV1;
use crate::headless::trace::HeadlessTraceEventV1;
use crate::scheduler::SchedulerTraceEventV1;

pub(super) const ARTIFACT_BYTES: usize = 65_536;
pub(super) const LINE_BYTES: usize = 1_024;
pub(super) const LINES: usize = 512;

const FIXED_HEADER: [&str; 4] = [
    "fenestra-headless-spine|1",
    "versions|fixture|1|schema|1|construction|1|style|1|trace|1|projection|1",
    "environment|platform|headless-fake|clock|scheduler|domain|8001",
    "projection-choices|full|vertical|rebuilt|reverse",
];

/// Encodes one typed V1 headless artifact in canonical bounded form.
pub fn encode_headless_artifact_v1(
    artifact: &HeadlessArtifactV1,
) -> Result<Vec<u8>, HeadlessArtifactEncodeErrorV1> {
    let mut measurement = OutputMeasurementV1::default();
    render_artifact(&mut measurement, artifact)?;
    measurement.validate()?;

    let mut output = OutputBytesV1::new(measurement.artifact_bytes)?;
    render_artifact(&mut output, artifact)?;
    debug_assert_eq!(output.bytes.len(), measurement.artifact_bytes);
    Ok(output.bytes)
}

fn render_artifact(
    output: &mut impl LineSinkV1,
    artifact: &HeadlessArtifactV1,
) -> Result<(), HeadlessArtifactEncodeErrorV1> {
    let headless_bytes = accounted_bytes(
        artifact.headless_events.len(),
        HeadlessTraceEventV1::ACCOUNTED_BYTES,
    )?;
    let scheduler_bytes = accounted_bytes(
        artifact.scheduler_events.len(),
        SchedulerTraceEventV1::ACCOUNTED_BYTES,
    )?;
    write_header(output, artifact)?;
    push_line(
        output,
        &format!(
            "headless-trace-begin|{}|{}",
            artifact.headless_events.len(),
            headless_bytes
        ),
    )?;
    for event in &artifact.headless_events {
        push_line(output, &event::headless_line(event))?;
    }
    push_line(output, "headless-trace-end")?;
    push_line(
        output,
        &format!(
            "scheduler-trace-begin|{}|{}",
            artifact.scheduler_events.len(),
            scheduler_bytes
        ),
    )?;
    for event in &artifact.scheduler_events {
        push_line(output, &event::scheduler_line(event))?;
    }
    push_line(output, "scheduler-trace-end")?;
    projection::write_projection(output, artifact)?;
    push_line(output, result_line(artifact.result))?;
    push_line(output, "end")
}

fn write_header(
    output: &mut impl LineSinkV1,
    artifact: &HeadlessArtifactV1,
) -> Result<(), HeadlessArtifactEncodeErrorV1> {
    push_line(output, FIXED_HEADER[0])?;
    push_line(output, FIXED_HEADER[1])?;
    let metadata = artifact.metadata;
    push_line(
        output,
        &format!(
            "fixture|headless-spine|{}|{}|{}|{}|{}|{}",
            metadata.fixture_revision,
            metadata.schema_format,
            metadata.schema_namespace,
            metadata.schema_revision,
            metadata.construction_format,
            metadata.style_format,
        ),
    )?;
    push_line(output, FIXED_HEADER[2])?;
    push_line(output, FIXED_HEADER[3])?;
    let capacities = artifact.capacities;
    push_values(output, "capacity-ir", &capacities.ir)?;
    push_values(output, "capacity-style", &[capacities.style])?;
    push_values(output, "capacity-runtime", &capacities.runtime)?;
    push_values(output, "capacity-projection", &capacities.projection)?;
    push_values(output, "capacity-scheduler", &capacities.scheduler)?;
    push_values(output, "capacity-renderer", &capacities.renderer)?;
    push_values(
        output,
        "capacity-scheduler-trace",
        &capacities.scheduler_trace,
    )?;
    push_values(
        output,
        "capacity-headless-trace",
        &capacities.headless_trace,
    )?;
    push_values(output, "capacity-artifact", &capacities.artifact)
}

fn push_values(
    output: &mut impl LineSinkV1,
    marker: &str,
    values: &[usize],
) -> Result<(), HeadlessArtifactEncodeErrorV1> {
    let mut line = marker.to_owned();
    for value in values {
        line.push('|');
        line.push_str(&value.to_string());
    }
    push_line(output, &line)
}

pub(super) fn push_line(
    output: &mut impl LineSinkV1,
    line: &str,
) -> Result<(), HeadlessArtifactEncodeErrorV1> {
    output.push_line(line)
}

pub(super) trait LineSinkV1 {
    fn push_line(&mut self, line: &str) -> Result<(), HeadlessArtifactEncodeErrorV1>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct OutputMeasurementV1 {
    artifact_bytes: usize,
    line_bytes: usize,
    lines: usize,
}

impl OutputMeasurementV1 {
    fn validate(self) -> Result<(), HeadlessArtifactEncodeErrorV1> {
        if self.artifact_bytes > ARTIFACT_BYTES {
            return Err(error(HeadlessArtifactEncodeErrorKindV1::ArtifactBytes));
        }
        if self.line_bytes > LINE_BYTES {
            return Err(error(HeadlessArtifactEncodeErrorKindV1::LineBytes));
        }
        if self.lines > LINES {
            return Err(error(HeadlessArtifactEncodeErrorKindV1::Lines));
        }
        Ok(())
    }
}

impl LineSinkV1 for OutputMeasurementV1 {
    fn push_line(&mut self, line: &str) -> Result<(), HeadlessArtifactEncodeErrorV1> {
        let artifact_bytes = self
            .artifact_bytes
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or_else(artifact_bytes_error)?;
        let lines = self.lines.checked_add(1).ok_or_else(artifact_bytes_error)?;
        self.artifact_bytes = artifact_bytes;
        self.line_bytes = self.line_bytes.max(line.len());
        self.lines = lines;
        Ok(())
    }
}

struct OutputBytesV1 {
    bytes: Vec<u8>,
}

impl OutputBytesV1 {
    fn new(capacity: usize) -> Result<Self, HeadlessArtifactEncodeErrorV1> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| artifact_bytes_error())?;
        Ok(Self { bytes })
    }
}

impl LineSinkV1 for OutputBytesV1 {
    fn push_line(&mut self, line: &str) -> Result<(), HeadlessArtifactEncodeErrorV1> {
        self.bytes.extend_from_slice(line.as_bytes());
        self.bytes.push(b'\n');
        Ok(())
    }
}

fn accounted_bytes(
    count: usize,
    bytes_per_record: usize,
) -> Result<usize, HeadlessArtifactEncodeErrorV1> {
    count
        .checked_mul(bytes_per_record)
        .ok_or_else(artifact_bytes_error)
}

const fn result_line(result: HeadlessResultV1) -> &'static str {
    match result {
        HeadlessResultV1::Pass => "result|pass",
        HeadlessResultV1::Adapt => "result|adapt",
        HeadlessResultV1::Stop => "result|stop",
    }
}

const fn error(kind: HeadlessArtifactEncodeErrorKindV1) -> HeadlessArtifactEncodeErrorV1 {
    HeadlessArtifactEncodeErrorV1::new(kind)
}

const fn artifact_bytes_error() -> HeadlessArtifactEncodeErrorV1 {
    error(HeadlessArtifactEncodeErrorKindV1::ArtifactBytes)
}

#[cfg(test)]
mod tests;

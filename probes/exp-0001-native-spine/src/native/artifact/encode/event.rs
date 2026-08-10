use std::fmt::{self, Write};

use fenestra_ui_runtime::prototype::SchedulerState;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceEventV1, NativeTraceStageV1,
};

pub(super) fn write_event(output: &mut impl Write, event: NativeTraceEventV1) -> fmt::Result {
    write!(
        output,
        "event|sequence={}|tick={}|scheduler_turn=",
        event.sequence(),
        event.tick().get(),
    )?;
    write_optional_u64(output, event.scheduler_turn())?;
    write!(
        output,
        concat!(
            "|stage={}|observation={}|outcome={}|source={}|state={}",
            "|generation={}|captured="
        ),
        stage(event.stage()),
        observation(event.observation()),
        outcome(event.outcome()),
        source(event.input_source()),
        scheduler_state(event.scheduler_state()),
        event.current_generation().get(),
    )?;
    write_optional_generation(output, event.captured_generation())?;
    write!(output, "|published=")?;
    write_optional_generation(output, event.published_generation())?;
    write_surface(output, event)?;
    write!(output, "|target=")?;
    write_target(output, event.target())?;
    write!(output, "|frame=")?;
    write_optional_u64(output, event.frame())?;
    write!(output, "|submission=")?;
    write_submission(output, event)?;
    write!(output, "|control=")?;
    write_optional_u64(output, event.control())?;
    write!(output, "|digest=")?;
    write_digest(output, event.staging_digest())?;
    write!(output, "|redraw={}", u8::from(event.redraw_armed()))?;
    let pending = event.pending();
    write!(
        output,
        "|pending={}:{}:{}",
        pending.surface(),
        pending.pointer(),
        pending.presenter()
    )?;
    write_lane(output, "deferred", event.deferred())?;
    write_lane(output, "controls", event.controls())?;
    write_lane(output, "visual", event.visual())?;
    write_lane(output, "in_flight", event.in_flight())?;
    writeln!(
        output,
        "|accounted_bytes={}",
        NativeTraceEventV1::ACCOUNTED_BYTES
    )
}

fn write_surface(output: &mut impl Write, event: NativeTraceEventV1) -> fmt::Result {
    if let Some(surface) = event.surface() {
        return write!(
            output,
            concat!("|native_generation={}|physical={}x{}|logical={}x{}|scale_micros={}"),
            surface.generation().get(),
            surface.physical().width(),
            surface.physical().height(),
            surface.logical_surface().width(),
            surface.logical_surface().height(),
            surface.scale().micros(),
        );
    }
    if let Some(observed) = event.surface_observation() {
        return write!(
            output,
            concat!("|native_generation=-|physical={}x{}|logical={}x{}|scale_micros={}"),
            observed.physical().width(),
            observed.physical().height(),
            observed.logical_surface().width(),
            observed.logical_surface().height(),
            observed.scale().micros(),
        );
    }
    write!(
        output,
        "|native_generation=-|physical=-|logical=-|scale_micros=-"
    )
}

fn write_lane(
    output: &mut impl Write,
    name: &str,
    lane: super::super::super::trace::NativeTraceLaneStatsV1,
) -> fmt::Result {
    write!(
        output,
        "|{name}={}:{}",
        lane.items(),
        lane.accounted_bytes()
    )
}

const fn stage(value: NativeTraceStageV1) -> &'static str {
    match value {
        NativeTraceStageV1::Manifest => "manifest",
        NativeTraceStageV1::Shell => "shell",
        NativeTraceStageV1::Platform => "platform",
        NativeTraceStageV1::Scheduler => "scheduler",
        NativeTraceStageV1::Renderer => "renderer",
        NativeTraceStageV1::Oracle => "oracle",
    }
}

const fn observation(value: NativeObservationV1) -> &'static str {
    match value {
        NativeObservationV1::Build => "build",
        NativeObservationV1::Resumed => "resumed",
        NativeObservationV1::Surface => "surface",
        NativeObservationV1::Scale => "scale",
        NativeObservationV1::Pointer => "pointer",
        NativeObservationV1::Redraw => "redraw",
        NativeObservationV1::Frame => "frame",
        NativeObservationV1::Present => "present",
        NativeObservationV1::Close => "close",
        NativeObservationV1::Completion => "completion",
        NativeObservationV1::Shutdown => "shutdown",
        NativeObservationV1::Timeout => "timeout",
    }
}

const fn outcome(value: NativeOutcomeV1) -> &'static str {
    match value {
        NativeOutcomeV1::Observed => "observed",
        NativeOutcomeV1::Coalesced => "coalesced",
        NativeOutcomeV1::Ignored => "ignored",
        NativeOutcomeV1::Deferred => "deferred",
        NativeOutcomeV1::Published => "published",
        NativeOutcomeV1::Armed => "armed",
        NativeOutcomeV1::Offered => "offered",
        NativeOutcomeV1::Accepted => "accepted",
        NativeOutcomeV1::Rejected => "rejected",
        NativeOutcomeV1::Completed => "completed",
        NativeOutcomeV1::Matched => "matched",
        NativeOutcomeV1::Stopped => "stopped",
        NativeOutcomeV1::Failed(cause) => failed(cause),
    }
}

const fn failed(value: NativeFailureCauseV1) -> &'static str {
    match value {
        NativeFailureCauseV1::InvalidScale => "failed:invalid-scale",
        NativeFailureCauseV1::InvalidPoint => "failed:invalid-point",
        NativeFailureCauseV1::Arithmetic => "failed:arithmetic",
        NativeFailureCauseV1::WidthLimit => "failed:width-limit",
        NativeFailureCauseV1::HeightLimit => "failed:height-limit",
        NativeFailureCauseV1::PixelLimit => "failed:pixel-limit",
        NativeFailureCauseV1::ByteLimit => "failed:byte-limit",
        NativeFailureCauseV1::UnsupportedAlpha => "failed:unsupported-alpha",
        NativeFailureCauseV1::Storage => "failed:storage",
        NativeFailureCauseV1::EnvironmentScaleChanged => "failed:environment-scale-changed",
        NativeFailureCauseV1::EnvironmentSurfaceChanged => "failed:environment-surface-changed",
        NativeFailureCauseV1::SurfaceRepaintUnavailable => "failed:surface-repaint-unavailable",
        NativeFailureCauseV1::Runtime => "failed:runtime",
        NativeFailureCauseV1::Oracle => "failed:oracle",
        NativeFailureCauseV1::Scheduler => "failed:scheduler",
        NativeFailureCauseV1::PrePresent => "failed:pre-present",
        NativeFailureCauseV1::Presenter => "failed:presenter",
        NativeFailureCauseV1::Trace => "failed:trace",
        NativeFailureCauseV1::Timeout => "failed:timeout",
        NativeFailureCauseV1::Invariant => "failed:invariant",
    }
}

const fn source(value: Option<NativeInputSourceV1>) -> &'static str {
    match value {
        Some(NativeInputSourceV1::Native) => "native",
        Some(NativeInputSourceV1::Scripted) => "scripted",
        None => "-",
    }
}

pub(super) const fn scheduler_state(value: SchedulerState) -> &'static str {
    match value {
        SchedulerState::Running => "running",
        SchedulerState::ShutdownQueued => "shutdown-queued",
        SchedulerState::Draining => "draining",
        SchedulerState::Stopped => "stopped",
        SchedulerState::Faulted => "faulted",
    }
}

fn write_target(output: &mut impl Write, value: Option<HeadlessPointerTargetV1>) -> fmt::Result {
    match value {
        Some(HeadlessPointerTargetV1::None) => write!(output, "none"),
        Some(HeadlessPointerTargetV1::StaticControl) => write!(output, "static-control"),
        Some(HeadlessPointerTargetV1::Key(key)) => write!(output, "key:{key}"),
        None => write!(output, "-"),
    }
}

fn write_submission(output: &mut impl Write, event: NativeTraceEventV1) -> fmt::Result {
    match event.submission() {
        Some(value) => write!(output, "{}:{}", value.epoch(), value.token()),
        None => write!(output, "-"),
    }
}

fn write_digest(output: &mut impl Write, value: Option<u64>) -> fmt::Result {
    match value {
        Some(value) => write!(output, "{value:016x}"),
        None => write!(output, "-"),
    }
}

fn write_optional_generation(
    output: &mut impl Write,
    value: Option<fenestra_ui_runtime::prototype::RuntimeGeneration>,
) -> fmt::Result {
    write_optional_u64(output, value.map(|generation| generation.get()))
}

fn write_optional_u64(output: &mut impl Write, value: Option<u64>) -> fmt::Result {
    match value {
        Some(value) => write!(output, "{value}"),
        None => write!(output, "-"),
    }
}

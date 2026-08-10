mod event;

use std::fmt::{self, Write};

use super::super::trace::{NativeFailureCauseV1, NativeTraceV1};
use super::types::{
    NativeArtifactManifestV1, NativeArtifactTerminalV1, NativeOsFamilyV1, NativeProbeResultV1,
    NativeTargetV1, NativeWindowSystemV1,
};
use super::{
    NATIVE_ARTIFACT_MAX_BYTES_V1, NATIVE_ARTIFACT_MAX_EVENTS_V1, NATIVE_ARTIFACT_MAX_LINES_V1,
    NATIVE_ARTIFACT_SCHEMA_REVISION_V1,
};

pub(crate) fn encode_native_artifact_v1(
    manifest: &NativeArtifactManifestV1,
    trace: &NativeTraceV1,
    terminal: &NativeArtifactTerminalV1,
) -> Result<String, NativeFailureCauseV1> {
    validate(manifest, trace, terminal)?;
    let mut measurement = MeasurementV1::new();
    render(&mut measurement, manifest, trace, terminal).map_err(map_format_error)?;
    if measurement.bytes > NATIVE_ARTIFACT_MAX_BYTES_V1
        || measurement.lines > NATIVE_ARTIFACT_MAX_LINES_V1
    {
        return Err(NativeFailureCauseV1::Trace);
    }
    let mut output = String::new();
    output
        .try_reserve_exact(measurement.bytes)
        .map_err(|_| NativeFailureCauseV1::Storage)?;
    render(&mut output, manifest, trace, terminal).map_err(map_format_error)?;
    if output.len() != measurement.bytes || !is_printable_ascii_lf(&output) {
        return Err(NativeFailureCauseV1::Invariant);
    }
    Ok(output)
}

fn validate(
    manifest: &NativeArtifactManifestV1,
    trace: &NativeTraceV1,
    terminal: &NativeArtifactTerminalV1,
) -> Result<(), NativeFailureCauseV1> {
    if !manifest.is_valid()
        || !terminal.is_stopped_and_empty()
        || trace.len() > NATIVE_ARTIFACT_MAX_EVENTS_V1
    {
        return Err(NativeFailureCauseV1::Invariant);
    }
    let accounted = trace
        .len()
        .checked_mul(super::super::trace::NativeTraceEventV1::ACCOUNTED_BYTES)
        .ok_or(NativeFailureCauseV1::Arithmetic)?;
    if accounted != trace.accounted_bytes() {
        return Err(NativeFailureCauseV1::Invariant);
    }
    for (sequence, event) in trace.events().iter().enumerate() {
        let sequence = u64::try_from(sequence).map_err(|_| NativeFailureCauseV1::Arithmetic)?;
        if event.schema_revision() != NATIVE_ARTIFACT_SCHEMA_REVISION_V1
            || event.sequence() != sequence
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
    }
    let Some(last) = trace.events().last() else {
        return Err(NativeFailureCauseV1::Invariant);
    };
    if last.current_generation().get() != terminal.runtime_generation()
        || last.scheduler_state() != terminal.scheduler_state()
        || last.deferred() != terminal.deferred()
        || last.controls() != terminal.controls()
        || last.visual() != terminal.visual()
        || last.in_flight() != terminal.in_flight()
        || last.redraw_armed() != terminal.redraw_armed()
        || last.pending() != terminal.pending()
    {
        return Err(NativeFailureCauseV1::Invariant);
    }
    Ok(())
}

fn render(
    output: &mut impl Write,
    manifest: &NativeArtifactManifestV1,
    trace: &NativeTraceV1,
    terminal: &NativeArtifactTerminalV1,
) -> fmt::Result {
    writeln!(
        output,
        "fenestra-native-artifact|{}",
        NATIVE_ARTIFACT_SCHEMA_REVISION_V1
    )?;
    write_manifest(output, *manifest)?;
    for event in trace.events() {
        event::write_event(output, *event)?;
    }
    write_terminal(output, *terminal)
}

fn write_manifest(output: &mut impl Write, manifest: NativeArtifactManifestV1) -> fmt::Result {
    let surface = manifest.surface();
    let dependency = environment(manifest.os(), manifest.target(), manifest.window_system());
    writeln!(
        output,
        concat!(
            "manifest|os={}|target={}|window={}",
            "|winit=0.30.13|winit_features={}",
            "|softbuffer=0.4.8|softbuffer_features={}",
            "|physical={}x{}|logical={}x{}|scale_micros={}",
            "|requested={}|detected={}|effective={}"
        ),
        dependency.os,
        dependency.target,
        dependency.window,
        dependency.winit_features,
        dependency.softbuffer_features,
        surface.physical().width(),
        surface.physical().height(),
        surface.logical_surface().width(),
        surface.logical_surface().height(),
        surface.scale().micros(),
        manifest.requested().bits(),
        manifest.detected().bits(),
        manifest.effective().bits(),
    )
}

fn write_terminal(output: &mut impl Write, terminal: NativeArtifactTerminalV1) -> fmt::Result {
    writeln!(
        output,
        concat!(
            "terminal|result={}|generation={}|scheduler={}",
            "|deferred={}:{}|controls={}:{}|visual={}:{}|in_flight={}:{}",
            "|redraw={}|pending={}:{}:{}"
        ),
        result(terminal.result()),
        terminal.runtime_generation(),
        event::scheduler_state(terminal.scheduler_state()),
        terminal.deferred().items(),
        terminal.deferred().accounted_bytes(),
        terminal.controls().items(),
        terminal.controls().accounted_bytes(),
        terminal.visual().items(),
        terminal.visual().accounted_bytes(),
        terminal.in_flight().items(),
        terminal.in_flight().accounted_bytes(),
        u8::from(terminal.redraw_armed()),
        terminal.pending().surface(),
        terminal.pending().pointer(),
        terminal.pending().presenter(),
    )
}

struct EnvironmentWordsV1 {
    os: &'static str,
    target: &'static str,
    window: &'static str,
    winit_features: &'static str,
    softbuffer_features: &'static str,
}

const fn environment(
    os: NativeOsFamilyV1,
    target: NativeTargetV1,
    window: NativeWindowSystemV1,
) -> EnvironmentWordsV1 {
    match (os, target, window) {
        (
            NativeOsFamilyV1::Linux,
            NativeTargetV1::X86_64UnknownLinuxGnu,
            NativeWindowSystemV1::Wayland,
        ) => EnvironmentWordsV1 {
            os: "linux",
            target: "x86_64-unknown-linux-gnu",
            window: "wayland",
            winit_features: "rwh_06,wayland,wayland-dlopen",
            softbuffer_features: "wayland,wayland-dlopen",
        },
        (
            NativeOsFamilyV1::Windows,
            NativeTargetV1::X86_64PcWindowsMsvc,
            NativeWindowSystemV1::Win32,
        ) => EnvironmentWordsV1 {
            os: "windows",
            target: "x86_64-pc-windows-msvc",
            window: "win32",
            winit_features: "rwh_06",
            softbuffer_features: "none",
        },
        _ => EnvironmentWordsV1 {
            os: "invalid",
            target: "invalid",
            window: "invalid",
            winit_features: "invalid",
            softbuffer_features: "invalid",
        },
    }
}

const fn result(value: NativeProbeResultV1) -> &'static str {
    match value {
        NativeProbeResultV1::Pass => "pass",
        NativeProbeResultV1::Adapt => "adapt",
        NativeProbeResultV1::Stop => "stop",
    }
}

const fn map_format_error(_: fmt::Error) -> NativeFailureCauseV1 {
    NativeFailureCauseV1::Trace
}

fn is_printable_ascii_lf(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte == b'\n' || (b' '..=b'~').contains(&byte))
        && value.ends_with('\n')
        && !value.contains('\r')
}

struct MeasurementV1 {
    bytes: usize,
    lines: usize,
}

impl MeasurementV1 {
    const fn new() -> Self {
        Self { bytes: 0, lines: 0 }
    }
}

impl Write for MeasurementV1 {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.bytes = self.bytes.checked_add(value.len()).ok_or(fmt::Error)?;
        self.lines = self
            .lines
            .checked_add(value.bytes().filter(|byte| *byte == b'\n').count())
            .ok_or(fmt::Error)?;
        Ok(())
    }
}

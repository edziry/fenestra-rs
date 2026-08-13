mod types;

use std::fmt::Write as _;

use crate::{
    GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1, InteractiveEvidenceV1, InteractiveMilestoneV1,
    InteractiveObservationV1, admit_adapter_v1,
};

use super::{ARTIFACT_LIMITS_V1, InteractiveArtifactErrorKindV1, verify_interactive_artifact_v1};
use crate::GpuAdapterObservationV1;

pub use types::{
    ArtifactAdaptReasonV1, ArtifactAdapterV1, ArtifactEventV1, ArtifactPresentV1,
    ArtifactSurfaceV1, ArtifactTerminalV1, SurfaceAlphaV1, SurfaceFormatV1, SurfacePresentModeV1,
};

/// Bounded typed builder for one release evidence artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveArtifactBuilderV1 {
    target: GpuTargetV1,
    encoded: String,
    evidence: InteractiveEvidenceV1,
    has_adapter: bool,
    has_surface: bool,
}

impl InteractiveArtifactBuilderV1 {
    /// Starts one release artifact with bounded hexadecimal OS version data.
    #[must_use = "artifact initialization failures must be handled"]
    pub fn new(
        target: GpuTargetV1,
        os_version: &[u8],
    ) -> Result<Self, InteractiveArtifactErrorKindV1> {
        let (target_name, rust_target, os) = target_names(target);
        let observed = if os_version.is_empty() {
            b"unknown".as_slice()
        } else {
            os_version
        };
        let mut builder = Self {
            target,
            encoded: String::new(),
            evidence: InteractiveEvidenceV1::new(),
            has_adapter: false,
            has_surface: false,
        };
        builder.push_line("fenestra-windows-gpu|artifact=1|probe=14")?;
        builder.push_line(&format!(
            "run|target={target_name}|rust-target={rust_target}|package={}|profile=release|os={os}|os-version-hex={}",
            env!("CARGO_PKG_VERSION"),
            hexadecimal(observed)
        ))?;
        Ok(builder)
    }

    /// Returns the current encoded record count.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.encoded.bytes().filter(|byte| *byte == b'\n').count()
    }

    /// Records one admitted adapter before any surface or event record.
    pub fn record_adapter(
        &mut self,
        adapter: ArtifactAdapterV1<'_>,
    ) -> Result<(), InteractiveArtifactErrorKindV1> {
        if self.has_adapter || self.has_surface || !self.evidence.milestones().is_empty() {
            return Err(InteractiveArtifactErrorKindV1::Grammar);
        }
        admit_adapter_v1(
            self.target,
            GpuAdapterObservationV1::new(
                adapter.backend,
                adapter.device_type,
                adapter.vendor,
                adapter.device,
            ),
        )
        .map_err(|_| InteractiveArtifactErrorKindV1::Coherence)?;
        self.push_line(&format!(
            "adapter|backend={}|device-type={}|vendor={}|device={}|name-hex={}|driver-hex={}|info-hex={}",
            backend_name(adapter.backend),
            device_type_name(adapter.device_type),
            adapter.vendor,
            adapter.device,
            hexadecimal(adapter.name),
            hexadecimal(adapter.driver),
            hexadecimal(adapter.driver_info)
        ))?;
        self.has_adapter = true;
        Ok(())
    }

    /// Records one selected surface tuple after adapter admission.
    pub fn record_surface(
        &mut self,
        surface: ArtifactSurfaceV1,
    ) -> Result<(), InteractiveArtifactErrorKindV1> {
        if !self.has_adapter || self.has_surface || !self.evidence.milestones().is_empty() {
            return Err(InteractiveArtifactErrorKindV1::Grammar);
        }
        self.push_line(&format!(
            "surface|format={}|present={}|alpha={}",
            format_name(surface.format),
            present_name(surface.present),
            alpha_name(surface.alpha)
        ))?;
        self.has_surface = true;
        Ok(())
    }

    /// Atomically records one ordered interactive event.
    pub fn observe(
        &mut self,
        event: ArtifactEventV1,
    ) -> Result<(), InteractiveArtifactErrorKindV1> {
        if !self.has_adapter || !self.has_surface {
            return Err(InteractiveArtifactErrorKindV1::Protocol);
        }
        let (observation, line) = event_record(event)?;
        self.evidence
            .observe(observation)
            .map_err(|_| InteractiveArtifactErrorKindV1::Protocol)?;
        self.push_line(&line)
    }

    /// Finishes and independently verifies one terminal artifact.
    #[must_use = "artifact encoding failures must be handled"]
    pub fn finish(
        mut self,
        terminal: ArtifactTerminalV1,
    ) -> Result<Vec<u8>, InteractiveArtifactErrorKindV1> {
        let line = match terminal {
            ArtifactTerminalV1::Pass => "result|kind=pass|reason=complete".to_owned(),
            ArtifactTerminalV1::Stop => "result|kind=stop|reason=operator-close".to_owned(),
            ArtifactTerminalV1::Adapt(reason) => {
                format!("result|kind=adapt|reason={}", adapt_name(reason))
            }
        };
        self.push_line(&line)?;
        let bytes = self.encoded.into_bytes();
        verify_interactive_artifact_v1(&bytes)?;
        Ok(bytes)
    }

    fn push_line(&mut self, line: &str) -> Result<(), InteractiveArtifactErrorKindV1> {
        if line.len() > ARTIFACT_LIMITS_V1.line_bytes()
            || self.record_count() >= ARTIFACT_LIMITS_V1.records()
            || self
                .encoded
                .len()
                .checked_add(line.len())
                .and_then(|bytes| bytes.checked_add(1))
                .is_none_or(|bytes| bytes > ARTIFACT_LIMITS_V1.artifact_bytes())
        {
            return Err(InteractiveArtifactErrorKindV1::Bounds);
        }
        self.encoded.push_str(line);
        self.encoded.push('\n');
        Ok(())
    }
}

fn event_record(
    event: ArtifactEventV1,
) -> Result<(InteractiveObservationV1, String), InteractiveArtifactErrorKindV1> {
    let pair = match event {
        ArtifactEventV1::Adapter => (
            InteractiveObservationV1::Adapter,
            "event|milestone=adapter".to_owned(),
        ),
        ArtifactEventV1::PointerMove => (
            InteractiveObservationV1::PointerMove,
            "event|milestone=pointer-move".to_owned(),
        ),
        ArtifactEventV1::PointerPress => (
            InteractiveObservationV1::PointerPress,
            "event|milestone=pointer-press".to_owned(),
        ),
        ArtifactEventV1::Resize(extent) => (
            InteractiveObservationV1::Resize,
            format!(
                "event|milestone=resize|physical={}x{}|logical={}x{}",
                extent.width(),
                extent.height(),
                extent.width(),
                extent.height()
            ),
        ),
        ArtifactEventV1::Suspend => (
            InteractiveObservationV1::Suspend,
            "event|milestone=suspend".to_owned(),
        ),
        ArtifactEventV1::Restore => (
            InteractiveObservationV1::Restore,
            "event|milestone=restore".to_owned(),
        ),
        ArtifactEventV1::Close => (
            InteractiveObservationV1::Close,
            "event|milestone=close".to_owned(),
        ),
        ArtifactEventV1::Present(present) => present_record(present)?,
    };
    Ok(pair)
}

fn present_record(
    present: ArtifactPresentV1,
) -> Result<(InteractiveObservationV1, String), InteractiveArtifactErrorKindV1> {
    let (name, observation) = match present.milestone {
        InteractiveMilestoneV1::InitialPresent => (
            "initial-present",
            InteractiveObservationV1::InitialPresent {
                generation: present.generation,
            },
        ),
        InteractiveMilestoneV1::MutationPresent => (
            "mutation-present",
            InteractiveObservationV1::MutationPresent {
                generation: present.generation,
            },
        ),
        InteractiveMilestoneV1::ResizePresent => (
            "resize-present",
            InteractiveObservationV1::ResizePresent {
                generation: present.generation,
            },
        ),
        InteractiveMilestoneV1::RestorePresent => (
            "restore-present",
            InteractiveObservationV1::RestorePresent {
                generation: present.generation,
            },
        ),
        _ => return Err(InteractiveArtifactErrorKindV1::Grammar),
    };
    let mut line = String::new();
    write!(
        line,
        "event|milestone={name}|generation={}|frame={}|submission={}|physical={}x{}|logical={}x{}|raster={:016x}",
        present.generation,
        present.frame,
        present.submission,
        present.extent.width(),
        present.extent.height(),
        present.extent.width(),
        present.extent.height(),
        present.raster_digest
    )
    .map_err(|_| InteractiveArtifactErrorKindV1::Bounds)?;
    Ok((observation, line))
}

const fn target_names(target: GpuTargetV1) -> (&'static str, &'static str, &'static str) {
    match target {
        GpuTargetV1::WindowsDx12 => ("windows-dx12", "x86_64-pc-windows-msvc", "windows"),
        GpuTargetV1::LinuxVulkan => ("linux-vulkan", "x86_64-unknown-linux-gnu", "linux"),
    }
}

const fn backend_name(backend: GpuBackendV1) -> &'static str {
    match backend {
        GpuBackendV1::Dx12 => "dx12",
        GpuBackendV1::Vulkan => "vulkan",
    }
}

const fn device_type_name(device_type: GpuDeviceTypeV1) -> &'static str {
    match device_type {
        GpuDeviceTypeV1::Other => "other",
        GpuDeviceTypeV1::Integrated => "integrated",
        GpuDeviceTypeV1::Discrete => "discrete",
        GpuDeviceTypeV1::Virtual => "virtual",
        GpuDeviceTypeV1::Cpu => "cpu",
    }
}

const fn format_name(format: SurfaceFormatV1) -> &'static str {
    match format {
        SurfaceFormatV1::Rgba8Unorm => "rgba8unorm",
        SurfaceFormatV1::Bgra8Unorm => "bgra8unorm",
    }
}

const fn present_name(present: SurfacePresentModeV1) -> &'static str {
    match present {
        SurfacePresentModeV1::Fifo => "fifo",
        SurfacePresentModeV1::FifoRelaxed => "fifo-relaxed",
        SurfacePresentModeV1::Immediate => "immediate",
        SurfacePresentModeV1::Mailbox => "mailbox",
    }
}

const fn alpha_name(alpha: SurfaceAlphaV1) -> &'static str {
    match alpha {
        SurfaceAlphaV1::Opaque => "opaque",
        SurfaceAlphaV1::PreMultiplied => "pre-multiplied",
        SurfaceAlphaV1::PostMultiplied => "post-multiplied",
        SurfaceAlphaV1::Inherit => "inherit",
    }
}

const fn adapt_name(reason: ArtifactAdaptReasonV1) -> &'static str {
    match reason {
        ArtifactAdaptReasonV1::AdapterUnavailable => "adapter-unavailable",
        ArtifactAdaptReasonV1::Backend => "backend",
        ArtifactAdaptReasonV1::DeviceType => "device-type",
        ArtifactAdaptReasonV1::Identity => "identity",
        ArtifactAdaptReasonV1::SurfaceFormat => "surface-format",
        ArtifactAdaptReasonV1::DeviceRequest => "device-request",
        ArtifactAdaptReasonV1::Renderer => "renderer",
        ArtifactAdaptReasonV1::Surface => "surface",
        ArtifactAdaptReasonV1::OutOfMemory => "out-of-memory",
        ArtifactAdaptReasonV1::Timeout => "timeout",
    }
}

fn hexadecimal(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

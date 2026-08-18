mod replay;
mod syntax;
mod writer;

use crate::{
    GpuAdapterObservationV1, GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1, InteractiveResultV1,
    admit_adapter_v1,
};

use replay::{ArtifactReplay, terminal_result};
use syntax::{exact_keys, fields, parse_target, parse_u32, require_hex};

pub use writer::{
    ArtifactAdaptReasonV1, ArtifactAdapterV1, ArtifactEventV1, ArtifactPresentV1,
    ArtifactSurfaceV1, ArtifactTerminalV1, InteractiveArtifactBuilderV1, SurfaceAlphaV1,
    SurfaceFormatV1, SurfacePresentModeV1,
};

/// Inclusive bounds for one interactive evidence artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InteractiveArtifactLimitsV1 {
    records: usize,
    line_bytes: usize,
    artifact_bytes: usize,
}

impl InteractiveArtifactLimitsV1 {
    /// Returns the inclusive record count limit.
    #[must_use]
    pub const fn records(self) -> usize {
        self.records
    }

    /// Returns the inclusive bytes-per-line limit, excluding LF.
    #[must_use]
    pub const fn line_bytes(self) -> usize {
        self.line_bytes
    }

    /// Returns the inclusive total artifact byte limit.
    #[must_use]
    pub const fn artifact_bytes(self) -> usize {
        self.artifact_bytes
    }
}

/// Registered inclusive evidence artifact limits.
pub const ARTIFACT_LIMITS_V1: InteractiveArtifactLimitsV1 = InteractiveArtifactLimitsV1 {
    records: 256,
    line_bytes: 512,
    artifact_bytes: 65_536,
};

/// Closed verifier failures for interactive evidence artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveArtifactErrorKindV1 {
    /// A registered record, line, or total-byte bound was exceeded.
    Bounds,
    /// Bytes were not printable ASCII with one final LF.
    Encoding,
    /// A forbidden privacy-sensitive key was present.
    Redaction,
    /// The record grammar or one closed value was invalid.
    Grammar,
    /// Target, platform, adapter, or surface facts were incoherent.
    Coherence,
    /// Milestones or presentation facts violated the interaction protocol.
    Protocol,
    /// The declared result did not match the replayed terminal state.
    Terminal,
}

/// Verified summary of one bounded interactive artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedInteractiveArtifactV1 {
    result: InteractiveResultV1,
    record_count: usize,
    byte_count: usize,
    last_generation: Option<u64>,
}

impl VerifiedInteractiveArtifactV1 {
    /// Returns the verified terminal classification.
    #[must_use]
    pub const fn result(self) -> InteractiveResultV1 {
        self.result
    }

    /// Returns the verified record count.
    #[must_use]
    pub const fn record_count(self) -> usize {
        self.record_count
    }

    /// Returns the verified total byte count.
    #[must_use]
    pub const fn byte_count(self) -> usize {
        self.byte_count
    }

    /// Returns the last successfully presented runtime generation.
    #[must_use]
    pub const fn last_generation(self) -> Option<u64> {
        self.last_generation
    }
}

/// Verifies grammar, bounds, privacy, coherence, protocol, and terminal state.
#[must_use = "artifact verification failures must be handled"]
pub fn verify_interactive_artifact_v1(
    bytes: &[u8],
) -> Result<VerifiedInteractiveArtifactV1, InteractiveArtifactErrorKindV1> {
    let lines = bounded_lines(bytes)?;
    reject_private_keys(bytes)?;
    if lines.first().copied() != Some("fenestra-windows-gpu|artifact=1|probe=14") {
        return Err(InteractiveArtifactErrorKindV1::Grammar);
    }
    let run = fields(lines.get(1).copied(), "run")?;
    exact_keys(
        &run,
        &[
            "target",
            "rust-target",
            "package",
            "profile",
            "os",
            "os-version-hex",
        ],
    )?;
    let target = parse_target(run[0].1)?;
    verify_run(target, &run)?;

    let mut cursor = 2;
    let mut has_adapter = false;
    if lines
        .get(cursor)
        .is_some_and(|line| line.starts_with("adapter|"))
    {
        verify_adapter(target, &fields(lines.get(cursor).copied(), "adapter")?)?;
        has_adapter = true;
        cursor += 1;
    }
    let mut has_surface = false;
    if lines
        .get(cursor)
        .is_some_and(|line| line.starts_with("surface|"))
    {
        verify_surface(&fields(lines.get(cursor).copied(), "surface")?)?;
        has_surface = true;
        cursor += 1;
    }

    let mut replay = ArtifactReplay::new();
    while let Some(line) = lines.get(cursor) {
        if line.starts_with("result|") {
            break;
        }
        replay.event(&fields(Some(line), "event")?)?;
        cursor += 1;
    }
    let result = fields(lines.get(cursor).copied(), "result")?;
    cursor += 1;
    if cursor != lines.len() {
        return Err(InteractiveArtifactErrorKindV1::Terminal);
    }
    exact_keys(&result, &["kind", "reason"])?;
    let declared = terminal_result(&result, &replay, has_adapter, has_surface)?;
    Ok(VerifiedInteractiveArtifactV1 {
        result: declared,
        record_count: lines.len(),
        byte_count: bytes.len(),
        last_generation: replay.last_generation,
    })
}

fn bounded_lines(bytes: &[u8]) -> Result<Vec<&str>, InteractiveArtifactErrorKindV1> {
    if bytes.len() > ARTIFACT_LIMITS_V1.artifact_bytes {
        return Err(InteractiveArtifactErrorKindV1::Bounds);
    }
    let record_count = bytes.iter().filter(|byte| **byte == b'\n').count();
    if record_count > ARTIFACT_LIMITS_V1.records {
        return Err(InteractiveArtifactErrorKindV1::Bounds);
    }
    if bytes
        .split(|byte| *byte == b'\n')
        .any(|line| line.len() > ARTIFACT_LIMITS_V1.line_bytes)
    {
        return Err(InteractiveArtifactErrorKindV1::Bounds);
    }
    if bytes.is_empty()
        || !bytes.ends_with(b"\n")
        || bytes.ends_with(b"\n\n")
        || bytes
            .iter()
            .any(|byte| *byte != b'\n' && !(b' '..=b'~').contains(byte))
    {
        return Err(InteractiveArtifactErrorKindV1::Encoding);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| InteractiveArtifactErrorKindV1::Encoding)?;
    Ok(text.lines().collect())
}

fn reject_private_keys(bytes: &[u8]) -> Result<(), InteractiveArtifactErrorKindV1> {
    const FORBIDDEN: [&[u8]; 10] = [
        b"|host=",
        b"|hostname=",
        b"|user=",
        b"|home=",
        b"|path=",
        b"|pid=",
        b"|handle=",
        b"|title=",
        b"|pointer-x=",
        b"|pointer-y=",
    ];
    if FORBIDDEN
        .iter()
        .any(|needle| bytes.windows(needle.len()).any(|window| window == *needle))
    {
        return Err(InteractiveArtifactErrorKindV1::Redaction);
    }
    Ok(())
}

fn verify_run(
    target: GpuTargetV1,
    run: &[(&str, &str)],
) -> Result<(), InteractiveArtifactErrorKindV1> {
    let coherent = match target {
        GpuTargetV1::WindowsDx12 => run[1].1 == "x86_64-pc-windows-msvc" && run[4].1 == "windows",
        GpuTargetV1::LinuxVulkan => run[1].1 == "x86_64-unknown-linux-gnu" && run[4].1 == "linux",
    };
    if !coherent || run[2].1 != env!("CARGO_PKG_VERSION") || run[3].1 != "release" {
        return Err(InteractiveArtifactErrorKindV1::Coherence);
    }
    require_hex(run[5].1)
}

fn verify_adapter(
    target: GpuTargetV1,
    adapter: &[(&str, &str)],
) -> Result<(), InteractiveArtifactErrorKindV1> {
    exact_keys(
        adapter,
        &[
            "backend",
            "device-type",
            "vendor",
            "device",
            "name-hex",
            "driver-hex",
            "info-hex",
        ],
    )?;
    let backend = match adapter[0].1 {
        "dx12" => GpuBackendV1::Dx12,
        "vulkan" => GpuBackendV1::Vulkan,
        _ => return Err(InteractiveArtifactErrorKindV1::Grammar),
    };
    let device_type = match adapter[1].1 {
        "other" => GpuDeviceTypeV1::Other,
        "integrated" => GpuDeviceTypeV1::Integrated,
        "discrete" => GpuDeviceTypeV1::Discrete,
        "virtual" => GpuDeviceTypeV1::Virtual,
        "cpu" => GpuDeviceTypeV1::Cpu,
        _ => return Err(InteractiveArtifactErrorKindV1::Grammar),
    };
    let vendor = parse_u32(adapter[2].1)?;
    let device = parse_u32(adapter[3].1)?;
    for value in &adapter[4..=6] {
        require_hex(value.1)?;
    }
    admit_adapter_v1(
        target,
        GpuAdapterObservationV1::new(backend, device_type, vendor, device),
    )
    .map_err(|_| InteractiveArtifactErrorKindV1::Coherence)
}

fn verify_surface(surface: &[(&str, &str)]) -> Result<(), InteractiveArtifactErrorKindV1> {
    exact_keys(surface, &["format", "present", "alpha"])?;
    if !matches!(surface[0].1, "rgba8unorm" | "bgra8unorm")
        || !matches!(
            surface[1].1,
            "fifo" | "fifo-relaxed" | "immediate" | "mailbox"
        )
        || !matches!(
            surface[2].1,
            "opaque" | "pre-multiplied" | "post-multiplied" | "inherit"
        )
    {
        return Err(InteractiveArtifactErrorKindV1::Coherence);
    }
    Ok(())
}

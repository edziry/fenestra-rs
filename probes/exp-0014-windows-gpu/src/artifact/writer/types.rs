use crate::{GpuBackendV1, GpuDeviceTypeV1, GpuSurfaceExtentV1, InteractiveMilestoneV1};

/// Borrowed adapter facts encoded by the artifact writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactAdapterV1<'a> {
    pub(super) backend: GpuBackendV1,
    pub(super) device_type: GpuDeviceTypeV1,
    pub(super) vendor: u32,
    pub(super) device: u32,
    pub(super) name: &'a [u8],
    pub(super) driver: &'a [u8],
    pub(super) driver_info: &'a [u8],
}

impl<'a> ArtifactAdapterV1<'a> {
    /// Creates one borrowed adapter environment record.
    #[must_use]
    pub const fn new(
        backend: GpuBackendV1,
        device_type: GpuDeviceTypeV1,
        vendor: u32,
        device: u32,
        name: &'a [u8],
        driver: &'a [u8],
        driver_info: &'a [u8],
    ) -> Self {
        Self {
            backend,
            device_type,
            vendor,
            device,
            name,
            driver,
            driver_info,
        }
    }
}

/// Closed native surface formats admitted by the artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceFormatV1 {
    /// Linear RGBA8.
    Rgba8Unorm,
    /// Linear BGRA8.
    Bgra8Unorm,
}

/// Closed native present modes admitted by the artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfacePresentModeV1 {
    /// FIFO vertical synchronization.
    Fifo,
    /// FIFO with relaxed synchronization.
    FifoRelaxed,
    /// Immediate presentation.
    Immediate,
    /// Mailbox presentation.
    Mailbox,
}

/// Closed native composite alpha modes admitted by the artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceAlphaV1 {
    /// Opaque composition.
    Opaque,
    /// Premultiplied alpha composition.
    PreMultiplied,
    /// Postmultiplied alpha composition.
    PostMultiplied,
    /// Platform-inherited composition.
    Inherit,
}

/// Selected surface capability tuple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactSurfaceV1 {
    pub(super) format: SurfaceFormatV1,
    pub(super) present: SurfacePresentModeV1,
    pub(super) alpha: SurfaceAlphaV1,
}

impl ArtifactSurfaceV1 {
    /// Creates one selected surface capability tuple.
    #[must_use]
    pub const fn new(
        format: SurfaceFormatV1,
        present: SurfacePresentModeV1,
        alpha: SurfaceAlphaV1,
    ) -> Self {
        Self {
            format,
            present,
            alpha,
        }
    }
}

/// Correlated facts for one completed presentation milestone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactPresentV1 {
    pub(super) milestone: InteractiveMilestoneV1,
    pub(super) generation: u64,
    pub(super) frame: u64,
    pub(super) submission: u64,
    pub(super) extent: GpuSurfaceExtentV1,
    pub(super) raster_digest: u64,
}

impl ArtifactPresentV1 {
    /// Creates one completed presentation record.
    #[must_use]
    pub const fn new(
        milestone: InteractiveMilestoneV1,
        generation: u64,
        frame: u64,
        submission: u64,
        extent: GpuSurfaceExtentV1,
        raster_digest: u64,
    ) -> Self {
        Self {
            milestone,
            generation,
            frame,
            submission,
            extent,
            raster_digest,
        }
    }
}

/// Typed event records accepted by the artifact writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactEventV1 {
    /// A compatible hardware adapter was admitted.
    Adapter,
    /// One GPU presentation completed.
    Present(ArtifactPresentV1),
    /// Native pointer movement was observed.
    PointerMove,
    /// Native primary-button press was observed.
    PointerPress,
    /// A distinct nonzero surface resize was observed.
    Resize(GpuSurfaceExtentV1),
    /// Presentation became absent.
    Suspend,
    /// Presentation became available after absence.
    Restore,
    /// The native window closed normally.
    Close,
}

/// Closed environment adaptation reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactAdaptReasonV1 {
    /// No compatible adapter was available.
    AdapterUnavailable,
    /// The effective backend was wrong.
    Backend,
    /// The adapter device class was not hardware.
    DeviceType,
    /// Stable adapter identity was absent.
    Identity,
    /// No admitted surface format was available.
    SurfaceFormat,
    /// Device creation failed.
    DeviceRequest,
    /// Vello renderer creation or execution failed.
    Renderer,
    /// Surface acquisition or presentation failed.
    Surface,
    /// The GPU reported out of memory.
    OutOfMemory,
    /// GPU completion exceeded the bounded wait.
    Timeout,
}

/// Terminal record requested from the typed writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactTerminalV1 {
    /// The complete operator protocol passed.
    Pass,
    /// The operator closed before completing the protocol.
    Stop,
    /// A closed environment limitation prevented the protocol.
    Adapt(ArtifactAdaptReasonV1),
}

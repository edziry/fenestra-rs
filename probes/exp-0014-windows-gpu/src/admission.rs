/// Closed native GPU backends admitted by the probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuBackendV1 {
    /// Direct3D 12 on Windows.
    Dx12,
    /// Vulkan on the Linux developer control.
    Vulkan,
}

/// Closed device classes observed by the candidate adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuDeviceTypeV1 {
    /// A device without a more specific hardware classification.
    Other,
    /// An integrated hardware GPU.
    Integrated,
    /// A discrete hardware GPU.
    Discrete,
    /// A virtual or hosted GPU.
    Virtual,
    /// A CPU or software adapter.
    Cpu,
}

/// Exact platform/backend tuple requested by one probe run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuTargetV1 {
    /// Windows with the DX12 backend.
    WindowsDx12,
    /// Linux with the Vulkan backend.
    LinuxVulkan,
}

impl GpuTargetV1 {
    const fn backend(self) -> GpuBackendV1 {
        match self {
            Self::WindowsDx12 => GpuBackendV1::Dx12,
            Self::LinuxVulkan => GpuBackendV1::Vulkan,
        }
    }
}

/// Candidate-neutral adapter facts required for target admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuAdapterObservationV1 {
    backend: GpuBackendV1,
    device_type: GpuDeviceTypeV1,
    vendor: u32,
    device: u32,
}

impl GpuAdapterObservationV1 {
    /// Creates one bounded adapter observation.
    #[must_use]
    pub const fn new(
        backend: GpuBackendV1,
        device_type: GpuDeviceTypeV1,
        vendor: u32,
        device: u32,
    ) -> Self {
        Self {
            backend,
            device_type,
            vendor,
            device,
        }
    }
}

/// Closed reasons an observed adapter cannot enter a target lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuAdmissionErrorKindV1 {
    /// The effective backend differs from the registered target backend.
    Backend,
    /// The adapter is not classified as an integrated or discrete GPU.
    DeviceType,
    /// The adapter does not expose stable nonzero vendor and device identities.
    Identity,
}

/// Applies the closed target admission policy in stable priority order.
#[must_use = "GPU admission failures must be handled"]
pub fn admit_adapter_v1(
    target: GpuTargetV1,
    adapter: GpuAdapterObservationV1,
) -> Result<(), GpuAdmissionErrorKindV1> {
    if adapter.backend != target.backend() {
        return Err(GpuAdmissionErrorKindV1::Backend);
    }
    if !matches!(
        adapter.device_type,
        GpuDeviceTypeV1::Integrated | GpuDeviceTypeV1::Discrete
    ) {
        return Err(GpuAdmissionErrorKindV1::DeviceType);
    }
    if adapter.vendor == 0 || adapter.device == 0 {
        return Err(GpuAdmissionErrorKindV1::Identity);
    }
    Ok(())
}

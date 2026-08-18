use crate::{
    ArtifactAdaptReasonV1, ArtifactAdapterV1, ArtifactSurfaceV1, GpuBackendV1, GpuDeviceTypeV1,
    GpuTargetV1, SurfaceAlphaV1, SurfaceFormatV1, SurfacePresentModeV1,
};

pub(in crate::native) struct GpuEnvironmentV1 {
    pub(in crate::native) backend: GpuBackendV1,
    pub(in crate::native) device_type: GpuDeviceTypeV1,
    pub(in crate::native) vendor: u32,
    pub(in crate::native) device: u32,
    pub(in crate::native) name: String,
    pub(in crate::native) driver: String,
    pub(in crate::native) driver_info: String,
    pub(in crate::native) surface: ArtifactSurfaceV1,
}

impl GpuEnvironmentV1 {
    pub(in crate::native) fn artifact_adapter(&self) -> ArtifactAdapterV1<'_> {
        ArtifactAdapterV1::new(
            self.backend,
            self.device_type,
            self.vendor,
            self.device,
            self.name.as_bytes(),
            self.driver.as_bytes(),
            self.driver_info.as_bytes(),
        )
    }
}

pub(super) const fn target_backends(target: GpuTargetV1) -> wgpu::Backends {
    match target {
        GpuTargetV1::WindowsDx12 => wgpu::Backends::DX12,
        GpuTargetV1::LinuxVulkan => wgpu::Backends::VULKAN,
    }
}

pub(super) const fn map_backend(backend: wgpu::Backend) -> Option<GpuBackendV1> {
    match backend {
        wgpu::Backend::Dx12 => Some(GpuBackendV1::Dx12),
        wgpu::Backend::Vulkan => Some(GpuBackendV1::Vulkan),
        _ => None,
    }
}

pub(super) const fn map_device_type(device_type: wgpu::DeviceType) -> GpuDeviceTypeV1 {
    match device_type {
        wgpu::DeviceType::Other => GpuDeviceTypeV1::Other,
        wgpu::DeviceType::IntegratedGpu => GpuDeviceTypeV1::Integrated,
        wgpu::DeviceType::DiscreteGpu => GpuDeviceTypeV1::Discrete,
        wgpu::DeviceType::VirtualGpu => GpuDeviceTypeV1::Virtual,
        wgpu::DeviceType::Cpu => GpuDeviceTypeV1::Cpu,
    }
}

pub(super) fn map_admission(error: crate::GpuAdmissionErrorKindV1) -> ArtifactAdaptReasonV1 {
    match error {
        crate::GpuAdmissionErrorKindV1::Backend => ArtifactAdaptReasonV1::Backend,
        crate::GpuAdmissionErrorKindV1::DeviceType => ArtifactAdaptReasonV1::DeviceType,
        crate::GpuAdmissionErrorKindV1::Identity => ArtifactAdaptReasonV1::Identity,
    }
}

pub(super) fn select_format(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::TextureFormat, SurfaceFormatV1), ArtifactAdaptReasonV1> {
    for (format, artifact) in [
        (wgpu::TextureFormat::Bgra8Unorm, SurfaceFormatV1::Bgra8Unorm),
        (wgpu::TextureFormat::Rgba8Unorm, SurfaceFormatV1::Rgba8Unorm),
    ] {
        if capabilities.formats.contains(&format) {
            return Ok((format, artifact));
        }
    }
    Err(ArtifactAdaptReasonV1::SurfaceFormat)
}

pub(super) fn select_present(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::PresentMode, SurfacePresentModeV1), ArtifactAdaptReasonV1> {
    capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
        .then_some((wgpu::PresentMode::Fifo, SurfacePresentModeV1::Fifo))
        .ok_or(ArtifactAdaptReasonV1::Surface)
}

pub(super) fn select_alpha(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::CompositeAlphaMode, SurfaceAlphaV1), ArtifactAdaptReasonV1> {
    capabilities
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::Opaque)
        .then_some((wgpu::CompositeAlphaMode::Opaque, SurfaceAlphaV1::Opaque))
        .ok_or(ArtifactAdaptReasonV1::Surface)
}

pub(super) fn bounded_adapter_identity(value: &str) -> String {
    const MAX_BYTES: usize = 48;

    if value.len() <= MAX_BYTES {
        return value.to_owned();
    }
    let mut end = MAX_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::bounded_adapter_identity;

    #[test]
    fn adapter_identity_is_bounded_without_splitting_utf8() {
        let exact = "a".repeat(48);
        assert_eq!(bounded_adapter_identity(&exact), exact);
        assert_eq!(bounded_adapter_identity(&"b".repeat(49)), "b".repeat(48));

        let multibyte = format!("{}\u{e9}x", "a".repeat(47));
        assert_eq!(bounded_adapter_identity(&multibyte), "a".repeat(47));
    }
}

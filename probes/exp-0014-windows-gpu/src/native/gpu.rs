use std::sync::Arc;
use std::time::Duration;

use vello::peniko::Color;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions};
use wgpu::util::TextureBlitter;
use winit::window::Window;

use crate::scene::prepare_vello_scene_v1;
use crate::{
    ArtifactAdaptReasonV1, ArtifactAdapterV1, ArtifactSurfaceV1, GpuAdapterObservationV1,
    GpuBackendV1, GpuDeviceTypeV1, GpuPortReceiptV1, GpuPresentErrorKindV1, GpuPresentPortV1,
    GpuSurfaceExtentV1, GpuTargetV1, SurfaceAlphaV1, SurfaceFormatV1, SurfacePresentModeV1,
    admit_adapter_v1,
};

const GPU_WAIT: Duration = Duration::from_secs(10);

pub(super) struct GpuEnvironmentV1 {
    pub(super) backend: GpuBackendV1,
    pub(super) device_type: GpuDeviceTypeV1,
    pub(super) vendor: u32,
    pub(super) device: u32,
    pub(super) name: String,
    pub(super) driver: String,
    pub(super) driver_info: String,
    pub(super) surface: ArtifactSurfaceV1,
}

impl GpuEnvironmentV1 {
    pub(super) fn artifact_adapter(&self) -> ArtifactAdapterV1<'_> {
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

pub(super) struct NativeGpuV1 {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    target_texture: wgpu::Texture,
    target_view: wgpu::TextureView,
    blitter: TextureBlitter,
    renderer: Renderer,
}

impl NativeGpuV1 {
    pub(super) fn new(
        window: Arc<Window>,
        target: GpuTargetV1,
        extent: GpuSurfaceExtentV1,
    ) -> Result<(Self, GpuEnvironmentV1), ArtifactAdaptReasonV1> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: target_backends(target),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });
        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|_| ArtifactAdaptReasonV1::Surface)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .map_err(|_| ArtifactAdaptReasonV1::AdapterUnavailable)?;
        let info = adapter.get_info();
        let backend = map_backend(info.backend).ok_or(ArtifactAdaptReasonV1::Backend)?;
        let device_type = map_device_type(info.device_type);
        admit_adapter_v1(
            target,
            GpuAdapterObservationV1::new(backend, device_type, info.vendor, info.device),
        )
        .map_err(map_admission)?;

        let capabilities = surface.get_capabilities(&adapter);
        let (format, artifact_format) = select_format(&capabilities)?;
        let (present_mode, artifact_present) = select_present(&capabilities)?;
        let (alpha_mode, artifact_alpha) = select_alpha(&capabilities)?;
        let available_features = adapter.features();
        let required_features =
            available_features & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("fenestra-exp-0014-device"),
            required_features,
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }))
        .map_err(|_| ArtifactAdaptReasonV1::DeviceRequest)?;
        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|_| ArtifactAdaptReasonV1::Renderer)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: extent.width(),
            height: extent.height(),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: Vec::new(),
        };
        surface.configure(&device, &config);
        let (target_texture, target_view) = create_target(&device, extent);
        let blitter = TextureBlitter::new(&device, format);
        let environment = GpuEnvironmentV1 {
            backend,
            device_type,
            vendor: info.vendor,
            device: info.device,
            name: bounded_adapter_identity(&info.name),
            driver: bounded_adapter_identity(&info.driver),
            driver_info: bounded_adapter_identity(&info.driver_info),
            surface: ArtifactSurfaceV1::new(artifact_format, artifact_present, artifact_alpha),
        };
        Ok((
            Self {
                instance,
                surface,
                window,
                device,
                queue,
                config,
                target_texture,
                target_view,
                blitter,
                renderer,
            },
            environment,
        ))
    }

    pub(super) fn resize(&mut self, extent: GpuSurfaceExtentV1) {
        if extent.width() == 0 || extent.height() == 0 {
            return;
        }
        self.config.width = extent.width();
        self.config.height = extent.height();
        self.surface.configure(&self.device, &self.config);
        (self.target_texture, self.target_view) = create_target(&self.device, extent);
    }

    fn acquire(&mut self) -> Result<wgpu::SurfaceTexture, GpuPresentErrorKindV1> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(texture),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                self.acquire_once()
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self
                    .instance
                    .create_surface(Arc::clone(&self.window))
                    .map_err(|_| GpuPresentErrorKindV1::Surface)?;
                self.surface.configure(&self.device, &self.config);
                self.acquire_once()
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                Err(GpuPresentErrorKindV1::Timeout)
            }
            wgpu::CurrentSurfaceTexture::Validation => Err(GpuPresentErrorKindV1::Surface),
        }
    }

    fn acquire_once(&self) -> Result<wgpu::SurfaceTexture, GpuPresentErrorKindV1> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(texture),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                Err(GpuPresentErrorKindV1::Timeout)
            }
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => Err(GpuPresentErrorKindV1::Surface),
        }
    }
}

impl GpuPresentPortV1 for NativeGpuV1 {
    fn present<A>(
        &mut self,
        frame: fenestra_ui_runtime::prototype::RuntimePaintFrameV2<'_>,
        extent: GpuSurfaceExtentV1,
        accept_once: A,
    ) -> Result<GpuPortReceiptV1, GpuPresentErrorKindV1>
    where
        A: FnOnce() -> Result<fenestra_ui_runtime::prototype::SubmissionId, GpuPresentErrorKindV1>,
    {
        if self.config.width != extent.width() || self.config.height != extent.height() {
            self.resize(extent);
        }
        let prepared =
            prepare_vello_scene_v1(frame.spatial()).map_err(|_| GpuPresentErrorKindV1::Raster)?;
        if prepared.width != extent.width() || prepared.height != extent.height() {
            return Err(GpuPresentErrorKindV1::Viewport);
        }
        let surface_texture = self.acquire()?;
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.window.pre_present_notify();
        let _accepted = accept_once()?;
        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &prepared.scene,
                &self.target_view,
                &RenderParams {
                    base_color: Color::from_rgba8(0, 0, 0, 0),
                    width: extent.width(),
                    height: extent.height(),
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|_| GpuPresentErrorKindV1::Renderer)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fenestra-exp-0014-blit"),
            });
        self.blitter
            .copy(&self.device, &mut encoder, &self.target_view, &surface_view);
        let submitted = self.queue.submit([encoder.finish()]);
        surface_texture.present();
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submitted),
                timeout: Some(GPU_WAIT),
            })
            .map_err(|error| match error {
                wgpu::PollError::Timeout => GpuPresentErrorKindV1::Timeout,
                wgpu::PollError::WrongSubmissionIndex(_, _) => GpuPresentErrorKindV1::Surface,
            })?;
        Ok(GpuPortReceiptV1::new(prepared.raster_digest))
    }
}

fn create_target(
    device: &wgpu::Device,
    extent: GpuSurfaceExtentV1,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fenestra-exp-0014-vello-target"),
        size: wgpu::Extent3d {
            width: extent.width(),
            height: extent.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

const fn target_backends(target: GpuTargetV1) -> wgpu::Backends {
    match target {
        GpuTargetV1::WindowsDx12 => wgpu::Backends::DX12,
        GpuTargetV1::LinuxVulkan => wgpu::Backends::VULKAN,
    }
}

const fn map_backend(backend: wgpu::Backend) -> Option<GpuBackendV1> {
    match backend {
        wgpu::Backend::Dx12 => Some(GpuBackendV1::Dx12),
        wgpu::Backend::Vulkan => Some(GpuBackendV1::Vulkan),
        _ => None,
    }
}

const fn map_device_type(device_type: wgpu::DeviceType) -> GpuDeviceTypeV1 {
    match device_type {
        wgpu::DeviceType::Other => GpuDeviceTypeV1::Other,
        wgpu::DeviceType::IntegratedGpu => GpuDeviceTypeV1::Integrated,
        wgpu::DeviceType::DiscreteGpu => GpuDeviceTypeV1::Discrete,
        wgpu::DeviceType::VirtualGpu => GpuDeviceTypeV1::Virtual,
        wgpu::DeviceType::Cpu => GpuDeviceTypeV1::Cpu,
    }
}

fn map_admission(error: crate::GpuAdmissionErrorKindV1) -> ArtifactAdaptReasonV1 {
    match error {
        crate::GpuAdmissionErrorKindV1::Backend => ArtifactAdaptReasonV1::Backend,
        crate::GpuAdmissionErrorKindV1::DeviceType => ArtifactAdaptReasonV1::DeviceType,
        crate::GpuAdmissionErrorKindV1::Identity => ArtifactAdaptReasonV1::Identity,
    }
}

fn select_format(
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

fn select_present(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::PresentMode, SurfacePresentModeV1), ArtifactAdaptReasonV1> {
    capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
        .then_some((wgpu::PresentMode::Fifo, SurfacePresentModeV1::Fifo))
        .ok_or(ArtifactAdaptReasonV1::Surface)
}

fn select_alpha(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(wgpu::CompositeAlphaMode, SurfaceAlphaV1), ArtifactAdaptReasonV1> {
    capabilities
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::Opaque)
        .then_some((wgpu::CompositeAlphaMode::Opaque, SurfaceAlphaV1::Opaque))
        .ok_or(ArtifactAdaptReasonV1::Surface)
}

fn bounded_adapter_identity(value: &str) -> String {
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

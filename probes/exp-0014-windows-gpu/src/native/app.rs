use std::sync::Arc;

use fenestra_ui_runtime::prototype::{
    QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerInput, SchedulerInputResult,
    SchedulerTick, UiScheduler,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use super::gpu::NativeGpuV1;
use crate::{
    ArtifactAdaptReasonV1, ArtifactEventV1, ArtifactPresentV1, ArtifactTerminalV1,
    GpuPresentErrorKindV1, GpuPresentationOutcomeV1, GpuSurfaceExtentV1,
    InteractiveArtifactBuilderV1, InteractiveMilestoneV1, InteractiveProbeErrorKindV1,
    build_registered_runtime_v1, present_gpu_offer_v1,
};

mod interaction;

pub(super) struct NativeGpuApplicationV1 {
    builder: Option<InteractiveArtifactBuilderV1>,
    window: Option<Arc<Window>>,
    gpu: Option<NativeGpuV1>,
    scheduler: Option<UiScheduler>,
    runtime_extent: Option<GpuSurfaceExtentV1>,
    last_present_extent: Option<GpuSurfaceExtentV1>,
    suspended: bool,
    redraw_armed: bool,
    next_tick: u64,
    output: Option<Vec<u8>>,
    failure: Option<InteractiveProbeErrorKindV1>,
}

impl NativeGpuApplicationV1 {
    pub(super) const fn new(builder: InteractiveArtifactBuilderV1) -> Self {
        Self {
            builder: Some(builder),
            window: None,
            gpu: None,
            scheduler: None,
            runtime_extent: None,
            last_present_extent: None,
            suspended: false,
            redraw_armed: false,
            next_tick: 0,
            output: None,
            failure: None,
        }
    }

    pub(super) fn into_output(self) -> Result<Vec<u8>, InteractiveProbeErrorKindV1> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.output.ok_or(InteractiveProbeErrorKindV1::Artifact)
    }

    fn initialize(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), InteractiveProbeErrorKindV1> {
        if self.window.is_some() {
            return Ok(());
        }
        let attributes = Window::default_attributes()
            .with_title("Fenestra GPU: initializing")
            .with_inner_size(LogicalSize::new(640_u32, 420_u32))
            .with_transparent(false);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|_| InteractiveProbeErrorKindV1::Window)?,
        );
        let physical = window.inner_size();
        let extent = GpuSurfaceExtentV1::new(physical.width, physical.height);
        if extent.width() == 0 || extent.height() == 0 {
            self.window = Some(window);
            self.finish_adapt(event_loop, ArtifactAdaptReasonV1::Surface);
            return Ok(());
        }
        let (gpu, environment) =
            match NativeGpuV1::new(Arc::clone(&window), super::target(), extent) {
                Ok(ready) => ready,
                Err(reason) => {
                    self.window = Some(window);
                    self.finish_adapt(event_loop, reason);
                    return Ok(());
                }
            };
        let builder = self
            .builder
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Artifact)?;
        builder
            .record_adapter(environment.artifact_adapter())
            .and_then(|()| builder.record_surface(environment.surface))
            .and_then(|()| builder.observe(ArtifactEventV1::Adapter))
            .map_err(|_| InteractiveProbeErrorKindV1::Artifact)?;
        let runtime = build_registered_runtime_v1(viewport(extent))
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        let mut scheduler = UiScheduler::new(runtime, scheduler_capacity())
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        let tick = self.take_tick()?;
        scheduler
            .request_current_frame(tick)
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        self.window = Some(window);
        self.gpu = Some(gpu);
        self.scheduler = Some(scheduler);
        self.runtime_extent = Some(extent);
        self.arm_next_action(tick)?;
        Ok(())
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if !self.redraw_armed || self.suspended {
            return;
        }
        self.redraw_armed = false;
        if let Err(error) = self.redraw_inner() {
            match error {
                RedrawFailureV1::Adapt(reason) => self.finish_adapt(event_loop, reason),
                RedrawFailureV1::Internal => {
                    self.abort(event_loop, InteractiveProbeErrorKindV1::Runtime)
                }
            }
        }
    }

    fn redraw_inner(&mut self) -> Result<(), RedrawFailureV1> {
        let tick = self.take_tick().map_err(|_| RedrawFailureV1::Internal)?;
        let scheduler = self.scheduler.as_mut().ok_or(RedrawFailureV1::Internal)?;
        if scheduler
            .process_input(SchedulerInput::FrameReady, tick)
            .map_err(|_| RedrawFailureV1::Internal)?
            != SchedulerInputResult::FrameReady
        {
            return Err(RedrawFailureV1::Internal);
        }
        let Some(SchedulerAction::OfferFrame(work)) = scheduler
            .next_action(tick)
            .map_err(|_| RedrawFailureV1::Internal)?
        else {
            return Err(RedrawFailureV1::Internal);
        };
        let extent = self.runtime_extent.ok_or(RedrawFailureV1::Internal)?;
        let outcome = present_gpu_offer_v1(
            scheduler,
            &work,
            extent,
            self.gpu.as_mut().ok_or(RedrawFailureV1::Internal)?,
            tick,
        )
        .map_err(|error| map_present_error(error.kind()))?;
        let GpuPresentationOutcomeV1::Completed(receipt) = outcome else {
            return Err(RedrawFailureV1::Internal);
        };
        if scheduler
            .next_action(tick)
            .map_err(|_| RedrawFailureV1::Internal)?
            .is_some()
        {
            return Err(RedrawFailureV1::Internal);
        }
        let milestone = match self.next_required() {
            Some(
                milestone @ (InteractiveMilestoneV1::InitialPresent
                | InteractiveMilestoneV1::MutationPresent
                | InteractiveMilestoneV1::ResizePresent
                | InteractiveMilestoneV1::RestorePresent),
            ) => milestone,
            _ => return Err(RedrawFailureV1::Internal),
        };
        self.builder
            .as_mut()
            .ok_or(RedrawFailureV1::Internal)?
            .observe(ArtifactEventV1::Present(ArtifactPresentV1::new(
                milestone,
                receipt.generation(),
                receipt.frame(),
                receipt.submission(),
                extent,
                receipt.raster_digest(),
            )))
            .map_err(|_| RedrawFailureV1::Internal)?;
        self.last_present_extent = Some(extent);
        self.update_title();
        Ok(())
    }

    fn arm_next_action(&mut self, tick: SchedulerTick) -> Result<(), InteractiveProbeErrorKindV1> {
        match self
            .scheduler
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?
            .next_action(tick)
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?
        {
            Some(SchedulerAction::RequestFrame) => {
                self.redraw_armed = true;
                self.window
                    .as_ref()
                    .ok_or(InteractiveProbeErrorKindV1::Window)?
                    .request_redraw();
                Ok(())
            }
            None if self.redraw_armed => Ok(()),
            _ => Err(InteractiveProbeErrorKindV1::Runtime),
        }
    }

    fn take_tick(&mut self) -> Result<SchedulerTick, InteractiveProbeErrorKindV1> {
        let tick = self.next_tick;
        self.next_tick = tick
            .checked_add(1)
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?;
        Ok(SchedulerTick::new(tick))
    }

    fn observe(&mut self, event: ArtifactEventV1) -> Result<(), InteractiveProbeErrorKindV1> {
        self.builder
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Artifact)?
            .observe(event)
            .map_err(|_| InteractiveProbeErrorKindV1::Artifact)
    }

    fn next_required(&self) -> Option<InteractiveMilestoneV1> {
        self.builder
            .as_ref()
            .and_then(InteractiveArtifactBuilderV1::next_required)
    }

    fn update_title(&self) {
        let title = match self.next_required() {
            Some(InteractiveMilestoneV1::InitialPresent) => "Fenestra GPU: presenting",
            Some(InteractiveMilestoneV1::PointerMove) => "Fenestra GPU: move the pointer",
            Some(InteractiveMilestoneV1::PointerPress) => "Fenestra GPU: press the primary button",
            Some(InteractiveMilestoneV1::MutationPresent) => "Fenestra GPU: presenting mutation",
            Some(InteractiveMilestoneV1::Resize) => "Fenestra GPU: resize the window",
            Some(InteractiveMilestoneV1::ResizePresent) => "Fenestra GPU: presenting resize",
            Some(InteractiveMilestoneV1::Suspend) => "Fenestra GPU: minimize the window",
            Some(InteractiveMilestoneV1::Restore) => "Fenestra GPU: restore the window",
            Some(InteractiveMilestoneV1::RestorePresent) => "Fenestra GPU: presenting restore",
            Some(InteractiveMilestoneV1::Close) => "Fenestra GPU: close the window",
            Some(InteractiveMilestoneV1::Adapter) | None => "Fenestra GPU",
        };
        if let Some(window) = &self.window {
            window.set_title(title);
        }
    }

    fn finish_adapt(&mut self, event_loop: &ActiveEventLoop, reason: ArtifactAdaptReasonV1) {
        self.finish(event_loop, ArtifactTerminalV1::Adapt(reason));
    }

    fn finish(&mut self, event_loop: &ActiveEventLoop, terminal: ArtifactTerminalV1) {
        let Some(builder) = self.builder.take() else {
            self.abort(event_loop, InteractiveProbeErrorKindV1::Artifact);
            return;
        };
        match builder.finish(terminal) {
            Ok(bytes) => self.output = Some(bytes),
            Err(_) => self.failure = Some(InteractiveProbeErrorKindV1::Artifact),
        }
        self.scheduler.take();
        self.gpu.take();
        self.window.take();
        event_loop.exit();
    }

    fn abort(&mut self, event_loop: &ActiveEventLoop, error: InteractiveProbeErrorKindV1) {
        self.failure.get_or_insert(error);
        self.scheduler.take();
        self.gpu.take();
        self.window.take();
        event_loop.exit();
    }
}

impl ApplicationHandler for NativeGpuApplicationV1 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.abort(event_loop, error);
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if self.suspend().is_err() {
            self.abort(event_loop, InteractiveProbeErrorKindV1::Runtime);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        let result = match event {
            WindowEvent::RedrawRequested => {
                self.redraw(event_loop);
                return;
            }
            WindowEvent::CursorMoved { .. } => self.pointer_move(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.pointer_press(),
            WindowEvent::Resized(size) => {
                self.resized(event_loop, GpuSurfaceExtentV1::new(size.width, size.height));
                return;
            }
            WindowEvent::Occluded(true) => self.suspend(),
            WindowEvent::Occluded(false) if self.suspended => {
                let Some(window) = &self.window else { return };
                let size = window.inner_size();
                self.restore(GpuSurfaceExtentV1::new(size.width, size.height))
            }
            WindowEvent::CloseRequested => {
                self.close(event_loop);
                return;
            }
            _ => Ok(()),
        };
        if result.is_err() {
            self.abort(event_loop, InteractiveProbeErrorKindV1::Runtime);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RedrawFailureV1 {
    Adapt(ArtifactAdaptReasonV1),
    Internal,
}

fn map_present_error(kind: GpuPresentErrorKindV1) -> RedrawFailureV1 {
    match kind {
        GpuPresentErrorKindV1::Raster | GpuPresentErrorKindV1::Renderer => {
            RedrawFailureV1::Adapt(ArtifactAdaptReasonV1::Renderer)
        }
        GpuPresentErrorKindV1::Acquire
        | GpuPresentErrorKindV1::Present
        | GpuPresentErrorKindV1::Surface => RedrawFailureV1::Adapt(ArtifactAdaptReasonV1::Surface),
        GpuPresentErrorKindV1::Timeout => RedrawFailureV1::Adapt(ArtifactAdaptReasonV1::Timeout),
        GpuPresentErrorKindV1::OutOfMemory => {
            RedrawFailureV1::Adapt(ArtifactAdaptReasonV1::OutOfMemory)
        }
        GpuPresentErrorKindV1::Viewport
        | GpuPresentErrorKindV1::Scheduler
        | GpuPresentErrorKindV1::Invariant => RedrawFailureV1::Internal,
    }
}

fn viewport(extent: GpuSurfaceExtentV1) -> SpatialViewportV2 {
    SpatialViewportV2::new(
        i32::try_from(extent.width()).unwrap_or(i32::MAX),
        i32::try_from(extent.height()).unwrap_or(i32::MAX),
    )
}

const fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 64),
        QueueCapacity::new(4, 128, 64),
        QueueCapacity::new(1, 40, 64),
        QueueCapacity::new(2, 80, 64),
    )
}

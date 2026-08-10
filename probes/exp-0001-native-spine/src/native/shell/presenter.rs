use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Buffer, Context, Surface};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use super::super::driver::PresenterPortV1;
use super::super::raster::CpuFrameV1;
use super::super::trace::NativeFailureCauseV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativePresenterBackendErrorV1 {
    OperationFailed,
}

pub(crate) trait NativePresenterSurfacePortV1 {
    type Buffer<'a>: NativePresenterBufferPortV1
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), NativePresenterBackendErrorV1>;

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, NativePresenterBackendErrorV1>;
}

pub(crate) trait NativePresenterBufferPortV1 {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), NativePresenterBackendErrorV1>;
    fn pre_present_notify(&mut self) -> Result<(), NativePresenterBackendErrorV1>;
    fn present(self) -> Result<(), NativePresenterBackendErrorV1>;
}

pub(crate) struct NativeOwnedSurfacePortV1 {
    surface: Surface<OwnedDisplayHandle, Arc<Window>>,
    _context: Context<OwnedDisplayHandle>,
    window: Arc<Window>,
}

pub(crate) struct NativeSoftbufferPresenterV1<S = NativeOwnedSurfacePortV1> {
    surface: S,
}

impl NativeSoftbufferPresenterV1 {
    pub(crate) fn from_owned_parts(
        context: Context<OwnedDisplayHandle>,
        surface: Surface<OwnedDisplayHandle, Arc<Window>>,
        window: Arc<Window>,
    ) -> Self {
        Self {
            surface: NativeOwnedSurfacePortV1 {
                surface,
                _context: context,
                window,
            },
        }
    }
}

impl<S> NativeSoftbufferPresenterV1<S> {
    #[cfg(test)]
    pub(crate) const fn from_surface_port_for_test(surface: S) -> Self {
        Self { surface }
    }
}

impl<S: NativePresenterSurfacePortV1> PresenterPortV1 for NativeSoftbufferPresenterV1<S> {
    fn present_offer<A>(
        &mut self,
        frame: CpuFrameV1,
        accept_once: A,
    ) -> Result<(), NativeFailureCauseV1>
    where
        A: FnOnce() -> Result<fenestra_ui_runtime::prototype::SubmissionId, NativeFailureCauseV1>,
    {
        let physical = frame.surface_tuple().physical();
        let width = NonZeroU32::new(physical.width()).ok_or(NativeFailureCauseV1::Invariant)?;
        let height = NonZeroU32::new(physical.height()).ok_or(NativeFailureCauseV1::Invariant)?;
        self.surface
            .resize(width, height)
            .map_err(|_| NativeFailureCauseV1::Presenter)?;
        let mut buffer = self
            .surface
            .acquire()
            .map_err(|_| NativeFailureCauseV1::Presenter)?;
        buffer
            .copy_pixels(frame.pixels())
            .map_err(|_| NativeFailureCauseV1::Presenter)?;
        buffer
            .pre_present_notify()
            .map_err(|_| NativeFailureCauseV1::PrePresent)?;
        accept_once()?;
        buffer
            .present()
            .map_err(|_| NativeFailureCauseV1::Presenter)
    }
}

pub(crate) struct NativeOwnedBufferV1<'a> {
    buffer: Buffer<'a, OwnedDisplayHandle, Arc<Window>>,
    window: &'a Window,
}

impl NativePresenterSurfacePortV1 for NativeOwnedSurfacePortV1 {
    type Buffer<'a>
        = NativeOwnedBufferV1<'a>
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), NativePresenterBackendErrorV1> {
        self.surface
            .resize(width, height)
            .map_err(|_| NativePresenterBackendErrorV1::OperationFailed)
    }

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, NativePresenterBackendErrorV1> {
        let window = self.window.as_ref();
        let buffer = self
            .surface
            .buffer_mut()
            .map_err(|_| NativePresenterBackendErrorV1::OperationFailed)?;
        Ok(NativeOwnedBufferV1 { buffer, window })
    }
}

impl NativePresenterBufferPortV1 for NativeOwnedBufferV1<'_> {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), NativePresenterBackendErrorV1> {
        if self.buffer.len() != pixels.len() {
            return Err(NativePresenterBackendErrorV1::OperationFailed);
        }
        self.buffer.copy_from_slice(pixels);
        Ok(())
    }

    fn pre_present_notify(&mut self) -> Result<(), NativePresenterBackendErrorV1> {
        self.window.pre_present_notify();
        Ok(())
    }

    fn present(self) -> Result<(), NativePresenterBackendErrorV1> {
        self.buffer
            .present()
            .map_err(|_| NativePresenterBackendErrorV1::OperationFailed)
    }
}

pub(crate) const fn native_pre_present_notify_source_v1() -> fn(&Window) {
    Window::pre_present_notify
}

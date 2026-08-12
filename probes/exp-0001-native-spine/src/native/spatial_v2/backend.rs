use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Buffer, Context, Surface};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpatialPresenterBackendErrorV2 {
    OperationFailed,
}

pub(crate) trait SpatialPresenterSurfacePortV2 {
    type Buffer<'a>: SpatialPresenterBufferPortV2
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), SpatialPresenterBackendErrorV2>;

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, SpatialPresenterBackendErrorV2>;
}

pub(crate) trait SpatialPresenterBufferPortV2 {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), SpatialPresenterBackendErrorV2>;
    fn pre_present_notify(&mut self) -> Result<(), SpatialPresenterBackendErrorV2>;
    fn present(self) -> Result<(), SpatialPresenterBackendErrorV2>;
}

pub(crate) struct SpatialOwnedSurfacePortV2 {
    surface: Surface<OwnedDisplayHandle, Arc<Window>>,
    _context: Context<OwnedDisplayHandle>,
    window: Arc<Window>,
}

impl SpatialOwnedSurfacePortV2 {
    pub(crate) fn new(
        context: Context<OwnedDisplayHandle>,
        surface: Surface<OwnedDisplayHandle, Arc<Window>>,
        window: Arc<Window>,
    ) -> Self {
        Self {
            surface,
            _context: context,
            window,
        }
    }
}

pub(crate) struct SpatialOwnedBufferV2<'a> {
    buffer: Buffer<'a, OwnedDisplayHandle, Arc<Window>>,
    window: &'a Window,
}

impl SpatialPresenterSurfacePortV2 for SpatialOwnedSurfacePortV2 {
    type Buffer<'a>
        = SpatialOwnedBufferV2<'a>
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.surface
            .resize(width, height)
            .map_err(|_| SpatialPresenterBackendErrorV2::OperationFailed)
    }

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, SpatialPresenterBackendErrorV2> {
        let window = self.window.as_ref();
        let buffer = self
            .surface
            .buffer_mut()
            .map_err(|_| SpatialPresenterBackendErrorV2::OperationFailed)?;
        Ok(SpatialOwnedBufferV2 { buffer, window })
    }
}

impl SpatialPresenterBufferPortV2 for SpatialOwnedBufferV2<'_> {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), SpatialPresenterBackendErrorV2> {
        if self.buffer.len() != pixels.len() {
            return Err(SpatialPresenterBackendErrorV2::OperationFailed);
        }
        self.buffer.copy_from_slice(pixels);
        Ok(())
    }

    fn pre_present_notify(&mut self) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.window.pre_present_notify();
        Ok(())
    }

    fn present(self) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.buffer
            .present()
            .map_err(|_| SpatialPresenterBackendErrorV2::OperationFailed)
    }
}

#[cfg(test)]
pub(crate) const fn spatial_pre_present_notify_source_v2() -> fn(&Window) {
    Window::pre_present_notify
}

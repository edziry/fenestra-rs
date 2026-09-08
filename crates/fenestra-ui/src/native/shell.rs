use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent as PlatformEvent;
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowId};

use super::input::{application_event, requests_redraw};
use super::presentation::copy_pixels;
use super::{NativeError, WindowContent, WindowOptions};
use crate::Size;

type NativeContext = Context<OwnedDisplayHandle>;
type NativeSurface = Surface<OwnedDisplayHandle, Arc<Window>>;

#[derive(Debug)]
pub(super) struct EventOutcome {
    pub(super) redraw: bool,
    pub(super) close: bool,
}

pub(super) struct NativeApplication<'a, C: WindowContent> {
    pub(super) content: &'a mut C,
    options: WindowOptions,
    pub(super) presented: bool,
    pub(super) drawable: bool,
    pub(super) size: Option<Size>,
    window: Option<Arc<Window>>,
    _context: Option<NativeContext>,
    surface: Option<NativeSurface>,
    pub(super) failure: Option<NativeError<C::Error>>,
}

impl<'a, C: WindowContent> NativeApplication<'a, C> {
    pub(super) fn new(content: &'a mut C, options: WindowOptions) -> Self {
        Self {
            content,
            options,
            presented: false,
            drawable: false,
            size: None,
            window: None,
            _context: None,
            surface: None,
            failure: None,
        }
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeError<C::Error>> {
        if self.window.is_some() {
            return Ok(());
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(&self.options.title)
                        .with_inner_size(LogicalSize::new(
                            self.options.size.width(),
                            self.options.size.height(),
                        ))
                        .with_transparent(false),
                )
                .map_err(|_| NativeError::Window)?,
        );
        let physical = window.inner_size();
        self.resize_content(physical.width, physical.height)?;
        let context =
            Context::new(event_loop.owned_display_handle()).map_err(|_| NativeError::Presenter)?;
        let surface =
            Surface::new(&context, Arc::clone(&window)).map_err(|_| NativeError::Presenter)?;
        self._context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.request_redraw();
        Ok(())
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window
            && self.drawable
        {
            window.request_redraw();
        }
    }

    pub(super) fn resize_content(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), NativeError<C::Error>> {
        if width == 0 || height == 0 {
            self.drawable = false;
            return Ok(());
        }
        self.content
            .resize(width, height)
            .map_err(NativeError::Application)?;
        self.size = Some(Size::new(width, height));
        self.drawable = true;
        Ok(())
    }

    fn redraw(&mut self) -> Result<(), NativeError<C::Error>> {
        if !self.drawable {
            return Ok(());
        }
        let raster = self.content.frame().map_err(NativeError::Application)?;
        if Some(raster.size()) != self.size {
            return Err(NativeError::Presenter);
        }
        let width = NonZeroU32::new(raster.size().width()).ok_or(NativeError::Presenter)?;
        let height = NonZeroU32::new(raster.size().height()).ok_or(NativeError::Presenter)?;
        let surface = self.surface.as_mut().ok_or(NativeError::Presenter)?;
        surface
            .resize(width, height)
            .map_err(|_| NativeError::Presenter)?;
        let mut buffer = surface.buffer_mut().map_err(|_| NativeError::Presenter)?;
        copy_pixels(&raster, &mut buffer).map_err(|_| NativeError::Presenter)?;
        let window = self.window.as_ref().ok_or(NativeError::Window)?;
        window.pre_present_notify();
        buffer.present().map_err(|_| NativeError::Presenter)?;
        self.presented = true;
        self.content.presented().map_err(NativeError::Application)
    }

    pub(super) fn process_event(
        &mut self,
        event: PlatformEvent,
    ) -> Result<EventOutcome, NativeError<C::Error>> {
        let redraw = requests_redraw(&event);
        let close = matches!(event, PlatformEvent::CloseRequested);
        match event {
            PlatformEvent::RedrawRequested => self.redraw()?,
            PlatformEvent::Resized(size) => self.resize_content(size.width, size.height)?,
            _ => {
                if let Some(event) =
                    application_event(&event).map_err(|_| NativeError::Presenter)?
                {
                    self.content
                        .event(event)
                        .map_err(NativeError::Application)?;
                }
            }
        }
        Ok(EventOutcome { redraw, close })
    }

    fn abort(&mut self, event_loop: &ActiveEventLoop, error: NativeError<C::Error>) {
        self.failure.get_or_insert(error);
        event_loop.exit();
    }
}

impl<C: WindowContent> ApplicationHandler for NativeApplication<'_, C> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.abort(event_loop, error);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: PlatformEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        match self.process_event(event) {
            Ok(outcome) => {
                if outcome.close {
                    event_loop.exit();
                } else if outcome.redraw {
                    self.request_redraw();
                }
            }
            Err(error) => self.abort(event_loop, error),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.options.smoke && self.presented {
            event_loop.exit();
        }
    }
}

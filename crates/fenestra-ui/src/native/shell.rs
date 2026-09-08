use std::num::NonZeroU32;
use std::sync::Arc;

use accesskit_winit::{Event as AccessibilityEvent, WindowEvent as AccessibilityEventKind};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent as PlatformEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy, OwnedDisplayHandle};
use winit::window::{Window, WindowId};

use super::accessibility::{Accessibility, action_kind, action_request};
use super::ime::{ImeBridge, WindowImeSink};
use super::input::{InputState, requests_redraw};
use super::presentation::copy_pixels;
use super::{NativeError, WindowContent, WindowOptions};
use crate::{AccessibilityAction, Size};

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
    accessibility: Option<Accessibility>,
    pub(super) event_loop_proxy: Option<EventLoopProxy<AccessibilityEvent>>,
    window: Option<Arc<Window>>,
    _context: Option<NativeContext>,
    surface: Option<NativeSurface>,
    input: InputState,
    ime: ImeBridge,
    window_focused: bool,
    pub(super) failure: Option<NativeError<C::Error>>,
}

impl<'a, C: WindowContent> NativeApplication<'a, C> {
    pub(super) fn new(content: &'a mut C, options: WindowOptions) -> Self {
        Self {
            content,
            ime: ImeBridge::new(options.ime_allowed),
            options,
            presented: false,
            drawable: false,
            size: None,
            accessibility: None,
            event_loop_proxy: None,
            window: None,
            _context: None,
            surface: None,
            input: InputState::default(),
            window_focused: true,
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
                        .with_visible(false)
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
        let tree = self
            .content
            .accessibility()
            .map_err(NativeError::Application)?;
        let context =
            Context::new(event_loop.owned_display_handle()).map_err(|_| NativeError::Presenter)?;
        let surface =
            Surface::new(&context, Arc::clone(&window)).map_err(|_| NativeError::Presenter)?;
        if tree.is_some() {
            let proxy = self
                .event_loop_proxy
                .as_ref()
                .ok_or(NativeError::EventLoop)?;
            let mut accessibility = Accessibility::new(event_loop, &window, proxy.clone());
            accessibility.update(
                tree,
                &self.options.title,
                Size::new(physical.width, physical.height),
            );
            self.accessibility = Some(accessibility);
        }
        self._context = Some(context);
        self.surface = Some(surface);
        let focused = window.has_focus();
        self.window = Some(window);
        self.refresh_ime(Some(focused), false)?;
        if let Some(window) = &self.window {
            window.set_visible(true);
        }
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
            return self.refresh_window_state(None, false);
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
        self.presentation_completed()
    }

    pub(super) fn presentation_completed(&mut self) -> Result<(), NativeError<C::Error>> {
        self.presented = true;
        self.content.presented().map_err(NativeError::Application)?;
        self.refresh_window_state(None, false)
    }

    pub(super) fn process_event(
        &mut self,
        event: PlatformEvent,
    ) -> Result<EventOutcome, NativeError<C::Error>> {
        let redraw = requests_redraw(&event);
        let close = matches!(event, PlatformEvent::CloseRequested);
        let focused = match &event {
            PlatformEvent::Focused(focused) => Some(*focused),
            _ => None,
        };
        let scale_changed = matches!(event, PlatformEvent::ScaleFactorChanged { .. });
        match event {
            PlatformEvent::RedrawRequested => {
                self.redraw()?;
                return Ok(EventOutcome { redraw, close });
            }
            PlatformEvent::Resized(size) => self.resize_content(size.width, size.height)?,
            _ => {
                let events = self
                    .input
                    .application_events(&event)
                    .map_err(|_| NativeError::Presenter)?;
                for event in events {
                    self.content
                        .event(event)
                        .map_err(NativeError::Application)?;
                }
            }
        }
        self.refresh_window_state(focused, scale_changed)?;
        Ok(EventOutcome { redraw, close })
    }

    fn refresh_window_state(
        &mut self,
        focused: Option<bool>,
        force_area: bool,
    ) -> Result<(), NativeError<C::Error>> {
        self.refresh_accessibility(false)?;
        self.refresh_ime(focused, force_area)
    }

    fn refresh_ime(
        &mut self,
        focused: Option<bool>,
        force_area: bool,
    ) -> Result<(), NativeError<C::Error>> {
        let focused = focused.unwrap_or(self.window_focused);
        let desired = self.content.ime_context();
        if let Some(window) = &self.window {
            self.ime
                .refresh(
                    desired,
                    self.drawable,
                    focused,
                    force_area,
                    &mut WindowImeSink(window),
                )
                .map_err(NativeError::Application)?;
        } else {
            desired.map_err(NativeError::Application)?;
        }
        self.window_focused = focused;
        Ok(())
    }

    fn refresh_accessibility(&mut self, force: bool) -> Result<(), NativeError<C::Error>> {
        let Some(accessibility) = &mut self.accessibility else {
            return Ok(());
        };
        let tree = self
            .content
            .accessibility()
            .map_err(NativeError::Application)?;
        if force {
            accessibility.invalidate();
        }
        accessibility.update(
            tree,
            &self.options.title,
            self.size.unwrap_or(self.options.size),
        );
        Ok(())
    }

    pub(super) fn accessibility_event(
        &mut self,
        event: AccessibilityEventKind,
    ) -> Result<bool, NativeError<C::Error>> {
        match event {
            AccessibilityEventKind::InitialTreeRequested => self.refresh_accessibility(true)?,
            AccessibilityEventKind::AccessibilityDeactivated => {
                if let Some(accessibility) = &mut self.accessibility {
                    accessibility.invalidate();
                }
            }
            AccessibilityEventKind::ActionRequested(request) => {
                if action_kind(&request).is_none() {
                    return Ok(false);
                }
                let tree = self
                    .content
                    .accessibility()
                    .map_err(NativeError::Application)?;
                let Some(request) = tree
                    .as_ref()
                    .and_then(|tree| action_request(tree, &request))
                else {
                    return Ok(false);
                };
                self.content
                    .accessibility_action(request)
                    .map_err(NativeError::Application)?;
                if request.action == AccessibilityAction::Focus
                    && let Some(window) = &self.window
                {
                    window.focus_window();
                }
                self.refresh_window_state(None, false)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn abort(&mut self, event_loop: &ActiveEventLoop, error: NativeError<C::Error>) {
        self.failure.get_or_insert(error);
        event_loop.exit();
    }

    pub(super) fn should_exit(&self) -> bool {
        self.content.should_close() || (self.options.smoke && self.presented)
    }
}

impl<C: WindowContent> ApplicationHandler<AccessibilityEvent> for NativeApplication<'_, C> {
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
        if let (Some(accessibility), Some(window)) = (&mut self.accessibility, &self.window) {
            accessibility.process_event(window, &event);
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AccessibilityEvent) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != event.window_id)
        {
            return;
        }
        match self.accessibility_event(event.window_event) {
            Ok(true) => self.request_redraw(),
            Ok(false) => (),
            Err(error) => self.abort(event_loop, error),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_exit() {
            event_loop.exit();
        }
    }
}

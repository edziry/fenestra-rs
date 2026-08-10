use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use super::super::driver::NativeDriverV1;
use super::super::shell::mapping::{NativeShellInputV1, map_window_event_v1};
use super::super::shell::presenter::NativeSoftbufferPresenterV1;
use super::super::shell::script::{NativeReferenceScriptV1, NativeRunDirectiveV1};
use super::super::shell::watchdog::{NativeWatchdogTokenV1, NativeWatchdogV1};
use super::{NativeUserEventV1, WinitWatchdogProxyV1};
use crate::NativeProbeErrorV1;

pub(super) type NativePresenterV1 = NativeSoftbufferPresenterV1;
pub(super) type NativeDriverStateV1 = NativeDriverV1<NativePresenterV1>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WatchdogExpectationV1 {
    Redraw,
    PresentSettled,
    Resize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ActiveWatchdogV1 {
    pub(super) token: NativeWatchdogTokenV1,
    pub(super) expectation: WatchdogExpectationV1,
}

pub(super) const fn surface_preempts_redraw_v1(
    pending_surface: bool,
    expectation: Option<WatchdogExpectationV1>,
) -> bool {
    pending_surface
        && matches!(
            expectation,
            Some(WatchdogExpectationV1::Redraw | WatchdogExpectationV1::PresentSettled)
        )
}

pub(super) const fn surface_preemption_watchdog_v1(
    pending_surface: bool,
    expectation: Option<WatchdogExpectationV1>,
) -> Option<WatchdogExpectationV1> {
    if surface_preempts_redraw_v1(pending_surface, expectation) {
        expectation
    } else {
        None
    }
}

pub(super) const fn surface_preempts_directive_v1(
    pending_surface: bool,
    directive_slot_empty: bool,
) -> bool {
    pending_surface && !directive_slot_empty
}

pub(super) fn requested_surface_matches_v1(
    directive: NativeRunDirectiveV1,
    surface: fenestra_ui_runtime::prototype::HeadlessSurface,
) -> bool {
    let NativeRunDirectiveV1::RequestLogicalResize { width, height } = directive else {
        return false;
    };
    i32::try_from(width) == Ok(surface.width()) && i32::try_from(height) == Ok(surface.height())
}

pub(super) fn active_resize_refuses_surface_v1(
    expectation: Option<WatchdogExpectationV1>,
    directive: NativeRunDirectiveV1,
    surface: Option<fenestra_ui_runtime::prototype::HeadlessSurface>,
) -> bool {
    expectation == Some(WatchdogExpectationV1::Resize)
        && surface.is_some_and(|surface| !requested_surface_matches_v1(directive, surface))
}

#[derive(Clone, Copy, Default)]
pub(super) struct ObservedCapabilitiesV1 {
    pub(super) window: bool,
    pub(super) surface: bool,
    pub(super) pointer: bool,
    pub(super) resize: bool,
    pub(super) shutdown: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum NativeDirectiveSlotStateV1 {
    #[default]
    Empty,
    Waiting(NativeRunDirectiveV1),
    Ready(NativeRunDirectiveV1),
}

#[derive(Default)]
pub(super) struct NativeDirectiveSlotV1(NativeDirectiveSlotStateV1);

impl NativeDirectiveSlotV1 {
    pub(super) fn defer_until_barrier(
        &mut self,
        directive: NativeRunDirectiveV1,
    ) -> Result<(), super::super::trace::NativeFailureCauseV1> {
        if self.0 != NativeDirectiveSlotStateV1::Empty {
            return Err(super::super::trace::NativeFailureCauseV1::Invariant);
        }
        self.0 = NativeDirectiveSlotStateV1::Waiting(directive);
        Ok(())
    }

    pub(super) fn release_barrier(
        &mut self,
    ) -> Result<(), super::super::trace::NativeFailureCauseV1> {
        let NativeDirectiveSlotStateV1::Waiting(directive) = self.0 else {
            return Err(super::super::trace::NativeFailureCauseV1::Invariant);
        };
        self.0 = NativeDirectiveSlotStateV1::Ready(directive);
        Ok(())
    }

    pub(super) fn take_ready(&mut self) -> Option<NativeRunDirectiveV1> {
        let NativeDirectiveSlotStateV1::Ready(directive) = self.0 else {
            return None;
        };
        self.0 = NativeDirectiveSlotStateV1::Empty;
        Some(directive)
    }

    pub(super) fn clear(&mut self) {
        self.0 = NativeDirectiveSlotStateV1::Empty;
    }

    pub(super) const fn is_empty(&self) -> bool {
        matches!(self.0, NativeDirectiveSlotStateV1::Empty)
    }

    #[cfg(test)]
    pub(super) const fn pending_count(&self) -> usize {
        if self.is_empty() { 0 } else { 1 }
    }
}

pub(super) struct NativeApplicationV1 {
    pub(super) driver: Option<NativeDriverStateV1>,
    pub(super) window: Option<Arc<Window>>,
    pub(super) watchdog: NativeWatchdogV1<WinitWatchdogProxyV1>,
    pub(super) active_watchdog: Option<ActiveWatchdogV1>,
    pub(super) script: NativeReferenceScriptV1,
    pub(super) pending_script: NativeDirectiveSlotV1,
    pub(super) initial_surface: Option<super::super::surface::NativeSurfaceTupleV1>,
    pub(super) capabilities: ObservedCapabilitiesV1,
    pub(super) surface_dirty: bool,
    pub(super) next_tick: u64,
    pub(super) result: Option<super::super::artifact::NativeProbeResultV1>,
    pub(super) output: Option<Vec<u8>>,
    pub(super) failure: Option<NativeProbeErrorV1>,
}

impl NativeApplicationV1 {
    pub(super) fn new(watchdog: NativeWatchdogV1<WinitWatchdogProxyV1>) -> Self {
        Self {
            driver: None,
            window: None,
            watchdog,
            active_watchdog: None,
            script: NativeReferenceScriptV1::new(),
            pending_script: NativeDirectiveSlotV1::default(),
            initial_surface: None,
            capabilities: ObservedCapabilitiesV1::default(),
            surface_dirty: false,
            next_tick: 0,
            result: None,
            output: None,
            failure: None,
        }
    }

    pub(super) fn into_output(mut self) -> Result<Vec<u8>, NativeProbeErrorV1> {
        self.watchdog
            .shutdown_and_join()
            .map_err(super::map_watchdog_error)?;
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.output.take().ok_or(NativeProbeErrorV1::Artifact)
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeProbeErrorV1> {
        if self.driver.is_some() {
            return Ok(());
        }
        let attributes = Window::default_attributes()
            .with_title("Fenestra EXP-0001 native spine")
            .with_inner_size(LogicalSize::new(320_u32, 240_u32))
            .with_transparent(false);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|_| NativeProbeErrorV1::Window)?,
        );
        let context = Context::new(event_loop.owned_display_handle())
            .map_err(|_| NativeProbeErrorV1::Presenter)?;
        let surface = Surface::new(&context, Arc::clone(&window))
            .map_err(|_| NativeProbeErrorV1::Presenter)?;
        let presenter =
            NativeSoftbufferPresenterV1::from_owned_parts(context, surface, Arc::clone(&window));
        let driver = NativeDriverV1::new(presenter).map_err(|_| NativeProbeErrorV1::Driver)?;
        self.capabilities.window = true;
        self.driver = Some(driver);
        self.window = Some(window);
        self.observe_resume()
    }

    fn map_event(&self, window_id: WindowId, event: &WindowEvent) -> Option<NativeShellInputV1> {
        let window = self.window.as_ref()?;
        map_window_event_v1(
            window.id(),
            window_id,
            event,
            window.inner_size(),
            window.scale_factor(),
        )
    }
}

impl ApplicationHandler<NativeUserEventV1> for NativeApplicationV1 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.abort(event_loop, error);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: NativeUserEventV1) {
        match event {
            NativeUserEventV1::Timeout(token) => self.handle_timeout(event_loop, token),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(input) = self.map_event(window_id, &event) else {
            return;
        };
        self.handle_window_input(event_loop, input);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface_dirty {
            self.drain_surface(event_loop);
        }
        if self.failure.is_none() && self.output.is_none() {
            self.run_deferred_script(event_loop);
        }
    }
}

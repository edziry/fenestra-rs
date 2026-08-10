use winit::event_loop::ActiveEventLoop;

use super::super::driver::NativeDriverActionV1;
use super::super::shell::mapping::NativeShellInputV1;
use super::super::shell::script::{NativeRunDirectiveV1, NativeRunEvidenceV1};
use super::super::shell::watchdog::NativeWatchdogTokenV1;
use super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::super::types::NativePhysicalExtentV1;
use super::app::{
    NativeApplicationV1, WatchdogExpectationV1, active_resize_refuses_surface_v1,
    surface_preempts_directive_v1, surface_preempts_redraw_v1,
};
use crate::NativeProbeErrorV1;

impl NativeApplicationV1 {
    pub(super) fn observe_resume(&mut self) -> Result<(), NativeProbeErrorV1> {
        let tick = self.take_tick()?;
        let (physical, scale) = {
            let window = self.window.as_ref().ok_or(NativeProbeErrorV1::Window)?;
            (window.inner_size(), window.scale_factor())
        };
        let driver = self.driver.as_mut().ok_or(NativeProbeErrorV1::Driver)?;
        driver
            .record_shell_resumed(tick)
            .map_err(|_| NativeProbeErrorV1::Driver)?;
        driver
            .observe_surface(
                NativePhysicalExtentV1::new(physical.width, physical.height),
                scale,
                tick,
            )
            .map_err(|_| NativeProbeErrorV1::Driver)?;
        self.surface_dirty = true;
        Ok(())
    }

    pub(super) fn handle_window_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        input: NativeShellInputV1,
    ) {
        let result = match input {
            NativeShellInputV1::Surface { physical, scale } => {
                self.observe_surface(physical, scale)
            }
            NativeShellInputV1::CursorMoved(point) => self.with_driver_tick(|driver, tick| {
                driver.cursor_moved(point, NativeInputSourceV1::Native, tick)
            }),
            NativeShellInputV1::PrimaryPressed => self.with_driver_tick(|driver, tick| {
                driver
                    .pointer_pressed(NativeInputSourceV1::Native, tick)
                    .map(|_| ())
            }),
            NativeShellInputV1::RedrawRequested => self.redraw(),
            NativeShellInputV1::CloseRequested => {
                self.close(event_loop, NativeInputSourceV1::Native)
            }
        };
        if let Err(cause) = result {
            self.fail_and_close(event_loop, cause);
        }
    }

    pub(super) fn drain_surface(&mut self, event_loop: &ActiveEventLoop) {
        let pending_surface = self
            .driver
            .as_ref()
            .is_some_and(|driver| driver.pending_surface().is_some());
        let expectation = self.active_watchdog.map(|active| active.expectation);
        if surface_preempts_redraw_v1(pending_surface, expectation)
            || surface_preempts_directive_v1(pending_surface, self.pending_script.is_empty())
        {
            self.surface_dirty = false;
            let result = self.next_scheduler_tick().and_then(|tick| {
                self.driver
                    .as_mut()
                    .ok_or(NativeFailureCauseV1::Invariant)?
                    .reject_environment_surface_between_directives(tick)
            });
            if let Err(cause) = result {
                self.fail_and_close(event_loop, cause);
            }
            return;
        }
        self.surface_dirty = false;
        let result = self.drain_once(event_loop);
        if let Err(cause) = result {
            self.fail_and_close(event_loop, cause);
        }
    }

    pub(super) fn handle_timeout(
        &mut self,
        event_loop: &ActiveEventLoop,
        token: NativeWatchdogTokenV1,
    ) {
        if self.active_watchdog.map(|active| active.token) != Some(token) {
            return;
        }
        self.active_watchdog = None;
        let recorded = self.with_driver_tick(|driver, tick| driver.record_shell_timeout(tick));
        if recorded.is_err() {
            self.abort(event_loop, NativeProbeErrorV1::Driver);
            return;
        }
        self.fail_and_close(event_loop, NativeFailureCauseV1::Timeout);
    }

    pub(super) fn abort(&mut self, event_loop: &ActiveEventLoop, error: NativeProbeErrorV1) {
        if self.failure.is_none() && self.output.is_none() {
            self.failure = Some(error);
        }
        self.driver.take();
        self.window.take();
        event_loop.exit();
    }

    pub(super) fn observe_surface(
        &mut self,
        physical: NativePhysicalExtentV1,
        scale: f64,
    ) -> Result<(), NativeFailureCauseV1> {
        self.with_driver_tick(|driver, tick| driver.observe_surface(physical, scale, tick))?;
        let observed_surface = self
            .driver
            .as_ref()
            .and_then(|driver| driver.pending_surface().or(driver.accepted_surface()))
            .map(|surface| surface.logical_surface());
        if active_resize_refuses_surface_v1(
            self.active_watchdog.map(|active| active.expectation),
            self.script.current(),
            observed_surface,
        ) {
            self.surface_dirty = false;
            self.cancel_watchdog(WatchdogExpectationV1::Resize)?;
            let tick = self.next_scheduler_tick()?;
            return self
                .driver
                .as_mut()
                .ok_or(NativeFailureCauseV1::Invariant)?
                .reject_environment_surface_between_directives(tick);
        }
        self.surface_dirty = self
            .driver
            .as_ref()
            .is_some_and(|driver| driver.pending_surface().is_some());
        Ok(())
    }

    fn drain_once(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeFailureCauseV1> {
        let tick = self.next_scheduler_tick()?;
        let action = self
            .driver
            .as_mut()
            .ok_or(NativeFailureCauseV1::Invariant)?
            .drain_scheduler(tick)?;
        self.handle_driver_action(event_loop, action, tick)
    }

    pub(super) fn handle_driver_action(
        &mut self,
        event_loop: &ActiveEventLoop,
        action: NativeDriverActionV1,
        tick: fenestra_ui_runtime::prototype::SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        match action {
            NativeDriverActionV1::Idle | NativeDriverActionV1::Suspended { .. } => Ok(()),
            NativeDriverActionV1::RequestFrame {
                generation,
                surface_generation,
            } => {
                let surface = self
                    .driver
                    .as_ref()
                    .and_then(|driver| driver.accepted_surface())
                    .ok_or(NativeFailureCauseV1::Invariant)?;
                let evidence = match self.script.current() {
                    NativeRunDirectiveV1::AwaitInitialPublication => {
                        self.initial_surface = Some(surface);
                        self.capabilities.surface = true;
                        NativeRunEvidenceV1::InitialPublished {
                            runtime_generation: generation.get(),
                            surface_generation: surface_generation.get(),
                            scale_micros: surface.scale().micros(),
                        }
                    }
                    NativeRunDirectiveV1::RequestLogicalResize { .. } => {
                        self.cancel_watchdog(WatchdogExpectationV1::Resize)?;
                        self.capabilities.resize = true;
                        NativeRunEvidenceV1::ResizePublished {
                            runtime_generation: generation.get(),
                            surface_generation: surface_generation.get(),
                            logical_width: surface.logical_surface().width(),
                            logical_height: surface.logical_surface().height(),
                        }
                    }
                    _ => return Err(NativeFailureCauseV1::Invariant),
                };
                let directive = self.script.advance(evidence)?;
                if directive != NativeRunDirectiveV1::AwaitRedraw {
                    return Err(NativeFailureCauseV1::Invariant);
                }
                self.arm_redraw()
            }
            NativeDriverActionV1::StopRenderer { control } => {
                let has_retiring = self
                    .driver
                    .as_ref()
                    .is_some_and(|driver| driver.scheduler_stats().in_flight().items() > 0);
                if has_retiring {
                    self.driver
                        .as_mut()
                        .ok_or(NativeFailureCauseV1::Invariant)?
                        .renderer_stopped(tick)?;
                }
                self.capabilities.shutdown = true;
                if self.result.is_none() {
                    let driver = self
                        .driver
                        .as_ref()
                        .ok_or(NativeFailureCauseV1::Invariant)?;
                    let surface_generation = driver
                        .accepted_surface()
                        .ok_or(NativeFailureCauseV1::Invariant)?
                        .generation()
                        .get();
                    let directive = self.script.advance(NativeRunEvidenceV1::Stopped {
                        control: control.get(),
                        runtime_generation: driver.runtime_generation().get(),
                        surface_generation,
                    })?;
                    let NativeRunDirectiveV1::Exit(result) = directive else {
                        return Err(NativeFailureCauseV1::Invariant);
                    };
                    self.result = Some(result);
                }
                self.finish(event_loop, tick)
                    .map_err(|_| NativeFailureCauseV1::Invariant)
            }
        }
    }
}

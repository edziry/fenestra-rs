use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;

use super::super::driver::NativeRedrawResultV1;
use super::super::shell::script::{NativeRunDirectiveV1, NativeRunEvidenceV1};
use super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::super::types::NativePhysicalExtentV1;
use super::app::{NativeApplicationV1, WatchdogExpectationV1, surface_preempts_redraw_v1};

impl NativeApplicationV1 {
    pub(super) fn redraw(&mut self) -> Result<(), NativeFailureCauseV1> {
        let expectation = self.active_watchdog.map(|active| active.expectation);
        let pending_surface = self
            .driver
            .as_ref()
            .is_some_and(|driver| driver.pending_surface().is_some());
        if surface_preempts_redraw_v1(pending_surface, expectation) {
            self.cancel_watchdog(expectation.ok_or(NativeFailureCauseV1::Invariant)?)?;
            let tick = self.next_scheduler_tick()?;
            return self
                .driver
                .as_mut()
                .ok_or(NativeFailureCauseV1::Invariant)?
                .reject_environment_surface_before_redraw(tick);
        }
        if expectation == Some(WatchdogExpectationV1::PresentSettled) {
            self.cancel_watchdog(WatchdogExpectationV1::PresentSettled)?;
            let tick = self.next_scheduler_tick()?;
            let redraw = self
                .driver
                .as_mut()
                .ok_or(NativeFailureCauseV1::Invariant)?
                .redraw_requested(tick)?;
            if redraw != NativeRedrawResultV1::Ignored {
                return Err(NativeFailureCauseV1::Invariant);
            }
            return self.pending_script.release_barrier();
        }
        let redraw_expected = expectation == Some(WatchdogExpectationV1::Redraw);
        if self
            .driver
            .as_ref()
            .is_some_and(|driver| driver.redraw_armed())
            != redraw_expected
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let tick = self.next_scheduler_tick()?;
        let redraw = self
            .driver
            .as_mut()
            .ok_or(NativeFailureCauseV1::Invariant)?
            .redraw_requested(tick)?;
        let NativeRedrawResultV1::Presented {
            frame,
            submission,
            completion_control,
        } = redraw
        else {
            return Ok(());
        };
        self.cancel_watchdog(WatchdogExpectationV1::Redraw)?;
        let surface = self
            .driver
            .as_ref()
            .and_then(|driver| driver.accepted_surface())
            .ok_or(NativeFailureCauseV1::Invariant)?;
        let directive = self.script.advance(NativeRunEvidenceV1::Presented {
            runtime_generation: self
                .driver
                .as_ref()
                .ok_or(NativeFailureCauseV1::Invariant)?
                .runtime_generation()
                .get(),
            surface_generation: surface.generation().get(),
            frame: frame.get(),
            submission: submission.token(),
            completion_control: completion_control.get(),
        })?;
        self.pending_script.defer_until_barrier(directive)?;
        self.arm_present_settled_barrier()
    }

    fn execute_directive(
        &mut self,
        event_loop: &ActiveEventLoop,
        directive: NativeRunDirectiveV1,
    ) -> Result<(), NativeFailureCauseV1> {
        match directive {
            NativeRunDirectiveV1::ScriptPrimaryPress { physical } => {
                let tick = self.next_scheduler_tick()?;
                let driver = self
                    .driver
                    .as_mut()
                    .ok_or(NativeFailureCauseV1::Invariant)?;
                driver.cursor_moved(physical, NativeInputSourceV1::Scripted, tick)?;
                let target = driver.pointer_pressed(NativeInputSourceV1::Scripted, tick)?;
                self.capabilities.pointer = true;
                let next = self
                    .script
                    .advance(NativeRunEvidenceV1::PointerTarget(target))?;
                self.execute_directive(event_loop, next)
            }
            NativeRunDirectiveV1::RequestLogicalResize { width, height } => {
                self.request_logical_resize(width, height)
            }
            NativeRunDirectiveV1::ScriptClose => {
                self.close(event_loop, NativeInputSourceV1::Scripted)
            }
            NativeRunDirectiveV1::AwaitInitialPublication
            | NativeRunDirectiveV1::AwaitRedraw
            | NativeRunDirectiveV1::Exit(_) => Ok(()),
        }
    }

    pub(super) fn run_deferred_script(&mut self, event_loop: &ActiveEventLoop) {
        let Some(directive) = self.pending_script.take_ready() else {
            return;
        };
        let result = self.execute_directive(event_loop, directive);
        if let Err(cause) = result {
            self.fail_and_close(event_loop, cause);
        }
    }

    fn request_logical_resize(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), NativeFailureCauseV1> {
        self.arm_watchdog(WatchdogExpectationV1::Resize)?;
        let window = self
            .window
            .as_ref()
            .ok_or(NativeFailureCauseV1::Invariant)?;
        if let Some(physical) = window.request_inner_size(LogicalSize::new(width, height)) {
            self.observe_surface(
                NativePhysicalExtentV1::new(physical.width, physical.height),
                window.scale_factor(),
            )?;
        }
        Ok(())
    }
}

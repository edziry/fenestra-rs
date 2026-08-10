use winit::event_loop::ActiveEventLoop;

use super::super::artifact::{
    NativeArtifactCapabilitiesV1, NativeArtifactManifestV1, NativeArtifactTerminalV1,
    NativeOsFamilyV1, NativeTargetV1, NativeWindowSystemV1, encode_native_artifact_v1,
};
use super::super::driver::NativeDriverActionV1;
use super::super::shell::script::classify_native_failure_v1;
use super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::app::{ActiveWatchdogV1, NativeApplicationV1, WatchdogExpectationV1};
use crate::NativeProbeErrorV1;

impl NativeApplicationV1 {
    pub(super) fn close(
        &mut self,
        event_loop: &ActiveEventLoop,
        source: NativeInputSourceV1,
    ) -> Result<(), NativeFailureCauseV1> {
        self.cancel_any_watchdog()?;
        self.pending_script.clear();
        self.surface_dirty = false;
        let tick = self.next_scheduler_tick()?;
        self.driver
            .as_mut()
            .ok_or(NativeFailureCauseV1::Invariant)?
            .close_requested(source, tick)?;
        self.drain_shutdown(event_loop)
    }

    pub(super) fn fail_and_close(
        &mut self,
        event_loop: &ActiveEventLoop,
        cause: NativeFailureCauseV1,
    ) {
        if self.result.is_none() {
            let result = classify_native_failure_v1(cause);
            if self.script.finish(result).is_err() {
                self.abort(event_loop, NativeProbeErrorV1::Driver);
                return;
            }
            self.result = Some(result);
        }
        if self
            .close(event_loop, NativeInputSourceV1::Scripted)
            .is_err()
        {
            self.abort(event_loop, NativeProbeErrorV1::Driver);
        }
    }

    fn drain_shutdown(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeFailureCauseV1> {
        for _ in 0..4 {
            let tick = self.next_scheduler_tick()?;
            let action = self
                .driver
                .as_mut()
                .ok_or(NativeFailureCauseV1::Invariant)?
                .drain_scheduler(tick)?;
            if matches!(action, NativeDriverActionV1::StopRenderer { .. }) {
                return self.handle_driver_action(event_loop, action, tick);
            }
        }
        Err(NativeFailureCauseV1::Invariant)
    }

    pub(super) fn arm_redraw(&mut self) -> Result<(), NativeFailureCauseV1> {
        self.arm_watchdog(WatchdogExpectationV1::Redraw)?;
        self.window
            .as_ref()
            .ok_or(NativeFailureCauseV1::Invariant)?
            .request_redraw();
        Ok(())
    }

    pub(super) fn arm_present_settled_barrier(&mut self) -> Result<(), NativeFailureCauseV1> {
        self.arm_watchdog(WatchdogExpectationV1::PresentSettled)?;
        self.window
            .as_ref()
            .ok_or(NativeFailureCauseV1::Invariant)?
            .request_redraw();
        Ok(())
    }

    pub(super) fn arm_watchdog(
        &mut self,
        expectation: WatchdogExpectationV1,
    ) -> Result<(), NativeFailureCauseV1> {
        if self.active_watchdog.is_some() {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let token = self
            .watchdog
            .arm()
            .map_err(|_| NativeFailureCauseV1::Timeout)?;
        self.active_watchdog = Some(ActiveWatchdogV1 { token, expectation });
        Ok(())
    }

    pub(super) fn cancel_watchdog(
        &mut self,
        expectation: WatchdogExpectationV1,
    ) -> Result<(), NativeFailureCauseV1> {
        let Some(active) = self.active_watchdog else {
            return Ok(());
        };
        if active.expectation != expectation {
            return Ok(());
        }
        self.active_watchdog = None;
        let canceled = self
            .watchdog
            .cancel(active.token)
            .map_err(|_| NativeFailureCauseV1::Timeout)?;
        if !canceled {
            return Err(NativeFailureCauseV1::Timeout);
        }
        Ok(())
    }

    fn cancel_any_watchdog(&mut self) -> Result<(), NativeFailureCauseV1> {
        let Some(active) = self.active_watchdog else {
            return Ok(());
        };
        self.cancel_watchdog(active.expectation)
    }

    pub(super) fn finish(
        &mut self,
        event_loop: &ActiveEventLoop,
        tick: fenestra_ui_runtime::prototype::SchedulerTick,
    ) -> Result<(), NativeProbeErrorV1> {
        if !self.pending_script.is_empty() {
            return Err(NativeProbeErrorV1::Artifact);
        }
        let result = self.result.ok_or(NativeProbeErrorV1::Artifact)?;
        let surface = self.initial_surface.ok_or(NativeProbeErrorV1::Artifact)?;
        let driver = self.driver.as_mut().ok_or(NativeProbeErrorV1::Driver)?;
        driver
            .record_shell_close_completed(tick)
            .map_err(|_| NativeProbeErrorV1::Driver)?;
        let terminal = NativeArtifactTerminalV1::try_from_driver(result, driver)
            .map_err(|_| NativeProbeErrorV1::Artifact)?;
        let capabilities = self.capabilities.artifact();
        let (os, target, window_system) = current_environment();
        let manifest = NativeArtifactManifestV1::new(
            os,
            target,
            window_system,
            surface,
            NativeArtifactCapabilitiesV1::new(true, true, true, true, true),
            capabilities,
            capabilities,
        );
        let encoded = encode_native_artifact_v1(&manifest, driver.trace(), &terminal)
            .map_err(|_| NativeProbeErrorV1::Artifact)?;
        self.output = Some(encoded.into_bytes());
        self.driver.take();
        self.window.take();
        event_loop.exit();
        Ok(())
    }

    pub(super) fn with_driver_tick<T>(
        &mut self,
        operation: impl FnOnce(
            &mut super::app::NativeDriverStateV1,
            fenestra_ui_runtime::prototype::SchedulerTick,
        ) -> Result<T, NativeFailureCauseV1>,
    ) -> Result<T, NativeFailureCauseV1> {
        let tick = self.next_scheduler_tick()?;
        operation(
            self.driver
                .as_mut()
                .ok_or(NativeFailureCauseV1::Invariant)?,
            tick,
        )
    }

    pub(super) fn next_scheduler_tick(
        &mut self,
    ) -> Result<fenestra_ui_runtime::prototype::SchedulerTick, NativeFailureCauseV1> {
        let tick = self.next_tick;
        self.next_tick = self
            .next_tick
            .checked_add(1)
            .ok_or(NativeFailureCauseV1::Arithmetic)?;
        Ok(fenestra_ui_runtime::prototype::SchedulerTick::new(tick))
    }

    pub(super) fn take_tick(
        &mut self,
    ) -> Result<fenestra_ui_runtime::prototype::SchedulerTick, NativeProbeErrorV1> {
        self.next_scheduler_tick()
            .map_err(|_| NativeProbeErrorV1::Driver)
    }
}

impl super::app::ObservedCapabilitiesV1 {
    fn artifact(self) -> NativeArtifactCapabilitiesV1 {
        NativeArtifactCapabilitiesV1::new(
            self.window,
            self.surface,
            self.pointer,
            self.resize,
            self.shutdown,
        )
    }
}

const fn current_environment() -> (NativeOsFamilyV1, NativeTargetV1, NativeWindowSystemV1) {
    if cfg!(target_os = "windows") {
        (
            NativeOsFamilyV1::Windows,
            NativeTargetV1::X86_64PcWindowsMsvc,
            NativeWindowSystemV1::Win32,
        )
    } else {
        (
            NativeOsFamilyV1::Linux,
            NativeTargetV1::X86_64UnknownLinuxGnu,
            NativeWindowSystemV1::Wayland,
        )
    }
}

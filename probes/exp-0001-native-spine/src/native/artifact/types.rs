use fenestra_ui_runtime::prototype::SchedulerState;

use super::super::driver::{NativeDriverV1, PresenterPortV1};
use super::super::surface::NativeSurfaceTupleV1;
use super::super::trace::{NativeFailureCauseV1, NativeTraceLaneStatsV1, NativeTracePendingV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeOsFamilyV1 {
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeTargetV1 {
    X86_64UnknownLinuxGnu,
    X86_64PcWindowsMsvc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeWindowSystemV1 {
    Wayland,
    Win32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeProbeResultV1 {
    Pass,
    Adapt,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeArtifactCapabilitiesV1(u8);

impl NativeArtifactCapabilitiesV1 {
    pub(crate) const fn new(
        window: bool,
        surface: bool,
        pointer: bool,
        resize: bool,
        shutdown: bool,
    ) -> Self {
        Self(
            bool_bit(window)
                | bool_bit(surface) << 1
                | bool_bit(pointer) << 2
                | bool_bit(resize) << 3
                | bool_bit(shutdown) << 4,
        )
    }

    pub(super) const fn bits(self) -> u8 {
        self.0
    }

    const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeArtifactManifestV1 {
    os: NativeOsFamilyV1,
    target: NativeTargetV1,
    window_system: NativeWindowSystemV1,
    surface: NativeSurfaceTupleV1,
    requested: NativeArtifactCapabilitiesV1,
    detected: NativeArtifactCapabilitiesV1,
    effective: NativeArtifactCapabilitiesV1,
}

impl NativeArtifactManifestV1 {
    pub(crate) const fn new(
        os: NativeOsFamilyV1,
        target: NativeTargetV1,
        window_system: NativeWindowSystemV1,
        surface: NativeSurfaceTupleV1,
        requested: NativeArtifactCapabilitiesV1,
        detected: NativeArtifactCapabilitiesV1,
        effective: NativeArtifactCapabilitiesV1,
    ) -> Self {
        Self {
            os,
            target,
            window_system,
            surface,
            requested,
            detected,
            effective,
        }
    }

    pub(super) const fn os(self) -> NativeOsFamilyV1 {
        self.os
    }

    pub(super) const fn target(self) -> NativeTargetV1 {
        self.target
    }

    pub(super) const fn window_system(self) -> NativeWindowSystemV1 {
        self.window_system
    }

    pub(super) const fn surface(self) -> NativeSurfaceTupleV1 {
        self.surface
    }

    pub(super) const fn requested(self) -> NativeArtifactCapabilitiesV1 {
        self.requested
    }

    pub(super) const fn detected(self) -> NativeArtifactCapabilitiesV1 {
        self.detected
    }

    pub(super) const fn effective(self) -> NativeArtifactCapabilitiesV1 {
        self.effective
    }

    pub(super) const fn is_valid(self) -> bool {
        let environment = matches!(
            (self.os, self.target, self.window_system),
            (
                NativeOsFamilyV1::Linux,
                NativeTargetV1::X86_64UnknownLinuxGnu,
                NativeWindowSystemV1::Wayland,
            ) | (
                NativeOsFamilyV1::Windows,
                NativeTargetV1::X86_64PcWindowsMsvc,
                NativeWindowSystemV1::Win32,
            )
        );
        environment
            && self.requested.contains(self.effective)
            && self.detected.contains(self.effective)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeArtifactTerminalV1 {
    result: NativeProbeResultV1,
    runtime_generation: u64,
    scheduler_state: SchedulerState,
    deferred: NativeTraceLaneStatsV1,
    controls: NativeTraceLaneStatsV1,
    visual: NativeTraceLaneStatsV1,
    in_flight: NativeTraceLaneStatsV1,
    redraw_armed: bool,
    pending: NativeTracePendingV1,
}

impl NativeArtifactTerminalV1 {
    pub(crate) fn try_from_driver<P: PresenterPortV1>(
        result: NativeProbeResultV1,
        driver: &NativeDriverV1<P>,
    ) -> Result<Self, NativeFailureCauseV1> {
        let stats = driver.scheduler_stats();
        let terminal = Self {
            result,
            runtime_generation: driver.runtime_generation().get(),
            scheduler_state: driver.scheduler_state(),
            deferred: lane(stats.deferred()),
            controls: lane(stats.controls()),
            visual: lane(stats.visual()),
            in_flight: lane(stats.in_flight()),
            redraw_armed: driver.redraw_armed(),
            pending: NativeTracePendingV1::new(
                usize::from(driver.pending_surface().is_some()),
                driver.pending_pointer_count(),
                driver.presenter_pending_count(),
            ),
        };
        if !terminal.is_stopped_and_empty() {
            return Err(NativeFailureCauseV1::Invariant);
        }
        Ok(terminal)
    }

    pub(crate) const fn result(self) -> NativeProbeResultV1 {
        self.result
    }

    pub(crate) const fn runtime_generation(self) -> u64 {
        self.runtime_generation
    }

    pub(crate) fn is_stopped_and_empty(self) -> bool {
        self.scheduler_state == SchedulerState::Stopped
            && lane_is_empty(self.deferred)
            && lane_is_empty(self.controls)
            && lane_is_empty(self.visual)
            && lane_is_empty(self.in_flight)
            && !self.redraw_armed
            && self.pending.surface() == 0
            && self.pending.pointer() == 0
            && self.pending.presenter() == 0
    }

    pub(super) const fn scheduler_state(self) -> SchedulerState {
        self.scheduler_state
    }

    pub(super) const fn deferred(self) -> NativeTraceLaneStatsV1 {
        self.deferred
    }

    pub(super) const fn controls(self) -> NativeTraceLaneStatsV1 {
        self.controls
    }

    pub(super) const fn visual(self) -> NativeTraceLaneStatsV1 {
        self.visual
    }

    pub(super) const fn in_flight(self) -> NativeTraceLaneStatsV1 {
        self.in_flight
    }

    pub(super) const fn redraw_armed(self) -> bool {
        self.redraw_armed
    }

    pub(super) const fn pending(self) -> NativeTracePendingV1 {
        self.pending
    }
}

const fn lane(stats: fenestra_ui_runtime::prototype::QueueStats) -> NativeTraceLaneStatsV1 {
    NativeTraceLaneStatsV1::new(stats.items(), stats.accounted_bytes())
}

const fn lane_is_empty(lane: NativeTraceLaneStatsV1) -> bool {
    lane.items() == 0 && lane.accounted_bytes() == 0
}

const fn bool_bit(value: bool) -> u8 {
    if value { 1 } else { 0 }
}

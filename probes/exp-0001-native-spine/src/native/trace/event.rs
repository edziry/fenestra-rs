mod validation;

use fenestra_ui_runtime::prototype::{RuntimeGeneration, SchedulerState, SchedulerTick};
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::surface::{NativeSurfaceObservationV1, NativeSurfaceTupleV1};
use super::types::{
    NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1, NativeTraceLaneStatsV1,
    NativeTracePendingV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
};

pub(super) use validation::validate_step_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTraceEventV1 {
    schema_revision: u16,
    sequence: u64,
    tick: SchedulerTick,
    scheduler_state: SchedulerState,
    current_generation: RuntimeGeneration,
    step: NativeTraceStepV1,
}

impl NativeTraceEventV1 {
    pub(crate) const ACCOUNTED_BYTES: usize = 192;

    pub(crate) const fn new(
        sequence: u64,
        tick: SchedulerTick,
        scheduler_state: SchedulerState,
        current_generation: RuntimeGeneration,
        step: NativeTraceStepV1,
    ) -> Self {
        Self {
            schema_revision: 1,
            sequence,
            tick,
            scheduler_state,
            current_generation,
            step,
        }
    }

    pub(crate) const fn schema_revision(self) -> u16 {
        self.schema_revision
    }

    pub(crate) const fn sequence(self) -> u64 {
        self.sequence
    }

    pub(crate) const fn tick(self) -> SchedulerTick {
        self.tick
    }

    pub(crate) const fn stage(self) -> NativeTraceStageV1 {
        self.step.stage
    }

    pub(crate) const fn observation(self) -> NativeObservationV1 {
        self.step.observation
    }

    pub(crate) const fn outcome(self) -> NativeOutcomeV1 {
        self.step.outcome
    }

    pub(crate) const fn input_source(self) -> Option<NativeInputSourceV1> {
        self.step.input_source
    }

    pub(crate) const fn scheduler_state(self) -> SchedulerState {
        self.scheduler_state
    }

    pub(crate) const fn current_generation(self) -> RuntimeGeneration {
        self.current_generation
    }

    pub(crate) const fn scheduler_turn(self) -> Option<u64> {
        self.step.scheduler_turn
    }

    pub(crate) const fn captured_generation(self) -> Option<RuntimeGeneration> {
        self.step.captured_generation
    }

    pub(crate) const fn published_generation(self) -> Option<RuntimeGeneration> {
        self.step.published_generation
    }

    pub(crate) const fn surface(self) -> Option<NativeSurfaceTupleV1> {
        self.step.surface
    }

    pub(crate) const fn surface_observation(self) -> Option<NativeSurfaceObservationV1> {
        self.step.surface_observation
    }

    pub(crate) const fn target(self) -> Option<HeadlessPointerTargetV1> {
        self.step.target
    }

    pub(crate) const fn frame(self) -> Option<u64> {
        self.step.frame
    }

    pub(crate) const fn submission(self) -> Option<NativeTraceSubmissionV1> {
        self.step.submission
    }

    pub(crate) const fn control(self) -> Option<u64> {
        self.step.control
    }

    pub(crate) const fn staging_digest(self) -> Option<u64> {
        self.step.staging_digest
    }

    pub(crate) const fn redraw_armed(self) -> bool {
        self.step.redraw_armed
    }

    pub(crate) const fn pending(self) -> NativeTracePendingV1 {
        self.step.pending
    }

    pub(crate) const fn deferred(self) -> NativeTraceLaneStatsV1 {
        self.step.deferred
    }

    pub(crate) const fn controls(self) -> NativeTraceLaneStatsV1 {
        self.step.controls
    }

    pub(crate) const fn visual(self) -> NativeTraceLaneStatsV1 {
        self.step.visual
    }

    pub(crate) const fn in_flight(self) -> NativeTraceLaneStatsV1 {
        self.step.in_flight
    }
}

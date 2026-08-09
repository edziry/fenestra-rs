use fenestra_ui_runtime::prototype::{
    ControlSequence, FrameId, HeadlessSurface, RuntimeGeneration, SchedulerTick, UiScheduler,
};

use crate::headless::HeadlessPointerTargetV1;
use crate::scheduler::{FakeClockDomainV1, FakeRendererV1};

use super::types::{
    HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceError, HeadlessTraceErrorKind,
    HeadlessTraceProjectionCountsV1, HeadlessTraceQueueStatsV1, HeadlessTraceRendererStatsV1,
    HeadlessTraceStageV1, HeadlessTraceState, HeadlessTraceStep,
};

/// One fixed-accounting, privacy-safe deterministic headless observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessTraceEventV1 {
    schema_revision: u32,
    sequence: u64,
    domain: FakeClockDomainV1,
    tick: SchedulerTick,
    stage: HeadlessTraceStageV1,
    input: HeadlessInputKindV1,
    outcome: HeadlessOutcomeV1,
    captured_generation: Option<RuntimeGeneration>,
    published_generation: Option<RuntimeGeneration>,
    target: HeadlessPointerTargetV1,
    frame: Option<FrameId>,
    control: Option<ControlSequence>,
    surface: HeadlessSurface,
    projection_counts: HeadlessTraceProjectionCountsV1,
    deferred: HeadlessTraceQueueStatsV1,
    controls: HeadlessTraceQueueStatsV1,
    visual: HeadlessTraceQueueStatsV1,
    in_flight: HeadlessTraceQueueStatsV1,
    renderer: HeadlessTraceRendererStatsV1,
}

impl HeadlessTraceEventV1 {
    /// Fixed V1 protocol-accounted event weight.
    pub const ACCOUNTED_BYTES: usize = 160;

    /// Returns the event schema revision.
    #[must_use]
    pub const fn schema_revision(self) -> u32 {
        self.schema_revision
    }
    /// Returns the dense zero-based sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
    /// Returns the shared fake-clock domain.
    #[must_use]
    pub const fn clock_domain(self) -> FakeClockDomainV1 {
        self.domain
    }
    /// Returns the logical scheduler tick.
    #[must_use]
    pub const fn tick(self) -> SchedulerTick {
        self.tick
    }
    /// Returns the closed trace stage.
    #[must_use]
    pub const fn stage(self) -> HeadlessTraceStageV1 {
        self.stage
    }
    /// Returns the closed causal input.
    #[must_use]
    pub const fn input(self) -> HeadlessInputKindV1 {
        self.input
    }
    /// Returns the closed event outcome.
    #[must_use]
    pub const fn outcome(self) -> HeadlessOutcomeV1 {
        self.outcome
    }
    /// Returns a captured committed generation, when applicable.
    #[must_use]
    pub const fn captured_generation(self) -> Option<RuntimeGeneration> {
        self.captured_generation
    }
    /// Returns a generation published by the operation, when applicable.
    #[must_use]
    pub const fn published_generation(self) -> Option<RuntimeGeneration> {
        self.published_generation
    }
    /// Returns the closed semantic target.
    #[must_use]
    pub const fn target(self) -> HeadlessPointerTargetV1 {
        self.target
    }
    /// Returns the named frame identity, when applicable.
    #[must_use]
    pub const fn frame(self) -> Option<FrameId> {
        self.frame
    }
    /// Returns the named control sequence, when applicable.
    #[must_use]
    pub const fn control(self) -> Option<ControlSequence> {
        self.control
    }
    /// Returns the committed headless surface.
    #[must_use]
    pub const fn surface(self) -> HeadlessSurface {
        self.surface
    }
    /// Returns counts for every projection family.
    #[must_use]
    pub const fn projection_counts(self) -> HeadlessTraceProjectionCountsV1 {
        self.projection_counts
    }
    /// Returns deferred lane accounting.
    #[must_use]
    pub const fn deferred(self) -> HeadlessTraceQueueStatsV1 {
        self.deferred
    }
    /// Returns control lane accounting.
    #[must_use]
    pub const fn controls(self) -> HeadlessTraceQueueStatsV1 {
        self.controls
    }
    /// Returns visual lane accounting.
    #[must_use]
    pub const fn visual(self) -> HeadlessTraceQueueStatsV1 {
        self.visual
    }
    /// Returns in-flight lane accounting.
    #[must_use]
    pub const fn in_flight(self) -> HeadlessTraceQueueStatsV1 {
        self.in_flight
    }
    /// Returns fake renderer ledger accounting.
    #[must_use]
    pub const fn renderer(self) -> HeadlessTraceRendererStatsV1 {
        self.renderer
    }
}

pub(super) fn project_event(
    sequence: u64,
    domain: FakeClockDomainV1,
    tick: SchedulerTick,
    step: HeadlessTraceStep,
    scheduler: &UiScheduler,
    renderer: &FakeRendererV1,
) -> Result<HeadlessTraceEventV1, HeadlessTraceError> {
    let snapshot = scheduler.committed();
    let projection = snapshot
        .headless_projection()
        .ok_or_else(|| HeadlessTraceError::new(HeadlessTraceErrorKind::ProjectionUnavailable))?;
    if projection.generation() != snapshot.generation() {
        return Err(HeadlessTraceError::new(
            HeadlessTraceErrorKind::IdentityMismatch,
        ));
    }
    let state = HeadlessTraceState {
        generation: snapshot.generation(),
        surface: projection.surface(),
        counts: HeadlessTraceProjectionCountsV1::new(
            projection.computed_style_count(),
            projection.geometry_count(),
            projection.semantic_count(),
            projection.hit_region_count(),
            projection.scene_rectangle_count(),
        ),
    };
    let stats = scheduler.stats();
    Ok(HeadlessTraceEventV1 {
        schema_revision: 1,
        sequence,
        domain,
        tick,
        stage: step.stage,
        input: step.input,
        outcome: step.outcome,
        captured_generation: step.captured_generation,
        published_generation: step.records_publication.then_some(state.generation),
        target: step.target,
        frame: step.frame,
        control: step.control,
        surface: state.surface,
        projection_counts: state.counts,
        deferred: HeadlessTraceQueueStatsV1::project(stats.deferred()),
        controls: HeadlessTraceQueueStatsV1::project(stats.controls()),
        visual: HeadlessTraceQueueStatsV1::project(stats.visual()),
        in_flight: HeadlessTraceQueueStatsV1::project(stats.in_flight()),
        renderer: HeadlessTraceRendererStatsV1::project(renderer.stats()),
    })
}

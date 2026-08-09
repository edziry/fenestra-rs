use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, ControlSequence, FrameId, QueueStats, RuntimeGeneration,
    SchedulerInput, SchedulerInputResult, SchedulerState, SchedulerTick, SubmissionId, UiScheduler,
};

use super::super::{FakeClockDomainV1, FakeRendererStatsV1, FakeRendererV1};
use super::oldest_residence_ticks;
use super::types::{
    SchedulerTraceActionV1, SchedulerTraceErrorV1, SchedulerTraceStageV1, SchedulerTraceStepV1,
};

/// Copyable bounded accounting for one runtime-owned scheduler lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTraceLaneStatsV1 {
    items: usize,
    accounted_bytes: usize,
    oldest_residence_ticks: Option<u64>,
}

impl SchedulerTraceLaneStatsV1 {
    fn project(stats: QueueStats, tick: SchedulerTick) -> Result<Self, SchedulerTraceErrorV1> {
        Ok(Self {
            items: stats.items(),
            accounted_bytes: stats.accounted_bytes(),
            oldest_residence_ticks: oldest_residence_ticks(stats, tick)?,
        })
    }

    /// Returns the accepted item count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns the protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }

    /// Returns the age of the oldest retained item at the event tick.
    #[must_use]
    pub const fn oldest_residence_ticks(self) -> Option<u64> {
        self.oldest_residence_ticks
    }
}

/// Copyable bounded accounting for the fake renderer retirement ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTraceRendererStatsV1 {
    items: usize,
    accounted_bytes: usize,
    oldest_residence_ticks: Option<u64>,
    last_accepted: Option<SubmissionId>,
    completed: Option<CompletionWatermark>,
    has_pending_control: bool,
}

impl SchedulerTraceRendererStatsV1 {
    fn project(
        stats: FakeRendererStatsV1,
        tick: SchedulerTick,
    ) -> Result<Self, SchedulerTraceErrorV1> {
        let oldest_residence_ticks = stats
            .earliest_tick()
            .map(|earliest| {
                tick.get().checked_sub(earliest.get()).ok_or_else(|| {
                    SchedulerTraceErrorV1::new(
                        super::types::SchedulerTraceErrorKindV1::TickRegression,
                    )
                })
            })
            .transpose()?;
        Ok(Self {
            items: stats.items(),
            accounted_bytes: stats.accounted_bytes(),
            oldest_residence_ticks,
            last_accepted: stats.last_accepted(),
            completed: stats.completed(),
            has_pending_control: stats.has_pending_control(),
        })
    }

    /// Returns the retained synthetic-resource count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns the retained protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }

    /// Returns the age of the oldest unretired resource use.
    #[must_use]
    pub const fn oldest_residence_ticks(self) -> Option<u64> {
        self.oldest_residence_ticks
    }

    /// Returns the latest submission accepted through the fake renderer.
    #[must_use]
    pub const fn last_accepted(self) -> Option<SubmissionId> {
        self.last_accepted
    }

    /// Returns the latest completion admitted through the fake renderer.
    #[must_use]
    pub const fn completed(self) -> Option<CompletionWatermark> {
        self.completed
    }

    /// Reports whether one renderer control is retained for retry.
    #[must_use]
    pub const fn has_pending_control(self) -> bool {
        self.has_pending_control
    }
}

/// One fixed-accounting, privacy-safe deterministic scheduler observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTraceEventV1 {
    schema_revision: u32,
    sequence: u64,
    domain: FakeClockDomainV1,
    tick: SchedulerTick,
    step: SchedulerTraceStepV1,
    lifecycle: SchedulerState,
    generation: RuntimeGeneration,
    frame: Option<FrameId>,
    control: Option<ControlSequence>,
    deferred: SchedulerTraceLaneStatsV1,
    controls: SchedulerTraceLaneStatsV1,
    visual: SchedulerTraceLaneStatsV1,
    in_flight: SchedulerTraceLaneStatsV1,
    renderer: SchedulerTraceRendererStatsV1,
}

impl SchedulerTraceEventV1 {
    /// Fixed V1 protocol-accounted event weight.
    pub const ACCOUNTED_BYTES: usize = 96;

    /// Returns the event schema revision.
    #[must_use]
    pub const fn schema_revision(self) -> u32 {
        self.schema_revision
    }

    /// Returns the zero-based deterministic sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    /// Returns the fake-clock domain captured by this trace.
    #[must_use]
    pub const fn clock_domain(self) -> FakeClockDomainV1 {
        self.domain
    }

    /// Returns the logical scheduler tick.
    #[must_use]
    pub const fn tick(self) -> SchedulerTick {
        self.tick
    }

    /// Returns the complete typed operation projection.
    #[must_use]
    pub const fn step(self) -> SchedulerTraceStepV1 {
        self.step
    }

    /// Returns the scheduler stage represented by the operation.
    #[must_use]
    pub const fn stage(self) -> SchedulerTraceStageV1 {
        self.step.stage()
    }

    /// Returns one-based callback nesting depth only for callback events.
    #[must_use]
    pub const fn callback_depth(self) -> Option<usize> {
        self.step.callback_depth()
    }

    /// Returns the post-transition scheduler lifecycle.
    #[must_use]
    pub const fn lifecycle(self) -> SchedulerState {
        self.lifecycle
    }

    /// Returns the post-transition committed generation.
    #[must_use]
    pub const fn generation(self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the frame identity named by this event, when present.
    #[must_use]
    pub const fn frame(self) -> Option<FrameId> {
        self.frame
    }

    /// Returns the control sequence admitted or delivered by this event.
    #[must_use]
    pub const fn control(self) -> Option<ControlSequence> {
        self.control
    }

    /// Returns deferred callback transaction accounting.
    #[must_use]
    pub const fn deferred(self) -> SchedulerTraceLaneStatsV1 {
        self.deferred
    }

    /// Returns non-droppable control accounting.
    #[must_use]
    pub const fn controls(self) -> SchedulerTraceLaneStatsV1 {
        self.controls
    }

    /// Returns replaceable visual work accounting.
    #[must_use]
    pub const fn visual(self) -> SchedulerTraceLaneStatsV1 {
        self.visual
    }

    /// Returns accepted renderer submission accounting.
    #[must_use]
    pub const fn in_flight(self) -> SchedulerTraceLaneStatsV1 {
        self.in_flight
    }

    /// Returns fake renderer retirement accounting.
    #[must_use]
    pub const fn renderer(self) -> SchedulerTraceRendererStatsV1 {
        self.renderer
    }
}

pub(super) fn project_event(
    sequence: u64,
    domain: FakeClockDomainV1,
    tick: SchedulerTick,
    step: SchedulerTraceStepV1,
    scheduler: &UiScheduler,
    renderer: &FakeRendererV1,
) -> Result<SchedulerTraceEventV1, SchedulerTraceErrorV1> {
    let stats = scheduler.stats();
    let generation = {
        let snapshot = scheduler.committed();
        snapshot.generation()
    };
    Ok(SchedulerTraceEventV1 {
        schema_revision: 1,
        sequence,
        domain,
        tick,
        lifecycle: scheduler.state(),
        generation,
        frame: frame(step),
        control: control(step),
        step,
        deferred: SchedulerTraceLaneStatsV1::project(stats.deferred(), tick)?,
        controls: SchedulerTraceLaneStatsV1::project(stats.controls(), tick)?,
        visual: SchedulerTraceLaneStatsV1::project(stats.visual(), tick)?,
        in_flight: SchedulerTraceLaneStatsV1::project(stats.in_flight(), tick)?,
        renderer: SchedulerTraceRendererStatsV1::project(renderer.stats(), tick)?,
    })
}

const fn frame(step: SchedulerTraceStepV1) -> Option<FrameId> {
    match step {
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::AcceptFrame(frame) | SchedulerInput::RejectFrame(frame),
            ..
        }
        | SchedulerTraceStepV1::Action(SchedulerTraceActionV1::OfferFrame(frame)) => Some(frame),
        _ => None,
    }
}

const fn control(step: SchedulerTraceStepV1) -> Option<ControlSequence> {
    match step {
        SchedulerTraceStepV1::Input {
            outcome:
                super::types::SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(
                    admission,
                )),
            ..
        } => Some(match admission {
            ControlAdmission::Accepted(sequence) | ControlAdmission::AlreadyAccepted(sequence) => {
                sequence
            }
        }),
        SchedulerTraceStepV1::Action(SchedulerTraceActionV1::StopRenderer(sequence)) => {
            Some(sequence)
        }
        _ => None,
    }
}

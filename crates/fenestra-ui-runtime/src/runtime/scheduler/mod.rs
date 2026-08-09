mod callback;
mod control;
mod frame;
mod residence;
mod stats;
mod submission;
mod types;

use std::collections::VecDeque;

use fenestra_ui_ir::prototype::InvalidationSet;

use super::transaction::{UiRuntime, UiTransaction};
use super::view::CommittedRuntimeSnapshot;

use callback::DeferredTransaction;
pub use callback::{CallbackFinish, CallbackScope, NestedCallbackScope};
use control::{CONTROL_ENVELOPE_BYTES, ControlQueue};
pub use control::{ControlAdmission, ControlSequence};
pub use frame::{
    CompletionWatermark, FrameId, FrameWork, RendererEpoch, SchedulerInput, SchedulerInputResult,
    SubmissionId,
};
use frame::{SubmittedFrame, VisualState, VisualWork};
pub use stats::{QueueStats, SchedulerStats};
use types::VISUAL_ENVELOPE_BYTES;
pub use types::{
    QueueCapacity, ScheduledCommit, SchedulerAction, SchedulerCapacity, SchedulerError,
    SchedulerErrorKind, SchedulerLane, SchedulerState, SchedulerTick, VisualCancelResult,
};

/// Single-owner bounded scheduler around one logical UI runtime.
pub struct UiScheduler {
    runtime: UiRuntime,
    capacity: SchedulerCapacity,
    deferred: Option<DeferredTransaction>,
    controls: ControlQueue,
    visual: Option<VisualState>,
    in_flight: VecDeque<SubmittedFrame>,
    in_flight_bytes: usize,
    renderer_epoch: RendererEpoch,
    next_frame_id: Option<u64>,
    next_submission_token: Option<u64>,
    last_accepted_token: Option<u64>,
    completed_token: Option<u64>,
    state: SchedulerState,
    terminal_pressure: Option<SchedulerLane>,
    last_tick: Option<SchedulerTick>,
}

impl UiScheduler {
    /// Creates a scheduler after validating every publication-critical bound.
    pub fn new(runtime: UiRuntime, capacity: SchedulerCapacity) -> Result<Self, SchedulerError> {
        validate_capacity(&runtime, capacity)?;
        Ok(Self {
            runtime,
            capacity,
            deferred: None,
            controls: ControlQueue::new(),
            visual: None,
            in_flight: VecDeque::new(),
            in_flight_bytes: 0,
            renderer_epoch: RendererEpoch::new(0),
            next_frame_id: Some(0),
            next_submission_token: Some(0),
            last_accepted_token: None,
            completed_token: None,
            state: SchedulerState::Running,
            terminal_pressure: None,
            last_tick: None,
        })
    }

    /// Returns an immutable handle to the current committed generation.
    #[must_use]
    pub fn committed(&self) -> CommittedRuntimeSnapshot {
        self.runtime.committed()
    }

    /// Begins a detached transaction against the current generation.
    #[must_use]
    pub fn begin_transaction(&self) -> UiTransaction {
        self.runtime.begin_transaction()
    }

    /// Returns the explicit scheduler lane capacities.
    #[must_use]
    pub const fn capacity(&self) -> SchedulerCapacity {
        self.capacity
    }

    /// Returns the current closed scheduler lifecycle state.
    #[must_use]
    pub const fn state(&self) -> SchedulerState {
        self.state
    }

    /// Commits one transaction and coalesces its visual publication.
    pub fn commit(
        &mut self,
        transaction: UiTransaction,
        tick: SchedulerTick,
    ) -> Result<ScheduledCommit, SchedulerError> {
        self.begin_regular_turn(tick)?;
        if self.state != SchedulerState::Running
            || self.controls_pending()
            || self.deferred.is_some()
            || self.offer_is_pending()
        {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ControlPending,
                None,
            ));
        }

        self.commit_ready(transaction, tick)
    }

    fn commit_ready(
        &mut self,
        transaction: UiTransaction,
        tick: SchedulerTick,
    ) -> Result<ScheduledCommit, SchedulerError> {
        let receipt = self.runtime.commit(transaction).map_err(|error| {
            SchedulerError::new(
                SchedulerErrorKind::Transaction(error.kind()),
                error.operation_index(),
            )
        })?;
        let summary = ScheduledCommit::new(
            receipt.generation(),
            receipt.mutations().len(),
            receipt.invalidation(),
        );
        drop(receipt);
        if summary.is_empty() {
            return Ok(summary);
        }

        self.coalesce_visual(self.runtime.committed(), summary.invalidation(), tick);
        Ok(summary)
    }

    /// Advances one adapter-facing action without invoking foreign code.
    pub fn next_action(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<Option<SchedulerAction>, SchedulerError> {
        self.observe_tick(tick)?;
        let pressure_was_latched = self.terminal_pressure.is_some();
        let pressure = self.observe_residence(tick)?;
        if pressure.is_some() && !pressure_was_latched {
            return Err(self.residence_error());
        }
        if self.controls_pending() {
            return self.process_next_control();
        }
        if pressure.is_some() {
            return Err(self.residence_error());
        }
        if self.state != SchedulerState::Running {
            return Ok(None);
        }
        if let Some(deferred) = self.deferred.take() {
            self.commit_ready(deferred.transaction, tick)?;
        }
        if let Some(VisualState::RequestOutstanding { action_pending, .. }) = self.visual.as_mut()
            && *action_pending
        {
            *action_pending = false;
            return Ok(Some(SchedulerAction::RequestFrame));
        }

        if !matches!(self.visual, Some(VisualState::PendingPublication(_)))
            || self.next_in_flight_bytes()?.is_none()
        {
            return Ok(None);
        }
        let frame_id = self.allocate_frame_id()?;
        let Some(VisualState::PendingPublication(work)) = self.visual.take() else {
            return Ok(None);
        };
        let frame = work.into_frame(frame_id);
        self.visual = Some(VisualState::OfferAwaitingDisposition(frame.clone()));
        Ok(Some(SchedulerAction::OfferFrame(frame)))
    }

    /// Processes one typed platform or renderer observation.
    pub fn process_input(
        &mut self,
        input: SchedulerInput,
        tick: SchedulerTick,
    ) -> Result<SchedulerInputResult, SchedulerError> {
        self.observe_tick(tick)?;
        let pressure = self.observe_residence(tick)?;
        match input {
            SchedulerInput::Complete(watermark) => self
                .admit_completion_control(watermark, tick)
                .map(SchedulerInputResult::Control),
            SchedulerInput::RendererLost(epoch) => self
                .admit_renderer_loss_control(epoch, tick)
                .map(SchedulerInputResult::Control),
            SchedulerInput::RequestShutdown => self
                .admit_shutdown_control(tick)
                .map(SchedulerInputResult::Control),
            SchedulerInput::FrameReady => {
                self.ensure_visual_input_allowed(pressure)?;
                self.frame_ready()
            }
            SchedulerInput::AcceptFrame(frame) => {
                self.ensure_visual_input_allowed(pressure)?;
                self.accept_frame(frame, tick)
            }
            SchedulerInput::RejectFrame(frame) => {
                self.ensure_visual_input_allowed(pressure)?;
                self.reject_frame(frame)
            }
        }
    }

    /// Cancels replaceable visual work without releasing accepted submissions.
    pub fn cancel_visual(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<VisualCancelResult, SchedulerError> {
        self.observe_tick(tick)?;
        self.observe_residence(tick)?;
        Ok(if self.visual.take().is_some() {
            VisualCancelResult::Canceled
        } else {
            VisualCancelResult::AlreadyEmpty
        })
    }

    /// Returns bounded lane accounting without retaining another snapshot.
    #[must_use]
    pub fn stats(&self) -> SchedulerStats {
        let deferred = self
            .deferred
            .as_ref()
            .map_or_else(QueueStats::empty, |work| {
                QueueStats::occupied_bytes(
                    work.accepted_tick,
                    work.accepted_tick,
                    work.accounted_bytes,
                )
            });
        let controls = self.controls.stats();
        let visual = self
            .visual
            .as_ref()
            .map_or_else(QueueStats::empty, |state| {
                let (earliest, latest) = state.ticks();
                QueueStats::occupied_bytes(earliest, latest, VISUAL_ENVELOPE_BYTES)
            });
        let in_flight = match (self.in_flight.front(), self.in_flight.back()) {
            (Some(first), Some(last)) => QueueStats::counted(
                self.in_flight.len(),
                self.in_flight_bytes,
                first.accepted_tick,
                last.accepted_tick,
            ),
            _ => QueueStats::empty(),
        };
        SchedulerStats::new(deferred, controls, visual, in_flight)
    }

    fn coalesce_visual(
        &mut self,
        snapshot: CommittedRuntimeSnapshot,
        invalidation: InvalidationSet,
        tick: SchedulerTick,
    ) {
        match self.visual.as_mut() {
            Some(VisualState::RequestOutstanding { work, .. })
            | Some(VisualState::PendingPublication(work)) => {
                work.replace(snapshot, invalidation, tick);
            }
            Some(VisualState::OfferAwaitingDisposition(_)) => {}
            None => {
                self.visual = Some(VisualState::RequestOutstanding {
                    work: VisualWork {
                        snapshot,
                        invalidation,
                        earliest_tick: tick,
                        latest_tick: tick,
                    },
                    action_pending: true,
                });
            }
        }
    }

    pub(super) fn offer_is_pending(&self) -> bool {
        matches!(self.visual, Some(VisualState::OfferAwaitingDisposition(_)))
    }

    fn ensure_visual_input_allowed(
        &self,
        pressure: Option<SchedulerLane>,
    ) -> Result<(), SchedulerError> {
        if pressure.is_some() {
            return Err(self.residence_error());
        }
        if self.state != SchedulerState::Running || self.controls_pending() {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ControlPending,
                None,
            ));
        }
        Ok(())
    }
}

fn validate_capacity(
    runtime: &UiRuntime,
    capacity: SchedulerCapacity,
) -> Result<(), SchedulerError> {
    let visual = capacity.visual();
    if visual.max_items() < 1 || visual.max_bytes() < VISUAL_ENVELOPE_BYTES {
        return Err(SchedulerError::new(
            SchedulerErrorKind::CapacityTooSmall(SchedulerLane::Visual),
            None,
        ));
    }
    let controls = capacity.controls();
    if controls.max_items() < 1 || controls.max_bytes() < CONTROL_ENVELOPE_BYTES {
        return Err(SchedulerError::new(
            SchedulerErrorKind::CapacityTooSmall(SchedulerLane::Controls),
            None,
        ));
    }
    let required_retained = capacity
        .in_flight()
        .max_items()
        .checked_add(1)
        .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
    if runtime.capacity.retained_generations() < required_retained {
        return Err(SchedulerError::new(
            SchedulerErrorKind::RetainedGenerationCapacity,
            None,
        ));
    }
    Ok(())
}

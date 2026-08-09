use std::collections::VecDeque;

use super::{
    CompletionWatermark, QueueCapacity, QueueStats, RendererEpoch, SchedulerAction, SchedulerError,
    SchedulerErrorKind, SchedulerLane, SchedulerState, SchedulerTick, UiScheduler,
};

pub(super) const CONTROL_ENVELOPE_BYTES: usize = 32;

/// Monotonic identity assigned when one non-droppable control is accepted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ControlSequence(u64);

impl ControlSequence {
    const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric identity used by deterministic scheduler traces.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Result of atomically admitting one idempotent control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlAdmission {
    /// A new control entered the ordered lane.
    Accepted(ControlSequence),
    /// The same logical control was accepted previously.
    AlreadyAccepted(ControlSequence),
}

#[derive(Clone, Copy)]
pub(super) enum ControlKind {
    Complete(CompletionWatermark),
    RendererLost(RendererEpoch),
    Shutdown,
}

#[derive(Clone, Copy)]
pub(super) struct ControlRecord {
    pub(super) sequence: ControlSequence,
    pub(super) accepted_tick: SchedulerTick,
    pub(super) kind: ControlKind,
}

pub(super) struct ControlQueue {
    records: VecDeque<ControlRecord>,
    accounted_bytes: usize,
    next_sequence: Option<u64>,
    projected_completion: Option<(CompletionWatermark, ControlSequence)>,
    renderer_loss: Option<(RendererEpoch, ControlSequence)>,
    shutdown: Option<ControlSequence>,
}

impl ControlQueue {
    pub(super) const fn new() -> Self {
        Self {
            records: VecDeque::new(),
            accounted_bytes: 0,
            next_sequence: Some(0),
            projected_completion: None,
            renderer_loss: None,
            shutdown: None,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub(super) fn earliest_tick(&self) -> Option<SchedulerTick> {
        self.records.front().map(|record| record.accepted_tick)
    }

    pub(super) fn stats(&self) -> QueueStats {
        match (self.records.front(), self.records.back()) {
            (Some(first), Some(last)) => QueueStats::counted(
                self.records.len(),
                self.accounted_bytes,
                first.accepted_tick,
                last.accepted_tick,
            ),
            _ => QueueStats::empty(),
        }
    }

    fn projected_completion(&self) -> Option<(CompletionWatermark, ControlSequence)> {
        self.projected_completion
    }

    fn renderer_loss(&self) -> Option<(RendererEpoch, ControlSequence)> {
        self.renderer_loss
    }

    fn shutdown(&self) -> Option<ControlSequence> {
        self.shutdown
    }

    fn admit_ordinary(
        &mut self,
        kind: ControlKind,
        tick: SchedulerTick,
        capacity: QueueCapacity,
    ) -> Result<ControlSequence, SchedulerError> {
        self.admit(kind, tick, capacity, self.shutdown.is_none())
    }

    fn admit_shutdown(
        &mut self,
        tick: SchedulerTick,
        capacity: QueueCapacity,
    ) -> Result<ControlSequence, SchedulerError> {
        self.admit(ControlKind::Shutdown, tick, capacity, false)
    }

    fn admit(
        &mut self,
        kind: ControlKind,
        tick: SchedulerTick,
        capacity: QueueCapacity,
        reserve_shutdown: bool,
    ) -> Result<ControlSequence, SchedulerError> {
        let (next_bytes, value) = self.project_admission(capacity, reserve_shutdown)?;
        let sequence = ControlSequence::new(value);
        self.records.push_back(ControlRecord {
            sequence,
            accepted_tick: tick,
            kind,
        });
        self.accounted_bytes = next_bytes;
        self.next_sequence = value.checked_add(1);
        Ok(sequence)
    }

    fn project_admission(
        &self,
        capacity: QueueCapacity,
        reserve_shutdown: bool,
    ) -> Result<(usize, u64), SchedulerError> {
        let next_items = self
            .records
            .len()
            .checked_add(1)
            .ok_or_else(arithmetic_error)?;
        let required_items = next_items
            .checked_add(usize::from(reserve_shutdown))
            .ok_or_else(arithmetic_error)?;
        let next_bytes = self
            .accounted_bytes
            .checked_add(CONTROL_ENVELOPE_BYTES)
            .ok_or_else(arithmetic_error)?;
        let required_bytes = next_bytes
            .checked_add(if reserve_shutdown {
                CONTROL_ENVELOPE_BYTES
            } else {
                0
            })
            .ok_or_else(arithmetic_error)?;
        if required_items > capacity.max_items() || required_bytes > capacity.max_bytes() {
            return Err(SchedulerError::new(
                SchedulerErrorKind::CapacityExceeded(SchedulerLane::Controls),
                None,
            ));
        }
        let value = self.next_sequence.ok_or_else(arithmetic_error)?;
        Ok((next_bytes, value))
    }

    fn ensure_shutdown_admissible(&self, capacity: QueueCapacity) -> Result<(), SchedulerError> {
        if self.shutdown.is_some() {
            return Ok(());
        }
        self.project_admission(capacity, false).map(|_| ())
    }

    fn front(&self) -> Option<ControlRecord> {
        self.records.front().copied()
    }

    fn retire_front(&mut self) -> Result<(), SchedulerError> {
        let remaining_bytes = self
            .accounted_bytes
            .checked_sub(CONTROL_ENVELOPE_BYTES)
            .ok_or_else(arithmetic_error)?;
        if self.records.pop_front().is_none() {
            return Err(arithmetic_error());
        }
        self.accounted_bytes = remaining_bytes;
        Ok(())
    }
}

impl UiScheduler {
    pub(super) fn controls_pending(&self) -> bool {
        !self.controls.is_empty()
    }

    pub(super) fn ensure_callback_shutdown_admissible(&self) -> Result<(), SchedulerError> {
        self.controls
            .ensure_shutdown_admissible(self.capacity.controls())
    }

    pub(super) fn admit_completion_control(
        &mut self,
        watermark: CompletionWatermark,
        tick: SchedulerTick,
    ) -> Result<ControlAdmission, SchedulerError> {
        self.validate_completion_watermark(watermark)?;
        if let Some((projected, sequence)) = self.controls.projected_completion() {
            if watermark.token() < projected.token() {
                return Err(SchedulerError::new(
                    SchedulerErrorKind::CompletionRegression,
                    None,
                ));
            }
            if watermark == projected {
                return Ok(ControlAdmission::AlreadyAccepted(sequence));
            }
        }

        let sequence = self.controls.admit_ordinary(
            ControlKind::Complete(watermark),
            tick,
            self.capacity.controls(),
        )?;
        self.controls.projected_completion = Some((watermark, sequence));
        Ok(ControlAdmission::Accepted(sequence))
    }

    pub(super) fn admit_renderer_loss_control(
        &mut self,
        epoch: RendererEpoch,
        tick: SchedulerTick,
    ) -> Result<ControlAdmission, SchedulerError> {
        if epoch != self.renderer_epoch {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ForeignRendererEpoch,
                None,
            ));
        }
        if let Some((accepted_epoch, sequence)) = self.controls.renderer_loss() {
            debug_assert_eq!(accepted_epoch, epoch);
            return Ok(ControlAdmission::AlreadyAccepted(sequence));
        }
        if self.terminal_pressure.is_some() {
            return Err(self.residence_error());
        }
        if self.controls.shutdown().is_some() || self.state != SchedulerState::Running {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ControlPending,
                None,
            ));
        }

        let sequence = self.controls.admit_ordinary(
            ControlKind::RendererLost(epoch),
            tick,
            self.capacity.controls(),
        )?;
        self.controls.renderer_loss = Some((epoch, sequence));
        Ok(ControlAdmission::Accepted(sequence))
    }

    pub(super) fn admit_shutdown_control(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<ControlAdmission, SchedulerError> {
        if let Some(sequence) = self.controls.shutdown() {
            return Ok(ControlAdmission::AlreadyAccepted(sequence));
        }

        let sequence = self
            .controls
            .admit_shutdown(tick, self.capacity.controls())?;
        self.controls.shutdown = Some(sequence);
        if self.state == SchedulerState::Running {
            self.state = SchedulerState::ShutdownQueued;
        }
        Ok(ControlAdmission::Accepted(sequence))
    }

    pub(super) fn latch_shutdown(&mut self, tick: SchedulerTick) {
        if self.admit_shutdown_control(tick).is_err() {
            self.state = SchedulerState::Faulted;
        }
    }

    pub(super) fn process_next_control(
        &mut self,
    ) -> Result<Option<SchedulerAction>, SchedulerError> {
        let Some(record) = self.controls.front() else {
            return Ok(None);
        };
        let action = match record.kind {
            ControlKind::Complete(watermark) => {
                self.apply_completion(watermark)?;
                if self.state == SchedulerState::Draining && self.in_flight.is_empty() {
                    self.state = SchedulerState::Stopped;
                }
                None
            }
            ControlKind::RendererLost(epoch) => {
                debug_assert_eq!(epoch, self.renderer_epoch);
                self.deferred = None;
                self.visual = None;
                self.state = SchedulerState::Faulted;
                None
            }
            ControlKind::Shutdown => {
                self.deferred = None;
                self.visual = None;
                if self.terminal_pressure.is_none() {
                    self.state = if self.in_flight.is_empty() {
                        SchedulerState::Stopped
                    } else {
                        SchedulerState::Draining
                    };
                }
                Some(SchedulerAction::StopRenderer(record.sequence))
            }
        };
        self.controls.retire_front()?;
        Ok(action)
    }
}

fn arithmetic_error() -> SchedulerError {
    SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None)
}

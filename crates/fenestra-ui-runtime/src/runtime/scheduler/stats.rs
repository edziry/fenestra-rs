use super::types::SchedulerTick;

/// Current accounting for one scheduler lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueStats {
    items: usize,
    accounted_bytes: usize,
    earliest_tick: Option<SchedulerTick>,
    latest_tick: Option<SchedulerTick>,
}

impl QueueStats {
    pub(super) const fn empty() -> Self {
        Self {
            items: 0,
            accounted_bytes: 0,
            earliest_tick: None,
            latest_tick: None,
        }
    }

    pub(super) const fn occupied_bytes(
        earliest: SchedulerTick,
        latest: SchedulerTick,
        accounted_bytes: usize,
    ) -> Self {
        Self {
            items: 1,
            accounted_bytes,
            earliest_tick: Some(earliest),
            latest_tick: Some(latest),
        }
    }

    pub(super) const fn counted(
        items: usize,
        accounted_bytes: usize,
        earliest_tick: SchedulerTick,
        latest_tick: SchedulerTick,
    ) -> Self {
        Self {
            items,
            accounted_bytes,
            earliest_tick: Some(earliest_tick),
            latest_tick: Some(latest_tick),
        }
    }

    /// Returns the accepted item count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns the accepted protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }

    /// Returns the first unconsumed acceptance tick.
    #[must_use]
    pub const fn earliest_tick(self) -> Option<SchedulerTick> {
        self.earliest_tick
    }

    /// Returns the latest replacement tick.
    #[must_use]
    pub const fn latest_tick(self) -> Option<SchedulerTick> {
        self.latest_tick
    }
}

/// Bounded scheduler accounting observed after a transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerStats {
    deferred: QueueStats,
    controls: QueueStats,
    visual: QueueStats,
    in_flight: QueueStats,
}

impl SchedulerStats {
    pub(super) const fn new(
        deferred: QueueStats,
        controls: QueueStats,
        visual: QueueStats,
        in_flight: QueueStats,
    ) -> Self {
        Self {
            deferred,
            controls,
            visual,
            in_flight,
        }
    }

    /// Returns deferred callback transaction accounting.
    #[must_use]
    pub const fn deferred(self) -> QueueStats {
        self.deferred
    }

    /// Returns accepted non-droppable control accounting.
    #[must_use]
    pub const fn controls(self) -> QueueStats {
        self.controls
    }

    /// Returns the replaceable visual lane accounting.
    #[must_use]
    pub const fn visual(self) -> QueueStats {
        self.visual
    }

    /// Returns accepted renderer submission accounting.
    #[must_use]
    pub const fn in_flight(self) -> QueueStats {
        self.in_flight
    }
}

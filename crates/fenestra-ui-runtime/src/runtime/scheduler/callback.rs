use super::super::transaction::UiTransaction;
use super::super::view::CommittedRuntimeSnapshot;
use super::{
    SchedulerError, SchedulerErrorKind, SchedulerLane, SchedulerState, SchedulerTick, UiScheduler,
};

pub(super) const DEFERRED_ENVELOPE_BYTES: usize = 16;
const DEFERRED_OPERATION_BYTES: usize = 64;

/// Result of finishing one outer callback scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackFinish {
    /// The callback staged no operations and occupied no deferred slot.
    NoChanges,
    /// One bounded transaction entered the deferred lane.
    Deferred {
        /// Number of closed mutation operations retained by the lane.
        operation_count: usize,
        /// Total V1 protocol-accounted bytes retained by the lane.
        accounted_bytes: usize,
    },
    /// Shutdown won over staged mutation and was latched exactly once.
    ShutdownRequested,
}

/// Exclusively borrowed callback view over one exact committed snapshot.
pub struct CallbackScope<'scheduler> {
    scheduler: &'scheduler mut UiScheduler,
    committed: CommittedRuntimeSnapshot,
    transaction: Option<UiTransaction>,
    accepted_tick: SchedulerTick,
    shutdown_requested: bool,
}

impl CallbackScope<'_> {
    /// Returns the immutable snapshot captured before callback code runs.
    #[must_use]
    pub const fn committed(&self) -> &CommittedRuntimeSnapshot {
        &self.committed
    }

    /// Returns the single detached transaction shared by every nested scope.
    pub fn transaction(&mut self) -> &mut UiTransaction {
        self.transaction
            .as_mut()
            .expect("a live callback scope must retain its transaction")
    }

    /// Returns one for the outermost callback scope.
    #[must_use]
    pub const fn depth(&self) -> usize {
        1
    }

    /// Opens a nested guard over the same snapshot and transaction.
    pub fn begin_nested(&mut self) -> NestedCallbackScope<'_> {
        NestedCallbackScope {
            committed: &self.committed,
            transaction: self
                .transaction
                .as_mut()
                .expect("a live callback scope must retain its transaction"),
            shutdown_requested: &mut self.shutdown_requested,
            depth: 2,
        }
    }

    /// Records one idempotent shutdown request for the outer drop boundary.
    pub fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }

    /// Finishes the outer callback without publishing its transaction.
    pub fn finish(mut self) -> Result<CallbackFinish, SchedulerError> {
        if self.shutdown_requested {
            self.transaction.take();
            return Ok(CallbackFinish::ShutdownRequested);
        }
        let operation_count = self.transaction().operation_count();
        if operation_count == 0 {
            self.transaction.take();
            return Ok(CallbackFinish::NoChanges);
        }
        let accounted_bytes = deferred_bytes(operation_count)?;
        let capacity = self.scheduler.capacity.deferred();
        if capacity.max_items() < 1 || accounted_bytes > capacity.max_bytes() {
            return Err(SchedulerError::new(
                SchedulerErrorKind::CapacityExceeded(SchedulerLane::Deferred),
                None,
            ));
        }
        if self.scheduler.deferred.is_some() {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ControlPending,
                None,
            ));
        }
        let transaction = self
            .transaction
            .take()
            .expect("a live callback scope must retain its transaction");
        self.scheduler.deferred = Some(DeferredTransaction {
            transaction,
            accounted_bytes,
            accepted_tick: self.accepted_tick,
        });
        Ok(CallbackFinish::Deferred {
            operation_count,
            accounted_bytes,
        })
    }
}

impl Drop for CallbackScope<'_> {
    fn drop(&mut self) {
        if self.shutdown_requested {
            self.scheduler.latch_shutdown(self.accepted_tick);
        }
    }
}

/// Nested callback guard sharing the outer snapshot and transaction.
pub struct NestedCallbackScope<'scope> {
    committed: &'scope CommittedRuntimeSnapshot,
    transaction: &'scope mut UiTransaction,
    shutdown_requested: &'scope mut bool,
    depth: usize,
}

impl NestedCallbackScope<'_> {
    /// Returns the immutable snapshot captured by the outer callback.
    #[must_use]
    pub const fn committed(&self) -> &CommittedRuntimeSnapshot {
        self.committed
    }

    /// Returns the outer callback's one shared detached transaction.
    pub fn transaction(&mut self) -> &mut UiTransaction {
        self.transaction
    }

    /// Returns this guard's explicit one-based nesting depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Opens another guard over the same callback-owned state.
    pub fn begin_nested(&mut self) -> NestedCallbackScope<'_> {
        NestedCallbackScope {
            committed: self.committed,
            transaction: &mut *self.transaction,
            shutdown_requested: &mut *self.shutdown_requested,
            depth: self.depth + 1,
        }
    }

    /// Records one idempotent shutdown request on the outer scope.
    pub fn request_shutdown(&mut self) {
        *self.shutdown_requested = true;
    }
}

pub(super) struct DeferredTransaction {
    pub(super) transaction: UiTransaction,
    pub(super) accounted_bytes: usize,
    pub(super) accepted_tick: SchedulerTick,
}

impl UiScheduler {
    /// Begins one outer callback against the exact current allocation.
    pub fn begin_callback(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<CallbackScope<'_>, SchedulerError> {
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
        self.ensure_callback_shutdown_admissible()?;
        let committed = self.runtime.committed();
        let transaction = self.runtime.begin_transaction();
        Ok(CallbackScope {
            scheduler: self,
            committed,
            transaction: Some(transaction),
            accepted_tick: tick,
            shutdown_requested: false,
        })
    }
}

fn deferred_bytes(operation_count: usize) -> Result<usize, SchedulerError> {
    let operations = operation_count
        .checked_mul(DEFERRED_OPERATION_BYTES)
        .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
    DEFERRED_ENVELOPE_BYTES
        .checked_add(operations)
        .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))
}

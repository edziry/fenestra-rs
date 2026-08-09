use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CallbackFinish, NodeId, RuntimeGeneration, SchedulerError, SchedulerErrorKind, SchedulerInput,
    SchedulerInputResult, SchedulerState, SchedulerTick, TransactionError, UiScheduler,
    UiTransaction,
};

/// Closed nesting depth used by one deterministic callback script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeCallbackDepthV1 {
    /// Run only the outer callback scope.
    Outer,
    /// Run one nested scope inside the outer callback.
    Nested,
    /// Run a grandchild scope inside two enclosing callback scopes.
    Grandchild,
}

/// One owned direct-property mutation in a deterministic callback script.
#[derive(Clone, Eq, PartialEq)]
pub struct FakeCallbackMutationV1 {
    node: NodeId,
    property: PropertyId,
    value: PropertyValue,
}

impl FakeCallbackMutationV1 {
    /// Creates one typed property mutation for the selected callback depth.
    #[must_use]
    pub const fn new(node: NodeId, property: PropertyId, value: PropertyValue) -> Self {
        Self {
            node,
            property,
            value,
        }
    }
}

/// Closed deterministic behavior performed by one fake platform callback.
#[derive(Clone, Eq, PartialEq)]
pub struct FakeCallbackScriptV1 {
    depth: FakeCallbackDepthV1,
    mutation: Option<FakeCallbackMutationV1>,
    request_shutdown: bool,
}

impl FakeCallbackScriptV1 {
    /// Creates a bounded callback script with at most one mutation.
    #[must_use]
    pub const fn new(
        depth: FakeCallbackDepthV1,
        mutation: Option<FakeCallbackMutationV1>,
        request_shutdown: bool,
    ) -> Self {
        Self {
            depth,
            mutation,
            request_shutdown,
        }
    }
}

/// Owned callback observations that do not retain a committed snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FakeCallbackReportV1 {
    captured_generation: RuntimeGeneration,
    deepest_depth: usize,
    shares_entry_snapshot: bool,
    finish: CallbackFinish,
}

impl FakeCallbackReportV1 {
    pub(crate) const fn from_parts(
        captured_generation: RuntimeGeneration,
        deepest_depth: usize,
        shares_entry_snapshot: bool,
        finish: CallbackFinish,
    ) -> Self {
        Self {
            captured_generation,
            deepest_depth,
            shares_entry_snapshot,
            finish,
        }
    }

    /// Returns the generation captured before callback code ran.
    #[must_use]
    pub const fn captured_generation(self) -> RuntimeGeneration {
        self.captured_generation
    }

    /// Returns the deepest one-based callback depth reached by the script.
    #[must_use]
    pub const fn deepest_depth(self) -> usize {
        self.deepest_depth
    }

    /// Reports whether every scope shared the entry snapshot allocation.
    #[must_use]
    pub const fn shares_entry_snapshot(self) -> bool {
        self.shares_entry_snapshot
    }

    /// Returns the scheduler's closed outer callback result.
    #[must_use]
    pub const fn finish(self) -> CallbackFinish {
        self.finish
    }
}

/// Result of atomically attempting one fake frame-ready delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeFrameReadyDeliveryV1 {
    /// The runtime accepted the frame-ready observation.
    Accepted,
    /// A transient control boundary made the fake retain one observation.
    Retained(SchedulerErrorKind),
    /// A terminal runtime transition made the retained observation obsolete.
    Canceled,
}

/// Closed failures produced by the deterministic fake platform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakePlatformErrorKindV1 {
    /// The runtime scheduler rejected the corresponding typed interaction.
    Scheduler(SchedulerErrorKind),
}

/// Privacy-safe fake platform failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FakePlatformErrorV1 {
    kind: FakePlatformErrorKindV1,
    operation_index: Option<usize>,
}

impl FakePlatformErrorV1 {
    const fn new(kind: FakePlatformErrorKindV1, operation_index: Option<usize>) -> Self {
        Self {
            kind,
            operation_index,
        }
    }

    /// Returns the closed fake platform failure category.
    #[must_use]
    pub const fn kind(self) -> FakePlatformErrorKindV1 {
        self.kind
    }

    /// Returns the staged operation associated with the underlying rejection.
    #[must_use]
    pub const fn operation_index(self) -> Option<usize> {
        self.operation_index
    }
}

impl fmt::Debug for FakePlatformErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakePlatformErrorV1")
            .field("kind", &self.kind)
            .field("operation_index", &self.operation_index)
            .finish()
    }
}

impl fmt::Display for FakePlatformErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fake platform failed: {:?}", self.kind)
    }
}

impl Error for FakePlatformErrorV1 {}

/// Deterministic single-owner platform adapter for scheduler experiments.
pub struct FakePlatformV1 {
    pending_frame_ready: bool,
}

impl FakePlatformV1 {
    /// Creates an empty fake platform without reading host platform state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending_frame_ready: false,
        }
    }

    /// Runs one bounded nested callback script against one captured snapshot.
    pub fn run_callback(
        &mut self,
        scheduler: &mut UiScheduler,
        script: FakeCallbackScriptV1,
        tick: SchedulerTick,
    ) -> Result<FakeCallbackReportV1, FakePlatformErrorV1> {
        if self.pending_frame_ready {
            return Err(input_order_error());
        }
        let entry = scheduler.committed();
        let captured_generation = entry.generation();
        let mut scope = scheduler.begin_callback(tick).map_err(scheduler_error)?;
        let mut deepest_depth = scope.depth();
        let mut shares_entry_snapshot = scope.committed().shares_state_with(&entry);

        match script.depth {
            FakeCallbackDepthV1::Outer => {
                if let Some(mutation) = script.mutation {
                    stage_mutation(scope.transaction(), mutation)?;
                }
                if script.request_shutdown {
                    scope.request_shutdown();
                }
            }
            FakeCallbackDepthV1::Nested => {
                let mut nested = scope.begin_nested();
                deepest_depth = nested.depth();
                shares_entry_snapshot &= nested.committed().shares_state_with(&entry);
                if let Some(mutation) = script.mutation {
                    stage_mutation(nested.transaction(), mutation)?;
                }
                if script.request_shutdown {
                    nested.request_shutdown();
                }
            }
            FakeCallbackDepthV1::Grandchild => {
                let mut nested = scope.begin_nested();
                shares_entry_snapshot &= nested.committed().shares_state_with(&entry);
                let mut grandchild = nested.begin_nested();
                deepest_depth = grandchild.depth();
                shares_entry_snapshot &= grandchild.committed().shares_state_with(&entry);
                if let Some(mutation) = script.mutation {
                    stage_mutation(grandchild.transaction(), mutation)?;
                }
                if script.request_shutdown {
                    grandchild.request_shutdown();
                }
            }
        }

        let finish = scope.finish().map_err(scheduler_error)?;
        Ok(FakeCallbackReportV1::from_parts(
            captured_generation,
            deepest_depth,
            shares_entry_snapshot,
            finish,
        ))
    }

    /// Delivers or retains one frame-ready observation without overwriting one.
    pub fn frame_ready(
        &mut self,
        scheduler: &mut UiScheduler,
        tick: SchedulerTick,
    ) -> Result<FakeFrameReadyDeliveryV1, FakePlatformErrorV1> {
        if self.pending_frame_ready {
            return Err(input_order_error());
        }
        self.deliver_frame_ready(scheduler, tick, false)
    }

    /// Retries the single retained frame-ready observation.
    pub fn retry_frame_ready(
        &mut self,
        scheduler: &mut UiScheduler,
        tick: SchedulerTick,
    ) -> Result<FakeFrameReadyDeliveryV1, FakePlatformErrorV1> {
        if !self.pending_frame_ready {
            return Err(input_order_error());
        }
        self.deliver_frame_ready(scheduler, tick, true)
    }

    /// Reports whether one frame-ready observation is retained for retry.
    #[must_use]
    pub const fn has_pending_frame_ready(&self) -> bool {
        self.pending_frame_ready
    }

    fn deliver_frame_ready(
        &mut self,
        scheduler: &mut UiScheduler,
        tick: SchedulerTick,
        is_retry: bool,
    ) -> Result<FakeFrameReadyDeliveryV1, FakePlatformErrorV1> {
        match scheduler.process_input(SchedulerInput::FrameReady, tick) {
            Ok(SchedulerInputResult::FrameReady) => {
                self.pending_frame_ready = false;
                Ok(FakeFrameReadyDeliveryV1::Accepted)
            }
            Ok(_) => Err(input_order_error()),
            Err(error)
                if error.kind() == SchedulerErrorKind::ControlPending
                    && scheduler.state() != SchedulerState::Running =>
            {
                self.pending_frame_ready = false;
                Ok(FakeFrameReadyDeliveryV1::Canceled)
            }
            Err(error) if is_retry && error.kind() == SchedulerErrorKind::InputOutOfOrder => {
                self.pending_frame_ready = false;
                Ok(FakeFrameReadyDeliveryV1::Canceled)
            }
            Err(error) if error.kind() == SchedulerErrorKind::ControlPending => {
                self.pending_frame_ready = true;
                Ok(FakeFrameReadyDeliveryV1::Retained(error.kind()))
            }
            Err(error) => Err(scheduler_error(error)),
        }
    }
}

impl Default for FakePlatformV1 {
    fn default() -> Self {
        Self::new()
    }
}

fn stage_mutation(
    transaction: &mut UiTransaction,
    mutation: FakeCallbackMutationV1,
) -> Result<(), FakePlatformErrorV1> {
    transaction
        .set_property(mutation.node, mutation.property, mutation.value)
        .map_err(transaction_error)
}

fn scheduler_error(error: SchedulerError) -> FakePlatformErrorV1 {
    FakePlatformErrorV1::new(
        FakePlatformErrorKindV1::Scheduler(error.kind()),
        error.operation_index(),
    )
}

fn transaction_error(error: TransactionError) -> FakePlatformErrorV1 {
    FakePlatformErrorV1::new(
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::Transaction(error.kind())),
        error.operation_index(),
    )
}

fn input_order_error() -> FakePlatformErrorV1 {
    FakePlatformErrorV1::new(
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder),
        None,
    )
}

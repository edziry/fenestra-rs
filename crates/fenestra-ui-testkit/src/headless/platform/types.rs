use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CallbackFinish, HeadlessPoint, NodeId, RuntimeGeneration, SchedulerErrorKind,
};

use crate::scheduler::{FakeCallbackDepthV1, FakeCallbackMutationV1, FakeCallbackReportV1};

/// Closed target vocabulary for deterministic headless pointer input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessPointerTargetV1 {
    /// The committed hit-test projection contains no target at the point.
    None,
    /// The fixed static control received the pointer.
    StaticControl,
    /// One keyed member received the pointer.
    Key(u64),
}

/// One target-relative property mutation for a fake pointer callback.
#[derive(Clone, Eq, PartialEq)]
pub struct HeadlessPointerMutationV1 {
    property: PropertyId,
    value: PropertyValue,
}

impl HeadlessPointerMutationV1 {
    /// Creates one typed mutation without exposing a runtime node identity.
    #[must_use]
    pub const fn new(property: PropertyId, value: PropertyValue) -> Self {
        Self { property, value }
    }

    pub(super) fn with_node(self, node: NodeId) -> FakeCallbackMutationV1 {
        FakeCallbackMutationV1::new(node, self.property, self.value)
    }
}

/// Bounded pointer query and optional callback mutation.
#[derive(Clone, Eq, PartialEq)]
pub struct HeadlessPointerScriptV1 {
    pub(super) point: HeadlessPoint,
    pub(super) depth: FakeCallbackDepthV1,
    pub(super) mutation: Option<HeadlessPointerMutationV1>,
}

impl HeadlessPointerScriptV1 {
    /// Creates one deterministic pointer callback script.
    #[must_use]
    pub const fn new(
        point: HeadlessPoint,
        depth: FakeCallbackDepthV1,
        mutation: Option<HeadlessPointerMutationV1>,
    ) -> Self {
        Self {
            point,
            depth,
            mutation,
        }
    }
}

/// Opaque pointer capability captured from one committed generation.
#[derive(Clone, Copy)]
pub struct HeadlessPointerCaptureV1 {
    pub(super) generation: RuntimeGeneration,
    pub(super) target: HeadlessPointerTargetV1,
    pub(super) node: Option<NodeId>,
}

impl HeadlessPointerCaptureV1 {
    /// Returns the generation used for the hit-test query.
    #[must_use]
    pub const fn generation(&self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the stable closed target without its runtime identity.
    #[must_use]
    pub const fn target(&self) -> HeadlessPointerTargetV1 {
        self.target
    }
}

impl fmt::Debug for HeadlessPointerCaptureV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessPointerCaptureV1")
            .field("generation", &self.generation.get())
            .field("target", &self.target)
            .finish()
    }
}

/// Callback result paired with its closed pointer target, if any.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessCallbackReportV1 {
    callback: FakeCallbackReportV1,
    target: HeadlessPointerTargetV1,
}

impl HeadlessCallbackReportV1 {
    pub(super) const fn new(
        callback: FakeCallbackReportV1,
        target: HeadlessPointerTargetV1,
    ) -> Self {
        Self { callback, target }
    }

    /// Returns the generation captured by the outer callback scope.
    #[must_use]
    pub const fn captured_generation(self) -> RuntimeGeneration {
        self.callback.captured_generation()
    }

    /// Returns the deepest one-based callback depth reached.
    #[must_use]
    pub const fn deepest_depth(self) -> usize {
        self.callback.deepest_depth()
    }

    /// Reports whether every callback scope shared the entry allocation.
    #[must_use]
    pub const fn shares_entry_snapshot(self) -> bool {
        self.callback.shares_entry_snapshot()
    }

    /// Returns the scheduler's closed callback result.
    #[must_use]
    pub const fn finish(self) -> CallbackFinish {
        self.callback.finish()
    }

    /// Returns the pointer target, or `None` for resize and pointer misses.
    #[must_use]
    pub const fn target(self) -> HeadlessPointerTargetV1 {
        self.target
    }
}

/// Closed failures produced by deterministic headless platform input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessPlatformErrorKindV1 {
    /// The scheduler rejected the corresponding callback interaction.
    Scheduler(SchedulerErrorKind),
    /// The committed runtime has no headless projection.
    HeadlessUnavailable,
    /// A runtime hit cannot be represented by the fixed target vocabulary.
    IdentityMismatch,
}

/// Privacy-safe headless platform failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessPlatformErrorV1 {
    kind: HeadlessPlatformErrorKindV1,
    operation_index: Option<usize>,
}

impl HeadlessPlatformErrorV1 {
    pub(super) const fn new(
        kind: HeadlessPlatformErrorKindV1,
        operation_index: Option<usize>,
    ) -> Self {
        Self {
            kind,
            operation_index,
        }
    }

    /// Returns the closed failure category.
    #[must_use]
    pub const fn kind(self) -> HeadlessPlatformErrorKindV1 {
        self.kind
    }

    /// Returns the staged operation associated with a scheduler rejection.
    #[must_use]
    pub const fn operation_index(self) -> Option<usize> {
        self.operation_index
    }
}

impl fmt::Debug for HeadlessPlatformErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessPlatformErrorV1")
            .field("kind", &self.kind)
            .field("operation_index", &self.operation_index)
            .finish()
    }
}

impl fmt::Display for HeadlessPlatformErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "headless platform failed: {:?}", self.kind)
    }
}

impl Error for HeadlessPlatformErrorV1 {}

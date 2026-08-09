use std::error::Error;
use std::fmt;

use fenestra_ui_runtime::prototype::RuntimeGeneration;

use crate::scheduler::SchedulerTraceV1;

use super::super::oracle::NormalizedHeadlessProjectionV1;
use super::super::trace::{HeadlessFailureCauseV1, HeadlessTraceV1};

/// Closed synthetic outcome of the registered headless run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessResultV1 {
    /// The synthetic script and oracle completed without a mismatch.
    Pass,
    /// The synthetic result requires a documented adaptation.
    Adapt,
    /// The synthetic result requires the feasibility path to stop.
    Stop,
}

/// Privacy-safe failure from the fixed headless runner.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessRunErrorV1 {
    kind: HeadlessFailureCauseV1,
}

impl HeadlessRunErrorV1 {
    pub(super) const fn new(kind: HeadlessFailureCauseV1) -> Self {
        Self { kind }
    }

    /// Returns the closed failure source.
    #[must_use]
    pub const fn kind(&self) -> HeadlessFailureCauseV1 {
        self.kind
    }
}

impl fmt::Debug for HeadlessRunErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessRunErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for HeadlessRunErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "headless spine run failed: {:?}", self.kind)
    }
}

impl Error for HeadlessRunErrorV1 {}

/// Owned deterministic evidence from one complete fixed headless run.
pub struct HeadlessRunV1 {
    pub(super) result: HeadlessResultV1,
    pub(super) final_generation: RuntimeGeneration,
    pub(super) final_projection: NormalizedHeadlessProjectionV1,
    pub(super) headless_trace: HeadlessTraceV1,
    pub(super) scheduler_trace: SchedulerTraceV1,
}

impl HeadlessRunV1 {
    /// Returns the closed synthetic result.
    #[must_use]
    pub const fn result(&self) -> HeadlessResultV1 {
        self.result
    }

    /// Returns the final committed runtime generation.
    #[must_use]
    pub const fn final_generation(&self) -> RuntimeGeneration {
        self.final_generation
    }

    /// Returns the owned final normalized projection.
    #[must_use]
    pub const fn final_projection(&self) -> &NormalizedHeadlessProjectionV1 {
        &self.final_projection
    }

    /// Returns the complete bounded headless trace.
    #[must_use]
    pub const fn headless_trace(&self) -> &HeadlessTraceV1 {
        &self.headless_trace
    }

    /// Returns the complete correlated scheduler trace.
    #[must_use]
    pub const fn scheduler_trace(&self) -> &SchedulerTraceV1 {
        &self.scheduler_trace
    }
}

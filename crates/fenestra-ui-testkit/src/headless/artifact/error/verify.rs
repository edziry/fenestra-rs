use std::error::Error;
use std::fmt;

use crate::headless::trace::HeadlessFailureCauseV1;

/// Registered capacity rows checked during semantic artifact verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactCapacityKindV1 {
    /// Validated IR storage limits.
    Ir,
    /// Linked style storage limit.
    Style,
    /// Runtime storage limits.
    Runtime,
    /// Headless projection storage limits.
    Projection,
    /// Scheduler lane storage limits.
    Scheduler,
    /// Fake renderer storage limits.
    Renderer,
    /// Scheduler trace storage limits.
    SchedulerTrace,
    /// Headless trace storage limits.
    HeadlessTrace,
    /// Encoded artifact storage limits.
    Artifact,
}

impl HeadlessArtifactCapacityKindV1 {
    /// Capacity verification priority in canonical wire-row order.
    pub const ALL: [Self; 9] = [
        Self::Ir,
        Self::Style,
        Self::Runtime,
        Self::Projection,
        Self::Scheduler,
        Self::Renderer,
        Self::SchedulerTrace,
        Self::HeadlessTrace,
        Self::Artifact,
    ];
}

/// Closed semantic failures from verifying a decoded headless artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactVerificationErrorKindV1 {
    /// Registered fixture metadata differed.
    FixtureMismatch,
    /// One registered capacity row differed.
    CapacityMismatch(HeadlessArtifactCapacityKindV1),
    /// The fresh fixed runner failed before producing evidence.
    ReplayFailed(HeadlessFailureCauseV1),
    /// The stored closed result differed.
    ResultMismatch,
    /// The final committed generation differed.
    FinalGenerationMismatch,
    /// The final logical surface differed.
    SurfaceMismatch,
    /// The complete headless trace differed.
    HeadlessTraceMismatch,
    /// The complete scheduler trace differed.
    SchedulerTraceMismatch,
    /// Computed-style records differed.
    ComputedStyleMismatch,
    /// Geometry records differed.
    GeometryMismatch,
    /// Semantic records differed.
    SemanticsMismatch,
    /// Hit-region records differed.
    HitMismatch,
    /// Scene rectangles differed.
    SceneMismatch,
}

/// Privacy-safe failure to verify one decoded headless V1 artifact.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessArtifactVerificationErrorV1 {
    kind: HeadlessArtifactVerificationErrorKindV1,
    index: Option<usize>,
}

impl HeadlessArtifactVerificationErrorV1 {
    pub(in crate::headless::artifact) const fn new(
        kind: HeadlessArtifactVerificationErrorKindV1,
    ) -> Self {
        Self { kind, index: None }
    }

    pub(in crate::headless::artifact) const fn at(
        kind: HeadlessArtifactVerificationErrorKindV1,
        index: usize,
    ) -> Self {
        Self {
            kind,
            index: Some(index),
        }
    }

    /// Returns the first closed semantic mismatch class.
    #[must_use]
    pub const fn kind(self) -> HeadlessArtifactVerificationErrorKindV1 {
        self.kind
    }

    /// Returns the first differing vector index when applicable.
    #[must_use]
    pub const fn index(self) -> Option<usize> {
        self.index
    }
}

impl fmt::Debug for HeadlessArtifactVerificationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessArtifactVerificationErrorV1")
            .field("kind", &self.kind)
            .field("index", &self.index)
            .finish()
    }
}

impl fmt::Display for HeadlessArtifactVerificationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "headless artifact verification failed: {:?}",
            self.kind
        )
    }
}

impl Error for HeadlessArtifactVerificationErrorV1 {}

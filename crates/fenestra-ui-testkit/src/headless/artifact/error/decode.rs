use std::error::Error;
use std::fmt;

/// Inclusive storage ceilings checked by the headless V1 decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactLimitKindV1 {
    /// Complete encoded artifact bytes.
    ArtifactBytes,
    /// Bytes in one line, excluding its line feed.
    LineBytes,
    /// Complete encoded line count.
    Lines,
    /// Headless trace event count.
    HeadlessEvents,
    /// Logical headless trace bytes.
    HeadlessTraceBytes,
    /// Scheduler trace event count.
    SchedulerEvents,
    /// Logical scheduler trace bytes.
    SchedulerTraceBytes,
    /// Final computed-style records.
    ComputedStyles,
    /// Final geometry records.
    Geometry,
    /// Final semantic records.
    Semantics,
    /// Final hit-region records.
    HitRegions,
    /// Final scene-rectangle records.
    SceneRectangles,
    /// Segments in one normalized node path.
    PathDepth,
}

impl HeadlessArtifactLimitKindV1 {
    /// Decoder limit priority from earliest to latest.
    pub const ALL: [Self; 13] = [
        Self::ArtifactBytes,
        Self::LineBytes,
        Self::Lines,
        Self::HeadlessEvents,
        Self::HeadlessTraceBytes,
        Self::SchedulerEvents,
        Self::SchedulerTraceBytes,
        Self::ComputedStyles,
        Self::Geometry,
        Self::Semantics,
        Self::HitRegions,
        Self::SceneRectangles,
        Self::PathDepth,
    ];
}

/// Declared counts checked after all storage ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactCountKindV1 {
    /// Headless trace events.
    HeadlessEvents,
    /// Logical headless trace bytes.
    HeadlessTraceBytes,
    /// Scheduler trace events.
    SchedulerEvents,
    /// Logical scheduler trace bytes.
    SchedulerTraceBytes,
    /// Computed-style records.
    ComputedStyles,
    /// Geometry records.
    Geometry,
    /// Semantic records.
    Semantics,
    /// Hit-region records.
    HitRegions,
    /// Scene-rectangle records.
    SceneRectangles,
}

impl HeadlessArtifactCountKindV1 {
    /// Declared-count priority from earliest to latest.
    pub const ALL: [Self; 9] = [
        Self::HeadlessEvents,
        Self::HeadlessTraceBytes,
        Self::SchedulerEvents,
        Self::SchedulerTraceBytes,
        Self::ComputedStyles,
        Self::Geometry,
        Self::Semantics,
        Self::HitRegions,
        Self::SceneRectangles,
    ];
}

/// Version slots in the canonical headless V1 envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactVersionKindV1 {
    /// Envelope version.
    Envelope,
    /// Fixture version.
    Fixture,
    /// Schema format version.
    Schema,
    /// Construction format version.
    Construction,
    /// Style format version.
    Style,
    /// Trace format version.
    Trace,
    /// Projection format version.
    Projection,
}

/// Ordered singleton and record sections in headless artifact V1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactSectionKindV1 {
    /// Envelope header.
    Header,
    /// Version declaration.
    Versions,
    /// Registered fixture metadata.
    Fixture,
    /// Closed synthetic environment.
    Environment,
    /// Closed projection choices.
    ProjectionChoices,
    /// Nine registered capacity rows.
    Capacities,
    /// Headless trace section.
    HeadlessTrace,
    /// Scheduler trace section.
    SchedulerTrace,
    /// Final projection declaration.
    Projection,
    /// Computed-style records.
    ComputedStyles,
    /// Geometry records.
    Geometry,
    /// Semantic records.
    Semantics,
    /// Hit-region records.
    HitRegions,
    /// Scene-rectangle records.
    SceneRectangles,
    /// Closed run result.
    Result,
    /// Terminal marker.
    End,
}

impl HeadlessArtifactSectionKindV1 {
    /// Canonical section order.
    pub const ALL: [Self; 16] = [
        Self::Header,
        Self::Versions,
        Self::Fixture,
        Self::Environment,
        Self::ProjectionChoices,
        Self::Capacities,
        Self::HeadlessTrace,
        Self::SchedulerTrace,
        Self::Projection,
        Self::ComputedStyles,
        Self::Geometry,
        Self::Semantics,
        Self::HitRegions,
        Self::SceneRectangles,
        Self::Result,
        Self::End,
    ];
}

/// Closed structural failures returned by the headless V1 decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactDecodeErrorKindV1 {
    /// One inclusive storage ceiling was crossed.
    LimitExceeded(HeadlessArtifactLimitKindV1),
    /// A source byte was outside printable ASCII plus line feed.
    InvalidAscii,
    /// The artifact did not end with a line feed.
    MissingFinalLineFeed,
    /// A record did not match the closed grammar.
    MalformedRecord,
    /// A declared format version is unsupported.
    UnsupportedVersion(HeadlessArtifactVersionKindV1),
    /// A primitive used a noncanonical spelling or overflowed its domain.
    NonCanonicalValue,
    /// A required singleton or section was absent.
    MissingSection(HeadlessArtifactSectionKindV1),
    /// A singleton or section occurred more than once.
    DuplicateSection(HeadlessArtifactSectionKindV1),
    /// Canonical records occurred in an invalid order.
    OrderingViolation,
    /// A declared count differed from the corresponding records.
    CountMismatch(HeadlessArtifactCountKindV1),
    /// An artifact-local reference did not resolve.
    InvalidReference,
    /// Records followed the terminal marker.
    TrailingData,
}

/// Privacy-safe failure to decode caller-owned headless V1 bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessArtifactDecodeErrorV1 {
    kind: HeadlessArtifactDecodeErrorKindV1,
    line: Option<u32>,
}

impl HeadlessArtifactDecodeErrorV1 {
    pub(in crate::headless::artifact) const fn new(
        kind: HeadlessArtifactDecodeErrorKindV1,
        line: Option<u32>,
    ) -> Self {
        Self { kind, line }
    }

    pub(in crate::headless::artifact) const fn at(
        kind: HeadlessArtifactDecodeErrorKindV1,
        line: u32,
    ) -> Self {
        Self::new(kind, Some(line))
    }

    /// Returns the closed structural failure class.
    #[must_use]
    pub const fn kind(self) -> HeadlessArtifactDecodeErrorKindV1 {
        self.kind
    }

    /// Returns the one-based source line when one exists.
    #[must_use]
    pub const fn line(self) -> Option<u32> {
        self.line
    }
}

impl fmt::Debug for HeadlessArtifactDecodeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessArtifactDecodeErrorV1")
            .field("kind", &self.kind)
            .field("line", &self.line)
            .finish()
    }
}

impl fmt::Display for HeadlessArtifactDecodeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "headless artifact decode failed: {:?}",
            self.kind
        )
    }
}

impl Error for HeadlessArtifactDecodeErrorV1 {}
